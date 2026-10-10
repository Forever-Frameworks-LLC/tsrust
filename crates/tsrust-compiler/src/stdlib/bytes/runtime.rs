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
