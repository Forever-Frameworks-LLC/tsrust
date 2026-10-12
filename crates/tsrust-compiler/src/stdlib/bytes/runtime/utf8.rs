// UTF-8 copies validate once and reserve before exposing an owned result.
fn from_utf8_with(text: &str, reserve: impl FnOnce(&mut Vec<u8>, usize) -> Result<(), ByteFailure>) -> Result<Vec<u8>, ByteFailure> {
    let length = i64::try_from(text.len()).map_err(|_| ByteFailure { code: "length-range", message: "Byte length must be nonnegative and fit native storage.", offset: None })?;
    checked_length(length)?;
    let mut output = Vec::new();
    if !text.is_empty() { reserve(&mut output, text.len())?; }
    output.extend_from_slice(text.as_bytes());
    Ok(output)
}
pub fn from_utf8(text: &str) -> Result<Vec<u8>, ByteFailure> {
    from_utf8_with(text, reserve)
}
fn to_utf8_with(bytes: &[u8], reserve: impl FnOnce(&mut std::string::String, usize) -> Result<(), ByteFailure>) -> Result<std::string::String, ByteFailure> {
    let text = std::str::from_utf8(bytes).map_err(|error| ByteFailure {
        code: "utf8", message: "Byte buffer contains invalid UTF-8.", offset: Some(error.valid_up_to() as i64),
    })?;
    let mut output = std::string::String::new();
    if !text.is_empty() { reserve(&mut output, text.len())?; }
    output.push_str(text);
    Ok(output)
}
pub fn to_utf8(bytes: &[u8]) -> Result<std::string::String, ByteFailure> {
    to_utf8_with(bytes, |output, length| output.try_reserve_exact(length).map_err(|_| allocation_failure()))
}

#[cfg(test)]
mod utf8_tests {
    use super::*;
    #[test]
    fn round_trips_preserve_unicode_nul_bom_and_empty() {
        for text in ["", "ASCII", "é😀", "A\0B", "\u{FEFF}z"] {
            let bytes = from_utf8(text).unwrap_or_else(|_| panic!("valid text"));
            assert_eq!(bytes, text.as_bytes());
            assert_eq!(to_utf8(&bytes).unwrap_or_else(|_| panic!("valid bytes")), text);
        }
        assert!(from_utf8_with("", |_, _| panic!("empty encoding needs no reservation")).is_ok());
        assert!(to_utf8_with(&[], |_, _| panic!("empty decoding needs no reservation")).is_ok());
    }
    #[test]
    fn invalid_sequences_report_their_start_before_reservation() {
        for (bytes, offset) in [(&[65,195,40][..],1),(&[65,226,130][..],1),(&[192,175][..],0),(&[237,160,128][..],0),(&[244,144,128,128][..],0),(&[128][..],0)] {
            let error = to_utf8_with(bytes, |_, _| panic!("invalid UTF-8 must not reserve")).err().expect("invalid encoding");
            assert_eq!((error.code,error.offset),("utf8",Some(offset)));
            assert_eq!(error.message,"Byte buffer contains invalid UTF-8.");
        }
    }
    #[test]
    fn reservation_failures_do_not_expose_partial_owned_results() {
        let error = from_utf8_with("é😀", |output,length| {assert!(output.is_empty());assert_eq!(length,6);Err(allocation_failure())}).err().expect("forced encoding reservation failure");
        assert_eq!((error.code,error.offset),("allocation",None));
        let error = to_utf8_with("é😀".as_bytes(), |output,length| {assert!(output.is_empty());assert_eq!(length,6);Err(allocation_failure())}).err().expect("forced decoding reservation failure");
        assert_eq!((error.code,error.offset),("allocation",None));
    }
}
