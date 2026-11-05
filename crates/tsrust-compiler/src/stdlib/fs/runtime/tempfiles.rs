//! Selected temporary-directory backend; ordinary file programs do not include it.
use crate::stdlib::fs::runtime::fs::FsFailure;

pub fn create_temp_dir(parent:Option<&str>,prefix:Option<&str>)->Result<String,FsFailure> {
    if let Some(prefix)=prefix {
        let bytes=prefix.as_bytes();
        let drive=bytes.len()>=2 && bytes[0].is_ascii_alphabetic() && bytes[1]==b':';
        if prefix.contains(['\0','/','\\']) || drive {
            return Err(FsFailure {code:"invalid-input",operation:"createTempDir",message:"Temporary directory prefix must stay within one filename component.".into(),os_code:None});
        }
    }
    let mut builder=tempfile::Builder::new();
    if let Some(prefix)=prefix {builder.prefix(prefix);}
    let directory=match parent {Some(parent)=>builder.tempdir_in(parent),None=>builder.tempdir()}
        .map_err(|error|FsFailure::io("createTempDir",error))?;
    // Keep cleanup ownership while checking the string boundary. A failed
    // conversion drops TempDir and attempts the backend's normal cleanup.
    let path=directory.path().to_str().ok_or_else(||FsFailure {code:"not-unicode",operation:"createTempDir",message:"Temporary directory path is not valid Unicode.".into(),os_code:None})?.to_owned();
    let _kept=directory.keep();
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creation_is_unique_kept_and_uses_defaults_without_fallback_creation() {
        let parent=tempfile::tempdir().unwrap();let parent=parent.path().to_str().unwrap();
        let first=create_temp_dir(Some(parent),None).unwrap();let second=create_temp_dir(Some(parent),Some("")).unwrap();
        assert_ne!(first,second);assert!(std::path::Path::new(&first).is_dir());assert!(std::path::Path::new(&second).is_dir());
        assert!(std::path::Path::new(&first).file_name().unwrap().to_str().unwrap().starts_with(".tmp"));
        assert_eq!(std::path::Path::new(&second).file_name().unwrap().to_str().unwrap().chars().count(),6);
        let default=create_temp_dir(None,Some("tsrust-test-")).unwrap();assert!(std::path::Path::new(&default).is_dir());std::fs::remove_dir(&default).unwrap();
        let missing=std::path::Path::new(parent).join("missing");let error=create_temp_dir(missing.to_str(),None).unwrap_err();assert_eq!((error.code,error.operation),("not-found","createTempDir"));assert!(!missing.exists());
    }
    #[test]
    fn invalid_prefix_fails_before_parent_access_without_extra_size_policy() {
        for prefix in ["../outside","folder/name","folder\\name","C:relative","C:\\absolute","a\0b"] {
            let error=create_temp_dir(Some("missing-parent"),Some(prefix)).unwrap_err();
            assert_eq!((error.code,error.operation,error.os_code),("invalid-input","createTempDir",None));
        }
        let parent=tempfile::tempdir().unwrap();let path=create_temp_dir(parent.path().to_str(),Some("é-🦀-")).unwrap();
        assert!(std::path::Path::new(&path).file_name().unwrap().to_str().unwrap().starts_with("é-🦀-"));
    }
}
