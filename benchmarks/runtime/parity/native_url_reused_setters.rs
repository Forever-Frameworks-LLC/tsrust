// Handwritten reused URL setters. Valid writes borrow their input directly.
type String = std::borrow::Cow<'static, str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut value = url::Url::parse(value2.as_ref()).unwrap();
    let mut checksum = value1;
    for _ in 0..value0 {
        value.set_path("/changed/path");
        value.set_query(Some("q=one+two"));
        value.set_port(Some(8443)).unwrap();
        let href = value.as_str();
        assert_eq!(href, "https://example.com:8443/changed/path?q=one+two");
        checksum = checksum.checked_add(i64::try_from(href.len()).unwrap()).unwrap();
    }
    checksum
}
fn main() {}
