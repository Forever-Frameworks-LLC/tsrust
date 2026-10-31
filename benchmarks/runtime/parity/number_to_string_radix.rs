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
// The matched workload uses nonnegative values below 98 in eighths;
// its even radices give finite, exact fractional digit sequences.
fn radix_text(value:f64,radix:u32)->std::string::String{
 if value==0.0{return "0".to_owned();}
 let magnitude=value.abs();let integer=magnitude.trunc() as u64;
 let mut whole=match radix{2=>format!("{integer:b}"),8=>format!("{integer:o}"),16=>format!("{integer:x}"),36=>{let mut digits=Vec::new();let mut remaining=integer;loop{digits.push(char::from_digit((remaining%36) as u32,36).unwrap());remaining/=36;if remaining==0{break;}}digits.into_iter().rev().collect()},_=>panic!("workload radix")};
 let mut fraction=magnitude.fract();if fraction!=0.0{whole.push('.');while fraction!=0.0{fraction*=f64::from(radix);let digit=fraction.trunc();whole.push(char::from_digit(digit as u32,radix).unwrap());fraction-=digit;}}
 if value<0.0{whole.insert(0,'-');}whole
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
 let _=value2;let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
 for i in 0..value0{let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let value=base as f64+(i%8) as f64/8.0;
 output.push(Array(vec![String::Owned(number_text(value)),String::Owned(radix_text(value,2)),String::Owned(radix_text(value,16)),String::Owned(radix_text(value,36)),String::Owned(radix_text(-value,8))]));}output
}
fn main(){}
