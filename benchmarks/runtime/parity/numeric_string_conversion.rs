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
fn leading_space(c:char)->bool{matches!(c,'\t'|'\n'|'\r'|'\u{b}'|'\u{c}'|' '|'\u{a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')}
// Standard parsing for the bounded decimal/radix corpus, including its exact
// Unicode whitespace and unsigned binary/octal/hex conversion rules.
fn number(text:&str)->f64{
 let text=text.trim_matches(leading_space);if text.is_empty(){return 0.0;}
 match text{"Infinity"|"+Infinity"=>return f64::INFINITY,"-Infinity"=>return f64::NEG_INFINITY,_=>{}}
 for (prefix,radix) in [("0x",16),("0X",16),("0b",2),("0B",2),("0o",8),("0O",8)]{if let Some(digits)=text.strip_prefix(prefix){return u128::from_str_radix(digits,radix).map(|n|n as f64).unwrap_or(f64::NAN);}}
 if !text.bytes().all(|b|b.is_ascii_digit()||matches!(b,b'+'|b'-'|b'.'|b'e'|b'E')){return f64::NAN;}
 text.parse::<f64>().unwrap_or(f64::NAN)
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
 let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
let texts=[value2.as_ref(),"0b101","0o17","\u{feff}12\u{a0}","-0","inf","1e999","-1e-999","0x20000000000003","12tail"];
 for i in 0..value0{let k=i.checked_add(value1).unwrap();let text=texts[usize::try_from(k%10).unwrap()];let value=if k%2==0{Some(text.to_owned())}else{None};let first=number(text);let second=number(text);let nullable=value.as_deref().map(number).unwrap_or(0.0);
 output.push(Array(vec![String::Owned(number_text(first)),String::Owned(number_text(second)),String::Owned(number_text(nullable)),String::Owned((first==0.0 && first.is_sign_negative()).to_string())]));}output
}
fn main(){}
