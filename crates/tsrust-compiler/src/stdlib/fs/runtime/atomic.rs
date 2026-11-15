//! Staged text replacement selected independently from ordinary file operations.
use crate::stdlib::fs::runtime::fs::FsFailure;

pub fn rename_new(path: &str, destination: &str) -> Result<(), FsFailure> {
    let io = |error| FsFailure::io("renameNew", error);
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        use std::ffi::CString;
        let source = CString::new(path).map_err(|e| io(e.into()))?;
        let target = CString::new(destination).map_err(|e| io(e.into()))?;
        #[cfg(target_os = "macos")]
        let result = unsafe { nix::libc::renamex_np(source.as_ptr(), target.as_ptr(), nix::libc::RENAME_EXCL) };
        #[cfg(target_os = "linux")]
        let result = unsafe { nix::libc::syscall(nix::libc::SYS_renameat2, nix::libc::AT_FDCWD, source.as_ptr(), nix::libc::AT_FDCWD, target.as_ptr(), nix::libc::RENAME_NOREPLACE) };
        if result == 0 { Ok(()) } else { Err(io(std::io::Error::last_os_error())) }
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        unsafe extern "system" { fn MoveFileW(existing: *const u16, new: *const u16) -> i32; }
        let wide = |s: &str| std::ffi::OsStr::new(s).encode_wide().chain(Some(0)).collect::<Vec<_>>();
        if path.contains('\0') || destination.contains('\0') { return Err(io(std::io::ErrorKind::InvalidInput.into())); }
        if unsafe { MoveFileW(wide(path).as_ptr(), wide(destination).as_ptr()) } != 0 { Ok(()) } else { Err(io(std::io::Error::last_os_error())) }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
    Err(io(std::io::ErrorKind::Unsupported.into()))
}

fn atomic_failure(code:&'static str,message:&str)->FsFailure {
    FsFailure {code,operation:"writeTextFileAtomic",message:message.into(),os_code:None}
}

#[cfg(unix)]
fn same_revision(left:&std::fs::Metadata,right:&std::fs::Metadata)->bool {
    use std::os::unix::fs::MetadataExt;
    (left.dev(),left.ino(),left.len(),left.mtime(),left.mtime_nsec(),left.ctime(),left.ctime_nsec(),left.mode(),left.uid(),left.gid(),left.nlink())==
    (right.dev(),right.ino(),right.len(),right.mtime(),right.mtime_nsec(),right.ctime(),right.ctime_nsec(),right.mode(),right.uid(),right.gid(),right.nlink())
}

#[cfg(unix)]
fn matches_text(file:&mut std::fs::File,expected:&str)->std::io::Result<bool> {
    use std::io::{Read,Seek,SeekFrom};
    file.seek(SeekFrom::Start(0))?;
    let mut offset=0;let mut buffer=[0u8;8192];
    loop {
        let count=match file.read(&mut buffer) {Err(error) if error.kind()==std::io::ErrorKind::Interrupted=>continue,result=>result?};
        if count==0{return Ok(offset==expected.len());}
        if !expected.as_bytes().get(offset..).is_some_and(|rest|rest.starts_with(&buffer[..count])){return Ok(false);}
        offset+=count;
    }
}

/// Stage in the destination directory. Only committed-not-durable can fail after replacement.
pub fn write_text_file_atomic(path:&str,text:&str,expected:Option<&str>)->Result<(),FsFailure> {
    write_text_file_atomic_with(path,text,expected,||Ok(()),std::fs::File::sync_all)
}

