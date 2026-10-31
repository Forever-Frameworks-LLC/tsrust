// Independent hand-written Rust; standard f64 operations and checked integer bookkeeping.
// Decimal/hex/base36 prefix parsing for the matched bounded digit corpus.
type String=std::borrow::Cow<'static,str>;
fn leading_space(c:char)->bool{matches!(c,'\t'|'\n'|'\r'|'\u{b}'|'\u{c}'|' '|'\u{a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')}
fn parse_integer(text:&str,radix:u32)->f64{
 let text=text.trim_start_matches(leading_space);let (negative,text)=if let Some(text)=text.strip_prefix('-'){(true,text)}else{(false,text.strip_prefix('+').unwrap_or(text))};
 let hex=text.starts_with("0x")||text.starts_with("0X");let radix=if radix==0{if hex{16}else{10}}else{radix};
 if !(2..=36).contains(&radix){return f64::NAN;}
 let text=if radix==16 && hex{&text[2..]}else{text};let mut value=0.0;let mut any=false;
 for byte in text.bytes(){let digit=match byte{b'0'..=b'9'=>u32::from(byte-b'0'),b'a'..=b'z'=>u32::from(byte-b'a')+10,b'A'..=b'Z'=>u32::from(byte-b'A')+10,_=>break};if digit>=radix{break;}any=true;value=value*f64::from(radix)+f64::from(digit);}
 if !any{f64::NAN}else if negative{-value}else{value}
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Vec<f64>> {
 let _=value2;let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
 for i in 0..value0{let base=i.checked_mul(13).unwrap().checked_add(value1).unwrap()%97;let digits=base.checked_mul(7).unwrap().checked_add(i).unwrap().to_string();
 output.push(vec![parse_integer(&(digits.clone()+"px"),0),parse_integer(&digits,16),parse_integer(&("0x".to_owned()+&digits),0),parse_integer(&("-".to_owned()+&digits),36),parse_integer(&("z".to_owned()+&digits),0)]);
 }output
}
fn main(){}
