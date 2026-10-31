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
fn locale(value:f64)->std::string::String{
 if value.is_nan(){return "NaN".to_owned();}if value.is_infinite(){return if value.is_sign_negative(){"-∞"}else{"∞"}.to_owned();}
 let raw=value.abs().to_string();let (whole,fraction)=raw.split_once('.').unwrap_or((&raw,""));let mut fraction=fraction.to_owned();while fraction.len()<4{fraction.push('0');}
 let mut scaled=whole.parse::<u128>().unwrap().checked_mul(1000).unwrap().checked_add(fraction[..3].parse::<u128>().unwrap()).unwrap();if fraction.as_bytes()[3]>=b'5'{scaled=scaled.checked_add(1).unwrap();}
 let whole=(scaled/1000).to_string();let mut text=std::string::String::new();if value.is_sign_negative(){text.push('-');}for (at,byte) in whole.bytes().enumerate(){if at>0 && (whole.len()-at)%3==0{text.push(',');}text.push(char::from(byte));}
 let digits=format!("{:03}",scaled%1000);let tail=digits.trim_end_matches('0');if !tail.is_empty(){text.push('.');text.push_str(tail);}text
}
struct Row{n:i64}
impl std::fmt::Display for Row{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("[object Object]")}}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let whole=base.checked_mul(1001).unwrap();let ints=[whole,whole.checked_neg().unwrap()];let numbers=[1000.5+base as f64,-0.0,1.2345];let words=[value2.as_ref(),"é😀",""];let rows=[Row{n:base},Row{n:whole}];let nested=[vec![1000.5,-0.0],vec![],vec![2000.2345+base as f64]];
 let int_text=ints.iter().map(|v|locale(*v as f64)).collect::<Vec<_>>().join(",");let number_text=numbers.iter().map(|v|locale(*v)).collect::<Vec<_>>().join(",");let word_text=join_display(&words,",");let bool_text=join_display(&[true,false],",");let null_text=join_display(&[Empty;2],",");let undefined_text=join_display(&[Empty;2],",");let row_text=join_display(&rows,",");let nested_text=nested.iter().map(|row|row.iter().map(|v|locale(*v)).collect::<Vec<_>>().join(",")).collect::<Vec<_>>().join(",");output.push(Array(vec![int_text,number_text,word_text,bool_text,null_text,undefined_text,row_text,nested_text].into_iter().map(String::Owned).collect()));}output}
fn main(){}
