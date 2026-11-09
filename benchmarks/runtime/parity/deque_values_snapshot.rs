fn function0(value0: i64, value1: i64) -> i64 {let mut checksum=value1;for i in 0..value0{let mut deque=std::collections::VecDeque::from([vec![value1+i],vec![value1+i+1]]);deque.pop_front();deque.push_back(vec![value1+i+2]);let mut rows:Vec<_>=deque.iter().cloned().collect();rows[0].push(9);checksum+=rows[0][0]+rows[0][1]+2*rows[1][0]+deque.front().unwrap().len() as i64;}checksum}
fn main(){}
