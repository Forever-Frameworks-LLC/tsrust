//! Optional, explicitly prepared Cargo vendor packs. Never runs during a build.
use std::{collections::BTreeMap, fs, io, path::{Path, PathBuf}, process::Command};
use sha2::{Digest, Sha256};

pub(super) const MANIFEST: &str = include_str!("../../native-dependencies/Cargo.toml");
pub(super) const LOCK: &str = include_str!("../../native-dependencies/Cargo.lock");
const LIB: &str = include_str!("../../native-dependencies/src/lib.rs");
const CONFIG: &str = "[source.crates-io]\nreplace-with = \"bundled\"\n[source.bundled]\ndirectory = \"vendor\"\n[net]\noffline = true\n";

fn error(message: impl Into<String>) -> io::Error {io::Error::other(message.into())}
fn digest(bytes: &[u8]) -> String {Sha256::digest(bytes).iter().map(|b|format!("{b:02x}")).collect()}
pub(super) fn cargo_home() -> io::Result<PathBuf> {
    let home=std::env::var_os("CARGO_HOME").map(PathBuf::from).or_else(||
        std::env::var_os(if cfg!(windows){"USERPROFILE"}else{"HOME"}).map(|p|PathBuf::from(p).join(".cargo")))
        .ok_or_else(||error("Cannot locate Cargo home; set CARGO_HOME"))?;
    Ok(if home.is_absolute(){home}else{std::env::current_dir()?.join(home)})
}
pub(super) fn location(manifest: &str, lock: &str) -> io::Result<PathBuf> {
    let base=match std::env::var_os("TSRUST_SOURCE_PACK_DIR") {Some(path)=>PathBuf::from(path),None=>cargo_home()?.join("tsrust/source-packs")};
    let base=if base.is_absolute(){base}else{std::env::current_dir()?.join(base)};
    let patch=tsrust_compiler::native_url_patch::source_identity(manifest)?;
    Ok(base.join(digest(format!("portable-source-pack-1\nnative-dependencies-1\n{manifest}{lock}\n{LIB}\n{CONFIG}{patch}").as_bytes())))
}
fn regular(path: &Path, directory: bool) -> io::Result<()> {
    let metadata=fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || if directory{!metadata.is_dir()}else{!metadata.is_file()}{return Err(error(format!("Source pack contains a link or unexpected file type: {}",path.display())));}
    Ok(())
}
fn files(root: &Path, directory: &Path, result: &mut Vec<PathBuf>) -> io::Result<()> {
    regular(directory,true)?;
    for entry in fs::read_dir(directory)? {
        let entry=entry?;let path=entry.path();let kind=entry.file_type()?;
        if kind.is_dir(){files(root,&path,result)?;}else{regular(&path,false)?;result.push(path.strip_prefix(root).map_err(|e|error(e.to_string()))?.to_owned());}
    }
    Ok(())
}
// The published archive omits its repository-root license. Emit the exact
// reviewed supplement outside vendor/; never rewrite Cargo's checksums or cache.
fn supplemental_notice(package:&str,checksum:&str)->io::Result<Option<&'static [u8]>> {
    if package!="valuable-0.1.1" {return Ok(None);}
    if checksum!="ba73ea9cf16a25df0c8caa16c51acb937d5712a8429db78a3ee29d5dcacd3a65" {return Err(error("valuable 0.1.1 archive identity changed"));}
    let bytes=include_bytes!("../../../../attributions/valuable-0.1.1-MIT.txt");
    if digest(bytes)!="ed60d479b8fd1f64e9cbc3de449a16a53ac1b3d1b6aeb9bf9d190a8e93061b44" {return Err(error("valuable 0.1.1 supplemental MIT notice changed"));}
    Ok(Some(bytes))
}

fn packages(lock: &str) -> io::Result<BTreeMap<String,String>> {
    let document=lock.parse::<toml_edit::DocumentMut>().map_err(|e|error(e.to_string()))?;
    let mut packages=BTreeMap::new();
    for package in document["package"].as_array_of_tables().ok_or_else(||error("Invalid pinned source lock"))? {
        if let Some(source)=package.get("source").and_then(|v|v.as_str()) {
            if source!="registry+https://github.com/rust-lang/crates.io-index" {return Err(error("Unsupported portable source registry"));}
            let value=|key:&str|package.get(key).and_then(|v|v.as_str()).ok_or_else(||error("Invalid pinned package"));
            packages.insert(format!("{}-{}",value("name")?,value("version")?),value("checksum")?.into());
        }
    }
    Ok(packages)
}

