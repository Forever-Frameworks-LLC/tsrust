//! Independent owned byte construction; no shared allocation or implicit clone.
pub struct ByteFailure {
    pub code: &'static str,
    pub message: &'static str,
    pub offset: Option<i64>,
}
pub fn from_array(values: &[i64]) -> Result<Vec<u8>, ByteFailure> {
    if i64::try_from(values.len()).is_err() {
        return Err(ByteFailure { code: "length-range", message: "Byte length exceeds the int range.", offset: None });
    }
    if let Some((index, _)) = values.iter().enumerate().find(|(_, value)| !(0..=255).contains(*value)) {
        return Err(ByteFailure { code: "byte-range", message: "Byte value must be between 0 and 255.", offset: Some(index as i64) });
    }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(values.len()).map_err(|_| ByteFailure {
        code: "allocation", message: "Unable to allocate byte buffer.", offset: None,
    })?;
    bytes.extend(values.iter().map(|value| *value as u8));
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn construction_validates_before_copying_and_reports_first_offset() {
        let input = [0, 65, 255];
        let mut bytes = from_array(&input).unwrap_or_else(|_| panic!("valid bytes"));
        assert_eq!(bytes, input.map(|value| value as u8));
        bytes[0] = 7;
        assert_eq!(input[0], 0);
        assert!(from_array(&[]).is_ok_and(|bytes| bytes.is_empty()));
        for input in [&[-1, 256][..], &[4, 256, -1][..]] {
            let error = from_array(input).err().expect("invalid byte");
            assert_eq!(error.code, "byte-range");
            assert_eq!(error.message, "Byte value must be between 0 and 255.");
            assert_eq!(error.offset, Some(if input[0] == -1 { 0 } else { 1 }));
        }
    }
}

fn index_failure(index: i64) -> ByteFailure {
    ByteFailure { code: "index-range", message: "Byte index must be within the buffer.", offset: Some(index) }
}
pub fn at(bytes: &[u8], index: i64) -> Result<i64, ByteFailure> {
    usize::try_from(index).ok().and_then(|index| bytes.get(index)).map(|value| i64::from(*value)).ok_or_else(|| index_failure(index))
}
pub fn set(bytes: &mut [u8], index: i64, value: i64) -> Result<(), ByteFailure> {
    let Some(slot) = usize::try_from(index).ok().and_then(|index| bytes.get_mut(index)) else { return Err(index_failure(index)); };
    if !(0..=255).contains(&value) {
        return Err(ByteFailure { code: "byte-range", message: "Byte value must be between 0 and 255.", offset: Some(index) });
    }
    *slot = value as u8;
    Ok(())
}

#[cfg(test)]
mod access_tests {
    use super::*;
    #[test]
    fn accesses_are_checked_and_failed_writes_are_atomic() {
        let mut bytes = [0, 65, 255];
        for (index, expected) in [(0, 0), (2, 255)] {
            assert_eq!(at(&bytes, index).unwrap_or_else(|_| panic!("valid index")), expected);
        }
        for index in [-1, 3, i64::MAX] {
            let error = at(&bytes, index).err().expect("invalid index");
            assert_eq!((error.code, error.offset), ("index-range", Some(index)));
            let error = set(&mut bytes, index, 256).err().expect("index checked before value");
            assert_eq!((error.code, error.offset), ("index-range", Some(index)));
            assert_eq!(bytes, [0, 65, 255]);
        }
        for value in [-1, 256] {
            let error = set(&mut bytes, 1, value).err().expect("invalid byte");
            assert_eq!((error.code, error.offset), ("byte-range", Some(1)));
            assert_eq!(bytes, [0, 65, 255]);
        }
        assert!(set(&mut bytes, 1, 200).is_ok());
        assert_eq!(bytes, [0, 200, 255]);
        assert!(at(&[], 0).is_err());
        assert!(set(&mut [], 0, 0).is_err());
    }
}

