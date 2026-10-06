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
            directory: root.join(format!("{:016x}", hash.finish())),
            identity,
        }
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
    /// Publish a complete sibling staging directory atomically; serialize repair per cache identity.
    pub fn publish(&self, executable: &Path) -> std::io::Result<()> {
        let root = self.directory.parent().expect("cache directory parent");
        std::fs::create_dir_all(root)?;
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
        // Keep the lock file: removing it can let another process lock a different inode.
        // The OS releases this advisory lock when the file closes, including on process exit.
        let publication_lock = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.directory.with_extension("lock"))?;
        publication_lock.lock()?;
        if self.lookup().is_some() {
            return Ok(());
        }
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
        }
    }
}
