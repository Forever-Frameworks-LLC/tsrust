//! Materialize compiler-owned source dependencies into a private build directory.
use std::{fs, io, path::Path};
#[path="portable_sources.rs"]
mod sources;
pub(crate) fn prepare(cargo: std::process::Command, offline: bool) -> io::Result<std::path::PathBuf> {sources::prepare(cargo,offline)}
pub(crate) fn materialize(root: &Path) -> io::Result<()> {sources::materialize(root)}

// Lockfile package checksums bind the vendored source closure. The manifest binds
// selected features; increment the protocol when build settings change.
pub(crate) const IDENTITY:&str=concat!("native-dependencies-1\n",include_str!("../../native-dependencies/Cargo.toml"),include_str!("../../native-dependencies/Cargo.lock"));

/// windows-targets records a native DLL import-library name in its rlib, but
/// Cargo's native search path is not retained by the application rustc command.
/// Recreate the host-matching imports from the same pinned source closure on
/// both fresh builds and cache restores; existing TSRLIB2 artifacts stay valid.
pub(crate) fn restore_import_libraries(root:&Path)->io::Result<usize> {
    #[cfg(not(all(windows,target_env="msvc")))] {let _=root;Ok(0)}
    #[cfg(all(windows,target_env="msvc"))] {
        use std::io::Write;
        if fs::symlink_metadata(root)?.file_type().is_symlink(){return Err(io::Error::other("Native dependency directory cannot be a symlink"));}
        let mut needed=false;
        for entry in fs::read_dir(root)? {let entry=entry?;if entry.file_type()?.is_file()&&entry.file_name().to_str().is_some_and(|name|name.starts_with("libwindows_targets-")&&name.ends_with(".rlib")){needed=true;break;}}
        if !needed{return Ok(0);}
        // These hashes were verified against the pinned 0.52.6 Cargo vendor
        // checksums. Keep only metadata in the compiler, never dependency payloads.
        let (architecture,expected)=match std::env::consts::ARCH {
            "x86"=>("i686","820506ebd44db30b5f429891fe629f31be412e583001b13df7ed8847886ad178"),
            "x86_64"=>("x86_64","24d8cbc445955b0d48041948a3c71ce2cddb948c089b25ec7106fadd9f3efde0"),
            "aarch64"=>("aarch64","ca85dd7757f338ba00d0bc19229f176ee426a707425ee4fcf205ec491a95bd5a"),
            _=>return Err(io::Error::other("Unsupported Windows import-library architecture")),
        };
        use sha2::{Digest,Sha256};
        let verify=|bytes:&[u8]|Sha256::digest(bytes).iter().map(|b|format!("{b:02x}")).collect::<String>()==expected;
        let name="windows.0.52.0.lib";let path=root.join(name);
        let existing=match fs::symlink_metadata(&path){Ok(metadata)=>Some(metadata),Err(error) if error.kind()==io::ErrorKind::NotFound=>None,Err(error)=>return Err(error)};
        if let Some(metadata)=existing{
            if metadata.file_type().is_symlink() || !metadata.is_file() || !verify(&fs::read(&path)?){return Err(io::Error::other(format!("Native import library differs from pinned bytes; preserved {}",path.display())));}
            return Ok(1);
        }
        let package=format!("windows_{architecture}_msvc-0.52.6");
        let mut candidates=Vec::new();
        let home=sources::cargo_home()?;
        let registry=home.join("registry/src");
        if registry.is_dir(){for entry in fs::read_dir(registry)?{let entry=entry?;if entry.file_type()?.is_dir(){candidates.push(entry.path().join(&package).join("lib").join(name));}}}
        candidates.push(sources::location()?.join("vendor").join(&package).join("lib").join(name));
        let bytes=candidates.iter().find_map(|candidate|fs::read(candidate).ok().filter(|bytes|verify(bytes)))
            .ok_or_else(||io::Error::other("Pinned Windows imports are absent; run `tsrust setup --dependencies --yes` or prepare a portable source pack"))?;
        match fs::OpenOptions::new().write(true).create_new(true).open(&path){
            Ok(mut output)=>output.write_all(&bytes)?,
            Err(error)=>return Err(error),
        }
        let count=1;
        Ok(count)
    }
}

