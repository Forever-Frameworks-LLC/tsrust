// Independent hand-written Rust; standard f64 operations and checked integer bookkeeping.
fn function0(value0: i64, value1: i64) -> f64 {
 let left:[f64;7]=[0.0, 0.25, -0.5, 0.75, 2.5, -3.0, -0.0];let right:[f64;7]=[0.5, -0.5, 2.0, -2.0, 4.0, -4.0, 1.0];let mut sum=value1 as f64;
 for i in 0..value0{let a=left[usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%7).unwrap()];let b=right[usize::try_from(i.checked_mul(17).unwrap().checked_add(value1).unwrap()%7).unwrap()];let result=a-b;sum+=result;}sum
}
fn main(){}
