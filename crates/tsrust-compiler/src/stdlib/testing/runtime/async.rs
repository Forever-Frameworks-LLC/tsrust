// Registered callbacks lend their captures/program storage only for invocation.
trait TsrTestAsyncCallback {
 type Output;
 fn invoke_async<'a,'p:'a>(&'a mut self,program:&'a mut TsrTestContext<'p>,registry:&'a mut TsrTestRegistry)->std::pin::Pin<Box<dyn std::future::Future<Output=Self::Output>+'a>>;
}
// Shared by emitted ProgramCallback adapters and direct runtime witnesses. Keep
// typed callback results intact; panic/timeout are runner failures, never checked
// user rejection payloads. Drain after the borrowed future has been dropped.
fn tsrust_testing_await<F:std::future::Future>(future:F)->F::Output {
 let deadline=if TSR_TEST_BODY.with(|value|value.get()){
  std::env::var("TSRUST_TEST_TIMEOUT_MS").ok().map(|value|{let milliseconds=value.parse::<u64>().unwrap_or_else(|_|panic!("Invalid explicit async test timeout"));tokio::time::Instant::now().checked_add(std::time::Duration::from_millis(milliseconds)).unwrap_or_else(||panic!("Explicit async test timeout exceeds backend Instant range"))})
 }else{None};
 tsrust_native_futures::testing_await(async{
  let scope=tsrust_native_futures::Scope::group();
  let result=tsrust_native_futures::in_scope(scope.clone(),async move{
   let mut future=std::pin::pin!(future);
   let checked=std::future::poll_fn(|context|match std::panic::catch_unwind(std::panic::AssertUnwindSafe(||std::future::Future::poll(future.as_mut(),context))){Ok(value)=>value.map(Ok),Err(error)=>std::task::Poll::Ready(Err(error))});
   if let Some(deadline)=deadline{tokio::time::timeout_at(deadline,checked).await.ok()}else{Some(checked.await)}
  }).await;
  scope.drain(!matches!(&result,Some(Ok(_)))).await;
  match result{Some(Ok(value))=>value,Some(Err(error))=>std::panic::resume_unwind(error),None=>panic!("TestTimeout: explicit async test deadline elapsed")}
 })
}
fn tsrust_testing_async_callback<C:TsrTestAsyncCallback<Output=Result<(),std::string::String>>+'static>(mut callback:C)->TsrTestCallback{Box::new(move|program,registry|tsrust_testing_await(callback.invoke_async(program,registry)))}
fn tsrust_testing_run_in_scope<R>(body:impl FnOnce()->R)->R{tsrust_native_futures::testing_scope(body)}