/// Verify Cargo's pinned package identities and every vendor file checksum before
/// copying. Cargo remains the vendor producer; packs contain no executable hooks.
fn visit(root: &Path, manifest: &str, lock: &str, mut copy: impl FnMut(&Path,&[u8])->io::Result<()>) -> io::Result<()> {
    regular(root,true)?;
    let normalized=tsrust_compiler::native_url_patch::normalize_manifest(manifest)?;
    tsrust_compiler::native_url_patch::visit(root,manifest,|relative,bytes|copy(relative,bytes))?;
    for (relative,expected) in [("Cargo.toml",normalized.as_str()),("Cargo.lock",lock),("src/lib.rs",LIB),(".cargo/config.toml",CONFIG)] {
        let path=root.join(relative);regular(path.parent().unwrap(),true)?;regular(&path,false)?;
        let bytes=fs::read(path)?;if bytes!=expected.as_bytes(){return Err(error(format!("Prepared source metadata changed: {relative}")));}copy(Path::new(relative),&bytes)?;
    }
    let vendor=root.join("vendor");regular(&vendor,true)?;let mut expected=packages(lock)?;
    for entry in fs::read_dir(&vendor)? {
        let entry=entry?;let name=entry.file_name().into_string().map_err(|_|error("Non-UTF8 vendor package"))?;
        let checksum=expected.remove(&name).ok_or_else(||error(format!("Unexpected vendor package: {name}")))?;
        let package=entry.path();regular(&package,true)?;
        let receipt=package.join(".cargo-checksum.json");regular(&receipt,false)?;
        let bytes=fs::read(&receipt)?;
        let metadata: serde_json::Value=serde_json::from_slice(&bytes).map_err(|e|error(e.to_string()))?;
        if metadata["package"].as_str()!=Some(checksum.as_str()){return Err(error(format!("Pinned package checksum differs: {name}")));}
        let mut hashes=metadata["files"].as_object().cloned().ok_or_else(||error("Missing Cargo file checksums"))?;
        copy(&Path::new("vendor").join(&name).join(".cargo-checksum.json"),&bytes)?;
        let mut paths=Vec::new();files(&package,&package,&mut paths)?;
        for relative in paths {
            if relative==Path::new(".cargo-checksum.json"){continue;}
            let key=relative.to_str().ok_or_else(||error("Non-UTF8 vendor file"))?.replace('\\',"/");
            let expected=hashes.remove(&key).ok_or_else(||error(format!("Unregistered vendor file: {name}/{key}")))?;
            let bytes=fs::read(package.join(&relative))?;
            if expected.as_str()!=Some(digest(&bytes).as_str()){return Err(error(format!("Vendor checksum changed: {name}/{key}")));}
            copy(&Path::new("vendor").join(&name).join(relative),&bytes)?;
        }
        if !hashes.is_empty(){return Err(error(format!("Missing vendor files: {name}")));}
        if let Some(notice)=supplemental_notice(&name,&checksum)? {
            copy(Path::new("attributions/dependencies/valuable-0.1.1/LICENSE"),notice)?;
        }
    }
    if !expected.is_empty(){return Err(error("Prepared source pack is incomplete"));}Ok(())
}

pub(super) fn materialize(root: &Path, manifest: &str, lock: &str) -> io::Result<()> {
    let source=location(manifest,lock)?;
    if !source.exists(){return Err(error("Portable sources are not prepared. Run `tsrust setup --portable --yes` (or --offline with cached sources); For bindings, run `tsrust bindings resolve <file.bindings.json> --portable` with the same Cargo home/source-pack directory. Export itself never invokes Cargo or downloads."));}
    fs::create_dir(root)?;
    let result=visit(&source,manifest,lock,|relative,bytes|{let path=root.join(relative);fs::create_dir_all(path.parent().unwrap())?;fs::write(path,bytes)});
    if result.is_err(){let _=fs::remove_dir_all(root);}result
}

pub(super) fn prepare(mut cargo: Command, offline: bool, manifest: &str, lock: &str) -> io::Result<PathBuf> {
    let destination=location(manifest,lock)?;
    // An existing pack that fails verification is reported, never overwritten.
    if destination.exists(){
        visit(&destination,manifest,lock,|_,_|Ok(())).map_err(|e|error(format!("Prepared source pack {} does not verify: {e}. It was left unchanged; remove that directory and run `tsrust setup --portable` again.",destination.display())))?;
        return Ok(destination);
    }
    let parent=destination.parent().unwrap();fs::create_dir_all(parent)?;
    let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|error(e.to_string()))?.as_nanos();
    let staging=parent.join(format!(".preparing-{}-{stamp}",std::process::id()));fs::create_dir(&staging)?;
    let result=(|| {
        fs::create_dir(staging.join("src"))?;fs::create_dir(staging.join(".cargo"))?;
        fs::write(staging.join("Cargo.toml"),tsrust_compiler::native_url_patch::normalize_manifest(manifest)?)?;
        tsrust_compiler::native_url_patch::install(&staging,manifest)?;fs::write(staging.join("Cargo.lock"),lock)?;fs::write(staging.join("src/lib.rs"),LIB)?;
        // Config is written after Cargo finishes so setup may explicitly fetch.
        cargo.args(["vendor","--locked","--versioned-dirs","--manifest-path"]).arg(staging.join("Cargo.toml")).arg(staging.join("vendor"))
            .env("CARGO_HOME",cargo_home()?).stdout(std::process::Stdio::null());
        if offline{cargo.arg("--offline");}
        if !cargo.status()?.success(){return Err(error("Portable source preparation failed; existing source packs were preserved"));}
        fs::write(staging.join(".cargo/config.toml"),CONFIG)?;
        visit(&staging,manifest,lock,|_,_|Ok(()))?;
        if destination.exists(){visit(&destination,manifest,lock,|_,_|Ok(()))?;}else{fs::rename(&staging,&destination)?;}
        Ok(destination)
    })();
    // Only the exclusively created staging directory is owned by this call.
    if staging.exists(){let _=fs::remove_dir_all(&staging);}result
}

#[cfg(test)]
mod supplemental_notice_tests {
    use super::*;
    #[test]
    fn valuable_notice_is_bound_to_the_exact_archive_and_kept_outside_vendor() {
        let bytes=supplemental_notice("valuable-0.1.1","ba73ea9cf16a25df0c8caa16c51acb937d5712a8429db78a3ee29d5dcacd3a65").unwrap().unwrap();
        assert_eq!(bytes.len(),1065);assert_eq!(digest(bytes),"ed60d479b8fd1f64e9cbc3de449a16a53ac1b3d1b6aeb9bf9d190a8e93061b44");
        assert!(supplemental_notice("valuable-0.1.1","changed").is_err());
        assert!(supplemental_notice("unrelated-0.1.1","changed").unwrap().is_none());
        assert!(supplemental_notice("valuable-0.1.2","changed").unwrap().is_none());
    }
}
