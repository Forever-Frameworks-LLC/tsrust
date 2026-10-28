//! Official release acquisition and validation, before any downloaded code executes.
use std::{collections::{BTreeMap,BTreeSet},fs,io::{Read,Write},path::Path,time::Duration};
use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};

const API:&str="https://api.github.com/repos/Forever-Frameworks-LLC/tsrust/releases/latest";
const DOWNLOAD:&str="https://github.com/Forever-Frameworks-LLC/tsrust/releases/download/";
const ARCHIVE_LIMIT:u64=256*1024*1024;
const EXPANDED_LIMIT:u64=1024*1024*1024;
pub(super) fn digest(bytes:&[u8])->String{Sha256::digest(bytes).iter().map(|b|format!("{b:02x}")).collect()}
pub(super) fn hash(value:&str)->bool{value.len()==64&&value.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))}
pub(super) fn version(value:&str)->Result<[u64;3],String>{
 let fields=value.split('.').collect::<Vec<_>>();
 if fields.len()!=3{return Err("Expected a stable semantic version".into());}
 let mut result=[0;3];for (index,field) in fields.iter().enumerate(){
  if field.is_empty()||field.len()>1&&field.starts_with('0')||!field.bytes().all(|b|b.is_ascii_digit()){return Err("Expected a stable semantic version".into());}
  result[index]=field.parse().map_err(|_|"Version component is too large")?;
 }Ok(result)
}
pub(super) fn host()->Result<&'static str,String>{match(std::env::consts::OS,std::env::consts::ARCH){
 ("macos","aarch64")=>Ok("aarch64-apple-darwin"),("macos","x86_64")=>Ok("x86_64-apple-darwin"),
 ("linux","x86_64") if cfg!(target_env="gnu")=>Ok("x86_64-unknown-linux-gnu"),
 ("windows","x86_64") if cfg!(target_env="msvc")=>Ok("x86_64-pc-windows-msvc"),
 _=>Err("No official prebuilt toolchain supports this host; use a source installation".into())
}}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct Release{pub version:String,pub target:String,pub name:String,pub url:String,pub sha256:String,pub bytes:u64}
pub(super) fn metadata(bytes:&[u8],target:&str)->Result<Release,String>{
 let value:serde_json::Value=serde_json::from_slice(bytes).map_err(|_|"Invalid GitHub release metadata")?;
 if value["draft"]!=false||value["prerelease"]!=false||value["immutable"]!=true||!value["published_at"].is_string(){return Err("The latest release must be published, stable and immutable".into());}
 let tag=value["tag_name"].as_str().ok_or("Missing release tag")?;
 let number=tag.strip_prefix('v').ok_or("Invalid release tag")?;version(number)?;
 let name=format!("tsrust-{tag}-{target}.tar.gz");
 let assets=value["assets"].as_array().ok_or("Missing release assets")?;
 let matching=assets.iter().filter(|asset|asset["name"]==name).collect::<Vec<_>>();
 if matching.len()!=1{return Err("Release must contain exactly one archive for this host".into());}
 let asset=matching[0];let url=format!("{DOWNLOAD}{tag}/{name}");
 if asset["browser_download_url"]!=url||asset["state"]!="uploaded"{return Err("Untrusted release asset origin or state".into());}
 let sha256=asset["digest"].as_str().and_then(|s|s.strip_prefix("sha256:")).filter(|s|hash(s)).ok_or("GitHub asset SHA-256 digest is missing or invalid")?;
 let size=asset["size"].as_u64().filter(|size|*size>0&&*size<=ARCHIVE_LIMIT).ok_or("Release archive exceeds the size limit")?;
 Ok(Release{version:number.into(),target:target.into(),name,url,sha256:sha256.into(),bytes:size})
}
fn client()->Result<reqwest::Client,String>{reqwest::Client::builder()
 .redirect(reqwest::redirect::Policy::none()).retry(reqwest::retry::never())
 .connect_timeout(Duration::from_secs(10)).timeout(Duration::from_secs(120))
 .user_agent(concat!("tsrust/",env!("CARGO_PKG_VERSION"))).build().map_err(|_|"Cannot initialize release transport".into())}
