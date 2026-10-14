mod tsrust_native_threads {
    pub struct Thread<T, E>(std::thread::JoinHandle<Result<T, E>>);
    pub enum JoinError<E> { Worker(E), Panic }

    pub fn spawn<T: Send + 'static, E: Send + 'static>(work: impl FnOnce() -> Result<T, E> + Send + 'static) -> Result<Thread<T, E>, std::io::Error> {
        std::thread::Builder::new().spawn(work).map(Thread)
    }
    pub fn join<T, E>(thread: Thread<T, E>) -> Result<T, JoinError<E>> {
        match thread.0.join() {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(JoinError::Worker(error)),
            Err(_) => Err(JoinError::Panic),
        }
    }
    pub fn detach<T, E>(thread: Thread<T, E>) { drop(thread); }
}
