use std::net::{IpAddr,SocketAddr};
fn function0(n:i64,seed:i64,text:&String)->i64{let address=text.parse::<IpAddr>().unwrap();let mut sum=0;for i in 0..n{let formatted=address.to_string();for byte in formatted.bytes(){sum=(sum*131+i64::from(byte))%1000000007;}}sum}fn main(){}
