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
            .output().unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        assert!(output.status.success(),"{}\n{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
    }
    #[cfg(unix)]
    #[test]
    fn non_unicode_child() {
        if std::env::var_os("TSRUST_ENV_CHILD").is_none(){return;}
        assert_eq!(get("TSRUST_RAW"),Err(EnvError::NotUnicode));
    }
}
