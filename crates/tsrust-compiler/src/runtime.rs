//! Safe support code copied into generated programs. Source ownership rules are
//! independent of whether immutable bytes or small array elements need a heap.
pub(crate) const LITERAL_STRING_ARRAYS: &str = r#"
struct LiteralStringArray { values: &'static [&'static str] }
impl LiteralStringArray {
    fn new(values: &'static [&'static str]) -> Self { Self { values } }
    fn len(&self) -> usize { self.values.len() }
    fn get(&self, index: usize) -> Option<String> { self.values.get(index).map(|value| String::Borrowed(value)) }
    fn literal_iter(&self) -> impl Iterator<Item = &'static str> + '_ { self.values.iter().copied() }
    fn clone_owned(&self) -> Array<String> {
        if self.len() <= 4 {
            let mut result = Array::from([]);
            for value in self.values { result.push(String::Borrowed(value)); }
            result
        } else { Array::Heap(self.values.iter().map(|value| String::Borrowed(value)).collect()) }
    }
    fn tsr_includes(&self, value: &String, from: i64, equal: impl Fn(&String, &String) -> bool) -> bool {
        self.values[tsr_search_start(self.len(), from)..].iter().any(|item| equal(&String::Borrowed(item), value))
    }
    fn tsr_index_of(&self, value: &String, from: i64, equal: impl Fn(&String, &String) -> bool, failure: &'static str) -> i64 {
        let start = tsr_search_start(self.len(), from);
        tsr_search_result(start, self.values[start..].iter().position(|item| equal(&String::Borrowed(item), value)), failure)
    }
}
"#;

// String support carries the number text tie helper so it stands alone.
pub(crate) const STRINGS: &str = concat!(include_str!("runtime_strings.rs"), include_str!("runtime_number_tie.rs"), include_str!("runtime_numeric_string.rs"));
pub(crate) const ARRAY_JOIN: &str = include_str!("runtime_join.rs");
pub(crate) const MATH_EXACT: &str = include_str!("runtime_math.rs");
pub(crate) const NUMBER_TIE: &str = include_str!("runtime_number_tie.rs");

// A split result needs ordinary owning array support; gate this separately from
// scalar string operations so unrelated string programs do not acquire arrays.
pub(crate) const STRING_SPLIT: &str = r#"
fn tsr_string_split(value: &String, separator: Option<&String>, limit: Option<i64>) -> Array<String> {
    let limit = limit.map_or(u32::MAX, |limit| limit as u32) as usize;
    if limit == 0 { return Array::Heap(Vec::new()); }
    let values = match value {
        String::Borrowed(text) => {
            let text: &'static str = text;
            match separator {
                None => vec![String::Borrowed(text)],
                Some(separator) if separator.is_empty() => text.char_indices().take(limit)
                    .map(|(start, scalar)| String::Borrowed(&text[start..start + scalar.len_utf8()])).collect(),
                Some(separator) => text.split(separator.as_ref()).take(limit).map(String::Borrowed).collect(),
            }
        }
        String::Owned(text) => match separator {
            None => vec![value.clone()],
            Some(separator) if separator.is_empty() => text.chars().take(limit)
                .map(|scalar| String::Owned(scalar.to_string())).collect(),
            Some(separator) => text.split(separator.as_ref()).take(limit)
                .map(|piece| String::Owned(piece.to_owned())).collect(),
        },
    };
    Array::Heap(values)
}
"#;

