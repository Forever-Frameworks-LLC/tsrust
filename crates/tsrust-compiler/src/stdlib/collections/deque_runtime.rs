//! Owned deque storage with borrowed reads and moving removals.
#[derive(Clone)]
struct TsrDeque<T> {
    values: std::collections::VecDeque<T>,
}
impl<T> TsrDeque<T> {
    fn new() -> Self { Self { values: std::collections::VecDeque::new() } }
    fn len(&self) -> usize { self.values.len() }
    fn push_front(&mut self, value: T) { self.values.push_front(value); }
    fn push_back(&mut self, value: T) { self.values.push_back(value); }
    fn pop_front(&mut self) -> Option<T> { self.values.pop_front() }
    fn pop_back(&mut self) -> Option<T> { self.values.pop_back() }
    fn front(&self) -> Option<&T> { self.values.front() }
    fn back(&self) -> Option<&T> { self.values.back() }
    fn at(&self, index: i64) -> Option<&T> {
        usize::try_from(index).ok().and_then(|index| self.values.get(index))
    }
    fn clear(&mut self) { self.values.clear(); }
    fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator { self.values.iter() }
}
