trait TaskCallback<A, R> {
    async fn invoke_once(self, args: A) -> R;
    async fn invoke_once_with_count(self, args: A, _count: usize) -> R where Self: Sized {
        self.invoke_once(args).await
    }
}
fn tsrust_async_block_on<F: std::future::Future>(future: F) -> F::Output {
    tsrust_native_futures::block_on(future)
}
// An asynchronous program runs its whole entrypoint on a thread with the common
// 8 MiB main-thread stack: Windows gives the main thread only 1 MiB, which an
// unoptimized async main can exceed. The name keeps panic reports unchanged.
#[allow(dead_code)]
fn tsrust_main_thread<R: Send>(entry: fn() -> R) -> R {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new().name("main".into()).stack_size(8 << 20)
            .spawn_scoped(scope, entry).expect("create the program's main thread");
        worker.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}
