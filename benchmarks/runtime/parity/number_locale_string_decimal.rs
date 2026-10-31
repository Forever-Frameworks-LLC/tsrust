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
// Decimal grouping and three-digit decimal rounding for the bounded corpus.
fn locale(value:f64)->std::string::String{
 if value.is_nan(){return "NaN".to_owned();}if value.is_infinite(){return if value.is_sign_negative(){"-∞"}else{"∞"}.to_owned();}
 let raw=value.abs().to_string();let (whole,fraction)=raw.split_once('.').unwrap_or((&raw,""));let mut fraction=fraction.to_owned();while fraction.len()<4{fraction.push('0');}
 let mut scaled=whole.parse::<u128>().unwrap().checked_mul(1000).unwrap().checked_add(fraction[..3].parse::<u128>().unwrap()).unwrap();if fraction.as_bytes()[3]>=b'5'{scaled=scaled.checked_add(1).unwrap();}
 let whole=(scaled/1000).to_string();let mut text=std::string::String::new();if value.is_sign_negative(){text.push('-');}for (at,byte) in whole.bytes().enumerate(){if at>0 && (whole.len()-at)%3==0{text.push(',');}text.push(char::from(byte));}
 let digits=format!("{:03}",scaled%1000);let tail=digits.trim_end_matches('0');if !tail.is_empty(){text.push('.');text.push_str(tail);}text
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
 let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
let fractions=[0.0,0.2345,0.9995];let specials=[-0.0004,-0.0005,f64::NAN,f64::INFINITY,f64::NEG_INFINITY];for i in 0..value0{let whole=(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97).checked_mul(1001).unwrap();let value=whole as f64+fractions[usize::try_from(i%3).unwrap()];let special=specials[usize::try_from(i%5).unwrap()];
 output.push(Array(vec![String::Owned(locale(whole as f64)),String::Owned(locale(value)),String::Owned(locale(-value)),String::Owned(locale(special)),String::Owned(locale(1e21))]));}output
}
fn main(){}
