// Shared untimed child executable for all public process benchmark engines.
// Child dispatch has no async runtime, allocator counter or measured-driver work.
use std::io::{Read,Write};
fn main(){let args:Vec<String>=std::env::args().collect();match args.get(1).map(String::as_str).unwrap_or("silent"){
 "silent"=>{},
 "echo"=>{let mut input=std::io::stdin().lock();let mut output=std::io::stdout().lock();std::io::copy(&mut input,&mut output).unwrap();output.flush().unwrap();},
 "eof"=>{let mut bytes=Vec::new();std::io::stdin().read_to_end(&mut bytes).unwrap();std::io::stdout().write_all(&bytes).unwrap();},
 "flood"=>{let size:usize=args[2].parse().unwrap();let seed:u64=args[3].parse().unwrap();let bytes:Vec<u8>=(0..size).map(|index|((seed+index as u64)%251) as u8).collect();let other:Vec<u8>=(0..size).map(|index|((seed+index as u64*3+17)%251) as u8).collect();let errors=std::thread::spawn(move||std::io::stderr().write_all(&other).unwrap());std::io::stdout().write_all(&bytes).unwrap();errors.join().unwrap();},
 "sleep"=>std::thread::sleep(std::time::Duration::from_secs(60)),
 "exit"=>std::process::exit(args[2].parse().unwrap()),
 "context"=>{println!("{}",std::env::current_dir().unwrap().display());println!("{}",std::env::var("TSRUST_PROCESS_CHILD").unwrap_or_default());println!("{}",std::env::var("TSRUST_PROCESS_PARENT").unwrap_or_default());for argument in &args[2..]{for byte in argument.as_bytes(){print!("{byte:02x}");}println!();}},
 mode=>panic!("unknown child helper mode: {mode}"),
}}
