mod tsrust_native_crypto {
    use sha2::{Digest,Sha256,Sha384,Sha512};
    use hmac::{Hmac,KeyInit,Mac};
    use rand::{SeedableRng,distr::{Distribution,Uniform},rngs::{StdRng,SysRng}};
    use subtle::ConstantTimeEq;
    pub type Error=(&'static str,&'static str);
    pub fn random_bytes(length:i64)->Result<Vec<u8>,Error>{
        let length=usize::try_from(length).map_err(|_|("Range","Random byte count must be nonnegative and fit the target size"))?;
        let mut bytes=vec![0;length];
        getrandom::fill(&mut bytes).map_err(|_|("Random","Operating system randomness unavailable"))?;Ok(bytes)
    }
    pub fn sha256(bytes:&[u8])->Vec<u8>{Sha256::digest(bytes).to_vec()}
    pub fn timing_safe_equal(left:&[u8],right:&[u8])->bool{bool::from(left.ct_eq(right))}

    pub fn sha384(bytes:&[u8])->Vec<u8>{Sha384::digest(bytes).to_vec()}
    pub fn sha512(bytes:&[u8])->Vec<u8>{Sha512::digest(bytes).to_vec()}
    pub enum Hasher{Sha256(Sha256),Sha384(Sha384),Sha512(Sha512)}
    impl Hasher{
        pub fn new(algorithm:&str)->Self{match algorithm{"sha256"=>Self::Sha256(Sha256::new()),"sha384"=>Self::Sha384(Sha384::new()),"sha512"=>Self::Sha512(Sha512::new()),_=>unreachable!("checked hash algorithm")}}
        pub fn update(&mut self,bytes:&[u8]){match self{Self::Sha256(value)=>value.update(bytes),Self::Sha384(value)=>value.update(bytes),Self::Sha512(value)=>value.update(bytes)}}
        pub fn finalize(self)->Vec<u8>{match self{Self::Sha256(value)=>value.finalize().to_vec(),Self::Sha384(value)=>value.finalize().to_vec(),Self::Sha512(value)=>value.finalize().to_vec()}}
    }
    pub fn create_hasher(algorithm:&str)->Hasher{Hasher::new(algorithm)}
    pub fn update(hasher:&mut Hasher,bytes:&[u8]){hasher.update(bytes)}
    pub fn finalize(hasher:Hasher)->Vec<u8>{hasher.finalize()}
    pub fn hmac_sha256(key:&[u8],input:&[u8])->Vec<u8>{let mut mac=Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");mac.update(input);mac.finalize().into_bytes().to_vec()}
    pub fn verify_hmac_sha256(key:&[u8],input:&[u8],tag:&[u8])->bool{let mut mac=Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");mac.update(input);mac.verify_slice(tag).is_ok()}
    pub fn hkdf_sha256(input:&[u8],salt:&[u8],info:&[u8],length:i64)->Result<Vec<u8>,Error>{
        let length=usize::try_from(length).map_err(|_|("Range","HKDF output length must be nonnegative and fit the target"))?;
        if length>255*Sha256::output_size(){return Err(("Range","HKDF SHA256 output exceeds 8160 bytes"));}
        let mut output=vec![0;length];hkdf::Hkdf::<Sha256>::new(Some(salt),input).expand(info,&mut output).map_err(|_|("Range","HKDF SHA256 output exceeds 8160 bytes"))?;Ok(output)
    }
    pub fn random_int(min:i64,max:i64)->Result<i64,Error>{
        let distribution=Uniform::new(min,max).map_err(|_|("Range","randomInt requires min strictly below max"))?;
        let mut rng=StdRng::try_from_rng(&mut SysRng).map_err(|_|("Random","Operating system randomness unavailable"))?;Ok(distribution.sample(&mut rng))
    }
}
