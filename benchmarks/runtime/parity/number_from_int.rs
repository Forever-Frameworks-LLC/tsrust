// Independent hand-written Rust; standard f64 operations and checked integer bookkeeping.
fn function0(value0: i64, value1: i64) -> f64 {
 let mut sum=0.0;for i in 0..value0{let integer=(i.checked_mul(37).unwrap().checked_add(value1).unwrap()%2001).checked_sub(1000).unwrap();sum+=integer as f64;}sum
}
fn main(){}
