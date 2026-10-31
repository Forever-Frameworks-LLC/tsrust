// Independent standard Rust arrays, Vec ownership and checked index arithmetic.
type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
#[derive(Clone)]struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {let values=[String::Owned(value2.to_string()),String::Borrowed("Pat"),String::Owned(value2.to_string()),String::Borrowed(""),String::Borrowed("é"),String::Borrowed("😀"),String::Borrowed("e\u{301}"),String::Borrowed("a\0b")];let starts=[0,0,1,-1,-8,-80,0,1,3,-3,100,-100,3,-2,-2,0];let ends=[0,0,0,0,0,0,0,3,3,-1,200,-80,1,-2,200,-1];let modes=[0,1,1,1,1,1,2,2,2,2,2,2,2,2,2,2];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%16).unwrap();let start=if modes[at]==0{0}else{bound(starts[at],values.len())};let end=if modes[at]<2{values.len()}else{bound(ends[at],values.len())};let copied=values[start..end.max(start)].to_vec();output.push(Array(copied));}output}
fn main(){}