async fn body(mut response:reqwest::Response,limit:u64)->Result<Vec<u8>,String>{
 if !response.status().is_success(){return Err(format!("Release request returned {}",response.status()));}
 if response.content_length().is_some_and(|n|n>limit){return Err("Release response exceeds its size limit".into());}
 let mut bytes=Vec::new();while let Some(chunk)=response.chunk().await.map_err(|_|"Cannot read release response")?{
  if bytes.len() as u64+chunk.len() as u64>limit{return Err("Release response exceeds its size limit".into());}bytes.extend_from_slice(&chunk);
 }Ok(bytes)
}
pub(super) async fn latest()->Result<Release,String>{
 let response=client()?.get(API).header("Accept","application/vnd.github+json").send().await.map_err(|_|"Cannot contact the official GitHub release API (TLS, connection or timeout)")?;
 if response.status()==reqwest::StatusCode::NOT_FOUND{return Err("No published stable official toolchain is available yet; the existing installation was not changed".into());}
 metadata(&body(response,2*1024*1024).await?,host()?)
}
fn redirect(value:&str)->Result<reqwest::Url,String>{
 let url=reqwest::Url::parse(value).map_err(|_|"Invalid release redirect")?;
 if url.scheme()!="https"||url.username()!=""||url.password().is_some()||url.fragment().is_some()||url.port().is_some_and(|p|p!=443)
  ||url.host_str()!=Some("release-assets.githubusercontent.com") {return Err("Release redirect left the trusted GitHub asset host".into());}Ok(url)
}
pub(super) async fn download(release:&Release)->Result<Vec<u8>,String>{
 let expected=format!("{DOWNLOAD}v{}/{}",release.version,release.name);
 if release.url!=expected{return Err("Untrusted release download URL".into());}
 let client=client()?;let mut response=client.get(&release.url).send().await.map_err(|_|"Release archive request failed")?;
 for _ in 0..3 {
  if !response.status().is_redirection(){break;}
  let url=redirect(response.headers().get("location").and_then(|v|v.to_str().ok()).ok_or("Missing release redirect")?)?;
  response=client.get(url).send().await.map_err(|_|"Release asset request failed")?;
 }
 let bytes=body(response,release.bytes.min(ARCHIVE_LIMIT)).await?;
 if bytes.len() as u64!=release.bytes||digest(&bytes)!=release.sha256{return Err("Release archive size or SHA-256 digest differs from authenticated metadata".into());}Ok(bytes)
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct FileRecord{pub sha256:String,pub bytes:u64}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub(super) struct Manifest{pub schema_version:u32,pub target:String,pub version:String,pub lsp_version:String,pub rust_toolchain:String,pub source_commit:String,pub source_tree_dirty:bool,pub distribution:String,pub files:BTreeMap<String,FileRecord>}
pub(super) fn safe_path(value:&str)->bool{
 !value.is_empty()&&value.is_ascii()&&value.split('/').all(|part|{
  let base=part.split('.').next().unwrap_or("").to_ascii_uppercase();
  !part.is_empty()&&part!="."&&part!=".."&&!part.ends_with(['.',' '])&&!part.bytes().any(|b|b<32||b==127||b"\\:<>\"|?*".contains(&b))
   &&!matches!(base.as_str(),"CON"|"PRN"|"AUX"|"NUL")
   &&!(base.len()==4&&(base.starts_with("COM")||base.starts_with("LPT"))&&(b'0'..=b'9').contains(&base.as_bytes()[3]))
 })
}
pub(super) fn verify(root:&Path,release:&Release)->Result<Manifest,String>{
 let root_metadata=fs::symlink_metadata(root).map_err(|e|e.to_string())?;
 if !root_metadata.file_type().is_dir(){return Err("Toolchain root must be a regular directory".into());}
 let metadata=fs::symlink_metadata(root.join("manifest.json")).map_err(|e|e.to_string())?;
 if !metadata.file_type().is_file()||metadata.len()>2*1024*1024{return Err("Toolchain manifest must be a bounded regular file".into());}
 let bytes=fs::read(root.join("manifest.json")).map_err(|e|e.to_string())?;
 if bytes.len()>2*1024*1024{return Err("Toolchain manifest is too large".into());}
 let manifest:Manifest=serde_json::from_slice(&bytes).map_err(|_|"Invalid toolchain manifest")?;
 if manifest.schema_version!=2||manifest.target!=release.target||manifest.version!=release.version||manifest.source_tree_dirty||manifest.distribution!="release"
  ||manifest.source_commit.len()!=40||!manifest.source_commit.bytes().all(|b|b.is_ascii_hexdigit()) {return Err("Toolchain manifest does not describe the selected clean release".into());}
 version(&manifest.version)?;version(&manifest.lsp_version)?;
 let suffix=if release.target.contains("-windows-"){".exe"}else{""};
 for name in [format!("bin/tsrust{suffix}"),format!("bin/tsrust-lsp{suffix}"),"LICENSE".into()]{if !manifest.files.contains_key(&name){return Err(format!("Missing required toolchain file {name}"));}}
 if !manifest.files.keys().any(|path|path.starts_with("attributions/dependencies/")){return Err("Release lacks dependency notices".into());}
 for (name,record) in &manifest.files{
  if !safe_path(name)||!hash(&record.sha256)||record.bytes>ARCHIVE_LIMIT{return Err("Invalid toolchain file identity".into());}
  let path=root.join(name);for parent in path.ancestors().skip(1).take_while(|p|*p!=root){if !fs::symlink_metadata(parent).map_err(|e|e.to_string())?.file_type().is_dir(){return Err("Toolchain files cannot traverse links or non-directories".into());}}
  let metadata=fs::symlink_metadata(&path).map_err(|e|e.to_string())?;if !metadata.file_type().is_file()||metadata.len()!=record.bytes{return Err("Toolchain file type or size differs from its manifest".into());}
  let bytes=fs::read(path).map_err(|e|e.to_string())?;
  if bytes.len() as u64!=record.bytes||digest(&bytes)!=record.sha256{return Err(format!("Toolchain file checksum differs: {name}"));}
 }Ok(manifest)
}
pub(super) fn unpack(root:&Path,release:&Release,bytes:&[u8])->Result<Manifest,String>{
 if bytes.len() as u64!=release.bytes||digest(bytes)!=release.sha256{return Err("Archive integrity check failed before extraction".into());}
 fs::create_dir(root).map_err(|e|e.to_string())?;
 let result=(||{
  let decoder=flate2::read::GzDecoder::new(bytes).take(EXPANDED_LIMIT+1);
  let mut archive=tar::Archive::new(decoder);let mut names=BTreeSet::new();let mut prefixes=BTreeMap::new();let mut files=BTreeSet::new();let mut total=0u64;
  let prefix=release.name.strip_suffix(".tar.gz").ok_or("Invalid archive name")?;
  for entry in archive.entries().map_err(|e|e.to_string())?{
   let mut entry=entry.map_err(|e|e.to_string())?;let kind=entry.header().entry_type();
   if !kind.is_file()&&!kind.is_dir(){return Err("Release archives cannot contain links or special entries".into());}
   let path=entry.path().map_err(|e|e.to_string())?.into_owned();let text=path.to_str().ok_or("Archive path must be UTF-8")?.trim_end_matches('/');
   if !safe_path(text)||!names.insert(text.to_ascii_lowercase())||names.len()>20000{return Err("Unsafe, duplicate or excessive archive paths".into());}
   let mut prefix_path=String::new();for part in text.split('/'){
    if !prefix_path.is_empty(){prefix_path.push('/');}prefix_path.push_str(part);
    if prefixes.insert(prefix_path.to_ascii_lowercase(),prefix_path.clone()).is_some_and(|old|old!=prefix_path){return Err("Archive directory names collide across platforms".into());}
   }
   if text==prefix&&kind.is_dir(){continue;}
   let relative=text.strip_prefix(&format!("{prefix}/")).ok_or("Unexpected archive root")?;let output=root.join(relative);
   if kind.is_dir(){fs::create_dir_all(&output).map_err(|e|e.to_string())?;continue;}
   let size=entry.size();total=total.checked_add(size).ok_or("Archive size overflow")?;
   if size>ARCHIVE_LIMIT||total>EXPANDED_LIMIT{return Err("Expanded archive exceeds its size limit".into());}
   fs::create_dir_all(output.parent().unwrap()).map_err(|e|e.to_string())?;
   let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&output).map_err(|e|e.to_string())?;
   if std::io::copy(&mut entry,&mut file).map_err(|e|e.to_string())?!=size{return Err("Truncated archive file".into());}
   file.flush().and_then(|_|file.sync_all()).map_err(|e|e.to_string())?;files.insert(relative.to_owned());
  }
  let manifest=verify(root,release)?;
  let mut expected=manifest.files.keys().cloned().collect::<BTreeSet<_>>();expected.extend(["manifest.json".into(),"README.md".into(),"rust-toolchain.toml".into()]);
  if files!=expected{return Err("Archive contains missing or unlisted files".into());}
  #[cfg(unix)]{use std::os::unix::fs::PermissionsExt;for name in ["tsrust","tsrust-lsp"]{fs::set_permissions(root.join("bin").join(name),fs::Permissions::from_mode(0o755)).map_err(|e|e.to_string())?;}}
  Ok(manifest)
 })();if result.is_err(){let _=fs::remove_dir_all(root);}result
}

