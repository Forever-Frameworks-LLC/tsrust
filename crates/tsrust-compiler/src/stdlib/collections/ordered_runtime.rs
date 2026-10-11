// Owned tree and heap backends. Keys use a compiler-checked total order.
#[derive(Clone, Copy, Debug)]
struct TsrOrderedNumber(f64);
impl TsrOrderedNumber {
    fn new(value: f64) -> Self {
        Self(if value == 0.0 { 0.0 } else if value.is_nan() { f64::NAN } else { value })
    }
    fn value(self) -> f64 { self.0 }
}
impl PartialEq for TsrOrderedNumber {
    fn eq(&self, other: &Self) -> bool { self.cmp(other) == std::cmp::Ordering::Equal }
}
impl Eq for TsrOrderedNumber {}
impl PartialOrd for TsrOrderedNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}
impl Ord for TsrOrderedNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => self.0.partial_cmp(&other.0).unwrap(),
        }
    }
}

#[derive(Clone)]
struct TsrOrderedMap<K, V> { values: std::collections::BTreeMap<K, V> }
impl<K: Ord, V> TsrOrderedMap<K, V> {
    fn new() -> Self { Self { values: std::collections::BTreeMap::new() } }
    fn len(&self) -> usize { self.values.len() }
    fn insert(&mut self, key: K, value: V) -> Option<V> { self.values.insert(key, value) }
    fn get(&self, key: &K) -> Option<&V> { self.values.get(key) }
    fn contains_key(&self, key: &K) -> bool { self.values.contains_key(key) }
    fn remove(&mut self, key: &K) -> Option<V> { self.values.remove(key) }
    fn clear(&mut self) { self.values.clear(); }
    fn iter(&self) -> std::collections::btree_map::Iter<'_, K, V> { self.values.iter() }
}

#[derive(Clone)]
struct TsrOrderedSet<K> { values: std::collections::BTreeSet<K> }
impl<K: Ord> TsrOrderedSet<K> {
    fn new() -> Self { Self { values: std::collections::BTreeSet::new() } }
    fn len(&self) -> usize { self.values.len() }
    fn insert(&mut self, value: K) -> bool { self.values.insert(value) }
    fn contains(&self, value: &K) -> bool { self.values.contains(value) }
    fn remove(&mut self, value: &K) -> bool { self.values.remove(value) }
    fn clear(&mut self) { self.values.clear(); }
    fn iter(&self) -> std::collections::btree_set::Iter<'_, K> { self.values.iter() }
}

#[derive(Clone)]
struct TsrHeap<T> { values: std::collections::BinaryHeap<T> }
impl<T: Ord> TsrHeap<T> {
    fn new() -> Self { Self { values: std::collections::BinaryHeap::new() } }
    fn len(&self) -> usize { self.values.len() }
    fn push(&mut self, value: T) { self.values.push(value); }
    fn peek(&self) -> Option<&T> { self.values.peek() }
    fn pop(&mut self) -> Option<T> { self.values.pop() }
    fn clear(&mut self) { self.values.clear(); }
}
