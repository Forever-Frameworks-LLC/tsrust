mod tsrust_native_threads {
    pub struct Thread<T, E>(Option<std::thread::JoinHandle<Result<T, E>>>);
    impl<T, E> Drop for Thread<T, E> {
        fn drop(&mut self) {
            if let Some(worker) = self.0.take() { let _ = worker.join(); }
        }
    }
    pub enum JoinError<E> { Worker(E), Panic }

    pub fn spawn<T: Send + 'static, E: Send + 'static>(work: impl FnOnce() -> Result<T, E> + Send + 'static) -> Result<Thread<T, E>, std::io::Error> {
        std::thread::Builder::new().spawn(work).map(|worker| Thread(Some(worker)))
    }
    pub fn join<T, E>(mut thread: Thread<T, E>) -> Result<T, JoinError<E>> {
        match thread.0.take().expect("owned worker").join() {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(JoinError::Worker(error)),
            Err(_) => Err(JoinError::Panic),
        }
    }
    pub fn detach<T, E>(mut thread: Thread<T, E>) { drop(thread.0.take()); }
}
