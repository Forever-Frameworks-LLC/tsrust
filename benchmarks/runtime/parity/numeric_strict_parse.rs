fn function0(value0: i64, value1: i64) -> i64 {let mut checksum=value1;for i in 0..value0{let integer=i64::from_str_radix("7fffffffffffffff",16).unwrap();let negative=i64::from_str_radix("-100",2).unwrap();let number="1.25e2".parse::<f64>().unwrap();assert_eq!(integer,i64::MAX);assert_eq!(number,125.0);checksum+=negative+125;}checksum}
fn main(){}
