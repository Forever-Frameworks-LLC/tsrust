// The async runtime is constructed once, outside every measured sample.
fn main(){
 let arguments:Vec<std::string::String>=std::env::args().collect();
 let iterations:i64=arguments[1].parse().unwrap();let seed:i64=arguments[2].parse().unwrap();
 let warmups:usize=arguments[3].parse().unwrap();let samples:usize=arguments[4].parse().unwrap();
 tsrust_native_futures::block_on(async {
  let mut expected=None;let mut validate=|value:i64|{assert!(value.unsigned_abs()<=9_007_199_254_740_991);if let Some(old)=expected{assert_eq!(old,value);}expected=Some(value);};
  for _ in 0..warmups{validate(benchmark_async_workload(std::hint::black_box(iterations),std::hint::black_box(seed)).await);}
  let mut durations=Vec::new();for _ in 0..samples{let start=std::time::Instant::now();let value=std::hint::black_box(benchmark_async_workload(std::hint::black_box(iterations),std::hint::black_box(seed)).await);durations.push(start.elapsed().as_secs_f64()*1000.0);validate(value);}
  println!("{{\"checksum\":{},\"durations\":{:?}}}",expected.unwrap(),durations);
 });
}
