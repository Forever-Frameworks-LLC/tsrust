use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
struct Counter;
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
// SAFETY: Every request is forwarded unchanged to System. Counters do not allocate.
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}
#[global_allocator] static ALLOCATOR: Counter = Counter;
#[derive(Clone, Debug, PartialEq)] struct Key(i64, u64);
impl TsrSetKey for Key {
    fn tsr_hash(&self) -> u64 { self.1 }
    fn tsr_same(&self, other: &Self) -> bool { self.0 == other.0 }
}
fn main() {
    let mut set = TsrSet::new();
    let mut map = TsrMap::<Key, i64>::new();
    set.slots.reserve(128); set.buckets.reserve(128);
    map.slots.reserve(128); map.buckets.reserve(128);
    let before = ALLOCATIONS.load(Relaxed);
    for i in 0..128 { set.add(Key(i, i as u64)); map.set(Key(i, i as u64), i * 3); }
    assert_eq!(ALLOCATIONS.load(Relaxed) - before, 0, "distinct hashes must not allocate bucket storage");
    set.clear(); map.clear();
    let mut model = Vec::<(i64, i64)>::new();
    for step in 0..2048 {
        let id = (step * 37) % 127;
        let key = Key(id, (id % 4) as u64);
        if step % 3 == 0 {
            let at = model.iter().position(|v| v.0 == id);
            assert_eq!(set.delete(key.clone()), at.is_some());
            assert_eq!(map.delete(key.clone()), at.is_some());
            if let Some(at) = at { model.remove(at); }
        } else {
            set.add(key.clone()); map.set(key.clone(), step);
            if let Some(v) = model.iter_mut().find(|v| v.0 == id) { v.1 = step; }
            else { model.push((id, step)); }
        }
        assert_eq!(set.iter().map(|k| k.0).collect::<Vec<_>>(), model.iter().map(|v| v.0).collect::<Vec<_>>());
        assert_eq!(map.iter().map(|(k,v)| (k.0,*v)).collect::<Vec<_>>(), model);
        assert_eq!(map.get(key.clone()).copied(), model.iter().find(|v| v.0 == id).map(|v| v.1));
        assert_eq!(set.has(key), model.iter().any(|v| v.0 == id));
    }
    let copied_set = set.clone(); let copied_map = map.clone();
    for (id, _) in &model { let key=Key(*id, (*id % 4) as u64); assert!(set.delete(key.clone())); assert!(map.delete(key)); }
    assert_eq!(set.len(), 0); assert_eq!(map.len(), 0);
    assert!(set.buckets.is_empty()); assert!(map.buckets.is_empty());
    assert_eq!(copied_set.len(), model.len()); assert_eq!(copied_map.len(), model.len());
    set.add(Key(1,0)); map.set(Key(1,0),9);
    assert!(set.has(Key(1,0))); assert_eq!(map.get(Key(1,0)),Some(&9));
}
