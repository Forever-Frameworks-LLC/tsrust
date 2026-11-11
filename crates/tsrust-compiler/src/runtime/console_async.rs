tokio::task_local! {static TSR_CONSOLE_ASYNC_TEST:TsrConsoleTestContext;}
pub fn tsr_console_test_future<F:std::future::Future>(future:F)->impl std::future::Future<Output=F::Output>{
    // Tokio enters the context on every poll and during the future's drop.
    TSR_CONSOLE_ASYNC_TEST.scope(tsr_console_test_context(),future)
}
