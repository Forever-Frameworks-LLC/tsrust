#[derive(Debug,PartialEq,Eq)]
pub(crate) struct OsError {pub code:&'static str,pub message:std::string::String}
pub(crate) fn available_parallelism()->Result<i64,OsError> {
    let count=std::thread::available_parallelism().map_err(|error|OsError{code:"Io",message:error.to_string()})?.get();
    checked_count(count)
}
fn checked_count(count:usize)->Result<i64,OsError>{i64::try_from(count).map_err(|_|OsError{code:"Range",message:"Available parallelism exceeds int".into()})}
#[cfg(test)]
mod tests {
    #[test]
    fn parallelism_is_a_positive_uncached_checked_observation(){
        for _ in 0..2 {match super::available_parallelism(){Ok(value)=>assert!(value>0),Err(error)=>{assert_eq!(error.code,"Io");assert!(!error.message.is_empty());}}}
        assert_eq!(super::checked_count(1),Ok(1));
        if usize::BITS==64 {let error=super::checked_count(usize::MAX).unwrap_err();assert_eq!(error.code,"Range");assert_eq!(error.message,"Available parallelism exceeds int");}
    }
}
