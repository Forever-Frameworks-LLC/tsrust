use sha2::{Digest,Sha256,Sha384,Sha512};use hmac::{Hmac,KeyInit,Mac};
fn input_bytes(size:usize,seed:i64)->Vec<u8>{(0..size).map(|i|((i+seed as usize)%251)as u8).collect()}
fn fold(mut sum:i64,bytes:&[u8])->i64{for byte in bytes{sum=(sum*131+i64::from(*byte))%1_000_000_007;}sum}
fn function0(n:i64,seed:i64)->i64{let size=4096i64;let input=input_bytes(usize::try_from(size).unwrap(),seed);let key=vec![7;32];let mut sum=0;for _ in 0..n{let mut h=Hmac::<Sha256>::new_from_slice(&key).unwrap();h.update(&input);let output=h.finalize().into_bytes().to_vec();sum=fold(sum,&output);}sum}
fn main(){}
