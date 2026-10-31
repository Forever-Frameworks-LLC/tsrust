// Independent hand-written Rust using standard f64 math and checked integer bookkeeping.
fn function0(value0: i64, value1: i64) -> f64 {let table0:[f64;8]=[0.25, 0.375, 0.5, 0.75, 1.25, 1.5, 2.0, 3.5];let mut sum=value1 as f64;
 for i in 0..value0{let position=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();let jitter=(i.checked_mul(17).unwrap().checked_add(value1).unwrap()%5) as f64;let argument0=table0[position]+jitter*0.015625;sum+=function1(argument0);}sum}
fn function1(value0: f64) -> f64 {value0.ln()}
fn main(){}
