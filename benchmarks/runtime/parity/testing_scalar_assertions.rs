// Independent standard Rust reference; retain pattern/default/control operations.
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
fn owned(value: impl Into<Text>) -> String {
    String::Owned(value.into())
}
fn function0(value0: i64, value1: i64) -> i64 {
    let values = [0_i64, 1, 4, 9, 16, 25, 36, 49];
    let mut checksum = 0;
    for i in 0..value0 {
        let at = add(mul(i, 13), value1) % 8;
        let actual = values[usize::try_from(at).unwrap()];
        let expected = mul(at, at);
        assert!(actual >= 0);
        assert_eq!(actual, expected);
        assert_ne!(actual, add(expected, 1));
        checksum = add(mul(checksum, 131), actual) % 1000000007;
    }
    checksum
}
fn main() {}
