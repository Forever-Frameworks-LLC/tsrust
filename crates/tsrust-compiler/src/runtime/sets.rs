//! Ordered key storage. Hash buckets retain indices rather than cloning keys.
pub(crate) const SET_RUNTIME: &str = r#"
// Hash collisions need a vector; the usual single-key bucket stays inline.
#[derive(Clone, Default)]
enum TsrSetBucket {
    #[default]
    Empty,
    One(usize),
    Many(Vec<usize>),
}
impl TsrSetBucket {
    fn iter(&self) -> std::slice::Iter<'_, usize> {
        match self {
            Self::Empty => [].iter(),
            Self::One(index) => std::slice::from_ref(index).iter(),
            Self::Many(indices) => indices.iter(),
        }
    }
    fn push(&mut self, index: usize) {
        match self {
            Self::Empty => *self = Self::One(index),
            Self::One(first) => *self = Self::Many(vec![*first, index]),
            Self::Many(indices) => indices.push(index),
        }
    }
    fn retain(&mut self, mut keep: impl FnMut(&usize) -> bool) {
        match self {
            Self::Empty => {},
            Self::One(index) => { if !keep(index) { *self = Self::Empty; } },
            Self::Many(indices) => {
                indices.retain(keep);
                match indices.as_slice() {
                    [] => *self = Self::Empty,
                    [index] => *self = Self::One(*index),
                    _ => {},
                }
            }
        }
    }
    fn is_empty(&self) -> bool { matches!(self, Self::Empty) }
}
trait TsrSetKey: Clone {
    fn tsr_hash(&self) -> u64;
    fn tsr_same(&self, other: &Self) -> bool;
    fn tsr_normalize(self) -> Self { self }
}
fn tsr_set_hash(tag: u64, value: impl std::hash::Hash) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut state = std::collections::hash_map::DefaultHasher::new();
    tag.hash(&mut state); value.hash(&mut state); state.finish()
}
fn tsr_set_number_hash(value: f64) -> u64 {
    tsr_set_hash(0, if value.is_nan() { 0x7ff8000000000000 } else if value == 0.0 { 0 } else { value.to_bits() })
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
    buckets: std::collections::HashMap<u64, TsrSetBucket>,
    length: usize,
}
impl<T: TsrSetKey> TsrSet<T> {
    fn tsr_owned_items(self)->impl Iterator<Item=T>{self.slots.into_iter().flatten()}

    fn new() -> Self { Self { slots: Vec::new(), buckets: std::collections::HashMap::new(), length: 0 } }
    fn from_values(values: impl IntoIterator<Item=T>) -> Self {
        let mut result = Self::new(); for value in values { result.add(value); } result
    }
    fn location(&self, value: &T, hash: u64) -> Option<usize> {
        self.buckets.get(&hash).and_then(|indices| indices.iter().copied().find(|&index| self.slots[index].as_ref().is_some_and(|stored| stored.tsr_same(value))))
    }
    fn add(&mut self, value: T) {
        let value=value.tsr_normalize(); let hash=value.tsr_hash();
        if self.location(&value,hash).is_some() { return; }
        let index=self.slots.len(); self.slots.push(Some(value));
        self.buckets.entry(hash).or_default().push(index); self.length+=1;
    }
    fn query_location<Q: TsrSetQuery<T>>(&self, query: &Q, hash: u64) -> Option<usize> {
        self.buckets.get(&hash).and_then(|indices| indices.iter().copied().find(|&index| self.slots[index].as_ref().is_some_and(|stored| query.tsr_query_same(stored))))
    }
    fn matches_with<'a,U,F:Fn(&U,&T)->bool>(&'a self,value:&'a U,hash:u64,compare:&'a F)->impl Iterator<Item=&'a T>+'a{
        self.buckets.get(&hash).into_iter().flat_map(|indices|indices.iter()).filter_map(move |&index|self.slots[index].as_ref().filter(|stored|compare(value,stored)))
    }
    fn has_with<U, F: Fn(&U, &T) -> bool>(&self, value: &U, hash: u64, compare: &F) -> bool {
        self.buckets.get(&hash).is_some_and(|indices|indices.iter().any(|&index|self.slots[index].as_ref().is_some_and(|stored|compare(value,stored))))
    }
    fn has(&self, value: impl TsrSetQuery<T>) -> bool { self.query_location(&value,value.tsr_query_hash()).is_some() }
    fn delete(&mut self, value: impl TsrSetQuery<T>) -> bool {
        let hash=value.tsr_query_hash(); let Some(index)=self.query_location(&value,hash) else { return false; };
        if let Some(indices)=self.buckets.get_mut(&hash) { indices.retain(|&stored|stored!=index); }
        if self.buckets.get(&hash).is_some_and(TsrSetBucket::is_empty) { self.buckets.remove(&hash); }
        self.slots[index].take(); self.length-=1; true
    }
    fn clear(&mut self) { self.slots.clear(); self.buckets.clear(); self.length=0; }
    fn len(&self) -> usize { self.length }
    fn iter(&self) -> impl Iterator<Item=&T> { self.slots.iter().filter_map(Option::as_ref) }
}
"#;
