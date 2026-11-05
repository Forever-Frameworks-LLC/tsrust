// Encoders own one output buffer; observers borrow the original byte storage.
fn codec_length(length: usize) -> Result<usize, ByteFailure> {
    let length = i64::try_from(length).map_err(|_| ByteFailure { code: "length-range", message: "Encoded length exceeds the int range.", offset: None })?;
    checked_length(length)
}
fn codec_buffer(length: usize) -> Result<Vec<u8>, ByteFailure> {
    let length = codec_length(length)?;
    let mut output = Vec::new();
    if length != 0 { reserve(&mut output, length)?; }
    Ok(output)
}
pub fn to_hex(bytes: &[u8]) -> Result<std::string::String, ByteFailure> {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let length = bytes.len().checked_mul(2).ok_or(ByteFailure { code: "length-range", message: "Encoded length exceeds native storage.", offset: None })?;
    let mut output = codec_buffer(length)?;
    for byte in bytes { output.push(DIGITS[usize::from(byte >> 4)]); output.push(DIGITS[usize::from(byte & 15)]); }
    Ok(std::string::String::from_utf8(output).expect("hex output is ASCII"))
}
pub fn from_hex(text: &str) -> Result<Vec<u8>, ByteFailure> {
    if text.len() % 2 != 0 { return Err(ByteFailure { code: "hex-length", message: "Hex text must contain an even number of bytes.", offset: None }); }
    let digit = |byte: u8, offset: usize| -> Result<u8, ByteFailure> {
        match byte { b'0'..=b'9' => Ok(byte - b'0'), b'a'..=b'f' => Ok(byte - b'a' + 10), b'A'..=b'F' => Ok(byte - b'A' + 10), _ => Err(ByteFailure { code: "hex-digit", message: "Hex text contains a non-hexadecimal byte.", offset: Some(offset as i64) }) }
    };
    let mut output = codec_buffer(text.len() / 2)?;
    for (index, pair) in text.as_bytes().chunks_exact(2).enumerate() { output.push((digit(pair[0], index * 2)? << 4) | digit(pair[1], index * 2 + 1)?); }
    Ok(output)
}
pub fn to_utf8_lossy(bytes: &[u8]) -> Result<std::string::String, ByteFailure> {
    match std::string::String::from_utf8_lossy(bytes) {
        std::borrow::Cow::Owned(output) => Ok(output),
        std::borrow::Cow::Borrowed(text) => {
            let mut output = std::string::String::new();
            output.try_reserve_exact(text.len()).map_err(|_| allocation_failure())?;
            output.push_str(text); Ok(output)
        },
    }
}
fn codec_little_endian(order: &str) -> Result<bool, ByteFailure> {
    match order { "little" => Ok(true), "big" => Ok(false), _ => Err(ByteFailure { code: "byte-order", message: "Byte order must be little or big.", offset: None }) }
}
pub fn from_utf16(text: &str, order: &str) -> Result<Vec<u8>, ByteFailure> {
    let little = codec_little_endian(order)?;
    let length = text.encode_utf16().count().checked_mul(2).ok_or(ByteFailure { code: "length-range", message: "Encoded length exceeds native storage.", offset: None })?;
    let mut output = codec_buffer(length)?;
    for unit in text.encode_utf16() { output.extend_from_slice(&if little { unit.to_le_bytes() } else { unit.to_be_bytes() }); }
    Ok(output)
}
pub fn to_utf16(bytes: &[u8], order: &str) -> Result<std::string::String, ByteFailure> {
    let little = codec_little_endian(order)?;
    if bytes.len() % 2 != 0 { return Err(ByteFailure { code: "utf16", message: "UTF-16 byte length must be even.", offset: Some((bytes.len() - 1) as i64) }); }
    let units = bytes.chunks_exact(2).map(|pair| if little { u16::from_le_bytes([pair[0], pair[1]]) } else { u16::from_be_bytes([pair[0], pair[1]]) });
    let mut output = std::string::String::new();
    output.try_reserve_exact(bytes.len()).map_err(|_| allocation_failure())?;
    let mut offset = 0;
    for scalar in char::decode_utf16(units) {
        let scalar = scalar.map_err(|_| ByteFailure { code: "utf16", message: "UTF-16 contains an unpaired surrogate.", offset: Some(offset) })?;
        offset += (scalar.len_utf16() * 2) as i64;
        output.push(scalar);
    }
    Ok(output)
}
