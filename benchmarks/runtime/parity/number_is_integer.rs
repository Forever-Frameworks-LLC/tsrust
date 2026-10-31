// Independent hand-written Rust; standard f64 operations and checked integer bookkeeping.
fn function0(value0: i64, value1: i64) -> i64 {
 let values:[f64;24]=[f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0, 0.0, 1.0, -1.0, 1.5, -1.5, 5e-324, -5e-324, 1.7976931348623157e308, -1.7976931348623157e308, 9007199254740991.0, -9007199254740991.0, 9007199254740992.0, -9007199254740992.0, 4503599627370495.5, 4503599627370496.0, 4503599627370497.0, 10.0, -10.0, 0.25, -0.25];let mut digest=value1;
 for i in 0..value0{let value=values[usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%24).unwrap()];let bit=i64::from(value.is_finite() && value.trunc()==value);digest=digest.checked_mul(131).unwrap().checked_add(bit).unwrap()%1000000007;}digest
}
fn main(){}
