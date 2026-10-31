// Independent hand-written Rust; standard f64 operations and checked integer bookkeeping.
fn function0(value0: i64, value1: i64) -> f64 {
 let texts=["", " -0 ", "2.5", "-3.125", " 1e2 ", "0.125", "-0.5"];let mut sum=0.0;
 for i in 0..value0{let text=texts[usize::try_from(i.checked_mul(13).unwrap().checked_add(value1).unwrap()%7).unwrap()].trim();let number=if text.is_empty(){0.0}else{text.parse::<f64>().unwrap()};sum+=number;}sum
}
fn main(){}
