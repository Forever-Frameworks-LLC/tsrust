mod support;
const RUST:&str=r#"
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize,AtomicBool,Ordering};
static DROPS:AtomicUsize=AtomicUsize::new(0);
static READY:AtomicBool=AtomicBool::new(false);
struct Guard;
impl Drop for Guard {fn drop(&mut self){DROPS.fetch_add(1,Ordering::SeqCst);}}
pub fn pointer(values:&[i64])->i64{values.as_ptr() as i64}
pub fn identity(rustInput0:Vec<i64>)->Vec<i64>{rustInput0}
pub fn numbers(mut values:Vec<f64>)->Vec<f64>{values[0]+=0.5;values}
pub fn flags(mut values:Vec<bool>)->Vec<bool>{values[0]=!values[0];values}
pub fn rejected(values:Vec<i64>)->Result<Vec<i64>,Vec<i64>>{Err(values)}
pub fn text(value:String)->String{value}
pub fn text_pointer(value:&str)->i64{value.as_ptr() as i64}
pub fn mutate(values:&mut [i64]){values[0]+=10;}
pub fn checked(value:i64)->Result<i64,String>{let _guard=Guard;if value<0{Err("negative".into())}else{Ok(value)}}
pub fn nested(values:Vec<i64>)->i64{let (sum,width)=rayon::join(||values.par_iter().sum::<i64>(),||rayon::current_num_threads() as i64);sum+width}
pub fn width()->i64{rayon::current_num_threads() as i64}
pub fn panics(){let _guard=Guard;rayon::join(||panic!("native panic"),||());}
pub fn listen(task:&TaskContext){let _guard=Guard;READY.store(true,Ordering::SeqCst);while !task.cancellation_requested(){std::thread::yield_now();}}
pub fn ready()->bool{READY.load(Ordering::SeqCst)}
pub fn unit()->Result<(),String>{Ok(())}
pub fn drops()->i64{DROPS.load(Ordering::SeqCst) as i64}
"#;
#[test]
fn rust_owned_buffers_results_rayon_cancellation_and_cleanup(){
 let dir=support::TempDirectory::new();std::fs::write(dir.0.join("compute.rs"),RUST).unwrap();
 let source=r#"
import {pointer,identity,text,text_pointer,mutate,checked,nested,width,panics,listen,ready,drops,unit,numbers,flags,rejected} from './compute.rs';
import {ThreadPool,spawn} from 'std:threads';import {strictEqual} from 'std:testing';
function main(){
 let values:int[]=[];for(let i=1;i<=20;i+=1){values.push(i);}
 const address=pointer(values);const returned=identity(values);strictEqual(address,pointer(returned));
 mutate(returned);strictEqual(returned[0],11);
 const action=identity;const throughCallback=action([1,2,3,4,5]);const retained=pointer(throughCallback);
 match(rejected(throughCallback)){Ok(_)=>{throw 'accepted rejection';},Err(value)=>strictEqual(pointer(value),retained)}
 const decimals=numbers([1.5,2.5]);strictEqual(decimals[0],2.0);const booleans=flags([true,false]);strictEqual(booleans[0],false);
 const word='hello'.repeat(8);const wordAddress=text_pointer(word);const saved=text(word);strictEqual(wordAddress,text_pointer(saved));
 unwrap(unit());strictEqual(unwrap(checked(7)),7);try{unwrap(checked(-1));}catch(error){strictEqual(error,'negative');}
 const pool=new ThreadPool({workers:2,capacity:2});
 const first=unwrap(pool.task(():int=>nested(returned)));const second=unwrap(pool.task(():int=>width()));
 strictEqual(unwrap(first.join()),222);strictEqual(unwrap(second.join()),2);
 const failed=unwrap(pool.task(():void=>panics()));match(failed.join()){Ok(_)=>{throw 'accepted panic';},Err(error)=>strictEqual(error.code,'panic')}
 const listener=unwrap(pool.task((task):void=>listen(task)));while(!ready()){}
 listener.cancel();match(listener.join()){Ok(_)=>{throw 'accepted cancellation';},Err(error)=>strictEqual(error.code,'cancelled')}
 strictEqual(drops(),4);unwrap(pool.shutdown());strictEqual(pool.stats().outstanding,0);
}
"#;
 let path=dir.source(&support::format_round_trip(source));support::success(&support::run(support::cli().arg("run").arg(path),90));
}
#[test]
fn rust_signatures_and_source_ownership_fail_closed(){
 for rust in ["pub unsafe fn bad()->i64{0}","pub fn bad<T>(value:T)->i64{0}","pub fn bad()-> &'static str{\"x\"}","pub fn bad(value:*const i64){}","pub fn bad(value:usize){}","pub fn bad(value:Vec<String>){}","mod hidden;pub fn bad(){}","pub fn bad(){include!(\"hidden.rs\");}","#[cfg(any())]pub fn bad(){}", "pub fn bad(){vec![include!(\"hidden.rs\")];}", "use std::include as vec;pub fn bad(){vec!(\"hidden.rs\");}"] {
  let dir=support::TempDirectory::new();std::fs::write(dir.0.join("compute.rs"),rust).unwrap();let path=dir.source("import {bad} from './compute.rs';function main(){}");
  let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"accepted {rust}");assert!(String::from_utf8_lossy(&output.stderr).contains("rust-bridge"));
 }
 for body in ["const values=[1,2];identity(values);console.log(values.length);","identity('wrong');","const values=[1,2];const worker=unwrap(spawn(():int=>pointer(values)));console.log(values.length);","const callback=identity;const values=[1,2];callback(values);console.log(values.length);"] {
  let dir=support::TempDirectory::new();std::fs::write(dir.0.join("compute.rs"),RUST).unwrap();let path=dir.source(&format!("import {{identity,pointer}} from './compute.rs';import {{spawn}} from 'std:threads';function main(){{{body}}}"));
  assert!(!support::run(support::cli().arg("check").arg(path),30).status.success(),"accepted {body}");
 }
}

#[test]
fn direct_rust_import_does_not_generate_a_worker_pool(){
 let dir=support::TempDirectory::new();std::fs::write(dir.0.join("compute.rs"),"pub fn double(value:i64)->i64{value*2}").unwrap();
 let path=dir.source(&support::format_round_trip("import {double} from './compute.rs';function main(){console.log(double(21));}"));
 let emitted=support::run(support::cli().arg("emit").arg(&path),30);support::success(&emitted);assert!(!String::from_utf8_lossy(&emitted.stdout).contains("mod tsrust_native_threads"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert_eq!(output.stdout,b"42\n");
}
