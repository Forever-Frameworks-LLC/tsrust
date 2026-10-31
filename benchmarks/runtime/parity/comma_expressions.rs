// Independent standard Rust formatting for this workload's float corpus.
// Explicit notation thresholds, exponent signs, nonfinite text and zero spelling
// match the observable workload contract. Precision is supplied to std::fmt.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn exponent_text(raw:std::string::String)->std::string::String{
 let (digits,exponent)=raw.split_once('e').unwrap();let exponent=exponent.parse::<i32>().unwrap();format!("{digits}e{exponent:+}")
}
fn special(value:f64)->Option<std::string::String>{if value.is_nan(){Some("NaN".to_owned())}else if value==f64::INFINITY{Some("Infinity".to_owned())}else if value==f64::NEG_INFINITY{Some("-Infinity".to_owned())}else{None}}
fn number_text(value:f64)->std::string::String{
 if let Some(text)=special(value){return text;}
 if value==0.0{return "0".to_owned();}
 if value.abs()>=1e21 || value.abs()<1e-6{exponent_text(format!("{value:e}"))}else{value.to_string()}
}
fn fixed(value:f64,digits:usize)->std::string::String{
 if !value.is_finite() || value.abs()>=1e21{return number_text(value);}
 let value=if value==0.0{0.0}else{value};format!("{value:.digits$}")
}
fn exponential(value:f64,digits:Option<usize>)->std::string::String{
 if let Some(text)=special(value){return text;}
 let value=if value==0.0{0.0}else{value};exponent_text(match digits{Some(digits)=>format!("{value:.digits$e}"),None=>format!("{value:e}")})
}
fn precision(value:f64,digits:usize)->std::string::String{
 if let Some(text)=special(value){return text;}
 let value=if value==0.0{0.0}else{value};let places=digits-1;let raw=format!("{value:.places$e}");let (_,exponent)=raw.split_once('e').unwrap();let exponent=exponent.parse::<i32>().unwrap();
 if exponent < -6 || exponent >= i32::try_from(digits).unwrap(){exponent_text(raw)}else{let places=usize::try_from(i32::try_from(digits).unwrap()-1-exponent).unwrap();format!("{value:.places$}")}
}
fn observe(trace:&mut Vec<i64>,value:i64)->i64{trace.push(value);value}
fn selected(trace:&mut Vec<i64>,value:i64)->i64{observe(trace,value);value.checked_add(1).unwrap()}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
 let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
let mut j=value1;for i in 0..value0{let mut trace=Vec::new();observe(&mut trace,j);observe(&mut trace,j.checked_add(2).unwrap());let value=j.checked_mul(3).unwrap();let result=selected(&mut trace,value);observe(&mut trace,j.checked_add(4).unwrap());let text=value2.to_string();let joined=trace.iter().map(i64::to_string).collect::<Vec<_>>().join(",");
 output.push(Array(vec![String::Owned(value.to_string()),String::Owned(result.to_string()),String::Owned(joined),String::Owned(text),String::Owned(i.to_string()),String::Owned(j.to_string())]));j=j.checked_add(1).unwrap();}output
}
fn main(){}
