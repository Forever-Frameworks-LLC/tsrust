use sha2::{Digest,Sha256,Sha384,Sha512};use hmac::{Hmac,KeyInit,Mac};
fn input_bytes(size:usize,seed:i64)->Vec<u8>{(0..size).map(|i|((i+seed as usize)%251)as u8).collect()}
fn fold(mut sum:i64,bytes:&[u8])->i64{for byte in bytes{sum=(sum*131+i64::from(*byte))%1_000_000_007;}sum}
fn function0(n:i64,seed:i64)->i64{let size=262144i64;let input=input_bytes(usize::try_from(size).unwrap(),seed);let salt=vec![7;16];let info=b"profile public info";let mut sum=0;for _ in 0..n{let mut output=vec![0;32];hkdf::Hkdf::<Sha256>::new(Some(&salt),&input).expand(info,&mut output).unwrap();sum=fold(sum,&output);}sum}
fn main(){}
