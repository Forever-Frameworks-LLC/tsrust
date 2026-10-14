//! Native ownership witness, not compiler or production-runner acceptance.
//! Compile directly with `rustc --edition=2024 --test` as well as through Cargo.
#![forbid(unsafe_code)]

use std::panic::{AssertUnwindSafe, catch_unwind};

trait ProgramFamily {
    type State<'p>;
}

// The lifetime belongs to each invocation, never to the registry type.
type TestCallback<P> = Box<
    dyn for<'p> FnMut(
        &mut <P as ProgramFamily>::State<'p>,
        &mut TestRegistry<P>,
    ) -> Result<(), String>,
>;

struct OwnedState<S>(std::marker::PhantomData<S>);
impl<S> ProgramFamily for OwnedState<S> {
    type State<'p> = S;
}

struct TestRegistry<S: ProgramFamily> {
    callbacks: Vec<TestCallback<S>>,
    running: bool,
}

impl<S: ProgramFamily> Default for TestRegistry<S> {
    fn default() -> Self {
        Self {
            callbacks: Vec::new(),
            running: false,
        }
    }
}

impl<S: ProgramFamily> TestRegistry<S> {
    fn register(&mut self, callback: TestCallback<S>) -> Result<(), String> {
        if self.running {
            return Err("registration during execution".into());
        }
        self.callbacks.push(callback);
        Ok(())
    }

    fn run(&mut self, state: &mut S::State<'_>) -> Result<Vec<Result<(), String>>, String> {
        if self.running {
            return Err("nested execution".into());
        }
        // No callback remains borrowed from the registry during invocation.
        let callbacks = std::mem::take(&mut self.callbacks);
        let mut results = Vec::new();
        for mut callback in callbacks {
            self.running = true;
            let result = catch_unwind(AssertUnwindSafe(|| callback(state, self)));
            self.running = false;
            results.push(result.unwrap_or_else(|_| Err("panic".into())));
        }
        Ok(results)
    }
}

#[test]
fn successive_callbacks_share_writable_program_storage() {
    let mut value = 0;
    let mut registry = TestRegistry::<OwnedState<usize>>::default();
    registry
        .register(Box::new(|state, _| {
            *state += 1;
            Ok(())
        }))
        .unwrap();
    registry
        .register(Box::new(|state, _| {
            assert_eq!(*state, 1);
            Ok(())
        }))
        .unwrap();
    registry
        .register(Box::new(|state, _| {
            *state += 2;
            Ok(())
        }))
        .unwrap();
    assert_eq!(registry.run(&mut value).unwrap(), vec![Ok(()); 3]);
    assert_eq!(value, 3);
    assert!(registry.run(&mut value).unwrap().is_empty());
}

#[test]
fn program_references_are_inputs_not_retained_captures() {
    struct Program<'p> {
        message: &'p str,
        count: &'p mut usize,
    }
    fn read_library_state(state: &Program<'_>) -> usize {
        state.message.len()
    }

    let message = String::from("hello");
    let mut count = 0;
    struct Family;
    impl ProgramFamily for Family {
        type State<'p> = Program<'p>;
    }
    let mut registry = TestRegistry::<Family>::default();
    registry
        .register(Box::new(|state, _| {
            *state.count = read_library_state(state);
            Ok(())
        }))
        .unwrap();
    registry
        .register(Box::new(|state, _| {
            assert_eq!(*state.count, 5);
            Ok(())
        }))
        .unwrap();
    let results = registry
        .run(&mut Program {
            message: &message,
            count: &mut count,
        })
        .unwrap();
    assert_eq!(results, vec![Ok(()); 2]);
    assert_eq!(count, 5);
    count = 7;
    registry
        .register(Box::new(|state, _| {
            assert_eq!(*state.count, 7);
            *state.count += 1;
            Ok(())
        }))
        .unwrap();
    assert_eq!(
        registry
            .run(&mut Program {
                message: &message,
                count: &mut count
            })
            .unwrap(),
        vec![Ok(())]
    );
    assert_eq!(count, 8);
    drop(registry);
}

