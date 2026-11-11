use std::collections::{HashMap,HashSet};
fn same(left:&HashMap<i64,Vec<i64>>,right:&HashMap<i64,Vec<i64>>)->bool{left.len()==right.len()&&left.iter().all(|(key,value)|right.get(key)==Some(value))}
fn function0(value0: i64, value1: i64) -> i64{let mut first=HashMap::new();let mut second=HashMap::new();let mut different=HashMap::new();let mut set_a=HashSet::new();let mut set_b=HashSet::new();for j in 0..8{first.insert(j,vec![j*j,j+1]);second.insert(7-j,vec![(7-j)*(7-j),8-j]);different.insert(j,vec![j*j,j+2]);set_a.insert(j);set_b.insert(7-j);}let mut checksum=0;for i in 0..value0{let at=(i*13+value1)%8;assert!(same(&first,&second));assert!(set_a==set_b);assert!(!same(&first,&different));checksum=(checksum*131+first[&at][0])%1000000007;}checksum}
fn main(){}
