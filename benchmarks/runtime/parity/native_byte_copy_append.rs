// Independent standard Rust native storage ownership and collection operations.
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
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn length<T>(values: &[T]) -> i64 {
    i64::try_from(values.len()).unwrap()
}

#[derive(Debug)]
struct ByteError {
    code: &'static str,
    offset: Option<i64>,
}
fn error(code: &'static str, offset: Option<i64>) -> ByteError {
    ByteError { code, offset }
}
fn code(error: &ByteError) -> i64 {
    i64::try_from(error.code.len()).unwrap()
}
fn validate(values: &[i64]) -> Result<(), ByteError> {
    for (at, value) in values.iter().enumerate() {
        if !(0..=255).contains(value) {
            return Err(error("byte-range", Some(i64::try_from(at).unwrap())));
        }
    }
    Ok(())
}
fn from_array(values: &[i64]) -> Result<Vec<u8>, ByteError> {
    validate(values)?;
    Ok(values.iter().map(|v| *v as u8).collect())
}
fn at(bytes: &[u8], index: i64) -> Result<i64, ByteError> {
    usize::try_from(index)
        .ok()
        .and_then(|at| bytes.get(at))
        .copied()
        .map(i64::from)
        .ok_or_else(|| error("index-range", Some(index)))
}
fn write(bytes: &mut [u8], index: i64, value: i64) -> Result<(), ByteError> {
    let target = usize::try_from(index)
        .ok()
        .and_then(|at| bytes.get_mut(at))
        .ok_or_else(|| error("index-range", Some(index)))?;
    let value = u8::try_from(value).map_err(|_| error("byte-range", Some(index)))?;
    *target = value;
    Ok(())
}
fn read(bytes: &[u8], index: i64) -> i64 {
    at(bytes, index).unwrap_or(-1)
}
fn fill_byte(fill: i64) -> Result<u8, ByteError> {
    u8::try_from(fill).map_err(|_| error("byte-range", None))
}
fn alloc(count: i64, fill: i64) -> Result<Vec<u8>, ByteError> {
    let count = usize::try_from(count).map_err(|_| error("length-range", None))?;
    let fill = fill_byte(fill)?;
    Ok(vec![fill; count])
}
fn resize(bytes: &mut Vec<u8>, count: i64, fill: i64) -> Result<(), ByteError> {
    let count = usize::try_from(count).map_err(|_| error("length-range", None))?;
    let fill = fill_byte(fill)?;
    bytes.resize(count, fill);
    Ok(())
}
fn append(bytes: &mut Vec<u8>, values: &[i64]) -> Result<(), ByteError> {
    bytes
        .len()
        .checked_add(values.len())
        .ok_or_else(|| error("length-range", None))?;
    validate(values)?;
    bytes.extend(values.iter().map(|v| *v as u8));
    Ok(())
}
fn copy(bytes: &[u8], start: i64, end: i64) -> Result<Vec<u8>, ByteError> {
    let len = length(bytes);
    if start < 0 || start > len {
        return Err(error("range", Some(start)));
    }
    if end < start || end > len {
        return Err(error("range", Some(end)));
    }
    Ok(bytes[usize::try_from(start).unwrap()..usize::try_from(end).unwrap()].to_vec())
}
fn from_utf8(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}
fn to_utf8(bytes: &[u8]) -> Result<std::string::String, ByteError> {
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|e| error("utf8", Some(i64::try_from(e.valid_up_to()).unwrap())))
}
fn invalid_utf8(values: &[i64]) -> Vec<i64> {
    let bytes = from_array(values).unwrap();
    match to_utf8(&bytes) {
        Ok(_) => vec![-1, -1],
        Err(e) => vec![code(&e), e.offset.unwrap_or(-1)],
    }
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let byte = add(i, value1) % 256;
        let mut bytes = from_array(&[byte, 65]).unwrap();
        let initial = length(&bytes);
        let success = if append(&mut bytes, &[255, 7]).is_ok() {
            1
        } else {
            -2
        };
        let failure = append(&mut bytes, &[8, 256]).unwrap_err();
        let preserved = read(&bytes, 3);
        let mut part = copy(&bytes, 0, 2).unwrap();
        let owner_write = if write(&mut bytes, 0, add(byte, 1) % 256).is_ok() {
            1
        } else {
            -1
        };
        let independent = read(&part, 0);
        let part_write = if write(&mut part, 1, 9).is_ok() {
            1
        } else {
            -1
        };
        let owner_retained = read(&bytes, 1);
        let empty = length(&copy(&bytes, 4, 4).unwrap());
        let range_offset = copy(&bytes, 0, 5).unwrap_err().offset.unwrap_or(-3);
        output.push(NumericArray(vec![
            initial,
            success,
            code(&failure),
            failure.offset.unwrap_or(-3),
            preserved,
            length(&part),
            owner_write,
            independent,
            part_write,
            owner_retained,
            empty,
            range_offset,
            length(&bytes),
        ]));
    }
    Array(output)
}
fn main() {}
