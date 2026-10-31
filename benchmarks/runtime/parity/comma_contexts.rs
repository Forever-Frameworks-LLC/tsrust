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
fn mark(trace:&mut Vec<i64>,value:i64){trace.push(value);}
struct Row{value:Option<i64>}
enum Alternative{Text(std::string::String),Int(i64)}
fn reset(row:&mut Row){row.value=None;}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
 let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
for i in 0..value0{let mut trace=Vec::new();let j=value1.checked_add(i).unwrap();let mut values=vec![j,j.checked_add(1).unwrap()];mark(&mut trace,1);values.push(j.checked_add(2).unwrap());mark(&mut trace,2);values[1..2].fill(8);mark(&mut trace,3);values.reverse();mark(&mut trace,4);let joined=values.iter().map(i64::to_string).collect::<Vec<_>>().join(",");
 let value=if i%2==0{Some(j)}else{None};mark(&mut trace,5);let narrowed=value.map(|v|v.checked_add(1).unwrap()).unwrap_or(0);let alternative=if i%2==0{Alternative::Text("yes".to_owned())}else{Alternative::Int(j)};mark(&mut trace,6);mark(&mut trace,7);let text=match alternative{Alternative::Text(text)=>text.to_uppercase(),Alternative::Int(_)=>"number".to_owned()};
 let mut row=Row{value:Some(j)};let before=row.value;reset(&mut row);let changed=before!=None && row.value==None;
 output.push(Array(vec![String::Owned(joined),String::Owned(trace.iter().map(i64::to_string).collect::<Vec<_>>().join(",")),String::Owned(narrowed.to_string()),String::Owned(text),String::Owned(changed.to_string()),String::Owned(value2.to_string())]));}output
}
fn main(){}
