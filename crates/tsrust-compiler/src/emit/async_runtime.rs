trait TaskCallback<A, R> {
    async fn invoke_once(self, args: A) -> R;
    async fn invoke_once_with_count(self, args: A, _count: usize) -> R where Self: Sized {
        self.invoke_once(args).await
    }
}
fn tsrust_async_block_on<F: std::future::Future>(future: F) -> F::Output {
    struct Notify(std::thread::Thread);
    impl std::task::Wake for Notify {
        fn wake(self: std::sync::Arc<Self>) { self.0.unpark(); }
        fn wake_by_ref(self: &std::sync::Arc<Self>) { self.0.unpark(); }
    }
    let waker = std::task::Waker::from(std::sync::Arc::new(Notify(std::thread::current())));
    let mut context = std::task::Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            std::task::Poll::Ready(value) => return value,
            std::task::Poll::Pending => std::thread::park(),
        }
    }
}
