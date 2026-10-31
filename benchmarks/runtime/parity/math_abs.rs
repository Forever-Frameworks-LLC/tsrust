// Independent hand-written Rust using standard f64 math and checked integer bookkeeping.
fn function0(value0: i64, value1: i64) -> f64 {let values:[f64;8]=[-2.5, -1.75, -0.25, -0.0, 0.0, 0.25, 1.75, 2.5];let mut sum=value1 as f64;
 for i in 0..value0{let value=values[usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%8).unwrap()];sum+=value.abs();}sum}

fn main(){}
