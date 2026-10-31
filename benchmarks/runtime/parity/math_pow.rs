// Independent hand-written Rust using standard f64 math and checked integer bookkeeping.
fn function0(value0: i64, value1: i64) -> f64 {let table0:[f64;8]=[0.5, 0.625, 0.75, 1.125, 1.5, 1.75, 2.25, 2.5];let table1:[f64;8]=[-2.5, -1.75, -1.25, -0.625, 0.375, 0.75, 1.25, 2.5];let mut sum=value1 as f64;
 for i in 0..value0{let position=usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap();let jitter=(i.checked_mul(17).unwrap().checked_add(value1).unwrap()%5) as f64;let argument0=table0[position]+jitter*0.015625;let argument1=table1[position]+jitter*0.0078125;sum+=function1(argument0,argument1);}sum}
fn function1(value0: f64, value1: f64) -> f64 {value0.powf(value1)}
fn main(){}
