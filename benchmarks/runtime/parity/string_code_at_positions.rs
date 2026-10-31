// Observer ABI string type only.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
// Independent Rust code extraction for the matched BMP-only corpus.
// BMP scalar values equal UTF-16 code units; no astral-code-unit claim is made.
fn code(text:&str,index:i64)->Option<u32>{usize::try_from(index).ok().and_then(|index|text.chars().nth(index)).map(u32::from)}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Vec<f64>> {
    let _=value2;let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let mode=i.checked_add(value1).unwrap()%3;let position=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%9;let text="aé€z".repeat(usize::try_from(mode.checked_add(1).unwrap()).unwrap());output.push(vec![code(&text,position).map(f64::from).unwrap_or(-1.0),code(&text,position).map(f64::from).unwrap_or(f64::NAN),code(&text,position.checked_add(20).unwrap()).map(f64::from).unwrap_or(-1.0),code(&text,(-1_i64).checked_sub(mode).unwrap()).map(f64::from).unwrap_or(f64::NAN),code(&text,mode).map(f64::from).unwrap_or(-1.0)]);}output
}
fn main(){}
