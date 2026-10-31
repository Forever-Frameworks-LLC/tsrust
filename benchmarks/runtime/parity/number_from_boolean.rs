// Independent hand-written Rust; standard f64 operations and checked integer bookkeeping.
fn function0(value0: i64, value1: i64) -> f64 {
 let mut sum=0.0;for i in 0..value0{let flag=i.checked_add(value1).unwrap()%2==0;sum+=u8::from(flag) as f64;}sum
}
fn main(){}
