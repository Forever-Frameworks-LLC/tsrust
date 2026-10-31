// Independent hand-written Rust; standard f64 operations and checked integer bookkeeping.
fn function0(value0: i64, value1: i64) -> i64 {
 let values:[f64;8]=[f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0, 0.0, 1.5, -2.25, 2.25];let mut sum=value1;
 for i in 0..value0{let a=values[usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap()];let b=values[usize::try_from(i.checked_mul(17).unwrap().checked_add(value1).unwrap()%8).unwrap()];let code=i64::from(a<=b);sum=sum.checked_mul(131).unwrap().checked_add(code).unwrap()%1000000007;}sum
}
fn main(){}
