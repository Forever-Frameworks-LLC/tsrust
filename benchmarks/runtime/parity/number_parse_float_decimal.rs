// Independent Rust decimal-prefix scanner and standard f64 parser.
// The explicit whitespace/Infinity/prefix rules match this API contract;
// precision, exponent range and signed zero are handled by Rust parse::<f64>.
type String=std::borrow::Cow<'static,str>;
fn leading_space(c:char)->bool{matches!(c,'\t'|'\n'|'\r'|'\u{b}'|'\u{c}'|' '|'\u{a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')}
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
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<f64> {
    let owned_input=value2.to_string();let values=[owned_input.as_str(),"0","-0","+17","0012","0.5",".5","5.","123.456","9007199254740991","-9007199254740991","1.0000000000000002","-0.000","3.141592653589793","12345678901234567890","000000.125"];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0{let at=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%16).unwrap();output.push(parse_float(values[at]));}output
}
fn main(){}
