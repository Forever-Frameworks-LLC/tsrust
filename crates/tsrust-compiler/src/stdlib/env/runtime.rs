/// Lookup failures never contain the environment value, which may be sensitive.
#[derive(Debug,PartialEq,Eq)]
pub(crate) enum EnvError { InvalidName, NotUnicode, Io(std::string::String) }

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

/// Program arguments exclude the executable name and retain exact Unicode values.
pub(crate) fn args()->Result<Vec<std::string::String>,EnvError> {
    std::env::args_os().skip(1).map(|value|value.into_string().map_err(|_|EnvError::NotUnicode)).collect()
}

/// A complete owned snapshot; names are not restricted by the lookup validator.
pub(crate) fn vars()->Result<Vec<(std::string::String,std::string::String)>,EnvError> {
    std::env::vars_os().map(|(name,value)|Ok((unicode(name)?,unicode(value)?))).collect()
}
fn unicode(value:std::ffi::OsString)->Result<std::string::String,EnvError> {
    value.into_string().map_err(|_|EnvError::NotUnicode)
}
fn path(value:std::path::PathBuf)->Result<std::string::String,EnvError> {unicode(value.into_os_string())}
fn io(error:std::io::Error)->EnvError {EnvError::Io(error.to_string())}
pub(crate) fn current_dir()->Result<std::string::String,EnvError> {
    path(std::env::current_dir().map_err(io)?)
}
pub(crate) fn current_exe()->Result<std::string::String,EnvError> {
    path(std::env::current_exe().map_err(io)?)
}
pub(crate) fn temp_dir()->Result<std::string::String,EnvError> {path(std::env::temp_dir())}
pub(crate) fn home_dir()->Result<Option<std::string::String>,EnvError> {
    std::env::home_dir().map(path).transpose()
}
