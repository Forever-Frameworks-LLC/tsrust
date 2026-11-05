// Endian conversions use safe, unaligned slice copies into native fixed arrays.
fn binary_order(order: &str) -> Result<bool, ByteFailure> {
    match order {"little" => Ok(true), "big" => Ok(false), _ => Err(ByteFailure {code: "byte-order", message: "Byte order must be little or big.", offset: None})}
}
fn binary_width(width: i64, unsigned: bool) -> Result<usize, ByteFailure> {
    match width {1|2|4 => Ok(width as usize), 8 if !unsigned => Ok(8), _ => Err(ByteFailure {code: "width", message: "Integer width is unsupported for this operation.", offset: None})}
}
fn binary_range(length: usize, offset: i64, width: usize) -> Result<std::ops::Range<usize>, ByteFailure> {
    let start = usize::try_from(offset).ok();
    let end = start.and_then(|start| start.checked_add(width)).filter(|end| *end <= length);
    match (start,end) { (Some(start),Some(end)) => Ok(start..end), _ => Err(ByteFailure {code: "range", message: "The complete value must fit at the byte offset.", offset: Some(offset)}) }
}
pub fn read_int(bytes: &[u8], offset: i64, width: i64, order: &str) -> Result<i64, ByteFailure> {
    let width = binary_width(width, false)?; let little = binary_order(order)?;
    let input = &bytes[binary_range(bytes.len(), offset, width)?];
    Ok(match width {
        1 => i64::from(i8::from_ne_bytes([input[0]])),
        2 => i64::from(if little {i16::from_le_bytes(input.try_into().expect("checked width"))} else {i16::from_be_bytes(input.try_into().expect("checked width"))}),
        4 => i64::from(if little {i32::from_le_bytes(input.try_into().expect("checked width"))} else {i32::from_be_bytes(input.try_into().expect("checked width"))}),
        8 => if little {i64::from_le_bytes(input.try_into().expect("checked width"))} else {i64::from_be_bytes(input.try_into().expect("checked width"))},
        _ => unreachable!("checked integer width"),
    })
}
pub fn read_uint(bytes: &[u8], offset: i64, width: i64, order: &str) -> Result<i64, ByteFailure> {
    let width = binary_width(width, true)?; let little = binary_order(order)?;
    let input = &bytes[binary_range(bytes.len(), offset, width)?];
    Ok(match width {
        1 => i64::from(input[0]),
        2 => i64::from(if little {u16::from_le_bytes(input.try_into().expect("checked width"))} else {u16::from_be_bytes(input.try_into().expect("checked width"))}),
        4 => i64::from(if little {u32::from_le_bytes(input.try_into().expect("checked width"))} else {u32::from_be_bytes(input.try_into().expect("checked width"))}),
        _ => unreachable!("checked unsigned width"),
    })
}
pub fn write_int(bytes: &mut [u8], offset: i64, value: i64, width: i64, order: &str) -> Result<(), ByteFailure> {
    let width = binary_width(width, false)?; let little = binary_order(order)?;
    let range = binary_range(bytes.len(), offset, width)?;
    if match width {1 => i8::try_from(value).is_err(), 2 => i16::try_from(value).is_err(), 4 => i32::try_from(value).is_err(), 8 => false, _ => unreachable!()} {
        return Err(ByteFailure {code: "integer-range", message: "Integer value does not fit the selected signed width.", offset: None});
    }
    let encoded = if little {value.to_le_bytes()} else {value.to_be_bytes()};
    bytes[range].copy_from_slice(if little {&encoded[..width]} else {&encoded[8-width..]}); Ok(())
}
pub fn write_uint(bytes: &mut [u8], offset: i64, value: i64, width: i64, order: &str) -> Result<(), ByteFailure> {
    let width = binary_width(width, true)?; let little = binary_order(order)?;
    let range = binary_range(bytes.len(), offset, width)?;
    if value < 0 || value > match width {1 => i64::from(u8::MAX), 2 => i64::from(u16::MAX), 4 => i64::from(u32::MAX), _ => unreachable!()} {
        return Err(ByteFailure {code: "integer-range", message: "Integer value does not fit the selected unsigned width.", offset: None});
    }
    let encoded = if little {value.to_le_bytes()} else {value.to_be_bytes()};
    bytes[range].copy_from_slice(if little {&encoded[..width]} else {&encoded[8-width..]}); Ok(())
}
pub fn read_float32(bytes: &[u8], offset: i64, order: &str) -> Result<f64, ByteFailure> {
    let little = binary_order(order)?; let input = &bytes[binary_range(bytes.len(), offset, 4)?];
    let value = if little {f32::from_le_bytes(input.try_into().expect("checked width"))} else {f32::from_be_bytes(input.try_into().expect("checked width"))};
    Ok(f64::from(value))
}
pub fn write_float32(bytes: &mut [u8], offset: i64, value: f64, order: &str) -> Result<(), ByteFailure> {
    let little = binary_order(order)?; let range = binary_range(bytes.len(), offset, 4)?;
    let encoded = if little {(value as f32).to_le_bytes()} else {(value as f32).to_be_bytes()};
    bytes[range].copy_from_slice(&encoded); Ok(())
}
pub fn read_float64(bytes: &[u8], offset: i64, order: &str) -> Result<f64, ByteFailure> {
    let little = binary_order(order)?; let input = &bytes[binary_range(bytes.len(), offset, 8)?];
    let value = if little {f64::from_le_bytes(input.try_into().expect("checked width"))} else {f64::from_be_bytes(input.try_into().expect("checked width"))};
    Ok(value)
}
pub fn write_float64(bytes: &mut [u8], offset: i64, value: f64, order: &str) -> Result<(), ByteFailure> {
    let little = binary_order(order)?; let range = binary_range(bytes.len(), offset, 8)?;
    let encoded = if little {(value).to_le_bytes()} else {(value).to_be_bytes()};
    bytes[range].copy_from_slice(&encoded); Ok(())
}
