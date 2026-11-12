//! Recursive directory snapshots using the compiler's traversal policy.
#[path = "discovery_walk.rs"]
mod walker;
use super::fs::{DirEntry, FsFailure};

pub fn walk_dir(path: &str, include_ignored: bool, max_entries: Option<i64>) -> Result<Vec<DirEntry>, FsFailure> {
    let failure = |code, message| FsFailure {code, operation: "walkDir", message, os_code: None};
    let limit = max_entries.map(|limit| usize::try_from(limit)
        .map_err(|_| failure("length-range", "Entry limit must be nonnegative and fit native storage.".into()))).transpose()?;
    let metadata = std::fs::symlink_metadata(path).map_err(|error| FsFailure::io("walkDir", error))?;
    if !metadata.is_dir() { return Err(failure("not-directory", "Choose a directory; symbolic-link roots are not followed.".into())); }
    let root = std::fs::canonicalize(path).map_err(|error| FsFailure::io("walkDir", error))?;
    let mut entries = Vec::new();
    let mut conversion_error = None;
    let result = walker::walk(&root, include_ignored, &mut || false, |entry| {
        let convert = || -> Result<DirEntry, FsFailure> {
            if limit.is_some_and(|limit| entries.len() >= limit) {
                return Err(failure("length-range", "Directory tree exceeds the requested entry limit.".into()));
            }
            let name = entry.file_name().to_os_string().into_string().map_err(|_| failure("not-unicode", "Filename is not valid Unicode.".into()))?;
            let path = entry.path().to_path_buf().into_os_string().into_string().map_err(|_| failure("not-unicode", "Path is not valid Unicode.".into()))?;
            let kind = entry.file_type().expect("walker yields regular files/directories");
            Ok(DirEntry {name, path, is_file: kind.is_file(), is_directory: kind.is_dir(), is_symlink: false})
        };
        match convert() {
            Ok(entry) => { entries.push(entry); Ok(()) },
            Err(error) => { conversion_error = Some(error); Err("entry conversion failed".into()) },
        }
    });
    if let Some(error) = conversion_error { return Err(error); }
    result.map_err(|message| failure("discovery", message))?;
    entries.sort_by(|a,b| a.path.cmp(&b.path));
    Ok(entries)
}
