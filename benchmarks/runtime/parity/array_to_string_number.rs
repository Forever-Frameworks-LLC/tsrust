// Independent standard Rust formatting and bounded reverse slice searches.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn join_display<T:std::fmt::Display>(values:&[T],separator:&str)->std::string::String{use std::fmt::Write as _;let mut output=std::string::String::new();for (index,value) in values.iter().enumerate(){if index>0{output.push_str(separator);}write!(&mut output,"{value}").unwrap();}output}
struct Number(f64);
impl std::fmt::Display for Number{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{let value=self.0;if value.is_nan(){f.write_str("NaN")}else if value==f64::INFINITY{f.write_str("Infinity")}else if value==f64::NEG_INFINITY{f.write_str("-Infinity")}else if value==0.0{f.write_str("0")}else{write!(f,"{value}")}}}
struct Nested(Vec<i64>);
impl std::fmt::Display for Nested{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{for (index,value) in self.0.iter().enumerate(){if index>0{f.write_str(",")?;}write!(f,"{value}")?;}Ok(())}}
#[derive(Clone,Copy)]struct Empty;
impl std::fmt::Display for Empty{fn fmt(&self,_:&mut std::fmt::Formatter<'_>)->std::fmt::Result{Ok(())}}
fn last_start(from:Option<i64>,len:usize)->Option<usize>{if len==0{return None;}let len=i64::try_from(len).unwrap();let index=match from{None=>len-1,Some(index) if index>=0=>index.min(len-1),Some(index)=>len.checked_add(index)?};usize::try_from(index).ok()}
fn number_item(base:i64)->f64{match base%8{0=>0.0,1=>-0.0,2=>1.25,3=>-2.5,4=>f64::NAN,5=>f64::INFINITY,6=>f64::NEG_INFINITY,_=>(base%17) as f64/4.0}}
fn function0(value0: i64, value1: i64) -> Array<String> {let values=[Number(value1 as f64),Number(-0.0),Number(0.5),Number(-2.5),Number(17.0),Number(f64::NAN),Number(f64::INFINITY),Number(f64::NEG_INFINITY)];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let count=usize::try_from(i.checked_add(value1).unwrap()%9).unwrap();let separator=",";let receiver=&values[..count];output.push(String::Owned(join_display(receiver,separator)));}Array(output)}
fn main(){}
