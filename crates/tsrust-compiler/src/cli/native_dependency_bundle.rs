//! Materialize compiler-owned source dependencies into a private build directory.
use std::{fs, io, path::Path};
include!(concat!(env!("OUT_DIR"), "/native_dependency_files.rs"));

pub(crate) fn materialize(root: &Path) -> io::Result<()> {
    // The caller supplies a fresh private directory. Never follow an existing
    // directory's symlinks or overwrite files belonging to another build.
    fs::create_dir(root)?;
    let result=(|| {
    for (relative, bytes) in FILES {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, bytes)?;
    }
    fs::create_dir(root.join(".cargo"))?;
    fs::write(root.join(".cargo/config.toml"), "[source.crates-io]\nreplace-with = \"bundled\"\n[source.bundled]\ndirectory = \"vendor\"\n[net]\noffline = true\n")
    })();
    if result.is_err(){let _=fs::remove_dir_all(root);}
    result
}

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
        let architecture=match std::env::consts::ARCH {"x86"=>"i686",value=>value};
        let prefix=format!("vendor/windows_{architecture}_msvc-");let mut count=0;
        for (relative,bytes) in FILES {
            if !relative.starts_with(prefix.as_str())||!relative.ends_with(".lib"){continue;}
            let name=Path::new(relative).file_name().ok_or_else(||io::Error::other("Invalid bundled native import path"))?;
            let path=root.join(name);
            match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut output)=>output.write_all(bytes)?,
                Err(error) if error.kind()==io::ErrorKind::AlreadyExists=>{
                    let metadata=fs::symlink_metadata(&path)?;
                    if !metadata.is_file()||metadata.file_type().is_symlink()||metadata.len()!=bytes.len() as u64||fs::read(&path)?.as_slice()!=*bytes{return Err(io::Error::other(format!("Native import library differs from pinned bytes; preserved {}",path.display())));}
                },
                Err(error)=>return Err(error),
            }
            count+=1;
        }
        if count==0{return Err(io::Error::other("Pinned native bundle lacks this Windows host's import libraries"));}
        Ok(count)
    }
}

pub(crate) fn pack(library:&Path,dependencies:&Path,destination:&Path)->io::Result<()> {
    use std::io::Write;
    let mut files=Vec::new();
    for entry in fs::read_dir(dependencies)? {
        let entry=entry?;if entry.file_type()?.is_file() && entry.path().extension().is_some_and(|e|e=="rlib" || e==std::env::consts::DLL_EXTENSION) {files.push(entry.path());}
    }
    files.push(library.to_owned());files.sort();
    let mut out=fs::File::create(destination)?;out.write_all(b"TSRLIB2\0")?;
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
    if bytes.len()>1024*1024*1024 || !bytes.starts_with(b"TSRLIB2\0") {return Err(invalid());}
    let mut remaining=&bytes[8..];let mut files=std::collections::BTreeMap::new();
    while !remaining.is_empty() {
        if remaining.len()<16 || files.len()>=512 {return Err(invalid());}
        let name_len=usize::try_from(u64::from_le_bytes(remaining[..8].try_into().unwrap())).map_err(|_|invalid())?;
        let length=usize::try_from(u64::from_le_bytes(remaining[8..16].try_into().unwrap())).map_err(|_|invalid())?;
        remaining=&remaining[16..];
        let total=name_len.checked_add(length).ok_or_else(invalid)?;
        if total>remaining.len(){return Err(invalid());}
        let name=std::str::from_utf8(&remaining[..name_len]).map_err(|_|invalid())?;
        if !(name.ends_with(".rlib") || name.ends_with(&format!(".{}",std::env::consts::DLL_EXTENSION))) || !name.bytes().all(|c|c.is_ascii_alphanumeric() || b"_-.".contains(&c)) || name.starts_with('.') {return Err(invalid());}
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
        let temporary=crate::TempDirectory::new().unwrap();let root=&temporary.0;
        let dependencies=root.join("deps");fs::create_dir(&dependencies).unwrap();
        fs::write(dependencies.join("libwindows_targets-witness.rlib"),b"target metadata").unwrap();
        let library=root.join("libtsrust_runtime.rlib");fs::write(&library,b"runtime").unwrap();
        assert!(restore_import_libraries(&dependencies).unwrap()>0);
        let imports=fs::read_dir(&dependencies).unwrap().map(Result::unwrap).filter(|file|file.path().extension().is_some_and(|value|value=="lib")).map(|file|(file.file_name(),fs::read(file.path()).unwrap())).collect::<Vec<_>>();
        assert!(!imports.is_empty());
        let packed=root.join("packed");pack(&library,&dependencies,&packed).unwrap();let restored=root.join("restored");unpack(&packed,&restored).unwrap();
        assert!(imports.iter().all(|(name,_)|!restored.join(name).exists()),"imports are derived from pinned compiler inputs, not a changed cache format");
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
        assert!(IDENTITY.starts_with("native-dependencies-1\n"));
        for name in ["Cargo.toml","Cargo.lock"] {
            let (_,bytes)=FILES.iter().find(|(path,_)|*path==name).unwrap();
            assert!(IDENTITY.contains(std::str::from_utf8(bytes).unwrap()));
        }
    }
}