// Inline storage is bounded by the emitter. Recursive and large element
// layouts retain Vec indirection; deeper array ranks do not grow inline.
pub(crate) const ARRAYS: &str = r#"
#[derive(Clone)]
enum Array<T> {
    Small { len: usize, values: [Option<T>; 4] },
    Heap(Vec<T>),
}
impl<T, const N: usize> From<[T; N]> for Array<T> {
    #[inline]
    fn from(values: [T; N]) -> Self {
        if N > 4 { return Self::Heap(Vec::from(values)); }
        let mut slots = [const { None }; 4];
        for (slot, value) in slots.iter_mut().zip(values) { *slot = Some(value); }
        Self::Small { len: N, values: slots }
    }
}
impl<T> Array<T> {
    #[inline]
    fn len(&self) -> usize {
        match self { Self::Small { len, .. } => *len, Self::Heap(values) => values.len() }
    }
    #[inline]
    fn get(&self, index: usize) -> Option<&T> {
        match self { Self::Small { values, .. } => values.get(index)?.as_ref(), Self::Heap(values) => values.get(index) }
    }
    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match self { Self::Small { values, .. } => values.get_mut(index)?.as_mut(), Self::Heap(values) => values.get_mut(index) }
    }
    #[inline]
    fn push(&mut self, value: T) {
        match self {
            Self::Heap(values) => { values.push(value); return; }
            Self::Small { len, values } if *len < 4 => { values[*len] = Some(value); *len += 1; return; }
            _ => {}
        }
        // Amortize growth for compact elements without reserving large records.
        let capacity = if std::mem::size_of::<T>() <= 8 { 32 } else { 8 };
        let mut heap = Vec::with_capacity(capacity);
        if let Self::Small { values, .. } = self {
            for slot in values { heap.push(slot.take().expect("occupied inline array slot")); }
        }
        heap.push(value);
        *self = Self::Heap(heap);
    }
    #[inline]
    fn reverse(&mut self) {
        match self {
            Self::Small { len, values } => values[..*len].reverse(),
            Self::Heap(values) => values.reverse(),
        }
    }
    #[inline]
    fn remove(&mut self, index: usize) -> T {
        match self {
            Self::Heap(values) => values.remove(index),
            Self::Small { len, values } => {
                assert!(index < *len);
                let result = values[index].take().expect("occupied inline array slot");
                for next in index + 1..*len { values[next - 1] = values[next].take(); }
                *len -= 1;
                result
            }
        }
    }
    #[inline]
    fn pop(&mut self) -> Option<T> {
        match self {
            Self::Heap(values) => values.pop(),
            Self::Small { len, values } => {
                if *len == 0 { return None; }
                *len -= 1;
                values[*len].take()
            }
        }
    }
    #[inline]
    fn iter(&self) -> ArrayIter<'_, T> {
        match self {
            Self::Small { len, values } => ArrayIter::Small(values[..*len].iter()),
            Self::Heap(values) => ArrayIter::Heap(values.iter()),
        }
    }
}
impl<T: Clone> Array<T> {
    fn tsr_fill(&mut self, value: T, start: usize, end: usize) {
        match self {
            Self::Small { len, values } => values[..*len][start..end].fill(Some(value)),
            Self::Heap(values) => values[start..end].fill(value),
        }
    }
    fn tsr_copy_within(&mut self, start: usize, end: usize, target: usize) {
        if start == end || start == target { return; }
        match self {
            Self::Small { len, values } => {
                let live = &mut values[..*len];
                let snapshot = live[start..end].to_vec();
                for (destination, value) in live[target..target + snapshot.len()].iter_mut().zip(snapshot) { *destination = value; }
            }
            Self::Heap(values) => {
                let snapshot = values[start..end].to_vec();
                for (destination, value) in values[target..target + snapshot.len()].iter_mut().zip(snapshot) { *destination = value; }
            }
        }
    }
    fn tsr_slice(&self, start: usize, end: usize) -> Self {
        match self {
            Self::Small { values, .. } => Self::Heap(values[start..end].iter().map(|value| value.as_ref().expect("occupied inline array slot").clone()).collect()),
            Self::Heap(values) => Self::Heap(values[start..end].to_vec()),
        }
    }
}
enum ArrayIter<'a, T> {
    Small(std::slice::Iter<'a, Option<T>>),
    Heap(std::slice::Iter<'a, T>),
}
impl<'a, T> Iterator for ArrayIter<'a, T> {
    type Item = &'a T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self { Self::Small(values) => values.next().map(|value| value.as_ref().expect("occupied inline array slot")), Self::Heap(values) => values.next() }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self { Self::Small(values) => values.size_hint(), Self::Heap(values) => values.size_hint() }
    }
}
impl<T> DoubleEndedIterator for ArrayIter<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self { Self::Small(values) => values.next_back().map(|value| value.as_ref().expect("occupied inline array slot")), Self::Heap(values) => values.next_back() }
    }
}
impl<T> ExactSizeIterator for ArrayIter<'_, T> {}
"#;

