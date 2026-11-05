// Independent handwritten Rust std/crate reference; no compiler runtime/bridge inclusion.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> { fn iter(&self) -> std::slice::Iter<'_,T> { self.0.iter() } }
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> { fn iter(&self) -> std::slice::Iter<'_,T> { self.0.iter() } }
use base64::Engine as _;
#[cfg(any(target_arch="x86_64",all(target_arch="aarch64",target_feature="neon")))]
fn engine()->&'static base64::engine::Simd {
    static ENGINE:std::sync::OnceLock<base64::engine::Simd>=std::sync::OnceLock::new();
    ENGINE.get_or_init(||base64::engine::Simd::standard(base64::engine::GeneralPurposeConfig::new()))
}
#[cfg(not(any(target_arch="x86_64",all(target_arch="aarch64",target_feature="neon"))))]
fn engine()->&'static base64::engine::GeneralPurpose {&base64::engine::general_purpose::STANDARD}
fn function0(value0: i64, value1: i64, value2: &String) -> NumericArray<i64> {
    let size=match value2.as_ref() {"4k"=>4096,"1m"=>1048576,_=>32};
    let text="é😀Ab".repeat(size/8);
    let encoded=engine().encode(text.as_bytes());
    let mut output=Vec::with_capacity(usize::try_from(value0.max(0)).unwrap().checked_mul(3).unwrap());
    for _ in 0..value0 { let result=engine().decode(&encoded).unwrap(); output.extend([result.len() as i64,i64::from(result[0]),i64::from(result[result.len()-1])]); }
    NumericArray(output)
}
fn main() {}
