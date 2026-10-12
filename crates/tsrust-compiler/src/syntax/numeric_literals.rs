//! Numeric spelling validation is independent of recursive expression parsing.
use super::{Token,TokenKind};
use crate::diagnostic::{Diagnostic,Span};
fn digit(byte:u8,radix:u32)->bool{(byte as char).to_digit(radix).is_some()}
fn digits(bytes:&[u8],mut at:usize,radix:u32,start:usize)->Result<usize,Diagnostic>{
    let begin=at;
    while let Some(&byte)=bytes.get(at){
        if digit(byte,radix){at+=1;continue;}
        if byte==b'_'{
            if at==begin||!bytes.get(at+1).is_some_and(|&next|digit(next,radix)){
                return Err(Diagnostic::new("syntax","A numeric separator must occur between digits of the same part.",Span::new(start,at+1)));
            }
            at+=1;continue;
        }
        break;
    }
    Ok(at)
}
#[inline(never)]
pub(super) fn scan(source:&str,start:usize)->Result<(Token,usize),Diagnostic>{
    let bytes=source.as_bytes();
    if bytes[start]==b'0'&&let Some(&prefix)=bytes.get(start+1)&&matches!(prefix,b'b'|b'B'|b'o'|b'O'|b'x'|b'X'){
        let radix=match prefix{b'b'|b'B'=>2,b'o'|b'O'=>8,_=>16};
        let at=digits(bytes,start+2,radix,start)?;
        if at==start+2{return Err(Diagnostic::new("syntax","A radix prefix requires digits.",Span::new(start,at)));}
        if bytes.get(at).is_some_and(|byte|byte.is_ascii_alphanumeric()||*byte==b'_'){
            return Err(Diagnostic::new("syntax","Invalid digit or suffix in a radix literal.",Span::new(start,at+1)));
        }
        return Ok((Token{text:source[start..at].into(),span:Span::new(start,at),kind:TokenKind::Integer},at));
    }
    let mut at=digits(bytes,start,10,start)?;let integer_end=at;let mut number=false;
    if bytes[start]==b'0'&&source[start..integer_end].contains('_'){
        return Err(Diagnostic::new("syntax","Numeric separators are not allowed after a leading decimal zero.",Span::new(start,integer_end)));
    }
    if bytes.get(at)==Some(&b'.'){
        if bytes.get(at+1)==Some(&b'_'){return Err(Diagnostic::new("syntax","A numeric separator cannot follow a decimal point.",Span::new(start,at+2)));}
        if !bytes.get(at+1).is_some_and(|byte|byte.is_ascii_alphabetic()&&!matches!(*byte,b'e'|b'E')){
            number=true;at=digits(bytes,at+1,10,start)?;
        }
    }
    if bytes.get(at).is_some_and(|byte|matches!(*byte,b'e'|b'E')){
        number=true;at+=1;
        if bytes.get(at).is_some_and(|byte|matches!(*byte,b'+'|b'-')){at+=1;}
        let exponent=at;at=digits(bytes,at,10,start)?;
        if at==exponent{return Err(Diagnostic::new("syntax","A numeric exponent needs decimal digits.",Span::new(start,at)));}
    }
    if bytes.get(at).is_some_and(|byte|byte.is_ascii_alphabetic()||*byte==b'_'){
        return Err(Diagnostic::new("unsupported","Numeric literal suffixes, including BigInt, are unsupported.",Span::new(start,at+1)));
    }
    Ok((Token{text:source[start..at].into(),span:Span::new(start,at),kind:if number{TokenKind::Number}else{TokenKind::Integer}},at))
}
/// Keep exact decimal integer spelling for later int/number contextual checking.
/// Once the magnitude exceeds 400 decimal digits it is above finite f64 range;
/// a bounded overflow spelling preserves both number infinity and int rejection.
#[inline(never)]
pub(crate) fn integer_spelling(raw:&str)->String{
    let radix=if raw.starts_with("0b")||raw.starts_with("0B"){2}else if raw.starts_with("0o")||raw.starts_with("0O"){8}else if raw.starts_with("0x")||raw.starts_with("0X"){16}else{return raw.replace('_',"");};
    let mut decimal=vec![0u8];
    for character in raw[2..].chars().filter(|&ch|ch!='_'){
        let mut carry=character.to_digit(radix).expect("validated radix digit");
        for place in &mut decimal{let value=u32::from(*place)*radix+carry;*place=(value%10)as u8;carry=value/10;}
        while carry>0{decimal.push((carry%10)as u8);carry/=10;}
        if decimal.len()>400{return format!("1{}","0".repeat(400));}
    }
    decimal.into_iter().rev().map(|place|char::from(b'0'+place)).collect()
}