#[test]
fn owned_captures_remain_owned_while_program_access_is_fresh() {
    let owned = String::from("owned callback data");
    let mut registry = TestRegistry::<OwnedState<usize>>::default();
    registry
        .register(Box::new(move |state, _| {
            *state += owned.len();
            Ok(())
        }))
        .unwrap();
    let mut count = 0;
    assert_eq!(registry.run(&mut count).unwrap(), vec![Ok(())]);
    assert_eq!(count, 19);
}

#[test]
fn registration_and_execution_guards_are_recoverable() {
    let mut registry = TestRegistry::<OwnedState<usize>>::default();
    registry
        .register(Box::new(|state, registry| {
            assert_eq!(
                registry.register(Box::new(|_, _| panic!("must not run"))),
                Err("registration during execution".into())
            );
            assert_eq!(registry.run(state), Err("nested execution".into()));
            *state += 1;
            Ok(())
        }))
        .unwrap();
    registry
        .register(Box::new(|state, _| {
            *state += 1;
            Ok(())
        }))
        .unwrap();
    let mut count = 0;
    assert_eq!(registry.run(&mut count).unwrap(), vec![Ok(()); 2]);
    assert_eq!(count, 2);
    assert!(!registry.running);
    assert!(registry.callbacks.is_empty());
}

#[test]
fn failures_restore_running_state_and_allow_later_callbacks() {
    let mut registry = TestRegistry::<OwnedState<usize>>::default();
    registry
        .register(Box::new(|_, _| Err("typed failure".into())))
        .unwrap();
    registry
        .register(Box::new(|_, _| panic!("native failure")))
        .unwrap();
    registry
        .register(Box::new(|state, _| {
            *state += 1;
            Ok(())
        }))
        .unwrap();
    let mut count = 0;
    assert_eq!(
        registry.run(&mut count).unwrap(),
        vec![Err("typed failure".into()), Err("panic".into()), Ok(())]
    );
    assert_eq!(count, 1);
    registry
        .register(Box::new(|state, _| {
            *state += 1;
            Ok(())
        }))
        .unwrap();
    assert_eq!(registry.run(&mut count).unwrap(), vec![Ok(())]);
    assert_eq!(count, 2);
}

#[test]
fn unrun_callback_environments_drop_before_program_storage() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct OrderedDrop(usize);
    impl Drop for OrderedDrop {
        fn drop(&mut self) {
            assert_eq!(NEXT.fetch_add(1, Ordering::SeqCst), self.0);
        }
    }
    {
        let _program = OrderedDrop(1);
        let mut registry = TestRegistry::<OwnedState<()>>::default();
        let capture = OrderedDrop(0);
        registry
            .register(Box::new(move |_, _| {
                std::hint::black_box(&capture);
                Ok(())
            }))
            .unwrap();
    }
    assert_eq!(NEXT.load(Ordering::SeqCst), 2);
}

#[test]
fn borrowed_locals_cannot_enter_owned_callback_environments() {
    let source = format!(
        "{}\nfn main() {{ let local = String::from(\"short lived\"); let mut registry = TestRegistry::<OwnedState<usize>>::default(); registry.register(Box::new(|state, _| {{ *state = local.len(); Ok(()) }})).unwrap(); drop(registry); }}",
        include_str!("program_scoped_test_registry_design.rs")
    );
    reject_native(&source, "E0597");
}

#[test]
fn invocation_borrow_cannot_escape_into_callback_storage() {
    let source = format!(
        "{}\nfn main() {{ let mut retained: Option<&mut usize> = None; let mut registry = TestRegistry::<OwnedState<usize>>::default(); registry.register(Box::new(move |state, _| {{ retained = Some(state); std::hint::black_box(&retained); Ok(()) }})).unwrap(); }}",
        include_str!("program_scoped_test_registry_design.rs")
    );
    reject_native(&source, "E0521");
}

fn reject_native(source: &str, code: &str) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let directory = Scratch(std::env::temp_dir().join(format!(
        "tsrust-program-registry-rejection-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir(&directory.0).unwrap();
    let path = directory.0.join("witness.rs");
    std::fs::write(&path, source).unwrap();
    let output = std::process::Command::new("rustc")
        .args(["--edition=2024", "--emit=metadata"])
        .arg(&path)
        .arg("-o")
        .arg(directory.0.join("witness.rmeta"))
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "borrowed storage unexpectedly escaped"
    );
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(errors.contains(&format!("error[{code}]")), "{errors}");
}
