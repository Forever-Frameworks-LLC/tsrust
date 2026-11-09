// Handwritten Rust URL parsing and borrowed component inspection.
type String = std::borrow::Cow<'static, str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let value = url::Url::parse(value2.as_ref()).unwrap();
        let href = value.as_str();
        let host = value.host_str().unwrap();
        let password = value.password().unwrap();
        // Include delimiters in the same views exposed by Tsrust.
        let protocol = &href[..value.scheme().len() + 1];
        let search = &value[url::Position::AfterPath..url::Position::AfterQuery];
        let hash = &value[url::Position::AfterQuery..];
        assert_eq!(href, "https://user:secret@example.com:8443/b?tag=one&tag=two#section");
        assert_eq!(host, "example.com");
        assert_eq!(value.port(), Some(8443));
        assert_eq!(value.path(), "/b");
        assert_eq!(protocol, "https:");
        assert_eq!(value.username(), "user");
        assert_eq!(password, "secret");
        assert_eq!(search, "?tag=one&tag=two");
        assert_eq!(hash, "#section");
        let observed = href.len() + host.len() + value.path().len() + protocol.len() + value.username().len() + password.len() + search.len() + hash.len() + 8443;
        checksum = checksum.checked_add(i64::try_from(observed).unwrap()).unwrap();
    }
    checksum
}
fn main() {}