fn write_text_file_atomic_with(path:&str,text:&str,expected:Option<&str>,before_commit:impl FnOnce()->Result<(),FsFailure>,sync_directory:impl FnOnce(&std::fs::File)->std::io::Result<()>)->Result<(),FsFailure> {
    // shortcut: implement metadata-preserving Windows replacement before Windows Studio acceptance.
    #[cfg(not(unix))]
    {let _=(path,text,expected,before_commit,sync_directory);Err(atomic_failure("unsupported","Atomic text replacement is not implemented on this platform."))}
    #[cfg(unix)]
    {
        use std::{io::Write,os::unix::fs::{MetadataExt,OpenOptionsExt},path::Path};
        let io=|error|FsFailure::io("writeTextFileAtomic",error);
        let path=Path::new(path);
        if path.file_name().is_none(){return Err(atomic_failure("invalid-input","Destination must name a file."));}
        let parent=path.parent().filter(|path|!path.as_os_str().is_empty()).unwrap_or(Path::new("."));
        let directory=std::fs::File::open(parent).map_err(io)?;
        let mut existing=match std::fs::symlink_metadata(path) {
            Ok(metadata)=>{
                if !metadata.is_file(){return Err(atomic_failure("not-file","Destination must be a regular file, not a directory or symbolic link."));}
                if metadata.nlink()!=1{return Err(atomic_failure("multiple-links","Atomic replacement would separate a file's hard links."));}
                if metadata.permissions().readonly(){return Err(atomic_failure("permission-denied","Destination is read-only."));}
                let file=std::fs::OpenOptions::new().read(true).write(true).custom_flags(nix::libc::O_NOFOLLOW).open(path).map_err(io)?;
                if !same_revision(&metadata,&file.metadata().map_err(io)?){return Err(atomic_failure("conflict","Destination changed while opening."));}
                Some((file,metadata))
            },
            Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{
                if expected.is_some(){return Err(atomic_failure("conflict","Destination was removed."));}
                None
            },
            Err(error)=>return Err(io(error)),
        };
        if let (Some((file,_)),Some(expected))=(&mut existing,expected) {
            if !matches_text(file,expected).map_err(io)?{return Err(atomic_failure("conflict","Destination differs from the expected text."));}
        }
        let mut staged=tempfile::Builder::new().prefix(".tsrust-write-").tempfile_in(parent).map_err(io)?;
        staged.write_all(text.as_bytes()).map_err(io)?;
        if let Some((file,metadata))=&existing {
            #[cfg(target_os="macos")]
            {
                use std::os::{fd::AsRawFd,macos::fs::MetadataExt};
                let flags=metadata.st_flags();
                if flags&(nix::libc::UF_IMMUTABLE|nix::libc::SF_IMMUTABLE|nix::libc::UF_APPEND|nix::libc::SF_APPEND)!=0 {
                    return Err(atomic_failure("permission-denied","Destination has immutable or append-only flags."));
                }
                // Copy ACLs, ownership, flags and xattrs without touching staged data.
                let result=unsafe {nix::libc::fcopyfile(file.as_raw_fd(),staged.as_file().as_raw_fd(),std::ptr::null_mut(),nix::libc::COPYFILE_METADATA)};
                if result!=0{return Err(io(std::io::Error::last_os_error()));}
                let copied=staged.as_file().metadata().map_err(io)?;
                if copied.uid()!=metadata.uid()||copied.gid()!=metadata.gid()||copied.mode()!=metadata.mode()||copied.st_flags()!=flags {
                    return Err(atomic_failure("metadata","Destination ownership, permissions or flags could not be preserved."));
                }
            }
            // shortcut: other Unix hosts preserve mode only; extend metadata support before Studio delivery there.
            #[cfg(not(target_os="macos"))]
            {let _=file;staged.as_file().set_permissions(metadata.permissions()).map_err(io)?;}
            staged.as_file().set_modified(std::time::SystemTime::now()).map_err(io)?;
        }
        staged.as_file().sync_all().map_err(io)?;
        before_commit()?;
        if let Some((file,metadata))=&mut existing {
            if let Some(expected)=expected {
                if !matches_text(file,expected).map_err(io)?{return Err(atomic_failure("conflict","Destination changed before replacement."));}
            }
            let current=std::fs::symlink_metadata(path).map_err(io)?;
            if !same_revision(metadata,&current){return Err(atomic_failure("conflict","Destination changed before replacement."));}
            // Observed revisions are checked; an uncooperative writer can still race the final rename.
            staged.persist(path).map_err(|error|io(error.error))?;
        }else{
            staged.persist_noclobber(path).map_err(|error|io(error.error))?;
        }
        sync_directory(&directory).map_err(|error|FsFailure {code:"committed-not-durable",operation:"writeTextFileAtomic",message:format!("New contents are committed, but directory synchronization failed: {error}"),os_code:error.raw_os_error().map(i64::from)})?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn failures_and_concurrent_replacements_preserve_original_or_external_data() {
        let root=tempfile::tempdir().unwrap();let path=root.path().join("document");let name=path.to_str().unwrap();
        std::fs::write(&path,"original").unwrap();
        let error=write_text_file_atomic_with(name,"draft",Some("original"),||Err(atomic_failure("io","injected staging failure")),std::fs::File::sync_all).unwrap_err();
        assert_eq!(error.code,"io");assert_eq!(std::fs::read_to_string(&path).unwrap(),"original");
        let error=write_text_file_atomic_with(name,"draft",Some("original"),||{std::fs::write(&path,"external").unwrap();Ok(())},std::fs::File::sync_all).unwrap_err();
        assert_eq!(error.code,"conflict");assert_eq!(std::fs::read_to_string(&path).unwrap(),"external");
        let error=write_text_file_atomic_with(name,"draft",Some("external"),||{
            let replacement=root.path().join("replacement");std::fs::write(&replacement,"external").unwrap();std::fs::rename(replacement,&path).unwrap();Ok(())
        },std::fs::File::sync_all).unwrap_err();
        assert_eq!(error.code,"conflict");assert_eq!(std::fs::read_to_string(&path).unwrap(),"external");
        let error=write_text_file_atomic_with(name,"committed",Some("external"),||Ok(()),|_|Err(std::io::Error::other("injected directory sync failure"))).unwrap_err();
        assert_eq!(error.code,"committed-not-durable");assert_eq!(std::fs::read_to_string(&path).unwrap(),"committed");
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(),1);
    }
    #[cfg(unix)]
    #[test]
    fn unsafe_targets_fail_without_modification_or_staging_leaks() {
        use std::os::unix::fs::{PermissionsExt,symlink};
        let root=tempfile::tempdir().unwrap();let path=root.path().join("document");
        std::fs::write(&path,"original").unwrap();
        std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o444)).unwrap();
        assert_eq!(write_text_file_atomic(path.to_str().unwrap(),"new",Some("original")).unwrap_err().code,"permission-denied");
        std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o640)).unwrap();
        let link=root.path().join("alias");symlink(&path,&link).unwrap();
        assert_eq!(write_text_file_atomic(link.to_str().unwrap(),"new",None).unwrap_err().code,"not-file");
        std::fs::remove_file(&link).unwrap();std::fs::hard_link(&path,&link).unwrap();
        assert_eq!(write_text_file_atomic(path.to_str().unwrap(),"new",None).unwrap_err().code,"multiple-links");
        assert_eq!(std::fs::read_to_string(&path).unwrap(),"original");
        assert_eq!(std::fs::read_to_string(&link).unwrap(),"original");
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(),2);
    }
    #[cfg(target_os="macos")]
    #[test]
    fn macos_replacement_preserves_permissions_ownership_xattrs_and_acl() {
        use std::{os::{fd::AsRawFd,unix::fs::{PermissionsExt,MetadataExt}},process::Command};
        use std::os::macos::fs::MetadataExt as _;
        let root=tempfile::tempdir().unwrap();let path=root.path().join("document");
        std::fs::write(&path,"original").unwrap();
        std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o640)).unwrap();
        assert!(Command::new("/bin/chmod").args(["+a","everyone allow read"]).arg(&path).status().unwrap().success());
        let acl=|path:&std::path::Path|{
            let output=Command::new("/bin/ls").arg("-le").arg(path).output().unwrap();assert!(output.status.success());
            String::from_utf8(output.stdout).unwrap().lines().skip(1).collect::<Vec<_>>().join("\n")
        };
        let before_acl=acl(&path);assert!(!before_acl.is_empty());
        let mut original=std::fs::File::open(&path).unwrap();
        original.set_modified(std::time::UNIX_EPOCH+std::time::Duration::from_secs(1)).unwrap();
        assert_eq!(unsafe {nix::libc::fchflags(original.as_raw_fd(),nix::libc::UF_HIDDEN)},0);
        let metadata=original.metadata().unwrap();
        let name=c"dev.tsrust.atomic-test";let value=b"preserved \0 bytes";
        assert_eq!(unsafe {nix::libc::fsetxattr(original.as_raw_fd(),name.as_ptr(),value.as_ptr().cast(),value.len(),0,0)},0);
        write_text_file_atomic(path.to_str().unwrap(),"edited 🦀\r\n",Some("original")).unwrap();
        let replacement=std::fs::File::open(&path).unwrap();let after=replacement.metadata().unwrap();
        assert_ne!(metadata.ino(),after.ino());assert_eq!((metadata.uid(),metadata.gid(),metadata.mode()),(after.uid(),after.gid(),after.mode()));
        assert_eq!(metadata.st_flags(),after.st_flags());assert!(after.modified().unwrap()>metadata.modified().unwrap());
        let mut actual=[0u8;64];let length=unsafe {nix::libc::fgetxattr(replacement.as_raw_fd(),name.as_ptr(),actual.as_mut_ptr().cast(),actual.len(),0,0)};
        assert_eq!(length,value.len()as isize);assert_eq!(&actual[..length as usize],value);
        assert_eq!(acl(&path),before_acl);assert_eq!(std::fs::read_to_string(&path).unwrap(),"edited 🦀\r\n");
        let mut old_text=String::new();std::io::Read::read_to_string(&mut original,&mut old_text).unwrap();assert_eq!(old_text,"original");
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(),1);
    }
}
