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
fn make<'a>(captured: &'a [u8], expected: i64) -> impl FnOnce(&'a [u8]) -> &'a [u8] {
    move |other| {
        let kept = captured;
        assert_eq!(first(kept), expected, "capture mismatch");
        other
    }
}
fn make_owner<'a>(captured: &'a [u8], expected: i64) -> impl FnOnce(&'a [u8]) -> &'a [u8] {
    move |other| {
        let kept = captured;
        assert_eq!(first(kept), expected, "capture mismatch");
        view(other, 1, 3).unwrap()
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let byte = add(i, value1) % 256;
        let next = add(byte, 1) % 256;
        let owner = from_array(&[byte, next, 255]);
        let picker = make(view(&owner, 0, 1).unwrap(), byte);
        let selected = picker(view(&owner, 1, 3).unwrap());
        let borrowed = make_owner(view(&owner, 0, 1).unwrap(), byte)(&owner);
        output.push(NumericArray(vec![
            length(selected),
            first(selected),
            length(borrowed),
            first(borrowed),
        ]));
    }
    Array(output)
}
fn main() {}
