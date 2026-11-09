#![allow(dead_code)]
#![cfg_attr(not(feature="allocation-probe"),forbid(unsafe_code))]
#[cfg(all(feature="miniz",feature="zlib-rs"))]compile_error!("Select exactly one compression backend");
#[cfg(not(any(feature="miniz",feature="zlib-rs")))]compile_error!("Select one compression backend");
include!("reference.rs");
#[cfg(feature="allocation-probe")]include!("../../probes/counting-allocator.rs");
#[cfg(feature="miniz")]const BACKEND:&str="miniz";
#[cfg(feature="zlib-rs")]const BACKEND:&str="zlib-rs";
fn operation(format:Format,strategy:Strategy,encode:bool,source:&[u8],level:i32,chunk:usize)->io::Result<Vec<u8>>{
 if encode{compress(format,source,level,chunk,strategy)}else{match strategy{Strategy::Reader=>decompress(format,source,chunk,None),Strategy::Writer=>decompress_writer(format,source,chunk,None)}}
}
fn main(){
 let args=std::env::args().collect::<Vec<_>>();
 if args.get(1).is_some_and(|arg|arg=="--check"){correctness();writer_contracts();short_io_contracts();return;}
 if args.get(1).is_some_and(|arg|arg=="--decode"){
  let format=Format::parse(&args[2]);let bytes=unhex(&args[3]);let limit=args.get(4).map(|limit|limit.parse::<usize>().unwrap());
  match decompress(format,&bytes,1,limit){Ok(output)=>println!("{{\"ok\":true,\"hex\":\"{}\"}}",hex(&output)),Err(_)=>println!("{{\"ok\":false}}")}return;
 }
 let fixtures=args.get(1).is_some_and(|arg|arg=="--fixtures");
 let group=if args.get(1).is_some_and(|arg|arg=="--group"){Some(args.get(2).expect("group value").as_str())}else{None};
 if let Some(group)=group{assert!(["gzip.compress","gzip.decompress","zlib.compress","zlib.decompress","raw.compress","raw.decompress"].contains(&group));}
 for format in[Format::Gzip,Format::Zlib,Format::Raw]{for strategy in[Strategy::Writer,Strategy::Reader]{for encode in[true,false]{for random in[false,true]{for selected_level in[0,1,-1,9]{for chunk in[64,8192,16384]{
  if fixtures&&!encode{continue;}
  if !fixtures&&group.is_some_and(|group|group!=format!("{}.{}",format.name(),if encode{"compress"}else{"decompress"})){continue;}
  let original=input(16384,random,7);
  let compressed=if encode{Vec::new()}else{
   let directory=std::env::var("TSRUST_COMPRESSION_CORPUS").expect("immutable common decode corpus");
   std::fs::read(std::path::Path::new(&directory).join(format!("{}-{}-{selected_level}.bin",format.name(),if random{"random"}else{"compressible"}))).unwrap()
  };
  let source=if encode{&original}else{&compressed};
  let expected=operation(format,strategy,encode,source,selected_level,chunk).unwrap();let length=expected.len();let digest=checksum(&expected);
  if encode{assert_eq!(decompress(format,&expected,1,None).unwrap(),original);}else{assert_eq!(expected,original);}
  let label=format!("\"backend\":\"{BACKEND}\",\"format\":\"{}\",\"strategy\":\"{}\",\"operation\":\"{}\",\"pattern\":\"{}\",\"level\":{selected_level},\"chunk\":{chunk},\"inputBytes\":{},\"inputChecksum\":{},\"outputBytes\":{length},\"checksum\":{digest}",format.name(),strategy.name(),if encode{"compress"}else{"decompress"},if random{"random"}else{"compressible"},source.len(),checksum(source));
  if fixtures{if encode{println!("{{\"kind\":\"fixture\",{label},\"hex\":\"{}\"}}",hex(&expected));}continue;}
  drop(expected);
  for _ in 0..8{std::hint::black_box(operation(format,strategy,encode,source,selected_level,chunk).unwrap());}
  #[cfg(feature="allocation-probe")]{
   let baseline=LIVE.load(std::sync::atomic::Ordering::Relaxed);ALLOCS.store(0,std::sync::atomic::Ordering::Relaxed);REALLOCS.store(0,std::sync::atomic::Ordering::Relaxed);REQUESTED.store(0,std::sync::atomic::Ordering::Relaxed);PEAK.store(baseline,std::sync::atomic::Ordering::Relaxed);
   let output=operation(format,strategy,encode,source,selected_level,chunk).unwrap();assert_eq!(output.len(),length);assert_eq!(checksum(&output),digest);
   let allocations=ALLOCS.load(std::sync::atomic::Ordering::Relaxed);let reallocations=REALLOCS.load(std::sync::atomic::Ordering::Relaxed);let requested=REQUESTED.load(std::sync::atomic::Ordering::Relaxed);let peak=PEAK.load(std::sync::atomic::Ordering::Relaxed).saturating_sub(baseline);drop(output);let retained=LIVE.load(std::sync::atomic::Ordering::Relaxed).saturating_sub(baseline);
   println!("{{\"kind\":\"allocation\",{label},\"allocations\":{allocations},\"reallocations\":{reallocations},\"requested\":{requested},\"peak\":{peak},\"retained\":{retained}}} ");
  }
  #[cfg(not(feature="allocation-probe"))]{
   let mut samples=Vec::with_capacity(31);let mut checksums=Vec::with_capacity(31);let mut output_lengths=Vec::with_capacity(31);
   for _ in 0..31{let start=std::time::Instant::now();let output=std::hint::black_box(operation(format,strategy,encode,source,selected_level,chunk).unwrap());let observed_length=output.len();let observed_checksum=checksum(&output);assert_eq!(observed_length,length);assert_eq!(observed_checksum,digest);drop(output);let elapsed=start.elapsed().as_nanos()as u64;samples.push(elapsed);checksums.push(observed_checksum);output_lengths.push(observed_length);}
   println!("{{\"kind\":\"timing\",{label},\"durations_ns\":{samples:?},\"checksums\":{checksums:?},\"outputLengths\":{output_lengths:?}}}");
  }
 }}}}}}
}
