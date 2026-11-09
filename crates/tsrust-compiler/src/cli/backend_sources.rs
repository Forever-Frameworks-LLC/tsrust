//! Small locked backend inputs. Cargo owns the shared registry and Git sources.
use std::{fs, path::{Path, PathBuf}};
use sha2::{Digest, Sha256};

pub(crate) const MANIFEST: &str = include_str!("../../backend-dependencies/Cargo.toml");
pub(crate) const LOCK: &str = include_str!("../../backend-dependencies/Cargo.lock");
const BASE_IDENTITY: &str = concat!("shared-backend-sources-2\n", include_str!("../../backend-dependencies/Cargo.toml"), include_str!("../../backend-dependencies/Cargo.lock"));

pub(crate) fn cargo_home() -> Result<PathBuf, String> {
    let home = std::env::var_os("CARGO_HOME").map(PathBuf::from).or_else(||
        std::env::var_os(if cfg!(windows) {"USERPROFILE"} else {"HOME"}).map(|home|PathBuf::from(home).join(".cargo")))
        .ok_or("Cannot locate Cargo home; set CARGO_HOME to the shared Cargo cache directory")?;
    super::absolute(&home)
}

pub(crate) fn build_root() -> Result<PathBuf, String> {
    let root=match std::env::var_os("TSRUST_CACHE_DIR") {
        Some(root)=>PathBuf::from(root),
        None=>super::project_config::default_root()?.join("target/tsrust-native"),
    };
    super::absolute(&root)
}

pub(crate) fn materialize(root: &Path) -> Result<(), String> { materialize_with(root, MANIFEST, LOCK) }

