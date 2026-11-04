// Independent Rust Unicode normalization through the established Unicode crate.
// Input tables match the workload corpus; all seven outputs are computed at runtime.
use unicode_normalization::UnicodeNormalization;
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> { fn iter(&self) -> std::slice::Iter<'_, T> { self.0.iter() } }
fn normalize(text: &str, form: &str) -> String {
    String::Owned(match form {
        "NFC" => text.nfc().collect(),
        "NFD" => text.nfd().collect(),
        "NFKC" => text.nfkc().collect(),
        "NFKD" => text.nfkd().collect(),
        _ => panic!("invalid normalization form"),
    })
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let _ = value2;
    let words = ["e\u{301}", "Åﬁ①", "각", "a\u{315}\u{300}", "é😀"];
    let forms = ["NFC", "NFD", "NFKC", "NFKD"];
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let base = i.checked_mul(13).unwrap().checked_add(value1).unwrap() % 97;
        let text = words[usize::try_from(i % 5).unwrap()].repeat(usize::try_from(base % 3 + 1).unwrap());
        output.push(Array(vec![normalize(&text, "NFC"), normalize(&text, "NFD"), normalize(&text, "NFKC"), normalize(&text, "NFKD"), normalize(&text, "NFC"), normalize(&text, "NFC"), normalize(&text, forms[usize::try_from(i % 4).unwrap()])]));
    }
    output
}
fn main() {}