pub(crate) fn pack(library:&Path,dependencies:&Path,destination:&Path)->io::Result<()> {
    use std::io::Write;
    restore_import_libraries(dependencies)?;
    let mut files=Vec::new();
    for entry in fs::read_dir(dependencies)? {
        let entry=entry?;if entry.file_type()?.is_file() && entry.path().extension().is_some_and(|e|e=="rlib" || e=="lib" || e==std::env::consts::DLL_EXTENSION) {files.push(entry.path());}
    }
    files.push(library.to_owned());files.sort();
    let mut out=fs::File::create(destination)?;out.write_all(b"TSRLIB3\0")?;
    for path in files {
        let name=path.file_name().unwrap().to_str().ok_or_else(||io::Error::other("non-UTF8 native artifact"))?;
        let bytes=fs::read(&path)?;
        out.write_all(&(name.len() as u64).to_le_bytes())?;out.write_all(&(bytes.len() as u64).to_le_bytes())?;
        out.write_all(name.as_bytes())?;out.write_all(&bytes)?;
    }
    Ok(())
}
pub(crate) fn unpack(source:&Path,destination:&Path)->io::Result<()> {
    let bytes=fs::read(source)?;
    let invalid=||io::Error::other("invalid native dependency artifact bundle");
    if bytes.len()>1024*1024*1024 || !(bytes.starts_with(b"TSRLIB2\0") || bytes.starts_with(b"TSRLIB3\0")) {return Err(invalid());}
    let mut remaining=&bytes[8..];let mut files=std::collections::BTreeMap::new();
    while !remaining.is_empty() {
        if remaining.len()<16 || files.len()>=512 {return Err(invalid());}
        let name_len=usize::try_from(u64::from_le_bytes(remaining[..8].try_into().unwrap())).map_err(|_|invalid())?;
        let length=usize::try_from(u64::from_le_bytes(remaining[8..16].try_into().unwrap())).map_err(|_|invalid())?;
        remaining=&remaining[16..];
        let total=name_len.checked_add(length).ok_or_else(invalid)?;
        if total>remaining.len(){return Err(invalid());}
        let name=std::str::from_utf8(&remaining[..name_len]).map_err(|_|invalid())?;
        if !(name.ends_with(".rlib") || (bytes.starts_with(b"TSRLIB3\0") && name.ends_with(".lib")) || name.ends_with(&format!(".{}",std::env::consts::DLL_EXTENSION))) || !name.bytes().all(|c|c.is_ascii_alphanumeric() || b"_-.".contains(&c)) || name.starts_with('.') {return Err(invalid());}
        if files.insert(name,&remaining[name_len..total]).is_some(){return Err(invalid());}
        remaining=&remaining[total..];
    }
    if !files.contains_key("libtsrust_runtime.rlib") {return Err(invalid());}
    fs::create_dir(destination)?;
    for (name,content) in files {fs::write(destination.join(name),content)?;}
    Ok(())
}

#[cfg(test)]
mod artifact_tests {
    use super::*;
    #[cfg(all(windows,target_env="msvc"))]
    #[test]
    fn windows_imports_restore_exactly_after_artifact_round_trip_and_preserve_unexpected_bytes(){
        // This source is compiled by both the CLI unit tests and an integration
        // crate; its witness must not depend on either crate's private helpers.
        let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root=std::env::temp_dir().join(format!("tsrust-import-roundtrip-{}-{stamp}",std::process::id()));
        fs::create_dir(&root).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(root.clone());
        let dependencies=root.join("deps");fs::create_dir(&dependencies).unwrap();
        fs::write(dependencies.join("libwindows_targets-witness.rlib"),b"target metadata").unwrap();
        let library=root.join("libtsrust_runtime.rlib");fs::write(&library,b"runtime").unwrap();
        assert!(restore_import_libraries(&dependencies).unwrap()>0);
        let imports=fs::read_dir(&dependencies).unwrap().map(Result::unwrap).filter(|file|file.path().extension().is_some_and(|value|value=="lib")).map(|file|(file.file_name(),fs::read(file.path()).unwrap())).collect::<Vec<_>>();
        assert!(!imports.is_empty());
        let packed=root.join("packed");pack(&library,&dependencies,&packed).unwrap();let restored=root.join("restored");unpack(&packed,&restored).unwrap();
        assert!(imports.iter().all(|(name,_)|restored.join(name).is_file()),"new artifacts retain verified imports for cache-only use");
        assert_eq!(restore_import_libraries(&restored).unwrap(),imports.len());
        for (name,bytes) in &imports{assert_eq!(fs::read(restored.join(name)).unwrap(),*bytes);}
        assert_eq!(restore_import_libraries(&restored).unwrap(),imports.len());
        let occupied=restored.join(&imports[0].0);fs::write(&occupied,b"owner bytes").unwrap();
        assert!(restore_import_libraries(&restored).is_err());assert_eq!(fs::read(occupied).unwrap(),b"owner bytes");
    }
    #[test]
    fn artifact_round_trip_preserves_bytes_and_dependency_identity() {
        let directory=std::env::temp_dir().join(format!("tsrust-bundle-roundtrip-{}",std::process::id()));
        fs::create_dir(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _=fs::remove_dir_all(&self.0); } }
        let _cleanup=Cleanup(directory.clone());
        let dependencies=directory.join("deps");fs::create_dir(&dependencies).unwrap();
        let library=directory.join("libtsrust_runtime.rlib");fs::write(&library,b"runtime\0bytes").unwrap();
        fs::write(dependencies.join("libdependency.rlib"),b"dependency").unwrap();
        let macro_name=format!("libmacro.{}",std::env::consts::DLL_EXTENSION);fs::write(dependencies.join(&macro_name),b"proc macro").unwrap();
        fs::write(dependencies.join("ignored.d"),b"metadata").unwrap();
        let bundle=directory.join("bundle");pack(&library,&dependencies,&bundle).unwrap();
        let output=directory.join("output");unpack(&bundle,&output).unwrap();
        assert_eq!(fs::read(output.join("libtsrust_runtime.rlib")).unwrap(),b"runtime\0bytes");
        assert_eq!(fs::read(output.join("libdependency.rlib")).unwrap(),b"dependency");
        assert!(!output.join("ignored.d").exists());
        assert_eq!(fs::read(output.join(macro_name)).unwrap(),b"proc macro");
        assert!(unpack(&bundle,&output).is_err());
        let mut legacy=fs::read(&bundle).unwrap();legacy[..8].copy_from_slice(b"TSRLIB2\0");
        fs::write(&bundle,legacy).unwrap();let old_output=directory.join("legacy-output");
        unpack(&bundle,&old_output).unwrap();
        assert_eq!(fs::read(old_output.join("libtsrust_runtime.rlib")).unwrap(),b"runtime\0bytes");
        assert!(IDENTITY.starts_with("native-dependencies-1\n"));
        assert!(IDENTITY.contains(sources::MANIFEST));
        assert!(IDENTITY.contains(sources::LOCK));
    }
}
