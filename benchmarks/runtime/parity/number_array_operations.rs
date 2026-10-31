// Independent growable Rust Vec mutation, clone and retained outputs.
fn function0(value0: i64, value1: i64) -> Vec<f64>{let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(8).unwrap()).unwrap());for i in 0..value0{let mut values=vec![value1.checked_add(i).unwrap() as f64+0.25,-0.0,2.25];values.push(i as f64+0.5);let saved=values.clone();values[0]=i as f64+1.5;values.reverse();let removed=values.remove(1);
 output.extend([saved[0],saved[1],removed,values[0],values[1],values[2],f64::from(u8::from(values.contains(&2.25))),values.iter().position(|&v|v==0.0).map(|n|n as f64).unwrap_or(-1.0)]);}output}
fn main(){}
