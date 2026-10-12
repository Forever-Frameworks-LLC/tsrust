#[path = "../src/stdlib/cache/lru.rs"]
mod lru;
use lru::Lru;
use std::{hash::{Hash,Hasher},num::NonZeroUsize};

#[derive(PartialEq,Eq,Debug)]
struct Key(u32);
impl Hash for Key {fn hash<H:Hasher>(&self,state:&mut H){0u8.hash(state);}}

#[test]
fn eviction_removal_clear_and_drop_dispose_each_value_once(){
 struct Value(std::rc::Rc<std::cell::Cell<usize>>);
 impl Drop for Value {fn drop(&mut self){self.0.set(self.0.get()+1);}}
 let drops=std::rc::Rc::new(std::cell::Cell::new(0));let value=||Value(drops.clone());
 let mut cache=Lru::new(NonZeroUsize::new(1).unwrap());
 assert!(cache.insert(Key(1),value()).is_none());let evicted=cache.insert(Key(2),value()).unwrap();
 assert_eq!(drops.get(),0);drop(evicted);assert_eq!(drops.get(),1);
 let removed=cache.remove(&Key(2)).unwrap();assert_eq!(drops.get(),1);drop(removed);assert_eq!(drops.get(),2);
 cache.insert(Key(3),value());cache.clear();assert_eq!(drops.get(),3);
 cache.insert(Key(4),value());drop(cache);assert_eq!(drops.get(),4);
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
