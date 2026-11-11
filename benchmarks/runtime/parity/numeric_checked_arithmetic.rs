fn function0(value0: i64, value1: i64) -> i64 {let mut checksum=value1;for i in 0..value0{let a=value1+i+10;checksum+=a.checked_add(3).unwrap()+a.checked_sub(3).unwrap()+a.checked_mul(3).unwrap()+a.checked_div(3).unwrap()+a.checked_rem(3).unwrap();}checksum}
fn main(){}