// Copied integer and boolean elements use initialized contiguous slots.
// Owned elements retain Array so this representation cannot share mutable values.
pub(crate) const NUMERIC_ARRAYS: &str = r#"
#[derive(Clone)]
enum NumericArray<T: Copy + Default> {
    Small { len: usize, values: [T; 4] },
    Heap(Vec<T>),
}
impl<T: Copy + Default, const N: usize> From<[T; N]> for NumericArray<T> {
    #[inline]
    fn from(values: [T; N]) -> Self {
        if N > 4 { return Self::Heap(Vec::from(values)); }
        let mut slots = [T::default(); 4];
        slots[..N].copy_from_slice(&values);
        Self::Small { len: N, values: slots }
    }
}
impl<T: Copy + Default> NumericArray<T> {
    #[inline]
    fn as_slice(&self) -> &[T] {
        match self { Self::Small { len, values } => &values[..*len], Self::Heap(values) => values }
    }
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] {
        match self { Self::Small { len, values } => &mut values[..*len], Self::Heap(values) => values }
    }
    #[inline]
    fn len(&self) -> usize { self.as_slice().len() }
    #[inline]
    fn get(&self, index: usize) -> Option<&T> { self.as_slice().get(index) }
    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut T> { self.as_mut_slice().get_mut(index) }
    #[inline]
    fn iter(&self) -> std::slice::Iter<'_, T> { self.as_slice().iter() }
    #[inline]
    fn push(&mut self, value: T) {
        match self {
            Self::Heap(values) => { values.push(value); return; }
            Self::Small { len, values } if *len < 4 => { values[*len] = value; *len += 1; return; }
            _ => {}
        }
        let mut heap = Vec::with_capacity(32);
        if let Self::Small { values, .. } = self { heap.extend_from_slice(values); }
        heap.push(value);
        *self = Self::Heap(heap);
    }
    #[inline]
    fn reverse(&mut self) { self.as_mut_slice().reverse(); }
    #[inline]
    fn tsr_fill(&mut self, value: T, start: usize, end: usize) { self.as_mut_slice()[start..end].fill(value); }
    #[inline]
    fn tsr_copy_within(&mut self, start: usize, end: usize, target: usize) { self.as_mut_slice().copy_within(start..end, target); }
    #[inline]
    fn remove(&mut self, index: usize) -> T {
        match self {
            Self::Heap(values) => values.remove(index),
            Self::Small { len, values } => {
                assert!(index < *len);
                let result = values[index];
                values.copy_within(index + 1..*len, index);
                *len -= 1;
                values[*len] = T::default();
                result
            }
        }
    }
    #[inline]
    fn pop(&mut self) -> Option<T> {
        match self {
            Self::Heap(values) => values.pop(),
            Self::Small { len, values } => {
                if *len == 0 { return None; }
                *len -= 1;
                Some(std::mem::take(&mut values[*len]))
            }
        }
    }

}
"#;

// Dense-array search normalizes an integer offset without overflowing at i64::MIN.
// Element equality remains private to the supported source primitive types.
pub(crate) const SEARCH_START: &str = r#"
#[inline]
fn tsr_search_start(len: usize, from: i64) -> usize {
    if from >= 0 { usize::try_from(from).unwrap_or(usize::MAX).min(len) }
    else { len.saturating_sub(usize::try_from(from.unsigned_abs()).unwrap_or(usize::MAX)) }
}
#[inline]
fn tsr_search_result(start: usize, offset: Option<usize>, failure: &'static str) -> i64 {
    offset.map_or(-1, |offset| i64::try_from(start + offset).expect(failure))
}
"#;

pub(crate) const SEARCH_NUMERIC: &str = r#"
impl<T: Copy + Default + PartialEq> NumericArray<T> {
    #[inline]
    fn tsr_includes(&self, value: &T, from: i64) -> bool {
        let values = self.as_slice();
        values[tsr_search_start(values.len(), from)..].contains(value)
    }
    #[inline]
    fn tsr_index_of(&self, value: &T, from: i64, failure: &'static str) -> i64 {
        let values = self.as_slice();
        let start = tsr_search_start(values.len(), from);
        let tail = &values[start..];
        if std::mem::size_of::<T>() == 1 {
            return tsr_search_result(start, tail.iter().position(|item| item == value), failure);
        }
        let mut groups = tail.chunks_exact(4);
        let mut base = start;
        for group in &mut groups {
            let a = group[0] == *value;
            let b = group[1] == *value;
            let c = group[2] == *value;
            let d = group[3] == *value;
            let mask = u8::from(a) | (u8::from(b) << 1) | (u8::from(c) << 2) | (u8::from(d) << 3);
            if mask != 0 { return tsr_search_result(base, Some(mask.trailing_zeros() as usize), failure); }
            base += 4;
        }
        tsr_search_result(base, groups.remainder().iter().position(|item| item == value), failure)
    }
}
"#;

