mod tsrust_native_decimal {
    use bigdecimal::{BigDecimal,FromPrimitive,ToPrimitive,Zero};
    type Error=(&'static str,&'static str);
    const RANGE:Error=("Range","Decimal exceeds 38 significant digits, 38 integer digits or 28 fractional places");
    const PRECISION:Error=("Precision","Operation would lose decimal precision");
    #[derive(Clone)]
    pub struct Decimal(BigDecimal);
    fn checked(value:BigDecimal)->Result<Decimal,Error>{
        let value=value.normalized();
        let (_,scale)=value.as_bigint_and_exponent();
        let digits=value.digits();
        if digits>38 || scale>28 || scale< -38 || (digits as i64)-scale>38{return Err(RANGE)}
        Ok(Decimal(value))
    }
    pub fn parse_decimal(text:&str)->Result<Decimal,Error>{
        if text.len()>128{return Err(RANGE)}
        let unsigned=text.strip_prefix('-').or_else(||text.strip_prefix('+')).unwrap_or(text);
        let mut parts=unsigned.split('.');
        let integer=parts.next().unwrap_or("");let fraction=parts.next();
        if integer.is_empty() || !integer.bytes().all(|v|v.is_ascii_digit()) || parts.next().is_some() || fraction.is_some_and(|v|v.is_empty() || !v.bytes().all(|v|v.is_ascii_digit())) {
            return Err(("Syntax","Expected signed decimal digits with an optional fractional part; no exponent or whitespace"));
        }
        checked(text.parse::<BigDecimal>().map_err(|_|("Syntax","Invalid decimal"))?)
    }
    pub fn from_int(value:i64)->Decimal{Decimal(BigDecimal::from(value))}
    pub fn format(value:&Decimal)->String{value.0.to_plain_string()}
    pub fn format_fixed(value:&Decimal,scale:i64)->Result<String,Error>{
        if !(0..=28).contains(&scale){return Err(("Scale","Scale must be between 0 and 28"))}
        let fixed=value.0.with_scale(scale);
        if fixed!=value.0{return Err(PRECISION)}
        Ok(fixed.to_plain_string())
    }
    pub fn add(left:&Decimal,right:&Decimal)->Result<Decimal,Error>{checked(&left.0+&right.0)}
    pub fn subtract(left:&Decimal,right:&Decimal)->Result<Decimal,Error>{checked(&left.0-&right.0)}
    pub fn multiply(left:&Decimal,right:&Decimal)->Result<Decimal,Error>{checked(&left.0*&right.0)}
    pub fn divide_exact(left:&Decimal,right:&Decimal)->Result<Decimal,Error>{
        if right.0.is_zero(){return Err(("DivideByZero","Decimal divisor must be nonzero"))}
        let quotient=&left.0/&right.0;
        // Division can round internally; only an exact inverse is accepted.
        if &quotient*&right.0!=left.0{return Err(PRECISION)}
        checked(quotient)
    }
    pub fn compare(left:&Decimal,right:&Decimal)->i64{match left.0.cmp(&right.0){std::cmp::Ordering::Less=>-1,std::cmp::Ordering::Equal=>0,std::cmp::Ordering::Greater=>1}}
    pub fn to_int(value:&Decimal)->Result<i64,Error>{
        if !value.0.is_integer(){return Err(PRECISION)}
        value.0.to_i64().ok_or(("Range","Decimal is outside signed 64-bit range"))
    }
    pub fn to_number(value:&Decimal)->Result<f64,Error>{
        let number=value.0.to_f64().filter(|v|v.is_finite()).ok_or(("Range","Decimal is outside finite number range"))?;
        if BigDecimal::from_f64(number).as_ref()!=Some(&value.0){return Err(PRECISION)}
        Ok(number)
    }
}
