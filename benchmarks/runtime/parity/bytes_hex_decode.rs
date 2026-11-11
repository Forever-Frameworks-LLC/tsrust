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
fn unhex(text: &str) -> Result<Vec<u8>,usize> {
    if text.len()%2!=0 { return Err(text.len()); }
    let digit = |byte:u8,index:usize| match byte { b'0'..=b'9'=>Ok(byte-b'0'), b'a'..=b'f'=>Ok(byte-b'a'+10), b'A'..=b'F'=>Ok(byte-b'A'+10), _=>Err(index) };
    let mut result = Vec::with_capacity(text.len()/2);
    for (index,pair) in text.as_bytes().chunks_exact(2).enumerate() { result.push((digit(pair[0],index*2)?<<4)|digit(pair[1],index*2+1)?); }
    Ok(result)
}
fn function0(value0: i64, value1: i64, value2: &String) -> NumericArray<i64> {
    let size=match value2.as_ref() {"4k"=>4096,"1m"=>1048576,_=>32};
    let text="é😀Ab".repeat(size/8);
    let encoded=hex(text.as_bytes());
    let mut output=Vec::with_capacity(usize::try_from(value0.max(0)).unwrap().checked_mul(3).unwrap());
    for _ in 0..value0 { let result=unhex(&encoded).unwrap(); output.extend([result.len() as i64,i64::from(result[0]),i64::from(result[result.len()-1])]); }
    NumericArray(output)
}
fn main() {}