#[cfg(test)]mod tests{
 use super::*;
 fn release()->serde_json::Value{let name="tsrust-v1.2.3-x86_64-pc-windows-msvc.tar.gz";serde_json::json!({"draft":false,"prerelease":false,"immutable":true,"published_at":"2026-10-28T00:00:00Z","tag_name":"v1.2.3","assets":[{"name":name,"state":"uploaded","browser_download_url":format!("{DOWNLOAD}v1.2.3/{name}"),"digest":format!("sha256:{}","a".repeat(64)),"size":100}]})}
 #[test]fn metadata_requires_authenticated_immutable_stable_host_asset(){let value=release();let parse=|v:&serde_json::Value|metadata(&serde_json::to_vec(v).unwrap(),"x86_64-pc-windows-msvc");assert!(parse(&value).is_ok());
  for (key,bad) in [("draft",true),("prerelease",true),("immutable",false)]{let mut v=value.clone();v[key]=bad.into();assert!(parse(&v).is_err());}
  for (key,bad) in [("digest",""),("browser_download_url","https://example.invalid/tool.tar.gz"),("state","new")]{let mut v=value.clone();v["assets"][0][key]=bad.into();assert!(parse(&v).is_err());}
  let mut v=value.clone();v["assets"].as_array_mut().unwrap().push(value["assets"][0].clone());assert!(parse(&v).is_err());
 }
 #[test]fn versions_paths_and_redirects_are_closed(){for v in ["1.2","01.2.3","1.2.3-beta","1.2.3+build","-1.2.3"]{assert!(version(v).is_err());}assert!(version("1.10.0").unwrap()>version("1.9.9").unwrap());
  for path in ["../bin/tsrust","/bin/tsrust","bin\\tsrust","C:/bin","bin/AUX.txt","bin/name.","bin/name ","bin/a:b","bin//a"]{assert!(!safe_path(path),"{path}");}
  for url in ["http://release-assets.githubusercontent.com/a","https://example.invalid/a","https://user@release-assets.githubusercontent.com/a"]{assert!(redirect(url).is_err());}
  assert!(redirect("https://release-assets.githubusercontent.com/a?token=opaque").is_ok());
 }
}
