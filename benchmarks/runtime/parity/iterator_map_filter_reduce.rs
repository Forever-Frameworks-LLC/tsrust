fn function0(value0: i64, value1: i64) -> i64 {let n=value0;let seed=value1;let mut result=0;for i in 0..n{let mut values=Vec::new();for j in 0..64{values.push((j*17+seed+i)%997);}let observed=values.into_iter().filter(|value|value%2==0).map(|value|value*3+1).fold(7,|sum,value|(sum*131+value)%1000000007);result=(result*131+observed)%1000000007;}result}
fn observe(values:Vec<i64>)->i64{values.into_iter().fold(7,|sum,value|(sum*131+value)%1000000007)}
fn main(){}
