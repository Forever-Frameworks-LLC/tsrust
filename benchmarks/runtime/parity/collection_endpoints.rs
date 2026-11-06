fn function0(value0: i64, value1: i64) -> i64 {let mut checksum=value1;for i in 0..value0{let values=std::collections::BTreeSet::from([value1+3,value1+1,value1+2]);let map=std::collections::BTreeMap::from([(value1+3,0),(value1+1,0),(value1+2,0)]);checksum+=values.first().unwrap()+values.last().unwrap()+map.first_key_value().unwrap().0+map.last_key_value().unwrap().0;}checksum}
fn main(){}
