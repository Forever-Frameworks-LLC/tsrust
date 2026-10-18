//! Local native artifact cache. Exact identity bytes validate hashed directory names.
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static PUBLICATION_SEQUENCE: AtomicU64 = AtomicU64::new(0);
fn checksum(bytes: &[u8]) -> [u8; 8] {
    let mut hash = DefaultHasher::new();
    bytes.hash(&mut hash);
    hash.finish().to_le_bytes()
}
pub struct NativeCache {
    directory: PathBuf,
    identity: Vec<u8>,
    budget: u64,
}
impl NativeCache {
    pub fn new(root: &Path, fields: &[&[u8]]) -> Self {
        let mut identity = Vec::new();
        for field in fields {
            identity.extend_from_slice(&(field.len() as u64).to_le_bytes());
            identity.extend_from_slice(field);
        }
        let mut hash = DefaultHasher::new();
        identity.hash(&mut hash);
        Self {
            directory: root.join("v2").join(format!("{:016x}", hash.finish())),
            identity,
            budget: 256 * 1024 * 1024,
        }
    }
    /// GPU runtime dependency metadata exceeds the application cache's budget.
    pub fn runtime(root: &Path, fields: &[&[u8]]) -> Self {
        let mut cache = Self::new(root, fields);
        cache.budget = 512 * 1024 * 1024;
        cache
    }
    pub fn lookup(&self) -> Option<PathBuf> {
        let executable = self
            .directory
            .join(format!("program{}", std::env::consts::EXE_SUFFIX));
        (std::fs::read(self.directory.join("identity")).ok()? == self.identity
            && std::fs::read(self.directory.join("checksum")).ok()?
                == checksum(&std::fs::read(&executable).ok()?))
        .then_some(executable)
    }
    /// Copy under the namespace lock so eviction cannot race a reader.
    pub fn copy_to(&self,destination:&Path)->bool {
        self.copy_with_lock(destination,true)
    }
    /// Dev supervision must not wait for another process's cache lock.
    pub fn try_copy_to(&self,destination:&Path)->bool {
        self.copy_with_lock(destination,false)
    }
    fn copy_with_lock(&self,destination:&Path,wait:bool)->bool {
        let Ok(_lock)=self.lock(wait) else{return false;};
        let Some(source)=self.lookup() else{return false;};
        if std::fs::copy(source,destination).is_err(){return false;}
        let _=std::fs::write(self.directory.join("used"),[]);true
    }
    fn lock(&self,wait:bool)->std::io::Result<std::fs::File> {
        let root=self.directory.parent().unwrap();std::fs::create_dir_all(root)?;
        let lock=std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(root.join(".cache.lock"))?;
        if wait {lock.lock()?;} else {lock.try_lock().map_err(std::io::Error::from)?;}Ok(lock)
    }
    fn prune(&self)->std::io::Result<()> {
        let root=self.directory.parent().unwrap();let mut entries=Vec::new();let mut total=0u64;
        for entry in std::fs::read_dir(root)? {
            let entry=entry?;if !entry.file_type()?.is_dir(){continue;}
            let name=entry.file_name();let name=name.to_string_lossy();
            // All staging writers hold the namespace lock. Anything left here
            // is from a terminated writer, never another active publication.
            if name.starts_with(".staging-"){std::fs::remove_dir_all(entry.path())?;continue;}
            if name.len()!=16 || !name.bytes().all(|c|c.is_ascii_hexdigit()){continue;}
            let path=entry.path();let mut size=0u64;
            for file in std::fs::read_dir(&path)? {let file=file?;if file.file_type()?.is_file(){size=size.saturating_add(file.metadata()?.len());}}
            let used=std::fs::metadata(path.join("used")).or_else(|_|std::fs::metadata(&path))?.modified()?;
            total=total.saturating_add(size);entries.push((path==self.directory,used,path,size));
        }
        entries.sort();let mut count=entries.len();
        for (_,_,path,size) in entries {
            if count<=32 && total<=self.budget {break;}
            std::fs::remove_dir_all(path)?;count-=1;total=total.saturating_sub(size);
        }
        Ok(())
    }
    /// Publish a complete sibling staging directory atomically; serialize repair per cache identity.
    pub fn publish(&self, executable: &Path) -> std::io::Result<()> {
        self.publish_with_lock(executable,true)
    }
    /// Return immediately when another process is copying or publishing.
    pub fn try_publish(&self, executable:&Path)->std::io::Result<()> {
        self.publish_with_lock(executable,false)
    }
    fn publish_with_lock(&self, executable:&Path,wait:bool)->std::io::Result<()> {
        let root = self.directory.parent().expect("cache directory parent");
        // New protocol has a separate namespace: older binaries may still hold
        // legacy per-artifact lock inodes, which must never be removed under them.
        let _lock=self.lock(wait)?;
        self.prune()?;
        if std::fs::metadata(executable)?.len().saturating_add(self.identity.len() as u64+8)>self.budget{return Ok(());}
        if self.lookup().is_some(){let _=std::fs::write(self.directory.join("used"),[]);return Ok(());}
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let sequence = PUBLICATION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let staging = root.join(format!(
            ".staging-{}-{stamp}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&staging)?;
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(staging.clone());
        let staged_executable = staging.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        std::fs::copy(executable, &staged_executable)?;
        std::fs::write(staging.join("identity"), &self.identity)?;
        std::fs::write(
            staging.join("checksum"),
            checksum(&std::fs::read(&staged_executable)?),
        )?;
        std::fs::write(staging.join("used"),[])?;
        match std::fs::rename(&staging, &self.directory) {
            Ok(()) => Ok(()),
            Err(_) if self.lookup().is_some() => Ok(()),
            Err(_) if self.directory.exists() => {
                let obsolete = staging.with_extension("obsolete");
                std::fs::rename(&self.directory, &obsolete)?;
                let _obsolete_cleanup = Cleanup(obsolete);
                std::fs::rename(&staging, &self.directory)
            }
            Err(error) => Err(error),
        }?;
        self.prune()
    }
}
