// Independent standard Rust language reference; preserve values and owned data operations.
type String = std::borrow::Cow<'static, str>;
type Text = std::string::String;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn owned(text: impl Into<Text>) -> String {
    String::Owned(text.into())
}
fn number_is(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}
fn number_int_is(a: f64, b: i64) -> bool {
    let converted = b as f64;
    converted as i128 == i128::from(b) && number_is(a, converted)
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<bool>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let mode = add(i, value1) % 3;
        let base = add(mul(i, 13), value1) % 97;
        let left = base as f64 / 4.0;
        let right = add(base, mode) as f64 / 4.0;
        let text = value2.repeat(usize::try_from(mode).unwrap());
        let lr = left - right;
        let rl = right - left;
        output.push(NumericArray(vec![
            number_is(left, right),
            number_is(lr, rl),
            number_is(lr / lr, rl / rl),
            text == value2.as_ref(),
            mode == 0,
            number_int_is(left, base),
        ]));
    }
    Array(output)
}
fn main() {}
