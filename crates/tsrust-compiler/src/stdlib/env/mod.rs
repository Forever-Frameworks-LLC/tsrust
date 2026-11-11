//! Read-only environment backend. Public bindings are integrated separately.
pub(crate) mod bindings;
pub(crate) mod emit;
#[cfg(test)]
include!("runtime.rs");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lookup_preserves_missing_empty_unicode_and_owned_values() {
        let exe=std::env::current_exe().unwrap();
        let output=std::process::Command::new(exe)
            .args(["--exact","stdlib::env::tests::environment_child","--nocapture"])
            .env_clear().env("TSRUST_ENV_CHILD","1")
            .env("TSRUST_EMPTY","").env("TSRUST_TEXT","héllo 🦀")
            .output().unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        assert!(output.status.success(),"{}\n{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
    }
    #[test]
    fn environment_child() {
        if std::env::var_os("TSRUST_ENV_CHILD").is_none(){return;}
        assert_eq!(args(),Ok(std::env::args().skip(1).collect::<Vec<_>>()));
        assert_eq!(get("TSRUST_MISSING"),Ok(None));
        assert_eq!(get("TSRUST_EMPTY"),Ok(Some(String::new())));
        let mut value=get("TSRUST_TEXT").unwrap().unwrap();
        assert_eq!(value,"héllo 🦀");value.clear();
        assert_eq!(get("TSRUST_TEXT"),Ok(Some("héllo 🦀".into())));
        for name in ["","A=B","A\0B"] {assert_eq!(get(name),Err(EnvError::InvalidName));}
    }
    #[cfg(unix)]
    #[test]
    fn non_unicode_values_are_errors_without_lossy_conversion() {
        use std::os::unix::ffi::OsStringExt;
        let output=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","stdlib::env::tests::non_unicode_child","--nocapture"])
            .env_clear().env("TSRUST_ENV_CHILD","1")
            .env("TSRUST_RAW",std::ffi::OsString::from_vec(vec![0xff]))
            .env("HOME",std::ffi::OsString::from_vec(vec![0xff]))
            .env("TMPDIR",std::ffi::OsString::from_vec(vec![0xff]))
            .output().unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        assert!(output.status.success(),"{}\n{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
    }
    #[cfg(unix)]
    #[test]
    fn non_unicode_child() {
        if std::env::var_os("TSRUST_ENV_CHILD").is_none(){return;}
        assert_eq!(get("TSRUST_RAW"),Err(EnvError::NotUnicode));
        assert_eq!(vars(),Err(EnvError::NotUnicode));
        assert_eq!(home_dir(),Err(EnvError::NotUnicode));
        assert_eq!(temp_dir(),Err(EnvError::NotUnicode));
    }
    #[test]
    fn owned_snapshot_and_process_path_observations_follow_std(){
        assert_eq!(path(std::path::PathBuf::new()),Ok(String::new()));
        let values=vars();
        match values {Ok(mut rows)=>{let expected=std::env::vars_os().map(|(name,value)|Ok((name.into_string().map_err(|_|EnvError::NotUnicode)?,value.into_string().map_err(|_|EnvError::NotUnicode)?))).collect::<Result<Vec<_>,EnvError>>().unwrap();assert_eq!(rows,expected);if let Some((name,value))=rows.iter_mut().find(|(name,_)|!name.is_empty()&&!name.contains(['=','\0'])){let original=get(name).unwrap();value.clear();assert_eq!(get(name).unwrap(),original);}},Err(error)=>assert_eq!(error,EnvError::NotUnicode)}
        assert_eq!(current_dir(),std::env::current_dir().map_err(|error|EnvError::Io(error.to_string())).and_then(|path|path.into_os_string().into_string().map_err(|_|EnvError::NotUnicode)));
        assert_eq!(current_exe(),std::env::current_exe().map_err(|error|EnvError::Io(error.to_string())).and_then(|path|path.into_os_string().into_string().map_err(|_|EnvError::NotUnicode)));
        assert_eq!(temp_dir(),std::env::temp_dir().into_os_string().into_string().map_err(|_|EnvError::NotUnicode));
        assert_eq!(home_dir(),std::env::home_dir().map(|path|path.into_os_string().into_string().map_err(|_|EnvError::NotUnicode)).transpose());
    }
    #[test]
    fn io_errors_keep_the_owned_os_message(){
        let EnvError::Io(message)=io(std::io::Error::new(std::io::ErrorKind::PermissionDenied,"controlled OS failure"))else{panic!("Io identity")};
        assert_eq!(message,"controlled OS failure");
    }
    #[cfg(unix)]
    #[test]
    fn non_unicode_snapshot_names_and_working_directory_are_errors(){
        use std::os::unix::ffi::OsStringExt;
        let parent=std::env::temp_dir().join(format!("tsrust-env-unicode-{}",std::process::id()));std::fs::create_dir(&parent).unwrap();
        let raw=parent.join(std::ffi::OsString::from_vec(vec![b'x',0xff]));
        assert_eq!(path(raw.clone()),Err(EnvError::NotUnicode));
        let raw_directory=match std::fs::create_dir(&raw) {
            Ok(())=>true,
            // macOS filesystems can reject non-Unicode names before cwd exists.
            Err(error) if cfg!(target_os="macos") && error.raw_os_error()==Some(92)=>false,
            Err(error)=>panic!("creating non-Unicode cwd: {error}"),
        };
        let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","stdlib::env::tests::non_unicode_observers_child","--nocapture"]).current_dir(if raw_directory{&raw}else{&parent}).env_clear().env("TSRUST_ENV_UNICODE_CHILD",if raw_directory{"raw-cwd"}else{"unicode-cwd"}).env(std::ffi::OsString::from_vec(vec![b'X',0xff]),"safe").output().unwrap();
        if raw_directory {std::fs::remove_dir(&raw).unwrap();}std::fs::remove_dir(parent).unwrap();
        assert!(output.status.success(),"{}\n{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
    }
    #[cfg(unix)]
    #[test]
    fn non_unicode_observers_child(){
        if std::env::var_os("TSRUST_ENV_UNICODE_CHILD").is_none(){return;}
        if std::env::var("TSRUST_ENV_UNICODE_CHILD").unwrap()=="raw-cwd" {assert_eq!(current_dir(),Err(EnvError::NotUnicode));}else{assert!(current_dir().is_ok());}
        assert_eq!(vars(),Err(EnvError::NotUnicode));assert_eq!(get("TSRUST_RAW"),Ok(None));
    }
    #[cfg(unix)]
    #[test]
    fn unavailable_cwd_is_a_recoverable_owned_io_error(){
        let directory=std::env::temp_dir().join(format!("tsrust-env-cwd-{}",std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","stdlib::env::tests::unavailable_cwd_child","--nocapture"]).current_dir(&directory).env("TSRUST_ENV_REMOVED_CWD","1").output().unwrap();
        let _=std::fs::remove_dir(&directory);
        assert!(output.status.success(),"{}\n{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
    }
    #[cfg(unix)]
    #[test]
    fn unavailable_cwd_child(){
        if std::env::var_os("TSRUST_ENV_REMOVED_CWD").is_none(){return;}
        std::fs::remove_dir(std::env::current_dir().unwrap()).unwrap();
        let EnvError::Io(message)=current_dir().unwrap_err()else{panic!("cwd must preserve its Io error")};
        assert!(!message.is_empty());
    }
}
