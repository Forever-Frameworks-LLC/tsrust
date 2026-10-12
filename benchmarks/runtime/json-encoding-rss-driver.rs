fn main(){
 let arguments:Vec<std::string::String>=std::env::args().collect();
 let n: i64 = arguments[1].parse().unwrap();
 let value=std::hint::black_box(function0(std::hint::black_box(n), std::hint::black_box(7)));
 let checksum = probe_checksum(&value);
 println!("{{\"checksum\":{}}}",checksum);
 std::hint::black_box(&value);
}
