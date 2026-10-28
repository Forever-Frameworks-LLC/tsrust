// A registered environment owns its captures, but borrows program storage only
// while a callback runs. The future cannot escape that invocation or require
// program state to be Send, cloned, or retained for the registry's lifetime.
trait TsrTestAsyncCallback {
    fn invoke_async<'a,'p:'a>(
        &'a mut self,
        program:&'a mut TsrTestContext<'p>,
        registry:&'a mut TsrTestRegistry,
    )->std::pin::Pin<Box<dyn std::future::Future<Output=Result<(),std::string::String>>+'a>>;
}
fn tsrust_testing_async_callback<C:TsrTestAsyncCallback+'static>(mut callback:C)->TsrTestCallback {
    Box::new(move |program,registry|tsrust_native_futures::testing_await(callback.invoke_async(program,registry)))
}
fn tsrust_testing_run_in_scope<R>(body:impl FnOnce()->R)->R {
    tsrust_native_futures::testing_scope(body)
}
