// Test-only candidate. The checked bridge deliberately rejects this Drop boundary.
use std::sync::{Mutex, atomic::{AtomicI64, Ordering}};
static NEXT: AtomicI64 = AtomicI64::new(1);
static RELEASES: Mutex<Vec<i64>> = Mutex::new(Vec::new());

pub struct Tracked { pub id: i64 }
impl Drop for Tracked {
    fn drop(&mut self) { RELEASES.lock().unwrap().push(self.id); }
}
pub fn tracked() -> Tracked {
    let id = NEXT.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_add(1)).expect("resource identity exhausted");
    Tracked { id }
}
pub fn close(value: Tracked) { drop(value); }
pub fn releases() -> Vec<i64> { RELEASES.lock().unwrap().clone() }
