//! Consuming resource lifecycle shared by bundled handle backends.

/// Closing must release ownership even when reporting an error, and must not panic.
/// A backend must not retry a released OS handle or turn close into implicit sync.
pub trait Backend {
    type Error;
    fn close(self) -> Result<(), Self::Error>;
}

/// A unique backend owner. Explicit close and fallback share one release path.
/// No clone, comparison or shared-ownership operation is provided.
pub struct Resource<B: Backend> {
    backend: Option<B>,
}
impl<B: Backend> Resource<B> {
    pub fn new(backend: B) -> Self { Self { backend: Some(backend) } }
    pub fn get(&self) -> &B { self.backend.as_ref().expect("live resource") }
    pub fn get_mut(&mut self) -> &mut B { self.backend.as_mut().expect("live resource") }
    pub fn close(mut self) -> Result<(), B::Error> {
        self.backend.take().expect("live resource").close()
    }
}
impl<B: Backend> Drop for Resource<B> {
    fn drop(&mut self) {
        if let Some(backend) = self.backend.take() {
            let _ = backend.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Probe<'a> {
        id: usize,
        releases: &'a RefCell<Vec<usize>>,
        fail: bool,
    }
    impl Backend for Probe<'_> {
        type Error = &'static str;
        fn close(self) -> Result<(), Self::Error> {
            self.releases.borrow_mut().push(self.id);
            if self.fail { Err("close-failed") } else { Ok(()) }
        }
    }
    fn open(releases: &RefCell<Vec<usize>>, id: usize, fail: bool) -> Resource<Probe<'_>> {
        Resource::new(Probe { id, releases, fail })
    }
    fn fail_after_open(releases: &RefCell<Vec<usize>>) -> Result<(), &'static str> {
        let _owner = open(releases, 4, false);
        Err("operation-failed")?;
        Ok(())
    }
    fn return_owner(releases: &RefCell<Vec<usize>>) -> Resource<Probe<'_>> {
        let _local = open(releases, 2, false);
        open(releases, 3, false)
    }

    #[test]
    fn close_consumes_success_and_failure_without_a_second_release() {
        let releases = RefCell::new(Vec::new());
        assert_eq!(open(&releases, 0, false).close(), Ok(()));
        assert_eq!(open(&releases, 1, true).close(), Err("close-failed"));
        assert_eq!(*releases.borrow(), [0, 1]);
    }

    #[test]
    fn native_scope_return_replacement_failure_and_closure_disposal_release_once() {
        let releases = RefCell::new(Vec::new());
        {
            let mut owner = open(&releases, 0, false);
            assert_eq!(owner.get().id, 0);
            owner = open(&releases, 1, true);
            assert_eq!(*releases.borrow(), [0]);
            assert_eq!(owner.get_mut().id, 1);
        }
        let returned = return_owner(&releases);
        assert_eq!(*releases.borrow(), [0, 1, 2]);
        drop(returned);
        assert_eq!(fail_after_open(&releases), Err("operation-failed"));
        let captured = open(&releases, 5, true);
        let callback = move || { drop(captured); Err::<(), _>("callback-failed") };
        assert_eq!(callback(), Err("callback-failed"));
        let captured = open(&releases, 6, false);
        let unused = move || drop(captured);
        drop(unused);
        assert_eq!(*releases.borrow(), [0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn fallback_ignores_reported_close_failure_during_unwind() {
        let releases = RefCell::new(Vec::new());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _owner = open(&releases, 0, true);
            panic!("operation panic");
        }));
        assert!(result.is_err());
        assert_eq!(*releases.borrow(), [0]);
    }
}
