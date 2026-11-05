// Borrowed slice operations; only concatenation owns a new allocation.
pub fn concat(left: &[u8], right: &[u8]) -> Result<Vec<u8>, ByteFailure> {
    let length = left.len().checked_add(right.len()).and_then(|length| i64::try_from(length).ok()).ok_or(ByteFailure {code: "length-range", message: "Combined byte length exceeds native storage.", offset: None})?;
    let length = checked_length(length)?;
    let mut output = Vec::new(); reserve(&mut output, length)?;
    output.extend_from_slice(left); output.extend_from_slice(right); Ok(output)
}
pub fn copy_into(target: &mut [u8], target_offset: i64, source: &[u8]) -> Result<(), ByteFailure> {
    let failure = || ByteFailure {code: "range", message: "The complete source must fit at the destination byte offset.", offset: Some(target_offset)};
    let start = usize::try_from(target_offset).map_err(|_| failure())?;
    let end = start.checked_add(source.len()).ok_or_else(failure)?;
    let destination = target.get_mut(start..end).ok_or_else(failure)?;
    destination.copy_from_slice(source); Ok(())
}
pub fn copy_within(bytes: &mut [u8], target_offset: i64, start: i64, end: i64) -> Result<(), ByteFailure> {
    let range = checked_range(bytes.len(), start, end)?;
    let target = usize::try_from(target_offset).ok().filter(|target| *target <= bytes.len()-range.len()).ok_or(ByteFailure {code: "range", message: "The complete source range must fit at the destination byte offset.", offset: Some(target_offset)})?;
    bytes.copy_within(range, target); Ok(())
}
pub fn equals(left: &[u8], right: &[u8]) -> bool { left == right }
pub fn compare(left: &[u8], right: &[u8]) -> i64 {
    match left.cmp(right) {std::cmp::Ordering::Less => -1, std::cmp::Ordering::Equal => 0, std::cmp::Ordering::Greater => 1}
}
