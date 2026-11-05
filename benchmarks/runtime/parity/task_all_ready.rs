mod tsrust_native_futures {pub fn block_on<F:std::future::Future>(future:F)->F::Output{tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(future)}}
async fn function0(n:i64,seed:i64)->i64{let mut sum=0;for i in 0..n{let values=tokio::join!(value(seed+i),value(seed+i+1),value(seed+i+2),value(seed+i+3));for value in [values.0,values.1,values.2,values.3]{sum=(sum+value)%1000000007;}}sum}
async fn value(value:i64)->i64{value}fn main(){}
