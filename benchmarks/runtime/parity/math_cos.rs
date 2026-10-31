// Independent hand-written Rust using standard f64 math and checked integer bookkeeping.
fn function0(value0: i64, value1: i64) -> f64 {let table0:[f64;8]=[-1.25, -1.0, -0.5, -0.125, 0.125, 0.5, 1.0, 1.125];let mut sum=value1 as f64;
 for i in 0..value0{let position=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();let jitter=(i.checked_mul(17).unwrap().checked_add(value1).unwrap()%5) as f64;let argument0=table0[position]+jitter*0.03125;sum+=function1(argument0);}sum}
fn function1(value0: f64) -> f64 {value0.cos()}
fn main(){}
