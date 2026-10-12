fn main(){
 let arguments:Vec<std::string::String>=std::env::args().collect();
 let n: i64 = arguments[1].parse().unwrap();
 let encode=||function0(std::hint::black_box(n), std::hint::black_box(7));
 let cold=std::time::Instant::now();
 let value=std::hint::black_box(encode());
 let cold_ms=cold.elapsed().as_secs_f64()*1000.0;
 let checksum = probe_checksum(&value);
 drop(value);
 let mut durations=Vec::new();
 for batch in 0..39 {
  let start=std::time::Instant::now();
  let mut last=None;
  for _ in 0..100 {last=Some(std::hint::black_box(encode()));}
  let elapsed=start.elapsed().as_secs_f64()*1000.0;
  let value=last.unwrap();
  assert_eq!(probe_checksum(&value),checksum);
  assert_eq!(&*value,arguments[2].as_str());
  drop(value);
  if batch>=8 {durations.push(elapsed);}
 }
 println!("{{\"checksum\":{},\"coldMs\":{},\"durations\":{:?},\"iterations\":100,\"warmups\":8}}",checksum,cold_ms,durations);
}
