// Independent safe Rust byte views: slices borrow live owners; no shared backing allocation.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn length<T>(values: &[T]) -> i64 {
    i64::try_from(values.len()).unwrap()
}
#[derive(Debug)]
struct Error {
    code: &'static str,
    offset: i64,
}
fn range(len: usize, start: i64, end: i64) -> Result<std::ops::Range<usize>, Error> {
    let len = i64::try_from(len).unwrap();
    if start < 0 || start > len {
        return Err(Error {
            code: "range",
            offset: start,
        });
    }
    if end < start || end > len {
        return Err(Error {
            code: "range",
            offset: end,
        });
    }
    Ok(usize::try_from(start).unwrap()..usize::try_from(end).unwrap())
}
fn view(bytes: &[u8], start: i64, end: i64) -> Result<&[u8], Error> {
    let range = range(bytes.len(), start, end)?;
    Ok(&bytes[range])
}
fn view_mut(bytes: &mut [u8], start: i64, end: i64) -> Result<&mut [u8], Error> {
    let range = range(bytes.len(), start, end)?;
    Ok(&mut bytes[range])
}
fn at(bytes: &[u8], index: i64) -> Result<i64, Error> {
    usize::try_from(index)
        .ok()
        .and_then(|i| bytes.get(i))
        .copied()
        .map(i64::from)
        .ok_or(Error {
            code: "index-range",
            offset: index,
        })
}
fn set(bytes: &mut [u8], index: i64, value: i64) -> Result<(), Error> {
    let target = usize::try_from(index)
        .ok()
        .and_then(|i| bytes.get_mut(i))
        .ok_or(Error {
            code: "index-range",
            offset: index,
        })?;
    *target = u8::try_from(value).map_err(|_| Error {
        code: "byte-range",
        offset: index,
    })?;
    Ok(())
}
fn from_array(values: &[i64]) -> Vec<u8> {
    values.iter().map(|v| u8::try_from(*v).unwrap()).collect()
}
fn window(owner: &[u8]) -> &[u8] {
    view(owner, 0, 2).unwrap()
}
fn first(part: &[u8]) -> i64 {
    at(part, 0).unwrap()
}
use std::collections::BTreeMap;
// Keys are inserted uniquely in ascending order; BTreeMap retains the observed order.
fn fill<'a>(values: &mut BTreeMap<i64, &'a [u8]>, owner: &'a [u8]) {
    for key in 0..8 {
        values.insert(key, window(owner));
    }
}
fn collect(owner: &[u8]) -> BTreeMap<i64, &[u8]> {
    let mut values = BTreeMap::new();
    fill(&mut values, owner);
    values
}
fn exposed(owner: &[u8]) -> &[u8] {
    let values = collect(owner);
    let selected = *values.get(&0).unwrap();
    view(selected, 0, 2).unwrap()
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let byte = add(i, value1) % 256;
        let owner = from_array(&[byte, 255]);
        let values = collect(&owner);
        let mut keys = 0;
        let mut sum = 0;
        for (key, part) in &values {
            keys = add(keys, *key);
            sum = add(sum, first(part));
        }
        let part = exposed(&owner);
        output.push(NumericArray(vec![
            i64::try_from(values.len()).unwrap(),
            keys,
            sum,
            first(part),
            length(part),
        ]));
    }
    Array(output)
}
fn main() {}