fn allocation_failure() -> ByteFailure {
    ByteFailure { code: "allocation", message: "Unable to allocate byte buffer.", offset: None }
}
fn checked_length(length: i64) -> Result<usize, ByteFailure> {
    usize::try_from(length).ok().filter(|length| *length <= isize::MAX as usize).ok_or(ByteFailure {
        code: "length-range", message: "Byte length must be nonnegative and fit native storage.", offset: None,
    })
}
fn checked_fill(fill: Option<i64>) -> Result<u8, ByteFailure> {
    let fill = fill.unwrap_or(0);
    if !(0..=255).contains(&fill) {
        return Err(ByteFailure { code: "byte-range", message: "Byte value must be between 0 and 255.", offset: None });
    }
    Ok(fill as u8)
}
fn reserve(bytes: &mut Vec<u8>, additional: usize) -> Result<(), ByteFailure> {
    bytes.try_reserve_exact(additional).map_err(|_| allocation_failure())
}
fn resize_with(bytes: &mut Vec<u8>, length: i64, fill: Option<i64>, reserve: impl FnOnce(&mut Vec<u8>, usize) -> Result<(), ByteFailure>) -> Result<(), ByteFailure> {
    let length = checked_length(length)?;
    let fill = checked_fill(fill)?;
    if length > bytes.len() { reserve(bytes, length - bytes.len())?; }
    bytes.resize(length, fill);
    Ok(())
}
pub fn alloc(length: i64, fill: Option<i64>) -> Result<Vec<u8>, ByteFailure> {
    let mut bytes = Vec::new();
    resize_with(&mut bytes, length, fill, reserve)?;
    Ok(bytes)
}
pub fn resize(bytes: &mut Vec<u8>, length: i64, fill: Option<i64>) -> Result<(), ByteFailure> {
    resize_with(bytes, length, fill, reserve)
}

#[cfg(test)]
mod growth_tests {
    use super::*;
    #[test]
    fn allocation_and_resize_validate_defaults_and_preserve_prefixes() {
        let mut bytes = alloc(3, None).unwrap_or_else(|_| panic!("valid allocation"));
        assert_eq!(bytes, [0, 0, 0]);
        bytes[0] = 7;
        assert!(resize(&mut bytes, 5, Some(255)).is_ok());
        assert_eq!(bytes, [7, 0, 0, 255, 255]);
        for (length, fill, code) in [(-1, Some(256), "length-range"), (0, Some(-1), "byte-range"), (5, Some(256), "byte-range")] {
            let error = resize(&mut bytes, length, fill).err().expect("invalid argument");
            assert_eq!((error.code, error.offset), (code, None));
            assert_eq!(bytes, [7, 0, 0, 255, 255]);
        }
        assert!(resize(&mut bytes, 1, None).is_ok());
        assert_eq!(bytes, [7]);
        assert!(resize(&mut bytes, 3, None).is_ok());
        assert_eq!(bytes, [7, 0, 0]);
        assert!(resize(&mut bytes, 0, None).is_ok());
        assert!(bytes.is_empty());
        assert!(alloc(0, None).is_ok_and(|bytes| bytes.is_empty()));
        assert!(alloc(2, Some(255)).is_ok_and(|bytes| bytes == [255, 255]));
    }
    #[test]
    fn failed_reservation_cannot_change_length_contents_pointer_or_capacity() {
        let mut bytes = vec![7, 65, 255];
        let pointer = bytes.as_ptr();
        let capacity = bytes.capacity();
        let error = resize_with(&mut bytes, 1024, Some(1), |old, additional| {
            assert_eq!(old.as_slice(), [7, 65, 255]);
            assert_eq!(additional, 1021);
            Err(allocation_failure())
        }).err().expect("forced private reservation failure");
        assert_eq!((error.code, error.offset), ("allocation", None));
        assert_eq!(bytes, [7, 65, 255]);
        assert_eq!(bytes.as_ptr(), pointer);
        assert_eq!(bytes.capacity(), capacity);
        for (length, fill) in [(-1, Some(256)), (0, Some(256))] {
            assert!(resize_with(&mut bytes, length, fill, |_, _| panic!("validation must precede reservation")).is_err());
        }
    }
}

