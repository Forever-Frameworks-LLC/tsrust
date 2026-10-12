//! Uniquely owned, item-bounded LRU storage. No key or value cloning.
use std::{collections::{BTreeSet,HashMap,hash_map::RandomState},hash::{BuildHasher,Hash},num::NonZeroUsize,time::Instant};

struct Entry<K,V> { key:K,value:V,hash:u64,deadline:Option<Instant>,previous:Option<usize>,next:Option<usize> }

pub struct Lru<K,V> {
 capacity:usize,
 hash:RandomState,
 // Hash buckets hold slot indices; the only owned key lives in its entry.
 index:HashMap<u64,Vec<usize>>,
 deadlines:BTreeSet<(Instant,usize)>,
 slots:Vec<Option<Entry<K,V>>>,
 free:Vec<usize>,
 first:Option<usize>,
 last:Option<usize>,
}
impl<K:Hash+Eq,V> Lru<K,V> {
 pub fn new(capacity:NonZeroUsize)->Self {
  Self{capacity:capacity.get(),hash:RandomState::new(),index:HashMap::new(),deadlines:BTreeSet::new(),slots:Vec::new(),free:Vec::new(),first:None,last:None}
 }
 pub fn len(&self)->usize {self.slots.len()-self.free.len()}
 fn find(&self,key:&K)->Option<usize>{
  self.index.get(&self.hash.hash_one(key))?.iter().copied().find(|&slot|self.slots[slot].as_ref().is_some_and(|entry|entry.key==*key))
 }
 pub fn peek(&self,key:&K)->Option<&V>{self.slots[self.find(key)?].as_ref().map(|entry|&entry.value)}
 pub fn get(&mut self,key:&K)->Option<&V>{
  let slot=self.find(key)?;self.unlink(slot);self.append(slot);
  self.slots[slot].as_ref().map(|entry|&entry.value)
 }
 /// Returns the replaced or evicted entry, transferring both key and value.
 pub fn insert(&mut self,key:K,value:V)->Option<(K,V)>{
  self.insert_until(key,value,None)
 }
 /// Stores an absolute monotonic deadline; the caller drives expiration explicitly.
 pub fn insert_until(&mut self,key:K,value:V,deadline:Option<Instant>)->Option<(K,V)>{
  let hash=self.hash.hash_one(&key);
  let replaced=if let Some(slot)=self.find(&key){Some(self.remove_slot(slot))}
   else if self.len()==self.capacity {Some(self.remove_slot(self.first.expect("full cache has an oldest entry")))} else {None};
  let slot=self.free.pop().unwrap_or_else(||{self.slots.push(None);self.slots.len()-1});
  self.slots[slot]=Some(Entry{key,value,hash,deadline,previous:None,next:None});
  if let Some(deadline)=deadline {self.deadlines.insert((deadline,slot));}
  self.index.entry(hash).or_default().push(slot);self.append(slot);replaced
 }
 pub fn remove(&mut self,key:&K)->Option<(K,V)>{let slot=self.find(key)?;Some(self.remove_slot(slot))}
 /// Moves out all entries whose deadline is at or before now. No sleeping or wall clock.
 pub fn expire(&mut self,now:Instant)->Vec<(K,V)>{
  let mut expired=Vec::new();
  while let Some(&(deadline,slot))=self.deadlines.first(){
   if deadline>now {break;}
   expired.push(self.remove_slot(slot));
  }
  expired
 }
 pub fn clear(&mut self){self.index.clear();self.deadlines.clear();self.slots.clear();self.free.clear();self.first=None;self.last=None;}
 fn unlink(&mut self,slot:usize){
  let entry=self.slots[slot].as_ref().expect("linked slot");let (previous,next)=(entry.previous,entry.next);
  if let Some(previous)=previous {self.slots[previous].as_mut().unwrap().next=next;} else {self.first=next;}
  if let Some(next)=next {self.slots[next].as_mut().unwrap().previous=previous;} else {self.last=previous;}
 }
 fn append(&mut self,slot:usize){
  let entry=self.slots[slot].as_mut().expect("occupied slot");entry.previous=self.last;entry.next=None;
  if let Some(last)=self.last {self.slots[last].as_mut().unwrap().next=Some(slot);} else {self.first=Some(slot);}
  self.last=Some(slot);
 }
 fn remove_slot(&mut self,slot:usize)->(K,V){
  self.unlink(slot);let entry=self.slots[slot].take().expect("occupied slot");
  if let Some(deadline)=entry.deadline {self.deadlines.remove(&(deadline,slot));}
  let bucket=self.index.get_mut(&entry.hash).expect("indexed slot");
  let position=bucket.iter().position(|&candidate|candidate==slot).expect("indexed position");bucket.swap_remove(position);
  if bucket.is_empty(){self.index.remove(&entry.hash);}
  self.free.push(slot);(entry.key,entry.value)
 }
 #[cfg(test)]
 pub fn assert_invariants(&self){
  assert!(self.slots.len()<=self.capacity);assert!(self.len()<=self.capacity);
  let mut seen=std::collections::HashSet::new();let mut next=self.first;let mut previous=None;
  while let Some(slot)=next {
   assert!(seen.insert(slot));let entry=self.slots[slot].as_ref().unwrap();assert_eq!(entry.previous,previous);
   assert_eq!(self.find(&entry.key),Some(slot));previous=Some(slot);next=entry.next;
  }
  assert_eq!(previous,self.last);assert_eq!(seen.len(),self.len());
  assert_eq!(self.index.values().map(Vec::len).sum::<usize>(),self.len());
  assert!(self.deadlines.len()<=self.len());
  assert_eq!(self.deadlines.len(),self.slots.iter().flatten().filter(|entry|entry.deadline.is_some()).count());
  for &(deadline,slot) in &self.deadlines {assert_eq!(self.slots[slot].as_ref().unwrap().deadline,Some(deadline));}
  let mut free=std::collections::HashSet::new();for &slot in &self.free {assert!(free.insert(slot));assert!(self.slots[slot].is_none());}
 }
}