pub(crate) fn materialize_with(root: &Path, manifest: &str, lock: &str) -> Result<(), String> {
    // Exclusive root creation: cleanup owns only this newly created directory.
    fs::create_dir(root).map_err(|e|e.to_string())?;
    let result = (|| -> Result<(), String> {
        fs::create_dir(root.join("src")).map_err(|e|e.to_string())?;
        fs::create_dir(root.join(".cargo")).map_err(|e|e.to_string())?;
        fs::write(root.join("Cargo.toml"), tsrust_compiler::native_url_patch::normalize_manifest(manifest).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        tsrust_compiler::native_url_patch::install(root,manifest).map_err(|e|e.to_string())?;
        fs::write(root.join("Cargo.lock"), lock).map_err(|e|e.to_string())?;
        fs::write(root.join("src/lib.rs"), include_str!("../../backend-dependencies/src/lib.rs")).map_err(|e|e.to_string())?;
        fs::write(root.join(".cargo/config.toml"), "[net]\noffline = true\n").map_err(|e|e.to_string())?;
        Ok(())
    })();
    if result.is_err() {let _=fs::remove_dir_all(root);}
    result
}

/// Caller holds the compiler-owned native workspace lock. Cache restoration may
/// preserve the directory while omitting generated source/configuration files.
pub(crate) fn restore_missing(root:&Path,manifest:&str,lock:&str)->Result<(),String>{
    let directories=[root.to_owned(),root.join("src"),root.join(".cargo")];
    let normalized=tsrust_compiler::native_url_patch::normalize_manifest(manifest).map_err(|e|e.to_string())?;
    let files=[("Cargo.toml",normalized.as_bytes(),false),("Cargo.lock",lock.as_bytes(),true),
        ("src/lib.rs",include_bytes!("../../backend-dependencies/src/lib.rs").as_slice(),false),
        (".cargo/config.toml",b"[net]\noffline = true\n".as_slice(),true)];
    // Refuse changed metadata and links before restoring anything. The manifest
    // and runtime source are subsequently written by the existing runtime builder.
    for path in &directories{match fs::symlink_metadata(path){
        Ok(meta) if !meta.file_type().is_symlink() && meta.is_dir()=>{},
        Ok(_)=>return Err(format!("Shared backend support requires a regular directory: {}",path.display())),
        Err(error) if error.kind()==std::io::ErrorKind::NotFound && path!=root=>{},
        Err(error)=>return Err(format!("Cannot inspect shared backend support {}: {error}",path.display())),
    }}
    let mut missing=Vec::new();
    for (name,bytes,exact) in files{let path=root.join(name);match fs::symlink_metadata(&path){
        Ok(meta) if !meta.file_type().is_symlink() && meta.is_file()=>{
            if exact && fs::read(&path).map_err(|e|format!("Cannot read shared backend support {}: {e}",path.display()))?!=bytes{
                return Err(format!("Shared backend support metadata was modified; preserved it: {}",path.display()));
            }
        },
        Ok(_)=>return Err(format!("Shared backend support requires a regular file: {}",path.display())),
        Err(error) if error.kind()==std::io::ErrorKind::NotFound=>missing.push((path,bytes)),
        Err(error)=>return Err(format!("Cannot inspect shared backend support {}: {error}",path.display())),
    }}
    for path in &directories[1..]{match fs::create_dir(path){Ok(())=>{},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>{},Err(e)=>return Err(format!("Cannot restore shared backend directory {}: {e}",path.display()))}}
    for (path,bytes) in missing{use std::io::Write;fs::OpenOptions::new().write(true).create_new(true).open(&path)
        .and_then(|mut file|file.write_all(bytes)).map_err(|e|format!("Cannot restore shared backend support {}: {e}",path.display()))?;}
    tsrust_compiler::native_url_patch::restore_missing(root,manifest).map_err(|e|format!("Cannot restore shared backend URL patch in {}: {e}",root.display()))
}

pub(crate) fn validate(root: &Path, manifest:&str, lock: &str) -> Result<(), String> {
    for path in [root.to_owned(),root.join("src"),root.join(".cargo"),root.join("Cargo.toml"),root.join("Cargo.lock"),root.join("src/lib.rs"),root.join(".cargo/config.toml")] {
        if fs::symlink_metadata(&path).map_err(|e|format!("Cannot inspect shared backend support {}: {e}",path.display()))?.file_type().is_symlink() {
            return Err(format!("Shared backend support cannot traverse a symlink: {}", path.display()));
        }
    }
    if fs::read(root.join("Cargo.lock")).map_err(|e|e.to_string())? != lock.as_bytes()
        || fs::read_to_string(root.join(".cargo/config.toml")).map_err(|e|e.to_string())? != "[net]\noffline = true\n" {
        return Err("Shared backend support metadata was modified; preserved it. Remove the affected private runtime workspace before retrying.".into());
    }
    tsrust_compiler::native_url_patch::visit(root,manifest,|_,_|Ok(())).map_err(|e|e.to_string())?;
    Ok(())
}

fn part(hash: &mut Sha256, bytes: &[u8]) {hash.update((bytes.len() as u64).to_le_bytes());hash.update(bytes);}

/// Only the digest is persisted. Config contents and environment credentials are
/// never written into cache identity files. Include all environment inputs because
/// upstream build scripts can declare arbitrary environment dependencies.
pub(crate) fn identity(root: &Path) -> Result<String, String> {
    let mut hash = Sha256::new();part(&mut hash, dependency_identity().as_bytes());
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
    let mut pending=configs.into_iter().map(|path|(path,false)).collect::<Vec<_>>();
    let mut visited=std::collections::BTreeSet::new();
    while let Some((path,required))=pending.pop() {
        part(&mut hash,path.as_os_str().as_encoded_bytes());
        let bytes=match fs::read(&path) {
            Ok(bytes)=>bytes,
            Err(error) if error.kind()==std::io::ErrorKind::NotFound && !required=>{part(&mut hash,b"absent");continue;},
            Err(error)=>return Err(format!("Cannot inspect Cargo configuration {}: {error}",path.display())),
        };
        part(&mut hash,b"present");part(&mut hash,&bytes);
        let canonical=fs::canonicalize(&path).map_err(|e|e.to_string())?;
        part(&mut hash,canonical.as_os_str().as_encoded_bytes());
        let parent=path.parent().ok_or("Cargo configuration requires a parent directory")?;
        let canonical_parent=fs::canonicalize(parent).map_err(|e|e.to_string())?;
        // Preserve alias-relative include bases while bounding repeated/cyclic files.
        if !visited.insert((canonical,canonical_parent)){continue;}
        let text=std::str::from_utf8(&bytes).map_err(|_|format!("Cargo configuration is not UTF-8: {}",path.display()))?;
        let document=text.parse::<toml_edit::DocumentMut>().map_err(|_|format!("Cargo configuration is not valid TOML: {}",path.display()))?;
        if let Some(include)=document.get("include") {
            if let Some(tables)=include.as_array_of_tables() {
                for table in tables {
                    let name=table.get("path").and_then(|v|v.as_str()).ok_or("Cargo include table requires a path string")?;
                    let optional=match table.get("optional"){None=>false,Some(value)=>value.as_bool().ok_or("Cargo include optional must be a boolean")?};
                    pending.push((parent.join(name),!optional));
                }
                continue;
            }
            let include=include.as_array().ok_or_else(||format!("Cargo include must be an array: {}",path.display()))?;
            for entry in include {
                let (name,optional)=if let Some(name)=entry.as_str(){(name,false)}else{
                    let table=entry.as_inline_table().ok_or("Cargo include entries require a path string or inline table")?;
                    let name=table.get("path").and_then(|v|v.as_str()).ok_or("Cargo include table requires a path string")?;
                    let optional=match table.get("optional"){None=>false,Some(value)=>value.as_bool().ok_or("Cargo include optional must be a boolean")?};
                    (name,optional)
                };
                pending.push((parent.join(name),!optional));
            }
        }
    }
    Ok(hash.finalize().iter().map(|byte|format!("{byte:02x}")).collect())
}

pub(crate) fn unchanged(expected: Option<&str>, root: &Path) -> bool {
    expected.is_none_or(|expected|identity(root).is_ok_and(|actual|actual==expected))
}

pub(crate) fn application_identity(source: &str, root: &Path) -> Result<Option<String>, String> {
    if tsrust_compiler::native_runtime::requires_dependencies(source) {Ok(Some(identity(root)?))} else {Ok(None)}
}

pub(crate) fn dependency_identity()-> &'static str {
 static VALUE:std::sync::OnceLock<String>=std::sync::OnceLock::new();
 VALUE.get_or_init(||tsrust_compiler::native_url_patch::cache_identity(BASE_IDENTITY)).as_str()
}

