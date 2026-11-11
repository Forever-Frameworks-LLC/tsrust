// Separate timing/allocation builds; compare transport and scaling, not public facade speed.
include!("reference.rs");
use std::{hint::black_box,time::Instant};
#[cfg(allocation_probe)]mod allocation{
 include!("../../probes/counting-allocator.rs");
 pub fn begin()->usize{let baseline=LIVE.load(Relaxed);ALLOCS.store(0,Relaxed);REALLOCS.store(0,Relaxed);REQUESTED.store(0,Relaxed);PEAK.store(baseline,Relaxed);baseline}
 pub fn end(baseline:usize)->(usize,usize,usize,usize,i128){(ALLOCS.load(Relaxed),REALLOCS.load(Relaxed),REQUESTED.load(Relaxed),PEAK.load(Relaxed).saturating_sub(baseline),LIVE.load(Relaxed) as i128-baseline as i128)}
}
async fn sample(candidate:&str,path:&std::path::Path,reads:usize)->(u64,f64,bool){
 use tokio::io::AsyncReadExt;
 // Reader setup/open and reusable destination allocation precede the timed body.
 let mut staged=if candidate=="staged"{Some(OwnedRead::new(std::fs::File::open(path).unwrap(),64))}else{None};
 let mut direct=if candidate=="tokio"{Some(tokio::fs::File::open(path).await.unwrap())}else{None};
 let mut destination=[0u8;64];let identity=destination.as_ptr();let mut sum=0u64;let start=Instant::now();
 for _ in 0..reads{if let Some(reader)=&mut staged{reader.read_exact_into(black_box(&mut destination)).await.unwrap();}else{direct.as_mut().unwrap().read_exact(black_box(&mut destination)).await.unwrap();}for value in black_box(&destination){sum=(sum*131+u64::from(*value))%1000000007;}}
 (sum,start.elapsed().as_secs_f64()*1000.0,identity==destination.as_ptr())
}
#[cfg(profile_probe)]fn main(){
 let args=std::env::args().collect::<Vec<_>>();let candidate=&args[1];assert!(matches!(candidate.as_str(),"staged"|"tokio"));let path=std::path::Path::new(&args[2]);let reads=args[3].parse::<usize>().unwrap();
 tsrust_native_futures::block_on(async{
  for _ in 0..8{black_box(sample(candidate,path,reads).await);}
  #[cfg(allocation_probe)]{
   // Aggregate counters include setup/drop; requested bytes and peak are reported
   // explicitly so a reusable buffer cannot conceal per-operation task allocations.
   // sample has returned: both reader owners and destination have been dropped.
   // Retained is signed because runtime-cache release can lower the live baseline.
   let baseline=allocation::begin();let(checksum,_,reused)=sample(candidate,path,reads).await;let(allocations,reallocations,requested,peak,retained)=allocation::end(baseline);
   println!("{{\"candidate\":{candidate:?},\"reads\":{reads},\"checksum\":{checksum},\"reusedDestination\":{reused},\"durations\":[],\"allocations\":{allocations},\"reallocations\":{reallocations},\"requested\":{requested},\"peak\":{peak},\"retained\":{retained}}}");
  }
  #[cfg(not(allocation_probe))]{let mut durations=Vec::with_capacity(31);let mut expected=None;let mut reused=true;for _ in 0..31{let(checksum,duration,identity)=sample(candidate,path,reads).await;if let Some(old)=expected{assert_eq!(old,checksum);}expected=Some(checksum);reused&=identity;durations.push(duration);}println!("{{\"candidate\":{candidate:?},\"reads\":{reads},\"checksum\":{},\"reusedDestination\":{reused},\"durations\":{durations:?}}}",expected.unwrap());}
 });
}
