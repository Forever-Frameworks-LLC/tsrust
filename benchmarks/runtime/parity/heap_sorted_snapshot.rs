fn function0(value0: i64, value1: i64) -> i64 {let mut checksum=value1;for i in 0..value0{let heap:std::collections::BinaryHeap<_>=[7,1,9,3,3,0,8,2].map(|v|v+value1+i).into_iter().collect();let mut rows:Vec<_>=heap.iter().copied().collect();rows.sort_unstable();for(j,v)in rows.iter().enumerate(){checksum+=(j as i64+1)*v;}checksum+=heap.len() as i64;}checksum}
fn main(){}
