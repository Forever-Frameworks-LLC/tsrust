// Independent handwritten Rust std/crate reference; no compiler runtime/bridge inclusion.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> { fn iter(&self) -> std::slice::Iter<'_,T> { self.0.iter() } }
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> { fn iter(&self) -> std::slice::Iter<'_,T> { self.0.iter() } }
fn hex(bytes: &[u8]) -> std::string::String {
    const DIGITS: &[u8;16] = b"0123456789abcdef";
    let mut result = std::string::String::with_capacity(bytes.len()*2);
    for byte in bytes { result.push(DIGITS[(byte>>4) as usize] as char); result.push(DIGITS[(byte&15) as usize] as char); }
    result
}
fn function0(value0: i64, value1: i64, value2: &String) -> NumericArray<i64> {
    let size=match value2.as_ref() {"4k"=>4096,"1m"=>1048576,_=>32};
    let text="é😀Ab".repeat(size/8);
    let bytes=text.as_bytes();
    let mut output=Vec::new();
    for _ in 0..value0 { let result=hex(bytes); output.extend([result.len() as i64,i64::from(result.starts_with("c3a9")),i64::from(result.ends_with("4162"))]); }
    NumericArray(output)
}
fn main() {}
