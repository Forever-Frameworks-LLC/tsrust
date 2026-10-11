//! Materialize compiler-owned source dependencies into a private build directory.
use std::{fs, io, path::Path};
include!(concat!(env!("OUT_DIR"), "/native_dependency_files.rs"));

pub(crate) fn materialize(root: &Path) -> io::Result<()> {
    // The caller supplies a fresh private directory. Never follow an existing
    // directory's symlinks or overwrite files belonging to another build.
    fs::create_dir(root)?;
    for (relative, bytes) in FILES {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, bytes)?;
    }
    fs::create_dir(root.join(".cargo"))?;
    fs::write(root.join(".cargo/config.toml"), "[source.crates-io]\nreplace-with = \"bundled\"\n[source.bundled]\ndirectory = \"vendor\"\n[net]\noffline = true\n")
}