pub(crate) const SEARCH_OWNED: &str = r#"
#[inline]
fn tsr_primitive_position<T>(values: &[T], value: &T, equal: impl Fn(&T, &T) -> bool) -> Option<usize> {
    let mut groups = values.chunks_exact(4);
    let mut base = 0;
    for group in &mut groups {
        if equal(&group[0], value) { return Some(base); }
        if equal(&group[1], value) { return Some(base + 1); }
        if equal(&group[2], value) { return Some(base + 2); }
        if equal(&group[3], value) { return Some(base + 3); }
        base += 4;
    }
    groups.remainder().iter().position(|item| equal(item, value)).map(|offset| base + offset)
}

impl<T> Array<T> {
    #[inline]
    fn tsr_includes(&self, value: &T, from: i64, equal: impl Fn(&T, &T) -> bool) -> bool {
        let start = tsr_search_start(self.len(), from);
        match self {
            Self::Small { len, values } => values[start..*len].iter().any(|item| equal(item.as_ref().expect("occupied inline array slot"), value)),
            Self::Heap(values) => tsr_primitive_position(&values[start..], value, equal).is_some(),
        }
    }
    #[inline]
    fn tsr_index_of(&self, value: &T, from: i64, equal: impl Fn(&T, &T) -> bool, failure: &'static str) -> i64 {
        let start = tsr_search_start(self.len(), from);
        let offset = match self {
            Self::Small { len, values } => values[start..*len].iter().position(|item| equal(item.as_ref().expect("occupied inline array slot"), value)),
            Self::Heap(values) => tsr_primitive_position(&values[start..], value, equal),
        };
        tsr_search_result(start, offset, failure)
    }
}
"#;

