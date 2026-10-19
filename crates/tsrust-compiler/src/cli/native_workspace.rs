//! Stable, locked dependency build paths survive runtime source changes.
use std::{fs,hash::{Hash,Hasher},path::{Path,PathBuf}};
pub(crate) struct Workspace {
    pub root:PathBuf,
    _lock:fs::File,
}
impl Workspace {
    pub fn acquire(root:&Path,identity:&[u8],opportunistic:bool)->Result<Option<Self>,String> {
        let mut hash=std::collections::hash_map::DefaultHasher::new();identity.hash(&mut hash);
        let namespace=root.join("runtime/build-v1");fs::create_dir_all(&namespace).map_err(|e|e.to_string())?;
        if fs::symlink_metadata(&namespace).map_err(|e|e.to_string())?.file_type().is_symlink(){return Err("Native build workspace namespace must not be a symlink".into());}
        let root=namespace.join(format!("{:016x}",hash.finish()));fs::create_dir_all(&root).map_err(|e|e.to_string())?;
        if fs::symlink_metadata(&root).map_err(|e|e.to_string())?.file_type().is_symlink(){return Err("Native build workspace must not be a symlink".into());}
        let lock=fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(root.join(".lock")).map_err(|e|e.to_string())?;
        if opportunistic {match lock.try_lock(){Ok(())=>{},Err(fs::TryLockError::WouldBlock)=>return Ok(None),Err(error)=>return Err(error.to_string())}}
        else {lock.lock().map_err(|e|e.to_string())?;}
        let path=root.join("identity");
        match fs::read(&path) {
            Ok(found) if found!=identity=>return Err("Native build workspace identity mismatch".into()),
            Ok(_)=>{},Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{fs::write(path,identity).map_err(|e|e.to_string())?;},
            Err(error)=>return Err(error.to_string()),
        }
        Ok(Some(Self{root,_lock:lock}))
    }
}
/// Keep Cargo fingerprints stable when inputs did not change.
pub(crate) fn write_changed(path:&Path,bytes:impl AsRef<[u8]>)->Result<(),String> {
    let bytes=bytes.as_ref();if fs::read(path).ok().as_deref()==Some(bytes){return Ok(());}
    fs::write(path,bytes).map_err(|e|e.to_string())
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn workspace_reuses_paths_serializes_writers_and_checks_exact_identity() {
        struct Cleanup(PathBuf);impl Drop for Cleanup{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
        let dir=Cleanup(std::env::temp_dir().join(format!("tsrust-workspace-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())));
        let workspace=Workspace::acquire(&dir.0,b"backend/profile/bundle",true).unwrap().unwrap();let root=workspace.root.clone();
        write_changed(&root.join("source"),"first").unwrap();let stamp=fs::metadata(root.join("source")).unwrap().modified().unwrap();
        write_changed(&root.join("source"),"first").unwrap();assert_eq!(fs::metadata(root.join("source")).unwrap().modified().unwrap(),stamp);
        assert!(Workspace::acquire(&dir.0,b"backend/profile/bundle",true).unwrap().is_none());drop(workspace);
        let workspace=Workspace::acquire(&dir.0,b"backend/profile/bundle",true).unwrap().unwrap();assert_eq!(workspace.root,root);assert_eq!(fs::read(root.join("source")).unwrap(),b"first");
        let other=Workspace::acquire(&dir.0,b"different backend/profile/bundle",true).unwrap().unwrap();assert_ne!(other.root,root);drop(workspace);
        fs::write(root.join("identity"),"corrupt").unwrap();assert!(Workspace::acquire(&dir.0,b"backend/profile/bundle",true).is_err());
    }
}
