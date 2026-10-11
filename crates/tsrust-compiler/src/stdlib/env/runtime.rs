/// Lookup failures never contain the environment value, which may be sensitive.
#[derive(Debug,PartialEq,Eq)]
pub(crate) enum EnvError { InvalidName, NotUnicode }

/// Returns an independent UTF-8 value; absence and an empty value stay distinct.
/// This backend never mutates the process environment.
pub(crate) fn get(name:&str)->Result<Option<std::string::String>,EnvError> {
    if name.is_empty() || name.contains(['=','\0']) {return Err(EnvError::InvalidName);}
    match std::env::var(name) {
        Ok(value)=>Ok(Some(value)),
        Err(std::env::VarError::NotPresent)=>Ok(None),
        Err(std::env::VarError::NotUnicode(_))=>Err(EnvError::NotUnicode),
    }
}
