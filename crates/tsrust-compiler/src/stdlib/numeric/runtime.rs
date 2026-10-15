type Error = (&'static str, &'static str);
const RANGE: Error = ("Range", "Value is outside the requested numeric range");
const PRECISION: Error = ("Precision", "Conversion would lose numeric precision");
const FINITE: Error = ("NonFinite", "Value must be finite");
pub fn number_to_int(value:f64)->Result<i64,Error>{
 if !value.is_finite(){return Err(FINITE)}
 if value.fract()!=0.0{return Err(("Fraction", "Integer conversion requires a whole number"))}
 if value < -9223372036854775808.0 || value >= 9223372036854775808.0{return Err(RANGE)}
 Ok(value as i64)
}
pub fn to_number(value:i64)->Result<f64,Error>{let result=value as f64;if result as i128 != i128::from(value){Err(PRECISION)}else{Ok(result)}}
pub fn number_to_float32(value:f64)->Result<f64,Error>{
 if !value.is_finite(){return Err(FINITE)}
 let result=(value as f32) as f64;
 if !result.is_finite(){return Err(RANGE)}
 if result!=value{return Err(PRECISION)}
 Ok(result)
}
pub fn int_to_float32(value:i64)->Result<f64,Error>{
 let result=(value as f32) as f64;
 if result as i128 != i128::from(value){Err(PRECISION)}else{Ok(result)}
}
pub fn check_signed(value:i64,bits:i64)->Result<i64,Error>{
 if !matches!(bits,8|16|32|64){return Err(("Width","Width must be 8, 16, 32 or 64 bits"))}
 let bound=1_i128<<(bits-1);
 if i128::from(value)< -bound || i128::from(value)>=bound{Err(RANGE)}else{Ok(value)}
}
pub fn check_unsigned(value:i64,bits:i64)->Result<i64,Error>{
 if !matches!(bits,8|16|32|64){return Err(("Width","Width must be 8, 16, 32 or 64 bits"))}
 if value<0 || i128::from(value)>=(1_i128<<bits){Err(RANGE)}else{Ok(value)}
}
pub fn parse_int(text:&str)->Result<i64,Error>{
 let digits=text.strip_prefix('-').or_else(||text.strip_prefix('+')).unwrap_or(text);
 if digits.is_empty() || !digits.bytes().all(|v|v.is_ascii_digit()){return Err(("Syntax","Expected an optional sign followed by decimal digits"))}
 text.parse::<i64>().map_err(|_|RANGE)
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn exact_boundaries(){
 assert_eq!(parse_int("9223372036854775807"),Ok(i64::MAX));
 assert_eq!(parse_int("-9223372036854775808"),Ok(i64::MIN));
 for text in ["","+"," 1","1.0","1e2","0x10","1_000"]{assert_eq!(parse_int(text).unwrap_err().0,"Syntax");}
 assert_eq!(parse_int("9223372036854775808"),Err(RANGE));
 assert_eq!(to_number(i64::MAX),Err(PRECISION));assert_eq!(to_number(i64::MIN),Ok(i64::MIN as f64));
 assert_eq!(to_number(9007199254740993),Err(PRECISION));
 assert_eq!(number_to_int(9223372036854775808.0),Err(RANGE));assert_eq!(number_to_int(-9223372036854775808.0),Ok(i64::MIN));
 assert_eq!(number_to_int(0.5).unwrap_err().0,"Fraction");
 assert_eq!(number_to_float32(0.1),Err(PRECISION));assert_eq!(number_to_float32(f64::MIN_POSITIVE),Err(PRECISION));
 assert_eq!(number_to_float32(f64::MAX),Err(RANGE));assert_eq!(number_to_float32(f64::NAN),Err(FINITE));
 assert!(number_to_float32(-0.0).unwrap().is_sign_negative());assert_eq!(number_to_float32(1.5),Ok(1.5));
 assert_eq!(int_to_float32(16777217),Err(PRECISION));assert_eq!(int_to_float32(i64::MAX),Err(PRECISION));
 for bits in [8,16,32,64]{assert_eq!(check_unsigned(-1,bits),Err(RANGE));assert!(check_signed(0,bits).is_ok());}
 assert_eq!(check_signed(128,8),Err(RANGE));assert_eq!(check_unsigned(255,8),Ok(255));assert_eq!(check_unsigned(256,8),Err(RANGE));
 assert_eq!(check_unsigned(i64::MAX,64),Ok(i64::MAX));assert_eq!(check_signed(0,7).unwrap_err().0,"Width");
 }
}
