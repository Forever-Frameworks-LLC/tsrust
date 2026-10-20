use std::sync::atomic::{AtomicU64,Ordering};

static NEXT_ID:AtomicU64=AtomicU64::new(1);

fn allocate(counter:&AtomicU64)->Result<String,String>{
 let value=counter.fetch_update(Ordering::Relaxed,Ordering::Relaxed,|value|value.checked_add(1)).map_err(|_|String::from("UI instance IDs are exhausted"))?;
 Ok(format!("tsrust-ui-{value}"))
}

/// Each call reserves a new ID; retaining the returned string preserves identity.
pub fn use_id()->Result<String,String>{allocate(&NEXT_ID)}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]
 fn ids_are_exact_independent_and_do_not_wrap(){
  let counter=AtomicU64::new(1);assert_eq!(allocate(&counter).unwrap(),"tsrust-ui-1");assert_eq!(allocate(&counter).unwrap(),"tsrust-ui-2");
  let counter=AtomicU64::new(u64::MAX-1);assert_eq!(allocate(&counter).unwrap(),format!("tsrust-ui-{}",u64::MAX-1));assert_eq!(allocate(&counter).unwrap_err(),"UI instance IDs are exhausted");assert_eq!(counter.load(Ordering::Relaxed),u64::MAX);
 }
 #[test]
 fn concurrent_allocations_have_no_duplicate_ids(){
  let counter=AtomicU64::new(1);let mut values=std::thread::scope(|scope|{let handles=(0..4).map(|_|scope.spawn(||(0..32).map(|_|allocate(&counter).unwrap()).collect::<Vec<_>>())).collect::<Vec<_>>();handles.into_iter().flat_map(|handle|handle.join().unwrap()).collect::<Vec<_>>()});
  values.sort();values.dedup();assert_eq!(values.len(),128);
 }
}
