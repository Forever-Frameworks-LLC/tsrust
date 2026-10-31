// Independent standard Rust reverse find; bound limits match starts,
// including overlapping matches extending beyond the selected start position.
type String=std::borrow::Cow<'static,str>;
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn byte_index(text:&str,needle:&str)->i64{text.find(needle).map(|x|i64::try_from(x).unwrap()).unwrap_or(-1)}
fn last_index_of(text:&str,needle:&str,position:Option<i64>)->i64{
    let at=position.map(|x|usize::try_from(x.max(0)).unwrap().min(text.len())).unwrap_or(text.len());
    assert!(text.is_char_boundary(at),"UTF-8 boundary required");if needle.is_empty(){return i64::try_from(at).unwrap();}
    let mut end=at.saturating_add(needle.len()).min(text.len());while !text.is_char_boundary(end){end-=1;}
    text[..end].rfind(needle).map(|x|i64::try_from(x).unwrap()).unwrap_or(-1)
}
fn function0(value0: i64, value1: i64, value2: &String) -> NumericArray<i64> {
    let needles=["","a","aa","aba","é","😀","é","missing\u{0000}","a\u{0000}b"];let positions=[i64::MIN,i64::MAX,-7,0,i64::try_from(value2.len()).unwrap(),byte_index(value2.as_ref(),"a"),byte_index(value2.as_ref(),"aa").checked_add(1).unwrap(),byte_index(value2.as_ref(),"é"),byte_index(value2.as_ref(),"😀"),byte_index(value2.as_ref(),"é"),byte_index(value2.as_ref(),"\0")];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let needle=needles[usize::try_from(i.checked_mul(17).unwrap().checked_add(value1).unwrap()%9).unwrap()];output.push(last_index_of(value2.as_ref(),needle,Some(positions[usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%11).unwrap()])).checked_add(1).unwrap());}NumericArray(output)
}
fn main(){}
