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
fn function0(value0: i64, value1: i64, value2: &String) -> NumericArray<i64> {
    let size=match value2.as_ref() {"4k"=>4096,"1m"=>1048576,_=>32};
    let text="é😀Ab".repeat(size/8);

    let mut output=Vec::with_capacity(usize::try_from(value0.max(0)).unwrap().checked_mul(3).unwrap());
    for _ in 0..value0 { let result=utf16(&text,false); output.extend([result.len() as i64,i64::from(result[0]),i64::from(result[result.len()-1])]); }
    NumericArray(output)
}
fn main() {}
