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
fn parse_integer(text:&str,radix:u32)->f64{
 let text=text.trim_start_matches(leading_space);let (negative,text)=if let Some(text)=text.strip_prefix('-'){(true,text)}else{(false,text.strip_prefix('+').unwrap_or(text))};
 let hex=text.starts_with("0x")||text.starts_with("0X");let radix=if radix==0{if hex{16}else{10}}else{radix};
 if !(2..=36).contains(&radix){return f64::NAN;}
 let text=if radix==16 && hex{&text[2..]}else{text};let mut value=0.0;let mut any=false;
 for byte in text.bytes(){let digit=match byte{b'0'..=b'9'=>u32::from(byte-b'0'),b'a'..=b'z'=>u32::from(byte-b'a')+10,b'A'..=b'Z'=>u32::from(byte-b'A')+10,_=>break};if digit>=radix{break;}any=true;value=value*f64::from(radix)+f64::from(digit);}
 if !any{f64::NAN}else if negative{-value}else{value}
}
fn parse_float(text:&str)->f64{
    let text=text.trim_start_matches(leading_space);let bytes=text.as_bytes();let mut at=usize::from(matches!(bytes.first(),Some(b'+')|Some(b'-')));
    if text[at..].starts_with("Infinity"){return if bytes.first()==Some(&b'-'){f64::NEG_INFINITY}else{f64::INFINITY};}
    let mut digits=0;while bytes.get(at).is_some_and(u8::is_ascii_digit){at+=1;digits+=1;}
    if bytes.get(at)==Some(&b'.'){at+=1;while bytes.get(at).is_some_and(u8::is_ascii_digit){at+=1;digits+=1;}}
    if digits==0{return f64::NAN;}
    let significand_end=at;
    if matches!(bytes.get(at),Some(b'e')|Some(b'E')){at+=1;if matches!(bytes.get(at),Some(b'+')|Some(b'-')){at+=1;}let start=at;while bytes.get(at).is_some_and(u8::is_ascii_digit){at+=1;}if at==start{at=significand_end;}}
    text[..at].parse::<f64>().unwrap_or(f64::NAN)
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
 let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
for i in 0..value0{let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let whole=parse_integer(&(base.to_string()+"px"),0);let hex=parse_integer(&base.to_string(),16);let part=parse_float(&(base.to_string()+".25x"));let missing=parse_integer(value2,0);let mut lowest=f64::INFINITY;if part<lowest{lowest=part;}let picked=if missing.is_nan(){f64::NAN}else{missing+1.0};
 output.push(Array(vec![String::Owned(number_text(whole+hex)),String::Owned(number_text(part)),String::Owned(missing.is_nan().to_string()),String::Owned((lowest.is_finite() && !f64::NEG_INFINITY.is_finite()).to_string()),String::Owned(number_text(picked))]));}output
}
fn main(){}
