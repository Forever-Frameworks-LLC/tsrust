// These application adapters keep the restricted bridge explicit and checked.
// Narrow values return widened i64/f64 storage; they do not create source width types.
pub fn signed(value: i64, width: i64) -> Result<i64, String> {
    match width {
        8 => i8::try_from(value).map(i64::from).map_err(|_| "Range".into()),
        16 => i16::try_from(value).map(i64::from).map_err(|_| "Range".into()),
        32 => i32::try_from(value).map(i64::from).map_err(|_| "Range".into()),
        64 => Ok(value),
        _ => Err("Width".into()),
    }
}
pub fn unsigned(value: i64, width: i64) -> Result<i64, String> {
    let value = u64::try_from(value).map_err(|_| "Range".to_owned())?;
    let checked = match width {
        8 => u8::try_from(value).map(u64::from).map_err(|_| "Range".to_owned())?,
        16 => u16::try_from(value).map(u64::from).map_err(|_| "Range".to_owned())?,
        32 => u32::try_from(value).map(u64::from).map_err(|_| "Range".to_owned())?,
        64 => value,
        _ => return Err("Width".into()),
    };
    i64::try_from(checked).map_err(|_| "Range".into())
}
pub fn float32(value: f64) -> Result<f64, String> {
    if !value.is_finite() { return Err("NonFinite".into()); }
    let narrowed = value as f32;
    if !narrowed.is_finite() { return Err("Range".into()); }
    if f64::from(narrowed) != value { return Err("Precision".into()); }
    Ok(f64::from(narrowed))
}
// Full-width unsigned integers remain validated text on the source side.
// This performs real Rust u128 arithmetic and rejects overflow before serialization.
pub fn incrementWide(value: &str, width: i64) -> Result<String, String> {
    if value.is_empty() || value.len() > 39 || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Syntax".into());
    }
    let value = value.parse::<u128>().map_err(|_| "Range".to_owned())?;
    let maximum = match width { 64 => u128::from(u64::MAX), 128 => u128::MAX, _ => return Err("Width".into()) };
    let next = value.checked_add(1).filter(|next| *next <= maximum).ok_or_else(|| "Range".to_owned())?;
    Ok(next.to_string())
}
pub fn bytes(values: Vec<i64>) -> Result<Vec<i64>, String> {
    let native = values.into_iter().map(u8::try_from).collect::<Result<Vec<u8>, _>>().map_err(|_| "Range".to_owned())?;
    Ok(native.into_iter().map(i64::from).collect())
}
pub enum OptionalInt { Absent, Present(i64) }
pub fn incrementOptional(value: OptionalInt) -> Result<OptionalInt, String> {
    let native: Option<i64> = match value { OptionalInt::Absent => None, OptionalInt::Present(value) => Some(value) };
    let result = native.map(|value| value.checked_add(1).ok_or_else(|| "Range".to_owned())).transpose()?;
    Ok(match result { None => OptionalInt::Absent, Some(value) => OptionalInt::Present(value) })
}
pub struct Pair { pub count: i64, pub label: String }
pub fn pair(value: Pair) -> Pair {
    let native: (i64, String) = (value.count, value.label);
    Pair { count: native.0, label: native.1 }
}
