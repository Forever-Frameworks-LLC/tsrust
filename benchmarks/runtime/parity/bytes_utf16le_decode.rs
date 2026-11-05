// Independent handwritten Rust std/crate reference; no compiler runtime/bridge inclusion.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> { fn iter(&self) -> std::slice::Iter<'_,T> { self.0.iter() } }
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> { fn iter(&self) -> std::slice::Iter<'_,T> { self.0.iter() } }
fn utf16(text:&str,little:bool)->Vec<u8> {
    let mut result = Vec::with_capacity(text.encode_utf16().count()*2);
    for unit in text.encode_utf16() { result.extend_from_slice(&if little {unit.to_le_bytes()} else {unit.to_be_bytes()}); }
    result
}
fn unutf16(bytes:&[u8],little:bool)->std::string::String {
    assert_eq!(bytes.len()%2,0);
    let units = bytes.chunks_exact(2).map(|pair| if little {u16::from_le_bytes([pair[0],pair[1]])} else {u16::from_be_bytes([pair[0],pair[1]])});
    let mut result = std::string::String::with_capacity(bytes.len());
    for scalar in char::decode_utf16(units) {result.push(scalar.unwrap());}
    result
}
fn function0(value0: i64, value1: i64, value2: &String) -> NumericArray<i64> {
    let size=match value2.as_ref() {"4k"=>4096,"1m"=>1048576,_=>32};
    let text="é😀Ab".repeat(size/8);
    let bytes=utf16(&text,true);
    let mut output=Vec::new();
    for _ in 0..value0 { let result=unutf16(&bytes,true); output.extend([result.len() as i64,i64::from(result.starts_with("é😀")),i64::from(result.ends_with("Ab"))]); }
    NumericArray(output)
}
fn main() {}
