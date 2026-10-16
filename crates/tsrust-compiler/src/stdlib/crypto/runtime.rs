mod tsrust_native_crypto {
    use sha2::{Digest,Sha256};
    use subtle::ConstantTimeEq;
    pub type Error=(&'static str,&'static str);
    pub fn random_bytes(length:i64)->Result<Vec<u8>,Error>{
        if !(1..=1048576).contains(&length){return Err(("Range","Random byte count must be 1 through 1048576"));}
        let mut bytes=vec![0;length as usize];
        getrandom::fill(&mut bytes).map_err(|_|("Random","Operating system randomness unavailable"))?;Ok(bytes)
    }
    pub fn sha256(bytes:&[u8])->Vec<u8>{Sha256::digest(bytes).to_vec()}
    pub fn timing_safe_equal(left:&[u8],right:&[u8])->bool{bool::from(left.ct_eq(right))}
}
