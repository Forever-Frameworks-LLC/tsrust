// memmem searches arbitrary bytes with a linear worst-case bound and no allocation.
pub fn index_of(bytes: &[u8], needle: &[u8], from_index: i64) -> Result<i64, ByteFailure> {
    let start = usize::try_from(from_index).ok().filter(|start| *start <= bytes.len()).ok_or(ByteFailure {code: "range", message: "Search start must be between zero and the byte length.", offset: Some(from_index)})?;
    Ok(memchr::memmem::find(&bytes[start..], needle).map_or(-1, |position| i64::try_from(start+position).expect("checked byte length")))
}
