// New protocol only; standard raw f64 observation after the workload timer.
fn main(){
 let arguments:Vec<std::string::String>=std::env::args().collect();
 let iterations:i64=arguments[1].parse().unwrap();let seed:i64=arguments[2].parse().unwrap();let warmups:usize=arguments[3].parse().unwrap();let samples:usize=arguments[4].parse().unwrap();
 let mut warmup_bits=Vec::with_capacity(warmups);let mut actual_bits=Vec::with_capacity(samples);let mut durations=Vec::with_capacity(samples);
 for _ in 0..warmups{let value=std::hint::black_box(function0(std::hint::black_box(iterations),std::hint::black_box(seed)));warmup_bits.push(format!("{:016x}",value.to_bits()));}
 for _ in 0..samples{let start=std::time::Instant::now();let value=std::hint::black_box(function0(std::hint::black_box(iterations),std::hint::black_box(seed)));durations.push(start.elapsed().as_secs_f64()*1000.0);actual_bits.push(format!("{:016x}",value.to_bits()));}
 // Vec<String> Debug is valid JSON for these restricted lowercase hex strings.
 println!("{{\"protocol\":\"mpfr-directed-f64-scalar-v1\",\"warmupBits\":{:?},\"actualBits\":{:?},\"durations\":{:?}}}",warmup_bits,actual_bits,durations);
}
