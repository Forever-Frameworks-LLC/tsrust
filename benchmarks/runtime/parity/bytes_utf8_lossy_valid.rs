// Independent handwritten Rust std/crate reference; no compiler runtime/bridge inclusion.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> { fn iter(&self) -> std::slice::Iter<'_,T> { self.0.iter() } }
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> { fn iter(&self) -> std::slice::Iter<'_,T> { self.0.iter() } }
fn function0(value0: i64, value1: i64, value2: &String) -> NumericArray<i64> {
    let size=match value2.as_ref() {"4k"=>4096,"1m"=>1048576,_=>32};
    let text="é😀Ab".repeat(size/8);
    let mut bytes=text.as_bytes().to_vec();
    let mut output=Vec::new();
    for _ in 0..value0 { let result=std::string::String::from_utf8_lossy(&bytes).into_owned(); output.extend([result.len() as i64,i64::from(result.starts_with("é😀")),i64::from(result.ends_with("Ab"))]); }
    NumericArray(output)
}
fn main() {}