#[cfg(test)] mod restoration_tests {
    use super::*;
    #[test] fn incomplete_owned_support_restores_missing_files_but_preserves_changed_metadata(){
        let directory=crate::TempDirectory::new().unwrap();
        let workspace=crate::native_workspace::Workspace::acquire(&directory.0,b"restore-missing-test",false).unwrap().unwrap();
        let package=workspace.root.join("native-support");materialize_with(&package,MANIFEST,LOCK).unwrap();
        let manifest=fs::read(package.join("Cargo.toml")).unwrap();
        fs::remove_dir_all(package.join("src")).unwrap();fs::remove_dir_all(package.join(".cargo")).unwrap();fs::remove_file(package.join("Cargo.lock")).unwrap();
        if package.join("patched-dependencies").exists(){fs::remove_dir_all(package.join("patched-dependencies")).unwrap();}
        restore_missing(&package,MANIFEST,LOCK).unwrap();validate(&package,MANIFEST,LOCK).unwrap();
        assert_eq!(fs::read(package.join("Cargo.toml")).unwrap(),manifest);
        fs::write(package.join("Cargo.lock"),b"changed").unwrap();fs::remove_file(package.join("src/lib.rs")).unwrap();
        assert!(restore_missing(&package,MANIFEST,LOCK).unwrap_err().contains("modified"));
        assert_eq!(fs::read(package.join("Cargo.lock")).unwrap(),b"changed");assert!(!package.join("src/lib.rs").exists());
        fs::write(package.join("Cargo.lock"),LOCK).unwrap();fs::write(package.join(".cargo/config.toml"),b"changed").unwrap();
        assert!(restore_missing(&package,MANIFEST,LOCK).is_err());assert!(!package.join("src/lib.rs").exists());
    }
    #[cfg(unix)]
    #[test] fn incomplete_support_does_not_follow_source_directory_symlinks(){
        let directory=crate::TempDirectory::new().unwrap();let package=directory.0.join("native-support");materialize_with(&package,MANIFEST,LOCK).unwrap();
        fs::remove_dir_all(package.join("src")).unwrap();std::os::unix::fs::symlink(&directory.0,package.join("src")).unwrap();
        assert!(restore_missing(&package,MANIFEST,LOCK).is_err());assert!(!directory.0.join("lib.rs").exists());
    }
}
