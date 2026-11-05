include!("reference.rs");
use std::{hint::black_box,time::Instant};
#[cfg(feature="allocation-probe")]
mod allocation {
    include!("../../probes/counting-allocator.rs");
    pub fn begin()->usize{let baseline=LIVE.load(Relaxed);ALLOCS.store(0,Relaxed);REALLOCS.store(0,Relaxed);REQUESTED.store(0,Relaxed);PEAK.store(baseline,Relaxed);baseline}
    pub fn end(baseline:usize)->(usize,usize,usize,i64,i64){(ALLOCS.load(Relaxed),REALLOCS.load(Relaxed),REQUESTED.load(Relaxed),LIVE.load(Relaxed)as i64-baseline as i64,PEAK.load(Relaxed)as i64-baseline as i64)}
}
struct Context { pattern:&'static str,text:String,compiled:Compiled,locations:regex::CaptureLocations }
impl Context {fn new(scenario:&str,size:usize)->Self{let(pattern,text)=match scenario{"unicode"=>("(é)(🦀)?","é🦀 é ".repeat(size)),"no-match"=>("([a-z]+)=([0-9]+)","!".repeat(size)),"scaling"=>("a+z|a","a".repeat(size)),_=>("([a-z]+)=([0-9]+)","key=123 other=4 ".repeat(size))};let compiled=Compiled::compile(pattern).unwrap();let locations=compiled.inner.capture_locations();Self{pattern,text,compiled,locations}}}
fn weight(value:&Range)->u64{u64::try_from(value.end-value.start+1).unwrap()}
fn kernel(name:&str,context:&mut Context)->u64{
 let text=black_box(context.text.as_str());let compiled=black_box(&context.compiled);let inner=&compiled.inner;
 match name{
  "compile_direct"=>black_box(Regex::new(black_box(context.pattern)).unwrap()).captures_len()as u64,
  "compile_adapter"=>black_box(Compiled::compile(black_box(context.pattern)).unwrap()).inner.captures_len()as u64,
  "is_match_direct"=>u64::from(inner.is_match(text)),"is_match_adapter"=>u64::from(compiled.is_match(text)),
  "find_direct"=>inner.find(text).map_or(0,|value|(value.len()+1)as u64),"find_adapter"=>compiled.find(text).as_ref().map_or(0,weight),
  "find_all_direct"|"scale_find_all_direct"=>inner.find_iter(text).map(|value|(value.len()+1)as u64).sum(),
  "find_all_ranges"|"scale_find_all_ranges"=>black_box(compiled.find_all(text)).iter().map(weight).sum(),
  "find_all_strings"=>black_box(inner.find_iter(text).map(|value|value.as_str().to_owned()).collect::<Vec<_>>()).iter().map(|value|(value.len()+1)as u64).sum(),
  "captures_direct"=>inner.captures(text).map_or(0,|value|value.iter().flatten().map(|value|(value.len()+1)as u64).sum()),
  "captures_locations"=>{if inner.captures_read(&mut context.locations,text).is_none(){0}else{(0..context.locations.len()).filter_map(|index|context.locations.get(index)).map(|(start,end)|(end-start+1)as u64).sum()}},
  "captures_adapter"=>black_box(compiled.captures(text)).map_or(0,|values|values.iter().flatten().map(weight).sum()),
  "captures_all_direct"=>inner.captures_iter(text).map(|value|value.iter().flatten().map(|value|(value.len()+1)as u64).sum::<u64>()).sum(),
  "captures_all_adapter"=>black_box(compiled.captures_all(text)).iter().flat_map(|values|values.iter().flatten()).map(weight).sum(),
  "replace_direct"=>black_box(inner.replace(text,"${2}:${1}")).len()as u64,"replace_adapter"=>black_box(compiled.replace(text,"${2}:${1}")).len()as u64,
  "replace_all_direct"=>black_box(inner.replace_all(text,"${2}:${1}")).len()as u64,"replace_all_adapter"=>black_box(compiled.replace_all(text,"${2}:${1}")).len()as u64,
  "split_direct"=>inner.split(text).map(|value|(value.len()+1)as u64).sum(),"split_ranges"=>black_box(compiled.split(text)).iter().map(weight).sum(),
  _=>panic!("unknown kernel"),
 }
}
fn measure(name:&str,scenario:&str,size:usize){
 let mut context=Context::new(scenario,size);
 #[cfg(feature="allocation-probe")]{for _ in 0..8{black_box(kernel(name,&mut context));}let baseline=allocation::begin();let checksum=black_box(kernel(name,&mut context));let(allocations,reallocations,requested,retained,peak)=allocation::end(baseline);println!("{{\"kind\":\"allocations\",\"kernel\":{name:?},\"scenario\":{scenario:?},\"size\":{size},\"checksum\":{checksum},\"allocations\":{allocations},\"reallocations\":{reallocations},\"requested\":{requested},\"retained\":{retained},\"peak\":{peak}}}");}
 #[cfg(not(feature="allocation-probe"))]{let rounds=if name.starts_with("compile_"){4}else if name.starts_with("scale_find_all_"){4}else{32};let mut durations=Vec::new();let mut checksums=Vec::new();for sample in 0..39{let started=Instant::now();let mut checksum=0;for _ in 0..rounds{checksum+=black_box(kernel(name,&mut context));}let elapsed=started.elapsed().as_nanos();if sample>=8{durations.push(elapsed);checksums.push(checksum);}}println!("{{\"kind\":\"timing\",\"kernel\":{name:?},\"scenario\":{scenario:?},\"size\":{size},\"rounds\":{rounds},\"durations_ns\":{durations:?},\"checksums\":{checksums:?}}}");}
}
fn main(){if std::env::args().any(|value|value=="--check"){correctness();return;}for name in ["compile_direct","compile_adapter","is_match_direct","is_match_adapter","find_direct","find_adapter","find_all_direct","find_all_ranges","find_all_strings","captures_direct","captures_locations","captures_adapter","captures_all_direct","captures_all_adapter","replace_direct","replace_adapter","replace_all_direct","replace_all_adapter","split_direct","split_ranges"]{for scenario in ["ascii","unicode","no-match"]{measure(name,scenario,32);}}for size in [64,256,1024]{for name in ["scale_find_all_direct","scale_find_all_ranges"]{measure(name,"scaling",size);}}}
