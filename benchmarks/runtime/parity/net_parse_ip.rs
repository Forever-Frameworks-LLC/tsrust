use std::net::{IpAddr,SocketAddr};
fn function0(n:i64,seed:i64,text:&String)->i64{let mut sum=0;for i in 0..n{let address=text.parse::<IpAddr>().unwrap();let family=if address.is_ipv6(){6}else{4};sum=(sum*131+family+seed+i)%1000000007;}sum}fn main(){}
