//! Operation-driven expiration; the clock is injected for deterministic checks.
use super::lru::{KeyPolicy,Lru};
use std::time::{Duration,Instant};

#[derive(Debug,PartialEq,Eq)]
pub enum DeadlineError { ZeroDuration, Overflow }
#[derive(Debug)]
pub struct InsertError<K,V> {pub kind:DeadlineError,pub key:K,pub value:V}

pub struct Timed<K,V,P,C> {storage:Lru<K,V,P>,clock:C}
impl<K,V,P:KeyPolicy<K>,C:FnMut()->Instant> Timed<K,V,P,C> {
 pub fn new(storage:Lru<K,V,P>,clock:C)->Self{Self{storage,clock}}
 fn expire(&mut self){let now=(self.clock)();drop(self.storage.expire(now));}
 pub fn len(&mut self)->usize{self.expire();self.storage.len()}
 pub fn get(&mut self,key:&K)->Option<&V>{self.expire();self.storage.get(key)}
 pub fn peek(&mut self,key:&K)->Option<&V>{self.expire();self.storage.peek(key)}
 pub fn remove(&mut self,key:&K)->Option<(K,V)>{self.expire();self.storage.remove(key)}
 pub fn insert(&mut self,key:K,value:V,ttl:Option<Duration>)->Result<Option<(K,V)>,InsertError<K,V>>{
  let now=(self.clock)();
  let deadline=match ttl {
   None=>None,
   Some(duration) if duration.is_zero()=>return Err(InsertError{kind:DeadlineError::ZeroDuration,key,value}),
   Some(duration)=>match now.checked_add(duration){Some(deadline)=>Some(deadline),None=>return Err(InsertError{kind:DeadlineError::Overflow,key,value})},
  };
  // Validation precedes cleanup, replacement and eviction. A rejected insertion
  // returns the exact incoming owners and leaves existing entries untouched.
  drop(self.storage.expire(now));Ok(self.storage.insert_until(key,value,deadline))
 }
}
