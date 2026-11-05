use base64::{Engine as _, engine::{GeneralPurposeConfig, DecodePaddingMode}};
#[cfg(any(target_arch = "x86_64", all(target_arch = "aarch64", target_feature = "neon")))]
fn codec_base64_engine(url: bool) -> &'static base64::engine::Simd {
    static STANDARD: std::sync::OnceLock<base64::engine::Simd> = std::sync::OnceLock::new();
    static URL: std::sync::OnceLock<base64::engine::Simd> = std::sync::OnceLock::new();
    if url { URL.get_or_init(|| base64::engine::Simd::url_safe(GeneralPurposeConfig::new().with_encode_padding(false).with_decode_padding_mode(DecodePaddingMode::RequireNone))) }
    else { STANDARD.get_or_init(|| base64::engine::Simd::standard(GeneralPurposeConfig::new())) }
}
#[cfg(not(any(target_arch = "x86_64", all(target_arch = "aarch64", target_feature = "neon"))))]
fn codec_base64_engine(url: bool) -> &'static base64::engine::GeneralPurpose {
    if url { &base64::engine::general_purpose::URL_SAFE_NO_PAD } else { &base64::engine::general_purpose::STANDARD }
}
pub fn to_base64(bytes: &[u8], url: bool) -> Result<std::string::String, ByteFailure> {
    let length = base64::encoded_len(bytes.len(), !url).ok_or(ByteFailure { code: "length-range", message: "Encoded length exceeds native storage.", offset: None })?;
    let mut output = codec_buffer(length)?; output.resize(length, 0);
    codec_base64_engine(url).encode_slice(bytes, &mut output).expect("checked base64 output size");
    Ok(std::string::String::from_utf8(output).expect("base64 output is ASCII"))
}
pub fn from_base64(text: &str, url: bool) -> Result<Vec<u8>, ByteFailure> {
    let length = base64::decoded_len_estimate(text.len());
    let mut output = codec_buffer(length)?; output.resize(length, 0);
    let length = codec_base64_engine(url).decode_slice(text, &mut output).map_err(|error| {
        use base64::{DecodeError, DecodeSliceError};
        let (code, message, offset) = match error {
            DecodeSliceError::DecodeError(DecodeError::InvalidByte(offset, _)) => ("base64-byte", "Base64 contains a byte outside the selected alphabet.", Some(offset as i64)),
            DecodeSliceError::DecodeError(DecodeError::InvalidLength(_)) => ("base64-length", "Base64 symbol count is invalid.", None),
            DecodeSliceError::DecodeError(DecodeError::InvalidLastSymbol { offset, .. }) => ("base64-trailing-bits", "Base64 contains nonzero trailing bits.", Some(offset as i64)),
            DecodeSliceError::DecodeError(DecodeError::InvalidPadding) => ("base64-padding", "Base64 padding does not match the selected encoding.", None),
            DecodeSliceError::OutputSliceTooSmall => unreachable!("conservative base64 output size"),
        };
        ByteFailure { code, message, offset }
    })?;
    output.truncate(length); Ok(output)
}
