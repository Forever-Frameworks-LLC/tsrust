// Benchmark observation after timing; no runtime/compiler changes.
trait NumberOutput {
    fn visit_numbers(&self, visitor: &mut dyn FnMut(f64));
    fn is_array(&self) -> bool;
}
impl NumberOutput for f64 {
    fn visit_numbers(&self, visitor: &mut dyn FnMut(f64)) { visitor(*self); }
    fn is_array(&self) -> bool { false }
}
impl NumberOutput for Vec<f64> {
    fn visit_numbers(&self, visitor: &mut dyn FnMut(f64)) { for &value in self { visitor(value); } }
    fn is_array(&self) -> bool { true }
}
fn canonical_number_bits(value: f64) -> u64 {
    if value.is_nan() { 0x7ff8_0000_0000_0000 } else { value.to_bits() }
}
// This profiler checksum visits bytes without allocating or changing disposal.
fn number_checksum<T: NumberOutput>(value: &T) -> i64 {
    let mut checksum = 0_i64;
    value.visit_numbers(&mut |number| {
        for byte in canonical_number_bits(number).to_be_bytes() {
            checksum = (checksum * 131 + i64::from(byte)) % 1_000_000_007;
        }
    });
    checksum
}
fn number_exact_bits<T: NumberOutput>(value: &T) -> std::string::String {
    use std::fmt::Write as _;
    let mut text = std::string::String::from(if value.is_array() { "a:" } else { "n:" });
    let mut first = true;
    value.visit_numbers(&mut |number| {
        if !first { text.push(','); }
        first = false;
        if number.is_nan() { text.push_str("nan"); }
        else { write!(&mut text, "{:016x}", number.to_bits()).unwrap(); }
    });
    text
}
fn probe_checksum<T: NumberOutput>(value: &T) -> i64 { number_checksum(value) }