// Front insertion validates the post-argument length and reserves before moving
// any existing inline owners. Arguments are already evaluated by the caller.
pub(crate) const UNSHIFT_VEC: &str = r#"
fn tsr_unshift_length(len: usize, added: usize, failure: &'static str) -> (usize, i64) {
    let length = len.checked_add(added).expect(failure);
    (length, i64::try_from(length).expect(failure))
}
trait TsrUnshift<T> {
    fn tsr_unshift<const N: usize>(&mut self, added: [T; N], failure: &'static str) -> i64;
    // added_count is the exact number of already staged items.
    fn tsr_unshift_iter<I: IntoIterator<Item=T>>(&mut self, added:I, added_count:usize, failure:&'static str)->i64;
}
impl<T> TsrUnshift<T> for Vec<T> {
    fn tsr_unshift<const N: usize>(&mut self, added: [T; N], failure: &'static str) -> i64 {
        let (_, length) = tsr_unshift_length(self.len(), N, failure);
        self.reserve(N);
        self.splice(0..0, added);
        length
    }
    fn tsr_unshift_iter<I: IntoIterator<Item=T>>(&mut self, added:I, added_count:usize, failure:&'static str)->i64 {
        let (_, length) = tsr_unshift_length(self.len(), added_count, failure);
        self.reserve(added_count);
        self.splice(0..0, added);
        length
    }
}
"#;
pub(crate) const UNSHIFT_ARRAY: &str = r#"
impl<T> TsrUnshift<T> for Array<T> {
    fn tsr_unshift<const N: usize>(&mut self, added: [T; N], failure: &'static str) -> i64 {
        let (new_len, length) = tsr_unshift_length(self.len(), N, failure);
        match self {
            Self::Heap(values) => { values.tsr_unshift(added, failure); }
            Self::Small { len, values } if new_len <= 4 => {
                values[..new_len].rotate_right(N);
                for (slot, value) in values[..N].iter_mut().zip(added) { *slot = Some(value); }
                *len = new_len;
            }
            Self::Small { len, values } => {
                let mut heap = Vec::new();
                heap.reserve(new_len);
                heap.extend(added);
                for slot in &mut values[..*len] { heap.push(slot.take().expect("occupied inline array slot")); }
                *self = Self::Heap(heap);
            }
        }
        length
    }
    fn tsr_unshift_iter<I: IntoIterator<Item=T>>(&mut self, added:I, added_count:usize, failure:&'static str)->i64 {
        let (new_len, length) = tsr_unshift_length(self.len(), added_count, failure);
        match self {
            Self::Heap(values) => { values.tsr_unshift_iter(added, added_count, failure); }
            Self::Small { len, values } if new_len <= 4 => {
                values[..new_len].rotate_right(added_count);
                for (slot, value) in values[..added_count].iter_mut().zip(added) { *slot = Some(value); }
                *len = new_len;
            }
            Self::Small { len, values } => {
                let mut heap = Vec::new();
                heap.reserve(new_len);
                heap.extend(added);
                for slot in &mut values[..*len] { heap.push(slot.take().expect("occupied inline array slot")); }
                *self = Self::Heap(heap);
            }
        }
        length
    }
}
"#;
pub(crate) const UNSHIFT_NUMERIC: &str = r#"
impl<T: Copy + Default> TsrUnshift<T> for NumericArray<T> {
    fn tsr_unshift<const N: usize>(&mut self, added: [T; N], failure: &'static str) -> i64 {
        let (new_len, length) = tsr_unshift_length(self.len(), N, failure);
        match self {
            Self::Heap(values) => { values.tsr_unshift(added, failure); }
            Self::Small { len, values } if new_len <= 4 => {
                values[..new_len].rotate_right(N);
                values[..N].copy_from_slice(&added);
                *len = new_len;
            }
            Self::Small { len, values } => {
                let mut heap = Vec::new();
                heap.reserve(new_len);
                heap.extend(added);
                heap.extend_from_slice(&values[..*len]);
                *self = Self::Heap(heap);
            }
        }
        length
    }
    fn tsr_unshift_iter<I: IntoIterator<Item=T>>(&mut self, added:I, added_count:usize, failure:&'static str)->i64 {
        let (new_len, length) = tsr_unshift_length(self.len(), added_count, failure);
        match self {
            Self::Heap(values) => { values.tsr_unshift_iter(added, added_count, failure); }
            Self::Small { len, values } if new_len <= 4 => {
                values[..new_len].rotate_right(added_count);
                for (slot, value) in values[..added_count].iter_mut().zip(added) { *slot = value; }
                *len = new_len;
            }
            Self::Small { len, values } => {
                let mut heap = Vec::new();
                heap.reserve(new_len);
                heap.extend(added);
                heap.extend_from_slice(&values[..*len]);
                *self = Self::Heap(heap);
            }
        }
        length
    }
}
"#;

// Return removed owners independently. Reserve both destinations before any
// receiver element moves; standard slice rotations preserve inline slot order.
pub(crate) const SPLICE_VEC: &str = r#"
fn tsr_splice_plan(len: usize, start: i64, delete: Option<i64>, added: usize, failure: &'static str) -> (usize, usize, usize) {
    let start=tsr_search_start(len,start);
    let deleted=delete.map_or(len-start,|count|usize::try_from(count.max(0)).unwrap_or(usize::MAX).min(len-start));
    i64::try_from(deleted).expect(failure);
    let new_len=(len-deleted).checked_add(added).expect(failure);
    i64::try_from(new_len).expect(failure);
    (start,deleted,new_len)
}
trait TsrSplice<T>: Sized {
    fn tsr_splice<const N:usize>(&mut self,start:i64,delete:Option<i64>,added:[T;N],failure:&'static str)->Self;
    // added_count is the exact number of already staged items.
    fn tsr_splice_iter<I: IntoIterator<Item=T>>(&mut self,start:i64,delete:Option<i64>,added:I,added_count:usize,failure:&'static str)->Self;
}
impl<T> TsrSplice<T> for Vec<T> {
    fn tsr_splice<const N:usize>(&mut self,start:i64,delete:Option<i64>,added:[T;N],failure:&'static str)->Self {
        let (start,deleted,_)=tsr_splice_plan(self.len(),start,delete,N,failure);
        let mut removed=Vec::with_capacity(deleted);
        self.reserve(N.saturating_sub(deleted));
        removed.extend(self.splice(start..start+deleted,added));
        removed
    }
    fn tsr_splice_iter<I: IntoIterator<Item=T>>(&mut self,start:i64,delete:Option<i64>,added:I,added_count:usize,failure:&'static str)->Self {
        let (start,deleted,_)=tsr_splice_plan(self.len(),start,delete,added_count,failure);
        let mut removed=Vec::with_capacity(deleted);
        self.reserve(added_count.saturating_sub(deleted));
        removed.extend(self.splice(start..start+deleted,added));
        removed
    }
}
"#;
pub(crate) const SPLICE_ARRAY: &str = r#"
impl<T> TsrSplice<T> for Array<T> {
    fn tsr_splice<const N:usize>(&mut self,start:i64,delete:Option<i64>,added:[T;N],failure:&'static str)->Self {
        let original_start=start;let original_delete=delete;
        let (start,deleted,new_len)=tsr_splice_plan(self.len(),start,delete,N,failure);
        match self {
            Self::Heap(values)=>Self::Heap(values.tsr_splice(original_start,original_delete,added,failure)),
            Self::Small {len,values} if new_len<=4=>{
                let mut removed=Self::from([]);
                for slot in &mut values[start..start+deleted] { removed.push(slot.take().expect("occupied inline array slot")); }
                values[start..*len].rotate_left(deleted);
                values[start..new_len].rotate_right(N);
                for (slot,value) in values[start..start+N].iter_mut().zip(added) { *slot=Some(value); }
                *len=new_len;
                removed
            }
            Self::Small {len,values}=>{
                let mut heap=Vec::new();heap.reserve(new_len);
                let mut removed=Self::from([]);
                for slot in &mut values[..start] { heap.push(slot.take().expect("occupied inline array slot")); }
                for slot in &mut values[start..start+deleted] { removed.push(slot.take().expect("occupied inline array slot")); }
                heap.extend(added);
                for slot in &mut values[start+deleted..*len] { heap.push(slot.take().expect("occupied inline array slot")); }
                *self=Self::Heap(heap);removed
            }
        }
    }
    fn tsr_splice_iter<I: IntoIterator<Item=T>>(&mut self,start:i64,delete:Option<i64>,added:I,added_count:usize,failure:&'static str)->Self {
        let original_start=start;let original_delete=delete;
        let (start,deleted,new_len)=tsr_splice_plan(self.len(),start,delete,added_count,failure);
        match self {
            Self::Heap(values)=>Self::Heap(values.tsr_splice_iter(original_start,original_delete,added,added_count,failure)),
            Self::Small {len,values} if new_len<=4=>{
                let mut removed=Self::from([]);
                for slot in &mut values[start..start+deleted] { removed.push(slot.take().expect("occupied inline array slot")); }
                values[start..*len].rotate_left(deleted);
                values[start..new_len].rotate_right(added_count);
                for (slot,value) in values[start..start+added_count].iter_mut().zip(added) { *slot=Some(value); }
                *len=new_len;
                removed
            }
            Self::Small {len,values}=>{
                let mut heap=Vec::new();heap.reserve(new_len);
                let mut removed=Self::from([]);
                for slot in &mut values[..start] { heap.push(slot.take().expect("occupied inline array slot")); }
                for slot in &mut values[start..start+deleted] { removed.push(slot.take().expect("occupied inline array slot")); }
                heap.extend(added);
                for slot in &mut values[start+deleted..*len] { heap.push(slot.take().expect("occupied inline array slot")); }
                *self=Self::Heap(heap);removed
            }
        }
    }
}
"#;
pub(crate) const SPLICE_NUMERIC: &str = r#"
impl<T:Copy+Default> TsrSplice<T> for NumericArray<T> {
    fn tsr_splice<const N:usize>(&mut self,start:i64,delete:Option<i64>,added:[T;N],failure:&'static str)->Self {
        let original_start=start;let original_delete=delete;
        let (start,deleted,new_len)=tsr_splice_plan(self.len(),start,delete,N,failure);
        match self {
            Self::Heap(values)=>Self::Heap(values.tsr_splice(original_start,original_delete,added,failure)),
            Self::Small {len,values} if new_len<=4=>{
                let mut removed=Self::from([]);
                for value in &values[start..start+deleted] { removed.push(*value); }
                values[start..*len].rotate_left(deleted);
                for value in &mut values[*len-deleted..*len] { *value=T::default(); }
                values[start..new_len].rotate_right(N);
                values[start..start+N].copy_from_slice(&added);
                *len=new_len;removed
            }
            Self::Small {len,values}=>{
                let mut heap=Vec::new();heap.reserve(new_len);
                let mut removed=Self::from([]);
                heap.extend_from_slice(&values[..start]);
                for value in &values[start..start+deleted] { removed.push(*value); }
                heap.extend(added);heap.extend_from_slice(&values[start+deleted..*len]);
                *self=Self::Heap(heap);removed
            }
        }
    }
    fn tsr_splice_iter<I: IntoIterator<Item=T>>(&mut self,start:i64,delete:Option<i64>,added:I,added_count:usize,failure:&'static str)->Self {
        let original_start=start;let original_delete=delete;
        let (start,deleted,new_len)=tsr_splice_plan(self.len(),start,delete,added_count,failure);
        match self {
            Self::Heap(values)=>Self::Heap(values.tsr_splice_iter(original_start,original_delete,added,added_count,failure)),
            Self::Small {len,values} if new_len<=4=>{
                let mut removed=Self::from([]);
                for value in &values[start..start+deleted] { removed.push(*value); }
                values[start..*len].rotate_left(deleted);
                for value in &mut values[*len-deleted..*len] { *value=T::default(); }
                values[start..new_len].rotate_right(added_count);
                for (slot, value) in values[start..start+added_count].iter_mut().zip(added) { *slot = value; }
                *len=new_len;removed
            }
            Self::Small {len,values}=>{
                let mut heap=Vec::new();heap.reserve(new_len);
                let mut removed=Self::from([]);
                heap.extend_from_slice(&values[..start]);
                for value in &values[start..start+deleted] { removed.push(*value); }
                heap.extend(added);heap.extend_from_slice(&values[start+deleted..*len]);
                *self=Self::Heap(heap);removed
            }
        }
    }
}
"#;

pub(crate) const SORT_VEC: &str = r#"
// Compare borrowed payloads before applying any permutation. An error leaves storage intact.
fn tsr_try_sort_slice<T, E, F: FnMut(&T, &T) -> Result<std::cmp::Ordering, E>>(values: &mut [T], mut compare: F) -> Result<(), E> {
    let mut positions: Vec<usize> = (0..values.len()).collect();
    let mut failure = None;
    positions.sort_by(|a, b| {
        if failure.is_some() { return std::cmp::Ordering::Equal; }
        match compare(&values[*a], &values[*b]) {
            Ok(order) => order,
            Err(error) => { failure=Some(error); std::cmp::Ordering::Equal }
        }
    });
    if let Some(error)=failure { return Err(error); }
    let mut destinations=vec![0;positions.len()];
    for (destination, original) in positions.into_iter().enumerate() { destinations[original]=destination; }
    for position in 0..destinations.len() {
        while destinations[position] != position {
            let destination=destinations[position];
            values.swap(position,destination);
            destinations.swap(position,destination);
        }
    }
    Ok(())
}
trait TsrSort<T> {
    fn tsr_try_sort<E, F: FnMut(&T, &T) -> Result<std::cmp::Ordering, E>>(&mut self, compare: F) -> Result<(), E>;
    fn tsr_sort<F: FnMut(&T, &T) -> std::cmp::Ordering>(&mut self, compare: F);
}
impl<T> TsrSort<T> for Vec<T> {
    fn tsr_try_sort<E, F: FnMut(&T, &T) -> Result<std::cmp::Ordering, E>>(&mut self, compare: F) -> Result<(), E> { tsr_try_sort_slice(self.as_mut_slice(), compare) }
    fn tsr_sort<F: FnMut(&T, &T) -> std::cmp::Ordering>(&mut self, compare: F) {
        self.as_mut_slice().sort_by(compare);
    }
}
impl<T, const N: usize> TsrSort<T> for [T; N] {
    fn tsr_try_sort<E, F: FnMut(&T, &T) -> Result<std::cmp::Ordering, E>>(&mut self, compare: F) -> Result<(), E> { tsr_try_sort_slice(self.as_mut_slice(), compare) }
    fn tsr_sort<F: FnMut(&T, &T) -> std::cmp::Ordering>(&mut self, compare: F) {
        self.as_mut_slice().sort_by(compare);
    }
}
impl<T> TsrSort<T> for [T] {
    fn tsr_try_sort<E, F: FnMut(&T, &T) -> Result<std::cmp::Ordering, E>>(&mut self, compare: F) -> Result<(), E> { tsr_try_sort_slice(self, compare) }
    fn tsr_sort<F: FnMut(&T, &T) -> std::cmp::Ordering>(&mut self, compare: F) {
        self.sort_by(compare);
    }
}
"#;
pub(crate) const SORT_ARRAY: &str = r#"
impl<T> TsrSort<T> for Array<T> {
    fn tsr_try_sort<E, F: FnMut(&T, &T) -> Result<std::cmp::Ordering, E>>(&mut self, mut compare: F) -> Result<(), E> {
        match self {
            Self::Small { len, values } => tsr_try_sort_slice(&mut values[..*len], |a,b| compare(a.as_ref().expect("live array slot"), b.as_ref().expect("live array slot"))),
            Self::Heap(values) => tsr_try_sort_slice(values.as_mut_slice(), compare),
        }
    }
    fn tsr_sort<F: FnMut(&T, &T) -> std::cmp::Ordering>(&mut self, mut compare: F) {
        match self {
            Self::Small { len, values } => values[..*len].sort_by(|a, b| compare(a.as_ref().expect("live array slot"), b.as_ref().expect("live array slot"))),
            Self::Heap(values) => values.sort_by(compare),
        }
    }
}
"#;
pub(crate) const SORT_NUMERIC: &str = r#"
impl<T: Copy + Default> TsrSort<T> for NumericArray<T> {
    fn tsr_try_sort<E, F: FnMut(&T, &T) -> Result<std::cmp::Ordering, E>>(&mut self, compare: F) -> Result<(), E> { tsr_try_sort_slice(self.as_mut_slice(), compare) }
    fn tsr_sort<F: FnMut(&T, &T) -> std::cmp::Ordering>(&mut self, compare: F) {
        self.as_mut_slice().sort_by(compare);
    }
}
"#;

// Standard iterator adapters over existing owning storage. No payload is cloned.
pub(crate) const OWNED_ITEMS_VEC: &str = r#"
trait TsrOwnedItems<T> { fn tsr_owned_items(self) -> impl Iterator<Item=T>; }
impl<T> TsrOwnedItems<T> for Vec<T> { fn tsr_owned_items(self) -> impl Iterator<Item=T> { self.into_iter() } }
impl<T, const N: usize> TsrOwnedItems<T> for [T; N] { fn tsr_owned_items(self) -> impl Iterator<Item=T> { self.into_iter() } }
fn tsr_flat_capacity(current: usize, additional: usize, failure: &'static str) -> usize {
    let length=current.checked_add(additional).expect(failure);
    i64::try_from(length).expect(failure);
    length
}
"#;
pub(crate) const OWNED_ITEMS_ARRAY: &str = r#"
impl<T> TsrOwnedItems<T> for Array<T> {
    fn tsr_owned_items(self) -> impl Iterator<Item=T> {
        let (slots,len,heap)=match self {
            Self::Small { values,len } => (values,len,Vec::new()),
            Self::Heap(values) => ([const { None };4],0,values),
        };
        slots.into_iter().take(len).map(|slot|slot.expect("occupied inline array slot")).chain(heap)
    }
}
"#;
pub(crate) const OWNED_ITEMS_NUMERIC: &str = r#"
impl<T:Copy+Default> TsrOwnedItems<T> for NumericArray<T> {
    fn tsr_owned_items(self) -> impl Iterator<Item=T> {
        let (slots,len,heap)=match self {
            Self::Small { values,len } => (values,len,Vec::new()),
            Self::Heap(values) => ([T::default();4],0,values),
        };
        slots.into_iter().take(len).chain(heap)
    }
}
"#;

// Standard disjoint mutable iterators lend existing owners without cloning or
// changing the user array representation.
pub(crate) const SPREAD_MUT_ITEMS_VEC: &str = r#"
trait TsrSpreadMutItems<T> {
    fn tsr_spread_mut_items<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T> where T: 'a;
}
impl<T> TsrSpreadMutItems<T> for Vec<T> {
    fn tsr_spread_mut_items<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T> where T: 'a { self.iter_mut() }
}
impl<T, const N: usize> TsrSpreadMutItems<T> for [T; N] {
    fn tsr_spread_mut_items<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T> where T: 'a { self.iter_mut() }
}
"#;
pub(crate) const SPREAD_MUT_ITEMS_ARRAY: &str = r#"
impl<T> TsrSpreadMutItems<T> for Array<T> {
    fn tsr_spread_mut_items<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T> where T: 'a {
        let (slots, heap): (&mut [Option<T>], &mut [T]) = match self {
            Self::Small { len, values } => (&mut values[..*len], &mut []),
            Self::Heap(values) => (&mut [], values.as_mut_slice()),
        };
        slots.iter_mut().map(|slot| slot.as_mut().expect("occupied inline array slot")).chain(heap.iter_mut())
    }
}
"#;
pub(crate) const SPREAD_MUT_ITEMS_NUMERIC: &str = r#"
impl<T: Copy + Default> TsrSpreadMutItems<T> for NumericArray<T> {
    fn tsr_spread_mut_items<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T> where T: 'a {
        let values = match self {
            Self::Small { len, values } => &mut values[..*len],
            Self::Heap(values) => values.as_mut_slice(),
        };
        values.iter_mut()
    }
}
"#;

pub(crate) const STRING_NORMALIZATION: &str = concat!(include_str!("runtime_normalization.rs"), include_str!("runtime_normalization_tables.rs"));

/// Typed completion support for exception lowering; payloads transfer by value.
pub(crate) const EXCEPTION_COMPLETIONS: &str = include_str!("runtime_completion.rs");
