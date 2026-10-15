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
