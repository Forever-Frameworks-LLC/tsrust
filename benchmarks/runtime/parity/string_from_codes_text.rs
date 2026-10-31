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
fn from_units(codes: impl IntoIterator<Item = i64>) -> Text {
    std::string::String::from_utf16_lossy(
        &codes
            .into_iter()
            .map(|code| code as u16)
            .collect::<Vec<_>>(),
    )
}
fn from_points(codes: impl IntoIterator<Item = i64>) -> Text {
    let mut units = Vec::new();
    for code in codes {
        let code = u32::try_from(code).expect("Invalid code point");
        assert!(code <= 0x10ffff, "Invalid code point");
        if (0xd800..=0xdfff).contains(&code) {
            units.push(u16::try_from(code).unwrap());
        } else {
            let mut pair = [0; 2];
            units.extend_from_slice(char::from_u32(code).unwrap().encode_utf16(&mut pair));
        }
    }
    std::string::String::from_utf16_lossy(&units)
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let codes = vec![
            add(65, base % 26),
            add(233, base),
            add(19968, base),
            add(128512, base % 64),
        ];
        output.push(Array(vec![
            owned(from_points(codes.iter().copied())),
            owned(from_units([add(72, base % 20), 105, 8364])),
            owned(from_points([codes[3], add(97, base % 26)])),
        ]));
    }
    output
}
fn main() {}
