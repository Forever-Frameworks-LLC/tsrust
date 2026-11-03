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
// Allocation offsets are absent; -3 is the workload's coalesced offset.
fn reserve(bytes: &mut Vec<u8>, additional: usize) -> Result<(), Error> {
    #[cfg(test)]
    if additional > bytes.capacity() - bytes.len() && tests::fail_reservation() {
        return Err(Error { code: "allocation", offset: -3 });
    }
    bytes.try_reserve_exact(additional).map_err(|_| Error { code: "allocation", offset: -3 })
}
fn copy(bytes: &[u8], start: i64, end: i64) -> Result<Vec<u8>, Error> {
    let source = view(bytes, start, end)?;
    let mut output = Vec::new();
    if !source.is_empty() { reserve(&mut output, source.len())?; }
    output.extend_from_slice(source);
    Ok(output)
}
fn from_array(values: &[i64]) -> Result<Vec<u8>, Error> {
    if let Some((index,_))=values.iter().enumerate().find(|(_,value)|!(0..=255).contains(*value)) {
        return Err(Error { code: "byte-range", offset: index as i64 });
    }
    let mut bytes = Vec::new();
    reserve(&mut bytes, values.len())?;
    bytes.extend(values.iter().map(|value| *value as u8));
    Ok(bytes)
}
fn window(owner: &[u8]) -> &[u8] {
    view(owner, 0, 2).unwrap()
}
fn first(part: &[u8]) -> i64 {
    at(part, 0).unwrap()
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let byte = add(i, value1) % 256;
        let next = add(byte, 1) % 256;
        let written = add(byte, 2) % 256;
        let mut bytes = match from_array(&[0, byte, next, 255, 7]) {
            Ok(bytes) => bytes,
            Err(_) => {
                output.push(NumericArray(vec![-1000000]));
                continue;
            }
        };
        let (size, head, tail, inner, copied) = {
            let part = view(&bytes, 1, 4).unwrap();
            let nested = view(part, 1, 2).unwrap();
            let copied = match copy(part, 0, 2) {
                Ok(owned) => length(&owned),
                Err(_) => -2,
            };
            (
                length(part),
                first(part),
                at(part, 2).unwrap(),
                first(nested),
                copied,
            )
        };
        let (success, write_code, write_offset) = {
            let part = view_mut(&mut bytes, 1, 3).unwrap();
            let success = if set(part, 1, written).is_ok() { 1 } else { -2 };
            let error = set(part, 2, written).unwrap_err();
            (
                success,
                i64::try_from(error.code.len()).unwrap(),
                error.offset,
            )
        };
        let through = at(&bytes, 2).unwrap();
        let error = view(&bytes, 4, 9).unwrap_err();
        let reversed = view(&bytes, 3, 1).unwrap_err().offset;
        output.push(NumericArray(vec![
            size,
            head,
            tail,
            inner,
            copied,
            success,
            write_code,
            write_offset,
            through,
            i64::try_from(error.code.len()).unwrap(),
            error.offset,
            reversed,
        ]));
    }
    Array(output)
}
fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    std::thread_local! { static FAIL_AFTER: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) }; }
    pub(super) fn fail_reservation() -> bool {
        FAIL_AFTER.with(|remaining| match remaining.get() {
            Some(0) => { remaining.set(None); true },
            Some(n) => { remaining.set(Some(n - 1)); false },
            None => false,
        })
    }
    #[test]
    fn allocation_errors_preserve_construction_and_copy_fallbacks_then_recover() {
        let expected = function0(3, 7, &String::Borrowed("bytes")).0;
        for (after, field) in [(0, None), (1, Some(4))] {
            FAIL_AFTER.with(|remaining| remaining.set(Some(after)));
            let actual = function0(3, 7, &String::Borrowed("bytes")).0;
            match field {
                None => assert_eq!(actual[0].0, vec![-1000000]),
                Some(field) => {
                    let mut row = expected[0].0.clone(); row[field] = -2;
                    assert_eq!(actual[0].0, row);
                },
            }
            assert_eq!(actual[1].0, expected[1].0);
            assert_eq!(actual[2].0, expected[2].0);
        }
        FAIL_AFTER.with(|remaining| remaining.set(Some(0)));
        assert_eq!(copy(&[], 0, 0).unwrap(), Vec::<u8>::new());
        assert_eq!(from_array(&[256]).unwrap_err().code, "byte-range");
        assert_eq!(from_array(&[0]).unwrap_err().code, "allocation");
        assert_eq!(from_array(&[0]).unwrap(), vec![0]);
    }
    #[test]
    fn construction_errors_keep_the_workload_fallback() {
        assert_eq!(from_array(&[0,255]).unwrap(),vec![0,255]);
        let error=from_array(&[0,256,-1]).unwrap_err();
        assert_eq!((error.code,error.offset),("byte-range",1));
        let rows=function0(3,-1,&String::Borrowed("bytes")).0;
        assert_eq!(rows[0].0,vec![-1000000]);
        assert_eq!(rows[1].0,vec![3,0,255,1,2,1,11,2,2,5,9,1]);
        assert_eq!(rows[2].0,vec![3,1,255,2,2,1,11,2,3,5,9,1]);
    }
}
