trait TaskCallback<A, R> {
    async fn invoke_once(self, args: A) -> R;
    async fn invoke_once_with_count(self, args: A, _count: usize) -> R where Self: Sized {
        self.invoke_once(args).await
    }
}
fn tsrust_async_block_on<F: std::future::Future>(future: F) -> F::Output {
    tsrust_native_futures::block_on(future)
}
