//! Ordered key storage. Hash buckets retain indices rather than cloning keys.
pub(crate) const SET_RUNTIME: &str = r#"
trait TsrSetKey: Clone {
    fn tsr_hash(&self) -> u64;
    fn tsr_same(&self, other: &Self) -> bool;
    fn tsr_normalize(self) -> Self { self }
}
/// One keyed hash per key. The key is random per process, so crafted keys
/// cannot predict collisions; Map and Set order comes from insertion, not hashes.
fn tsr_set_hash(tag: u64, value: impl std::hash::Hash) -> u64 {
    use std::hash::{BuildHasher, Hash, Hasher};
    static KEYS: std::sync::OnceLock<std::collections::hash_map::RandomState> = std::sync::OnceLock::new();
    let mut state = KEYS.get_or_init(std::collections::hash_map::RandomState::new).build_hasher();
    tag.hash(&mut state); value.hash(&mut state); state.finish()
}
/// Collections with at most this many slots scan them instead of hashing.
const TSR_SMALL_SLOTS: usize = 8;
const TSR_INDEX_EMPTY: usize = usize::MAX;
const TSR_INDEX_DELETED: usize = usize::MAX - 1;
/// Open-addressing table from a key's hash to its slot index. Each entry keeps
/// the full hash beside the index, so one linear probe finds a key and most
/// mismatches never read the slot vector. Keys live only in the slots.
#[derive(Clone, Default)]
struct TsrSetBuckets {
    entries: Vec<(u64, usize)>,
    live: usize,
    used: usize,
}
impl TsrSetBuckets {
    fn is_empty(&self) -> bool { self.live == 0 }
    fn clear(&mut self) {
        for entry in &mut self.entries { entry.1 = TSR_INDEX_EMPTY; }
        self.live = 0; self.used = 0;
    }
    /// Room for `additional` more indices without rebuilding (7/8 load).
    fn reserve(&mut self, additional: usize) {
        let needed = self.used + additional;
        if needed * 8 < self.entries.len() * 7 { return; }
        let live = self.live + additional;
        let mut capacity = self.entries.len().max(16);
        while live * 8 >= capacity * 7 / 2 { capacity *= 2; }
        let old = std::mem::replace(&mut self.entries, vec![(0, TSR_INDEX_EMPTY); capacity]);
        self.used = self.live;
        let mask = capacity - 1;
        for (hash, index) in old {
            if index >= TSR_INDEX_DELETED { continue; }
            let mut position = hash as usize & mask;
            while self.entries[position].1 != TSR_INDEX_EMPTY { position = (position + 1) & mask; }
            self.entries[position] = (hash, index);
        }
    }
    fn find(&self, hash: u64, mut same: impl FnMut(usize) -> bool) -> Option<usize> {
        if self.entries.is_empty() { return None; }
        let mask = self.entries.len() - 1;
        let mut position = hash as usize & mask;
        loop {
            let (stored, index) = self.entries[position];
            if index == TSR_INDEX_EMPTY { return None; }
            if index != TSR_INDEX_DELETED && stored == hash && same(index) { return Some(index); }
            position = (position + 1) & mask;
        }
    }
    /// Every index stored under `hash`, for keys compared by a caller's predicate.
    fn indices(&self, hash: u64) -> impl Iterator<Item=usize> + '_ {
        let mask = self.entries.len().wrapping_sub(1);
        let mut position = hash as usize & mask;
        let mut done = self.entries.is_empty();
        std::iter::from_fn(move || {
            while !done {
                let (stored, index) = self.entries[position];
                position = (position + 1) & mask;
                if index == TSR_INDEX_EMPTY { done = true; return None; }
                if index != TSR_INDEX_DELETED && stored == hash { return Some(index); }
            }
            None
        })
    }
    fn insert(&mut self, hash: u64, index: usize) {
        self.reserve(1);
        let mask = self.entries.len() - 1;
        let mut position = hash as usize & mask;
        while self.entries[position].1 < TSR_INDEX_DELETED { position = (position + 1) & mask; }
        if self.entries[position].1 == TSR_INDEX_EMPTY { self.used += 1; }
        self.entries[position] = (hash, index); self.live += 1;
    }
    fn remove(&mut self, hash: u64, index: usize) {
        if self.entries.is_empty() { return; }
        let mask = self.entries.len() - 1;
        let mut position = hash as usize & mask;
        loop {
            let (stored, current) = self.entries[position];
            if current == TSR_INDEX_EMPTY { return; }
            if current == index && stored == hash { self.entries[position].1 = TSR_INDEX_DELETED; self.live -= 1; return; }
            position = (position + 1) & mask;
        }
    }
}
/// Numbers (int and float alike, so 1 and 1.0 are one key) hash only their bits.
fn tsr_set_number_hash(value: f64) -> u64 {
    use std::hash::{BuildHasher, Hasher};
    static KEYS: std::sync::OnceLock<std::collections::hash_map::RandomState> = std::sync::OnceLock::new();
    let mut state = KEYS.get_or_init(std::collections::hash_map::RandomState::new).build_hasher();
    state.write_u64(if value.is_nan() { 0x7ff8000000000000 } else if value == 0.0 { 0 } else { value.to_bits() });
    state.finish()
}
fn tsr_set_number_same(left: f64, right: f64) -> bool { left == right || left.is_nan() && right.is_nan() }
trait TsrSetQuery<K: TsrSetKey> {
    fn tsr_query_hash(&self) -> u64;
    fn tsr_query_same(&self, stored: &K) -> bool;
}
impl<K: TsrSetKey> TsrSetQuery<K> for K {
    fn tsr_query_hash(&self) -> u64 { self.tsr_hash() }
    fn tsr_query_same(&self, stored: &K) -> bool { self.tsr_same(stored) }
}
impl<K: TsrSetKey> TsrSetQuery<K> for &K {
    fn tsr_query_hash(&self) -> u64 { (*self).tsr_hash() }
    fn tsr_query_same(&self, stored: &K) -> bool { (*self).tsr_same(stored) }
}
impl TsrSetKey for i64 {
    fn tsr_hash(&self) -> u64 { tsr_set_number_hash(*self as f64) }
    fn tsr_same(&self, other: &Self) -> bool { self == other }
}
impl TsrSetKey for f64 {
    fn tsr_hash(&self) -> u64 { tsr_set_number_hash(*self) }
    fn tsr_same(&self, other: &Self) -> bool { tsr_set_number_same(*self,*other) }
    fn tsr_normalize(self) -> Self { if self == 0.0 { 0.0 } else { self } }
}
impl TsrSetKey for bool {
    fn tsr_hash(&self) -> u64 { tsr_set_hash(1,*self) }
    fn tsr_same(&self, other: &Self) -> bool { self == other }
}
impl TsrSetKey for String {
    fn tsr_hash(&self) -> u64 { let value: &str = self.as_ref(); tsr_set_hash(2,value) }
    fn tsr_same(&self, other: &Self) -> bool { self.as_ref() == other.as_ref() }
}
impl TsrSetKey for TsrNull {
    fn tsr_hash(&self) -> u64 { tsr_set_hash(3,0u8) }
    fn tsr_same(&self, _: &Self) -> bool { true }
}
impl TsrSetKey for TsrUndefined {
    fn tsr_hash(&self) -> u64 { tsr_set_hash(4,0u8) }
    fn tsr_same(&self, _: &Self) -> bool { true }
}
#[derive(Clone)]
struct TsrSet<T: TsrSetKey> {
    slots: Vec<Option<T>>,
    buckets: TsrSetBuckets,
    length: usize,
}
impl<T: TsrSetKey> TsrSet<T> {
    fn tsr_owned_items(self)->impl Iterator<Item=T>{self.slots.into_iter().flatten()}

