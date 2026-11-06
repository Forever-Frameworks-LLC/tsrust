#![allow(dead_code)]
include!("reference.rs");
#[cfg(candidate_boxed)]const CANDIDATE:&str="boxed";
#[cfg(candidate_enum)]const CANDIDATE:&str="finite-enum";
#[cfg(not(any(candidate_boxed,candidate_enum)))]const CANDIDATE:&str="static";
#[cfg(allocation_probe)]mod allocation{
 include!("../../probes/counting-allocator.rs");
 pub fn begin()->usize{let baseline=LIVE.load(Relaxed);ALLOCS.store(0,Relaxed);REALLOCS.store(0,Relaxed);REQUESTED.store(0,Relaxed);PEAK.store(baseline,Relaxed);baseline}
 pub fn end(baseline:usize)->(usize,usize,usize,i64,i64){(ALLOCS.load(Relaxed),REALLOCS.load(Relaxed),REQUESTED.load(Relaxed),LIVE.load(Relaxed)as i64-baseline as i64,PEAK.load(Relaxed)as i64-baseline as i64)}
}
fn main(){
 if std::env::args().any(|arg|arg=="--check"){
  correctness();let static_chain=chain_a(Vec::new());let enum_chain=joined(Vec::new(),true);let boxed_chain=boxed(Vec::new(),true);
  println!("{{\"kind\":\"representation\",\"staticBytes\":{},\"enumBytes\":{},\"boxedHandleBytes\":{},\"boxedAllocationNotIncluded\":true}}",std::mem::size_of_val(&static_chain),std::mem::size_of_val(&enum_chain),std::mem::size_of_val(&boxed_chain));
  for name in ["reduce","some","take-array","partial-drop","flat-array","flat-iterator"]{if CANDIDATE!="static"&&name.starts_with("flat-"){continue;}for(size,seed)in[(0,0),(1,7),(2,19),(17,3),(17,4),(4096,7)]{let value=kernel(name,size,seed);println!("{{\"kind\":\"correctness\",\"candidate\":\"{CANDIDATE}\",\"kernel\":\"{name}\",\"size\":{size},\"seed\":{seed},\"checksum\":{value}}}");}}
  return;
 }
 for name in ["reduce","some","take-array","partial-drop","flat-array","flat-iterator"]{
  if CANDIDATE!="static"&&name.starts_with("flat-"){continue;}
  for size in [0,1,64,4096]{for seed in [7_i64,8]{
   for _ in 0..8{std::hint::black_box(kernel(std::hint::black_box(name),std::hint::black_box(size),std::hint::black_box(seed)));}
   #[cfg(allocation_probe)]{let baseline=allocation::begin();let checksum=std::hint::black_box(kernel(std::hint::black_box(name),std::hint::black_box(size),std::hint::black_box(seed)));let(allocations,reallocations,requested,retained,peak)=allocation::end(baseline);println!("{{\"kind\":\"allocation\",\"candidate\":\"{CANDIDATE}\",\"kernel\":\"{name}\",\"size\":{size},\"seed\":{seed},\"checksum\":{checksum},\"allocations\":{allocations},\"reallocations\":{reallocations},\"requested\":{requested},\"retained\":{retained},\"peak\":{peak}}}");}
   #[cfg(not(allocation_probe))]{let mut durations=Vec::with_capacity(31);let mut checksums=Vec::with_capacity(31);for _ in 0..31{let start=std::time::Instant::now();let mut checksum=0;for _ in 0..16{checksum+=std::hint::black_box(kernel(std::hint::black_box(name),std::hint::black_box(size),std::hint::black_box(seed)));}durations.push(start.elapsed().as_nanos());checksums.push(checksum);}println!("{{\"kind\":\"timing\",\"candidate\":\"{CANDIDATE}\",\"kernel\":\"{name}\",\"size\":{size},\"seed\":{seed},\"queries\":16,\"durations_ns\":{durations:?},\"checksums\":{checksums:?}}}");}
  }}
 }
}
