//! Small locked backend inputs. Cargo owns the shared registry and Git sources.
use std::{fs, path::{Path, PathBuf}};
use sha2::{Digest, Sha256};

pub(crate) const MANIFEST: &str = include_str!("../../backend-dependencies/Cargo.toml");
pub(crate) const LOCK: &str = include_str!("../../backend-dependencies/Cargo.lock");
pub(crate) const IDENTITY: &str = concat!("shared-backend-sources-1\n", include_str!("../../backend-dependencies/Cargo.toml"), include_str!("../../backend-dependencies/Cargo.lock"));

pub(crate) fn shared(source: &str) -> Result<bool, String> {
    Ok(!source.contains("pub mod tsrust_native_ui {") && super::rust_binding_packages::packages(source)?.is_empty())
}

pub(crate) fn cargo_home() -> Result<PathBuf, String> {
    let home = std::env::var_os("CARGO_HOME").map(PathBuf::from).or_else(||
        std::env::var_os(if cfg!(windows) {"USERPROFILE"} else {"HOME"}).map(|home|PathBuf::from(home).join(".cargo")))
        .ok_or("Cannot locate Cargo home; set CARGO_HOME to the shared Cargo cache directory")?;
    super::absolute(&home)
}

pub(crate) fn materialize(root: &Path) -> Result<(), String> {
    // Exclusive root creation: cleanup owns only this newly created directory.
    fs::create_dir(root).map_err(|e|e.to_string())?;
    let result = (|| -> Result<(), String> {
        fs::create_dir(root.join("src")).map_err(|e|e.to_string())?;
        fs::create_dir(root.join(".cargo")).map_err(|e|e.to_string())?;
        fs::write(root.join("Cargo.toml"), MANIFEST).map_err(|e|e.to_string())?;
        fs::write(root.join("Cargo.lock"), LOCK).map_err(|e|e.to_string())?;
        fs::write(root.join("src/lib.rs"), include_str!("../../backend-dependencies/src/lib.rs")).map_err(|e|e.to_string())?;
        fs::write(root.join(".cargo/config.toml"), "[net]\noffline = true\n").map_err(|e|e.to_string())?;
        Ok(())
    })();
    if result.is_err() {let _=fs::remove_dir_all(root);}
    result
}

pub(crate) fn validate(root: &Path) -> Result<(), String> {
    for path in [root.to_owned(),root.join("src"),root.join(".cargo"),root.join("Cargo.toml"),root.join("Cargo.lock"),root.join("src/lib.rs"),root.join(".cargo/config.toml")] {
        if fs::symlink_metadata(&path).map_err(|e|e.to_string())?.file_type().is_symlink() {
            return Err(format!("Shared backend support cannot traverse a symlink: {}", path.display()));
        }
    }
    if fs::read(root.join("Cargo.lock")).map_err(|e|e.to_string())? != LOCK.as_bytes()
        || fs::read_to_string(root.join(".cargo/config.toml")).map_err(|e|e.to_string())? != "[net]\noffline = true\n" {
        return Err("Shared backend support metadata was modified; preserved it. Remove the affected private runtime workspace before retrying.".into());
    }
    Ok(())
}

fn part(hash: &mut Sha256, bytes: &[u8]) {hash.update((bytes.len() as u64).to_le_bytes());hash.update(bytes);}

/// Only the digest is persisted. Config contents and environment credentials are
/// never written into cache identity files. Include all environment inputs because
/// upstream build scripts can declare arbitrary environment dependencies.
pub(crate) fn identity(root: &Path) -> Result<String, String> {
    let mut hash = Sha256::new();part(&mut hash, IDENTITY.as_bytes());
    let environment = std::env::vars_os().collect::<std::collections::BTreeMap<_,_>>();
    for (name,value) in environment {part(&mut hash,name.as_encoded_bytes());part(&mut hash,value.as_encoded_bytes());}
    let root = super::absolute(root)?;
    let home = cargo_home()?;
    part(&mut hash,home.as_os_str().as_encoded_bytes());
    let mut configs = std::collections::BTreeSet::new();
    for directory in root.ancestors() {
        for name in ["config", "config.toml"] {configs.insert(directory.join(".cargo").join(name));}
    }
    for name in ["config", "config.toml"] {configs.insert(home.join(name));}
    for path in configs {
        part(&mut hash,path.as_os_str().as_encoded_bytes());
        match fs::read(&path) {
            Ok(bytes)=>{part(&mut hash,b"present");part(&mut hash,&bytes);},
            Err(error) if error.kind()==std::io::ErrorKind::NotFound=>part(&mut hash,b"absent"),
            Err(error)=>return Err(format!("Cannot inspect Cargo configuration {}: {error}",path.display())),
        }
    }
    Ok(format!("{:x}",hash.finalize()))
}

pub(crate) fn application_identity(source: &str, root: &Path) -> Result<Option<String>, String> {
    if tsrust_compiler::native_runtime::separate(source).dependencies && shared(source)? {Ok(Some(identity(root)?))} else {Ok(None)}
}