    fn new() -> Self { Self { slots: Vec::new(), buckets: TsrSetBuckets::default(), length: 0 } }
    fn from_values(values: impl IntoIterator<Item=T>) -> Self {
        let mut result = Self::new(); for value in values { result.add(value); } result
    }
    fn small(&self) -> bool { self.slots.len() <= TSR_SMALL_SLOTS }
    fn add(&mut self, value: T) {
        let value=value.tsr_normalize();
        let hash = if self.small() {
            if self.slots.iter().any(|stored| stored.as_ref().is_some_and(|stored| stored.tsr_same(&value))) { return; }
            None
        } else {
            let hash=value.tsr_hash();
            if self.buckets.find(hash, |index| self.slots[index].as_ref().is_some_and(|stored| stored.tsr_same(&value))).is_some() { return; }
            Some(hash)
        };
        let index=self.slots.len();
        self.slots.push(Some(value)); self.length+=1;
        if index==TSR_SMALL_SLOTS {
            // Leaving the small form: index every live slot once.
            self.buckets.reserve(self.slots.len());
            for index in 0..self.slots.len() { if let Some(stored)=&self.slots[index] { let hash=stored.tsr_hash(); self.buckets.insert(hash, index); } }
        } else if index>TSR_SMALL_SLOTS {
            let hash=hash.unwrap_or_else(|| self.slots[index].as_ref().unwrap().tsr_hash());
            self.buckets.insert(hash, index);
        }
    }
    fn query_location<Q: TsrSetQuery<T>>(&self, query: &Q) -> Option<(usize, u64)> {
        if self.small() { return self.slots.iter().position(|stored| stored.as_ref().is_some_and(|stored| query.tsr_query_same(stored))).map(|index| (index, 0)); }
        let hash=query.tsr_query_hash();
        self.buckets.find(hash, |index| self.slots[index].as_ref().is_some_and(|stored| query.tsr_query_same(stored))).map(|index| (index, hash))
    }
    /// Candidate slots for `value`: every slot of a small set, else those stored under its hash.
    /// A small set has no indexed entries, so only one of the two parts yields.
    fn candidates_for<'a, U: TsrSetKey>(&'a self, value: &U) -> impl Iterator<Item=usize> + 'a {
        let small = self.small();
        (0..if small { self.slots.len() } else { 0 }).chain(self.buckets.indices(if small { 0 } else { value.tsr_hash() }))
    }
    fn matches_with<'a,U:TsrSetKey,F:Fn(&U,&T)->bool>(&'a self,value:&'a U,compare:&'a F)->impl Iterator<Item=&'a T>+'a{
        self.candidates_for(value).filter_map(move |index|self.slots[index].as_ref().filter(|stored|compare(value,stored)))
    }
    fn has_with<U:TsrSetKey, F: Fn(&U, &T) -> bool>(&self, value: &U, compare: &F) -> bool {
        self.candidates_for(value).any(|index|self.slots[index].as_ref().is_some_and(|stored|compare(value,stored)))
    }
    fn has(&self, value: impl TsrSetQuery<T>) -> bool { self.query_location(&value).is_some() }
    fn delete(&mut self, value: impl TsrSetQuery<T>) -> bool {
        let Some((index, hash))=self.query_location(&value) else { return false; };
        if !self.small() { self.buckets.remove(hash, index); }
        self.slots[index].take(); self.length-=1; true
    }
    fn clear(&mut self) { self.slots.clear(); self.buckets.clear(); self.length=0; }
    fn len(&self) -> usize { self.length }
    fn iter(&self) -> impl Iterator<Item=&T> { self.slots.iter().filter_map(Option::as_ref) }
}
"#;
