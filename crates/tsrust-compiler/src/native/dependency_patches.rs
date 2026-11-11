//! Exact compiler-owned dependency patches; Cargo registry checksums stay untouched.
use std::{collections::BTreeSet,fs,io,path::{Path,PathBuf}};
use sha2::{Digest,Sha256};
const DIRECTORY:&str="patched-dependencies/url";
const FILES:&[(&str,&[u8])]=&[
    ("Cargo.toml",include_bytes!("../../patched-dependencies/url/Cargo.toml")),
    ("LICENSE-APACHE",include_bytes!("../../patched-dependencies/url/LICENSE-APACHE")),
    ("LICENSE-MIT",include_bytes!("../../patched-dependencies/url/LICENSE-MIT")),
    ("README.md",include_bytes!("../../patched-dependencies/url/README.md")),
    ("TSRUST-PATCH.md",include_bytes!("../../patched-dependencies/url/TSRUST-PATCH.md")),
    ("src/host.rs",include_bytes!("../../patched-dependencies/url/src/host.rs")),
    ("src/lib.rs",include_bytes!("../../patched-dependencies/url/src/lib.rs")),
    ("src/origin.rs",include_bytes!("../../patched-dependencies/url/src/origin.rs")),
    ("src/parser.rs",include_bytes!("../../patched-dependencies/url/src/parser.rs")),
    ("src/path_segments.rs",include_bytes!("../../patched-dependencies/url/src/path_segments.rs")),
    ("src/quirks.rs",include_bytes!("../../patched-dependencies/url/src/quirks.rs")),
    ("src/slicing.rs",include_bytes!("../../patched-dependencies/url/src/slicing.rs")),
    ("tests/expected_failures.txt",include_bytes!("../../patched-dependencies/url/tests/expected_failures.txt")),
    ("tests/setters_tests.json",include_bytes!("../../patched-dependencies/url/tests/setters_tests.json")),
    ("tests/unit.rs",include_bytes!("../../patched-dependencies/url/tests/unit.rs")),
    ("tests/urltestdata.json",include_bytes!("../../patched-dependencies/url/tests/urltestdata.json")),
    ("tests/wpt.rs",include_bytes!("../../patched-dependencies/url/tests/wpt.rs")),
];
include!("gpui_patch_files.rs");
struct Patch {name:&'static str,directory:&'static str,files:&'static [(&'static str,&'static [u8])]}
const PATCHES:&[Patch]=&[
    Patch{name:"url",directory:DIRECTORY,files:FILES},
    Patch{name:"gpui-base",directory:"patched-dependencies/gpui-base",files:GPUI_FILES},
];
fn invalid(message:impl Into<String>)->io::Error {io::Error::other(message.into())}
fn selected(source:&str,patch:&Patch)->io::Result<Option<std::ops::Range<usize>>> {
    let document=toml_edit::Document::parse(source).map_err(|e|invalid(e.to_string()))?;
    let Some(item)=document.get("patch").and_then(|v|v.get("crates-io")).and_then(|v|v.get(patch.name)) else{return Ok(None);};
    let path=item.get("path").ok_or_else(||invalid(format!("The compiler-owned {} patch path is missing.",patch.name)))?;
    if !path.as_str().is_some_and(|value|value==patch.directory || value==format!("../{}",patch.directory)) {
        return Err(invalid(format!("The compiler-owned {} patch path was changed.",patch.name)));
    }
    Ok(Some(path.as_value().and_then(|value|value.span()).ok_or_else(||invalid("Dependency patch path has no source range."))?))
}
fn active_files(manifest:&str)->io::Result<Vec<(PathBuf,&'static [u8])>> {
    let mut files=Vec::new();
    for patch in PATCHES {if selected(manifest,patch)?.is_some(){for (name,bytes) in patch.files{files.push((Path::new(patch.directory).join(name),*bytes));}}}
    Ok(files)
}
pub fn normalize_manifest(source:&str)->io::Result<String> {
    let mut edits=Vec::new();
    for patch in PATCHES {if let Some(range)=selected(source,patch)?{edits.push((range,format!("{:?}",patch.directory)));}}
    edits.sort_by_key(|(range,_)|std::cmp::Reverse(range.start));let mut normalized=source.to_owned();
    for (range,value) in edits{normalized.replace_range(range,&value);}Ok(normalized)
}
pub fn source_identity(manifest:&str)->io::Result<String> {
    if selected(manifest,&PATCHES[1])?.is_none(){return Ok(if selected(manifest,&PATCHES[0])?.is_some(){cache_identity("url-patch-1")}else{String::new()});}
    let mut hash=Sha256::new();hash.update(b"dependency-patches-1");
    for (name,bytes) in active_files(manifest)? {for part in [name.to_str().ok_or_else(||invalid("Non-UTF8 patch path"))?.as_bytes(),bytes]{hash.update((part.len() as u64).to_le_bytes());hash.update(part);}}
    Ok(hash.finalize().iter().map(|b|format!("{b:02x}")).collect())
}
/// Preserve the existing backend/URL cache protocol; GPUI callers add source_identity separately.
pub fn cache_identity(base:&str)->String {
    let mut hash=Sha256::new();
    for bytes in std::iter::once(base.as_bytes()).chain(FILES.iter().flat_map(|(name,bytes)|[name.as_bytes(),*bytes])) {
        hash.update((bytes.len() as u64).to_le_bytes());hash.update(bytes);
    }
    hash.finalize().iter().map(|b|format!("{b:02x}")).collect()
}
fn regular(path:&Path,directory:bool)->io::Result<()> {
    let metadata=fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || if directory{!metadata.is_dir()}else{!metadata.is_file()} {return Err(invalid(format!("Dependency patch has a link or unexpected file type: {}",path.display())));}
    Ok(())
}
fn parents(root:&Path,relative:&Path)->io::Result<()> {
    let mut directory=root.to_owned();
    for component in relative.parent().unwrap().components() {
        directory.push(component);
        match fs::create_dir(&directory) {Ok(())=>{},Err(error) if error.kind()==io::ErrorKind::AlreadyExists=>regular(&directory,true)?,Err(error)=>return Err(error)}
    }
    Ok(())
}
/// The caller owns a newly created root and cleans it up if installation fails.
pub fn install(root:&Path,manifest:&str)->io::Result<()> {
    let files=active_files(manifest)?;if files.is_empty(){return Ok(());}
    regular(root,true)?;
    for (relative,bytes) in files {
        parents(root,&relative)?;
        use io::Write;
        fs::OpenOptions::new().write(true).create_new(true).open(root.join(relative))?.write_all(bytes)?;
    }
    Ok(())
}
/// The caller holds the workspace lock. Cache restoration can keep a support
/// directory while omitting payload files: recreate only absent registered files.
/// Present changed files and links are rejected before anything is written.
pub fn restore_missing(root:&Path,manifest:&str)->io::Result<()> {
    let files=active_files(manifest)?;if files.is_empty(){return Ok(());}
    regular(root,true)?;
    let mut missing=Vec::new();
    for (relative,bytes) in files {
        let mut ancestor=root.to_owned();
        for component in relative.parent().unwrap().components() {
            ancestor.push(component);
            match fs::symlink_metadata(&ancestor) {Ok(_)=>regular(&ancestor,true)?,Err(error) if error.kind()==io::ErrorKind::NotFound=>break,Err(error)=>return Err(error)}
        }
        match fs::symlink_metadata(root.join(&relative)) {
            Ok(_)=>{regular(&root.join(&relative),false)?;if fs::read(root.join(&relative))?!=bytes{return Err(invalid(format!("Dependency patch source changed: {}",relative.display())));}},
            Err(error) if error.kind()==io::ErrorKind::NotFound=>missing.push((relative,bytes)),
            Err(error)=>return Err(error),
        }
    }
    for (relative,bytes) in missing {
        parents(root,&relative)?;
        use io::Write;
        fs::OpenOptions::new().write(true).create_new(true).open(root.join(relative))?.write_all(bytes)?;
    }
    Ok(())
}
fn collect(directory:&Path,paths:&mut Vec<PathBuf>)->io::Result<()> {
    regular(directory,true)?;
    for entry in fs::read_dir(directory)? {
        let entry=entry?;let path=entry.path();
        if entry.file_type()?.is_dir(){collect(&path,paths)?;}else{regular(&path,false)?;paths.push(path);}
    }
    Ok(())
}
/// Verify every payload byte and reject injected files before copying or building.
pub fn visit(root:&Path,manifest:&str,mut copy:impl FnMut(&Path,&[u8])->io::Result<()>)->io::Result<()> {
    let files=active_files(manifest)?;if files.is_empty(){return Ok(());}
    regular(root,true)?;
    let mut expected=files.iter().map(|(name,_)|name.clone()).collect::<BTreeSet<_>>();
    let mut paths=Vec::new();collect(&root.join("patched-dependencies"),&mut paths)?;
    for path in paths {
        let relative=path.strip_prefix(root).map_err(|e|invalid(e.to_string()))?;
        if !expected.remove(relative){return Err(invalid(format!("Unregistered Dependency patch file: {}",relative.display())));}
    }
    if !expected.is_empty(){return Err(invalid("Dependency patch payload is incomplete."));}
    for (relative,bytes) in files {
        if fs::read(root.join(&relative))?!=bytes{return Err(invalid(format!("Dependency patch source changed: {}",relative.display())));}
        copy(&relative,bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const MANIFEST:&str="[package]\nname=\"probe\"\nversion=\"0.0.0\"\n[patch.crates-io]\nurl={path=\"../patched-dependencies/url\"}\n";
    #[test]
    fn gpui_and_url_payloads_relocate_together_without_changing_backend_identity() {
        let manifest=format!("{MANIFEST}gpui-base={{path=\"../patched-dependencies/gpui-base\"}}\n");
        let normalized=normalize_manifest(&manifest).unwrap();
        assert_eq!(normalize_manifest(&normalized).unwrap(),normalized);
        assert_eq!(source_identity(MANIFEST).unwrap(),cache_identity("url-patch-1"));
        assert_ne!(source_identity(&manifest).unwrap(),source_identity(MANIFEST).unwrap());
        assert!(normalize_manifest(&manifest.replace("../patched-dependencies/gpui-base","/tmp/gpui-base")).is_err());
        let directory=tempfile::tempdir().unwrap();let root=directory.path();
        install(root,&manifest).unwrap();let mut count=0;
        visit(root,&normalized,|_,_|{count+=1;Ok(())}).unwrap();assert_eq!(count,FILES.len()+GPUI_FILES.len());
        let panel=root.join("patched-dependencies/gpui-base/src/resizable/panel.rs");
        let original=fs::read(&panel).unwrap();fs::write(&panel,b"modified").unwrap();
        let missing=root.join(DIRECTORY).join("src/lib.rs");fs::remove_file(&missing).unwrap();
        assert!(restore_missing(root,&manifest).is_err());assert!(!missing.exists());
        assert!(visit(root,&manifest,|_,_|Ok(())).is_err());fs::write(&panel,original).unwrap();
        restore_missing(root,&manifest).unwrap();visit(root,&manifest,|_,_|Ok(())).unwrap();
    }
    #[test]
    fn relocation_preserves_exact_patch_and_rejects_modified_or_injected_source() {
        let normalized=normalize_manifest(MANIFEST).unwrap();
        assert_eq!(normalized.parse::<toml_edit::DocumentMut>().unwrap()["patch"]["crates-io"]["url"]["path"].as_str(),Some(DIRECTORY));
        assert_eq!(normalize_manifest(&normalized).unwrap(),normalized);
        assert!(normalize_manifest(&MANIFEST.replace("../patched-dependencies/url","/tmp/url")).is_err());
        assert_eq!(normalize_manifest("[dependencies]\n").unwrap(),"[dependencies]\n");
        assert_eq!(source_identity("[dependencies]\n").unwrap(),"");
        assert_ne!(source_identity(MANIFEST).unwrap(),"");
        assert_ne!(cache_identity("first"),cache_identity("second"));
        let root=std::env::temp_dir().join(format!("tsrust-url-payload-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();
        struct Cleanup(PathBuf);impl Drop for Cleanup{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}let _cleanup=Cleanup(root.clone());
        install(&root,MANIFEST).unwrap();
        let mut count=0;visit(&root,&normalized,|_,_|{count+=1;Ok(())}).unwrap();assert_eq!(count,FILES.len());
        let path=root.join(DIRECTORY).join("src/parser.rs");let original=fs::read(&path).unwrap();fs::write(&path,b"modified").unwrap();assert!(visit(&root,MANIFEST,|_,_|Ok(())).is_err());fs::write(path,original).unwrap();
        let injected=root.join(DIRECTORY).join("build.rs");fs::write(&injected,b"fn main(){}").unwrap();assert!(visit(&root,MANIFEST,|_,_|Ok(())).is_err());fs::remove_file(injected).unwrap();
        fs::remove_file(root.join(DIRECTORY).join("src/parser.rs")).unwrap();assert!(visit(&root,MANIFEST,|_,_|Ok(())).is_err());
        fs::remove_dir_all(root.join(DIRECTORY).join("tests")).unwrap();
        restore_missing(&root,MANIFEST).unwrap();visit(&root,MANIFEST,|_,_|Ok(())).unwrap();
        fs::remove_dir_all(root.join("patched-dependencies")).unwrap();
        restore_missing(&root,MANIFEST).unwrap();visit(&root,MANIFEST,|_,_|Ok(())).unwrap();
        fs::write(root.join(DIRECTORY).join("README.md"),b"changed").unwrap();fs::remove_file(root.join(DIRECTORY).join("src/lib.rs")).unwrap();
        assert!(restore_missing(&root,MANIFEST).unwrap_err().to_string().contains("changed"));
        assert_eq!(fs::read(root.join(DIRECTORY).join("README.md")).unwrap(),b"changed");assert!(!root.join(DIRECTORY).join("src/lib.rs").exists());
        assert!(restore_missing(&root,"[dependencies]\n").is_ok());
    }
    #[test]
    #[cfg(unix)]
    fn patch_payload_rejects_directory_symlinks() {
        let root=std::env::temp_dir().join(format!("tsrust-url-link-{}",std::process::id()));fs::create_dir(&root).unwrap();
        std::os::unix::fs::symlink(std::env::temp_dir(),root.join("patched-dependencies")).unwrap();
        assert!(install(&root,MANIFEST).is_err());assert!(restore_missing(&root,MANIFEST).is_err());assert!(visit(&root,MANIFEST,|_,_|Ok(())).is_err());
        fs::remove_file(root.join("patched-dependencies")).unwrap();fs::remove_dir(root).unwrap();
    }
}
