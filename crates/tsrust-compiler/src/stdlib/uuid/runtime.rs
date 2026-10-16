mod tsrust_native_uuid {
    type Error=(&'static str,&'static str);
    #[derive(Clone)]
    pub struct Uuid(uuid::Uuid);
    pub fn parse_uuid(text:&str)->Result<Uuid,Error>{
        if text.len()!=36 || !text.bytes().enumerate().all(|(i,b)|if matches!(i,8|13|18|23){b==b'-'}else{b.is_ascii_hexdigit()}){return Err(("Syntax","Expected 8-4-4-4-12 hexadecimal UUID text"))}
        uuid::Uuid::parse_str(text).map(Uuid).map_err(|_|("Syntax","Invalid UUID text"))
    }
    pub fn format_uuid(value:&Uuid)->String{value.0.hyphenated().to_string()}
    pub fn from_bytes(bytes:&[u8])->Result<Uuid,Error>{uuid::Uuid::from_slice(bytes).map(Uuid).map_err(|_|("Length","UUID bytes must contain exactly 16 bytes"))}
    pub fn to_bytes(value:&Uuid)->Vec<u8>{value.0.as_bytes().to_vec()}
    pub fn compare(left:&Uuid,right:&Uuid)->i64{match left.0.cmp(&right.0){std::cmp::Ordering::Less=>-1,std::cmp::Ordering::Equal=>0,std::cmp::Ordering::Greater=>1}}
    pub fn version(value:&Uuid)->i64{i64::from(value.0.as_bytes()[6]>>4)}
    fn random_with(fill:impl FnOnce(&mut [u8])->Result<(),()>)->Result<Uuid,Error>{
        let mut bytes=[0u8;16];fill(&mut bytes).map_err(|_|("Entropy","Operating system random source failed"))?;
        Ok(Uuid(uuid::Builder::from_random_bytes(bytes).into_uuid()))
    }
    pub fn random_uuid()->Result<Uuid,Error>{random_with(|bytes|getrandom::fill(bytes).map_err(|_|()))}
    #[cfg(test)] pub(super) fn failed_entropy()->bool{random_with(|bytes|{bytes[0]=255;Err(())}).is_err()}
    #[cfg(test)] pub(super) fn fixed_entropy()->Uuid{random_with(|bytes|{bytes.fill(0);Ok(())}).unwrap()}
}