fn append_with(bytes: &mut Vec<u8>, values: &[i64], reserve: impl FnOnce(&mut Vec<u8>, usize) -> Result<(), ByteFailure>) -> Result<(), ByteFailure> {
    let length = bytes.len().checked_add(values.len()).and_then(|length| i64::try_from(length).ok()).ok_or(ByteFailure {
        code: "length-range", message: "Byte length must be nonnegative and fit native storage.", offset: None,
    })?;
    checked_length(length)?;
    if let Some((index, _)) = values.iter().enumerate().find(|(_, value)| !(0..=255).contains(*value)) {
        return Err(ByteFailure { code: "byte-range", message: "Byte value must be between 0 and 255.", offset: Some(index as i64) });
    }
    if !values.is_empty() { reserve(bytes, values.len())?; }
    bytes.extend(values.iter().map(|value| *value as u8));
    Ok(())
}
pub fn append(bytes: &mut Vec<u8>, values: &[i64]) -> Result<(), ByteFailure> {
    append_with(bytes, values, reserve)
}
fn copy_range(bytes: &[u8], start: i64, end: i64) -> Result<&[u8], ByteFailure> {
    let invalid = |offset| ByteFailure { code: "range", message: "Byte range must satisfy 0 <= start <= end <= length.", offset: Some(offset) };
    let start_index = usize::try_from(start).ok().filter(|index| *index <= bytes.len()).ok_or_else(|| invalid(start))?;
    let end_index = usize::try_from(end).ok().filter(|index| *index >= start_index && *index <= bytes.len()).ok_or_else(|| invalid(end))?;
    Ok(&bytes[start_index..end_index])
}
fn copy_with(bytes: &[u8], start: i64, end: i64, reserve: impl FnOnce(&mut Vec<u8>, usize) -> Result<(), ByteFailure>) -> Result<Vec<u8>, ByteFailure> {
    let source = copy_range(bytes, start, end)?;
    let mut output = Vec::new();
    if !source.is_empty() { reserve(&mut output, source.len())?; }
    output.extend_from_slice(source);
    Ok(output)
}
pub fn copy(bytes: &[u8], start: i64, end: i64) -> Result<Vec<u8>, ByteFailure> {
    copy_with(bytes, start, end, reserve)
}

#[cfg(test)]
mod copy_append_tests {
    use super::*;
    #[test]
    fn append_validates_before_reserving_and_preserves_owner_on_failure() {
        let mut bytes = vec![0, 65];
        let pointer = bytes.as_ptr();
        let capacity = bytes.capacity();
        let error = append_with(&mut bytes, &[7, 256, -1], |_, _| panic!("invalid bytes must not reserve")).err().expect("invalid input");
        assert_eq!((error.code, error.offset), ("byte-range", Some(1)));
        let error = append_with(&mut bytes, &[255], |old, additional| {
            assert_eq!(old, &[0, 65]); assert_eq!(additional, 1); Err(allocation_failure())
        }).err().expect("private forced reservation failure");
        assert_eq!((error.code, error.offset), ("allocation", None));
        assert_eq!(bytes, [0, 65]); assert_eq!(bytes.as_ptr(), pointer); assert_eq!(bytes.capacity(), capacity);
        assert!(append_with(&mut bytes, &[], |_, _| panic!("empty append needs no reservation")).is_ok());
        assert!(append(&mut bytes, &[0, 255]).is_ok()); assert_eq!(bytes, [0, 65, 0, 255]);
    }
    #[test]
    fn copied_ranges_are_checked_independent_and_fallible() {
        let mut bytes = vec![0, 65, 255];
        let mut part = copy(&bytes, 1, 3).unwrap_or_else(|_| panic!("valid range"));
        bytes[1] = 7; part[1] = 8; assert_eq!(part, [65, 8]); assert_eq!(bytes, [0, 7, 255]);
        for (start, end, offset) in [(-1, 99, -1), (4, 0, 4), (0, 4, 4), (2, 1, 1), (0, -1, -1), (0, i64::MAX, i64::MAX)] {
            let error = copy_with(&bytes, start, end, |_, _| panic!("bad range must not reserve")).err().expect("invalid range");
            assert_eq!((error.code, error.offset), ("range", Some(offset)));
        }
        assert!(copy_with(&bytes, 3, 3, |_, _| panic!("empty copy needs no reservation")).is_ok_and(|value| value.is_empty()));
        assert!(copy(&[], 0, 0).is_ok_and(|value| value.is_empty()));
        let error = copy_with(&bytes, 0, 3, |_, _| Err(allocation_failure())).err().expect("forced reservation failure");
        assert_eq!((error.code, error.offset), ("allocation", None)); assert_eq!(bytes, [0, 7, 255]);
    }
}
