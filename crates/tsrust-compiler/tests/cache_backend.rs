#[path = "../src/stdlib/cache/lru.rs"]
mod lru;
mod support;
#[path = "../src/runtime/sets.rs"]
mod source_keys;
use lru::Lru;
use std::{hash::{Hash,Hasher},num::NonZeroUsize};

#[derive(PartialEq,Eq,Debug)]
struct Key(u32);
impl Hash for Key {fn hash<H:Hasher>(&self,state:&mut H){0u8.hash(state);}}

#[test]
fn cache_uses_existing_source_key_equality_and_normalization(){
 let source=format!("mod tsr_cache {{ {} }}\n{}\n{}\n{}",include_str!("../src/stdlib/cache/lru.rs"),source_keys::SET_RUNTIME,include_str!("../src/stdlib/cache/key_policy.rs"),r#"
type String=std::borrow::Cow<'static,str>;
#[derive(Clone)] struct TsrNull;
#[derive(Clone)] struct TsrUndefined;
fn main(){
 let mut cache=tsr_cache::Lru::with_policy(std::num::NonZeroUsize::new(3).unwrap(),TsrCacheKeys);
 cache.insert(-0.0,11);assert_eq!(cache.get(&0.0),Some(&11));
 let old=cache.insert(0.0,22).unwrap();assert_eq!(old.0.to_bits(),0.0f64.to_bits());
 let first=f64::from_bits(0x7ff8000000000001);let second=f64::from_bits(0x7ff8000000000002);
 cache.insert(first,33);assert_eq!(cache.get(&second),Some(&33));assert_eq!(cache.insert(second,44).unwrap().1,33);
 cache.insert(f64::INFINITY,55);assert_eq!(cache.len(),3);assert_eq!(cache.remove(&f64::NAN).unwrap().1,44);
 let mut exact=tsr_cache::Lru::with_policy(std::num::NonZeroUsize::new(2).unwrap(),TsrCacheKeys);
 exact.insert(9007199254740992i64,1);exact.insert(9007199254740993i64,2);
 assert_eq!(exact.get(&9007199254740992),Some(&1));assert_eq!(exact.get(&9007199254740993),Some(&2));
}
"#);
 let directory=support::TempDirectory::new();let path=directory.0.join("keys.rs");std::fs::write(&path,source).unwrap();let binary=directory.0.join("keys");
 support::success(&support::run(support::rustc().args(["--edition=2024","--error-format=short"]).arg(path).arg("-o").arg(&binary),60));
 support::success(&support::run(&mut std::process::Command::new(binary),10));
}

#[test]
fn deadlines_expire_at_boundary_without_stale_replacement_entries(){
 use std::time::{Instant,Duration};
 let start=Instant::now();let deadline=start+Duration::from_secs(5);
 let mut cache=Lru::new(NonZeroUsize::new(3).unwrap());
 cache.insert_until(Key(1),11,Some(deadline));cache.insert_until(Key(2),22,Some(deadline));cache.insert(Key(3),33);
 assert!(cache.expire(deadline-Duration::from_nanos(1)).is_empty());assert_eq!(cache.get(&Key(1)),Some(&11));
 // Replacement cancels the old deadline; access alone never changes deadlines.
 assert_eq!(cache.insert_until(Key(2),44,Some(deadline+Duration::from_secs(1))),Some((Key(2),22)));
 assert_eq!(cache.expire(deadline),vec![(Key(1),11)]);assert_eq!(cache.peek(&Key(2)),Some(&44));
 assert_eq!(cache.insert(Key(2),55),Some((Key(2),44)));assert!(cache.expire(deadline+Duration::from_secs(2)).is_empty());
 // Reusing a freed slot must not inherit any deadline from its old occupant.
 cache.insert_until(Key(4),66,Some(start));assert_eq!(cache.expire(deadline+Duration::from_secs(2)),vec![(Key(4),66)]);
 assert_eq!(cache.len(),2);cache.assert_invariants();
 cache.clear();assert!(cache.expire(deadline).is_empty());cache.assert_invariants();
}

#[test]
fn expiration_index_stays_bounded_during_refresh_and_eviction(){
 use std::time::{Instant,Duration};
 let start=Instant::now();let mut cache=Lru::new(NonZeroUsize::new(2).unwrap());
 for step in 0..5000 {
  cache.insert_until(Key(step%3),step,Some(start+Duration::from_secs(u64::from(step))));
  cache.assert_invariants();
 }
 let mut expired=cache.expire(start+Duration::from_secs(5000));expired.sort_by_key(|(key,_)|key.0);
 assert_eq!(expired,vec![(Key(0),4998),(Key(1),4999)]);assert_eq!(cache.len(),0);cache.assert_invariants();
}

#[test]
fn eviction_removal_clear_and_drop_dispose_each_value_once(){
 struct Value(std::rc::Rc<std::cell::Cell<usize>>);
 impl Drop for Value {fn drop(&mut self){self.0.set(self.0.get()+1);}}
 let drops=std::rc::Rc::new(std::cell::Cell::new(0));let value=||Value(drops.clone());
 let mut cache=Lru::new(NonZeroUsize::new(1).unwrap());
 assert!(cache.insert(Key(1),value()).is_none());let evicted=cache.insert(Key(2),value()).unwrap();
 assert_eq!(drops.get(),0);drop(evicted);assert_eq!(drops.get(),1);
 let removed=cache.remove(&Key(2)).unwrap();assert_eq!(drops.get(),1);drop(removed);assert_eq!(drops.get(),2);
 let now=std::time::Instant::now();cache.insert_until(Key(3),value(),Some(now));
 let expired=cache.expire(now);assert_eq!(drops.get(),2);drop(expired);assert_eq!(drops.get(),3);
 cache.insert(Key(4),value());cache.clear();assert_eq!(drops.get(),4);
 cache.insert(Key(5),value());drop(cache);assert_eq!(drops.get(),5);
}

#[test]
fn owned_collision_keys_and_values_move_without_clone(){
 let mut cache=Lru::new(NonZeroUsize::new(2).unwrap());
 let value=Box::new(11);let pointer=&*value as *const i32;
 assert!(cache.insert(Key(1),value).is_none());assert!(cache.insert(Key(2),Box::new(22)).is_none());
 assert_eq!(&**cache.get(&Key(1)).unwrap() as *const i32,pointer);
 assert_eq!(cache.insert(Key(3),Box::new(33)),Some((Key(2),Box::new(22))));
 let replaced=cache.insert(Key(1),Box::new(44)).unwrap();assert_eq!(&*replaced.1 as *const i32,pointer);
 assert_eq!(cache.peek(&Key(3)),Some(&Box::new(33)));
 assert_eq!(cache.insert(Key(4),Box::new(55)),Some((Key(3),Box::new(33))));
 assert_eq!(cache.remove(&Key(1)),Some((Key(1),Box::new(44))));assert_eq!(cache.len(),1);
 cache.clear();assert_eq!(cache.len(),0);assert!(cache.get(&Key(4)).is_none());cache.assert_invariants();
}

#[test]
fn churn_matches_independent_recency_model(){
 let mut cache=Lru::new(NonZeroUsize::new(7).unwrap());let mut model=std::collections::VecDeque::new();let mut seed=17u32;
 for step in 0..5000 {
  seed=seed.wrapping_mul(1664525).wrapping_add(1013904223);let key=(seed>>12)%23;
  let position=model.iter().position(|(candidate,_)|*candidate==key);
  match seed%4 {
   0=>{let old=position.and_then(|position|model.remove(position)).or_else(||if model.len()==7 {model.pop_front()} else {None});model.push_back((key,step));assert_eq!(cache.insert(Key(key),step).map(|(key,value)|(key.0,value)),old);},
   1=>{let found=position.and_then(|position|model.remove(position));if let Some(value)=found {model.push_back(value);}assert_eq!(cache.get(&Key(key)).copied(),found.map(|(_,value)|value));},
   2=>assert_eq!(cache.remove(&Key(key)).map(|(key,value)|(key.0,value)),position.and_then(|position|model.remove(position))),
   _=>assert_eq!(cache.peek(&Key(key)).copied(),position.map(|position|model[position].1)),
  }
  assert_eq!(cache.len(),model.len());cache.assert_invariants();
  for key in 0..23 {assert_eq!(cache.peek(&Key(key)).copied(),model.iter().find(|(candidate,_)|*candidate==key).map(|(_,value)|*value));}
 }
}
