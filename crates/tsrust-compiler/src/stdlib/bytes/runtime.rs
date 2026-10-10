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
