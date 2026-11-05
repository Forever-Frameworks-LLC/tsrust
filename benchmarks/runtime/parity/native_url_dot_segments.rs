// Handwritten root-dot-segment scaling corpus with independent expected serialization.
type String = std::borrow::Cow<'static, str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let size = value2.len().checked_mul(8192).unwrap();
    let authority = if value2.starts_with('L') { "a".repeat(size) } else { "a".to_owned() };
    let text = format!("https://{authority}@example.com/{}x", "../".repeat(size));
    let expected = format!("https://{authority}@example.com/x");
    let mut checksum = value1;
    for _ in 0..value0 {
        let value = url::Url::parse(&text).unwrap();
        let href = value.as_str();
        assert_eq!(href, expected);
        assert_eq!(value.username().len(), authority.len());
        assert_eq!(value.path(), "/x");
        let observed = href.len() + value.username().len() + value.path().len();
        checksum = checksum.checked_add(i64::try_from(observed).unwrap()).unwrap();
    }
    checksum
}
fn main() {}
