use std::{env,fs,path::PathBuf,time::{SystemTime,UNIX_EPOCH}};

pub(super) struct TempDirectory(pub(super) PathBuf);
impl TempDirectory {
    pub(super) fn new() -> std::io::Result<Self> {
        Self::new_in(&env::temp_dir())
    }
    /// A private directory under `parent`, created if missing.
    pub(super) fn new_in(parent: &std::path::Path) -> std::io::Result<Self> {
        fs::create_dir_all(parent)?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for attempt in 0..100 {
            let path =
                parent.join(format!("tsrust-{}-{stamp}-{attempt}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Could not create a private build directory",
        ))
    }
}
impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

