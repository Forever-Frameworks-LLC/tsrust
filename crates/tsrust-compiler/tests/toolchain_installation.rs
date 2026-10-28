mod support;
#[cfg(windows)]#[path="../src/cli/temp_directory.rs"]mod temp_directory;
#[cfg(windows)]use temp_directory::TempDirectory;
#[allow(dead_code)]#[path="../src/cli/toolchain_release.rs"]mod toolchain_release;
#[allow(dead_code)]#[path="../src/cli/toolchain_install.rs"]mod toolchain_install;
#[cfg(windows)]#[allow(dead_code)]#[path="../src/cli/dev_windows.rs"]mod dev_windows;
use std::{collections::BTreeMap,fs,path::Path,process::{Command,Stdio}};
use toolchain_release::{Release,Manifest,FileRecord,digest};

fn fixture(root:&Path,version:&str)->(Release,Vec<u8>){
 let source=root.join(format!("probe-{version}.rs"));let binary=root.join(format!("probe-{version}{}",std::env::consts::EXE_SUFFIX));
 fs::write(&source,format!(r#"fn main(){{if std::env::args().any(|a|a=="--hold"){{println!("holding");loop{{std::thread::sleep(std::time::Duration::from_secs(1));}}}}let name=if std::env::current_exe().unwrap().file_name().unwrap().to_string_lossy().contains("lsp"){{"tsrust-lsp"}}else{{"tsrust"}};println!("{{name}} {version}");}}"#)).unwrap();
 support::success(&support::run(Command::new("rustup").args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"rustc","--edition=2024","--crate-name","toolchain_fixture"]).arg(&source).arg("-o").arg(&binary),60));
 archive(version,version,fs::read(&binary).unwrap(),fs::read(binary).unwrap())
}
fn archive(version:&str,lsp_version:&str,compiler:Vec<u8>,server:Vec<u8>)->(Release,Vec<u8>){
 let suffix=std::env::consts::EXE_SUFFIX;
 let files=BTreeMap::from([(format!("bin/tsrust{suffix}"),compiler),(format!("bin/tsrust-lsp{suffix}"),server),("LICENSE".into(),b"MIT fixture\n".to_vec()),("attributions/dependencies/example/LICENSE".into(),b"dependency fixture\n".to_vec())]);
 let manifest=Manifest{schema_version:2,target:toolchain_release::host().unwrap().into(),version:version.into(),lsp_version:lsp_version.into(),rust_toolchain:tsrust_compiler::RUST_BACKEND_VERSION.into(),source_commit:"a".repeat(40),source_tree_dirty:false,distribution:"release".into(),files:files.iter().map(|(name,bytes)|(name.clone(),FileRecord{sha256:digest(bytes),bytes:bytes.len() as u64})).collect()};
 let mut files=files;files.insert("manifest.json".into(),serde_json::to_vec(&manifest).unwrap());files.insert("README.md".into(),b"fixture".to_vec());files.insert("rust-toolchain.toml".into(),b"fixture".to_vec());
 let name=format!("tsrust-v{version}-{}.tar.gz",manifest.target);let prefix=name.strip_suffix(".tar.gz").unwrap();
 let mut archive=tar::Builder::new(flate2::write::GzEncoder::new(Vec::new(),flate2::Compression::fast()));
 for (path,bytes) in files{let mut header=tar::Header::new_gnu();header.set_size(bytes.len() as u64);header.set_mode(0o644);header.set_cksum();archive.append_data(&mut header,format!("{prefix}/{path}"),bytes.as_slice()).unwrap();}
 let bytes=archive.into_inner().unwrap().finish().unwrap();
 (Release{version:version.into(),target:manifest.target,name:name.clone(),url:format!("https://github.com/Forever-Frameworks-LLC/tsrust/releases/download/v{version}/{name}"),sha256:digest(&bytes),bytes:bytes.len() as u64},bytes)
}
fn launcher()->std::path::PathBuf{env!("CARGO_BIN_EXE_tsrust").into()}
fn run(root:&Path,name:&str)->String{
 let output=support::run(Command::new(root.join("bin").join(format!("{name}{}",std::env::consts::EXE_SUFFIX))).arg("--version").env("PATH",root.join("no-tools")).env("RUSTUP_HOME",root.join("no-rust")),20);
 support::success(&output);String::from_utf8(output.stdout).unwrap().trim().into()
}

#[test]fn real_cli_and_lsp_launch_without_development_tools_from_the_selected_pair(){
 let scratch=support::TempDirectory::new();let root=scratch.0.join("real-installation");
 let server=std::env::var_os("TSRUST_LSP").map(std::path::PathBuf::from).unwrap_or_else(||launcher().parent().unwrap().join(format!("tsrust-lsp{}",std::env::consts::EXE_SUFFIX)));
 let lsp_version=fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../tsrust-lsp/Cargo.toml")).unwrap();let lsp_version=lsp_version.lines().find_map(|line|line.strip_prefix("version = \"").and_then(|v|v.strip_suffix('"'))).unwrap();
 let (release,bytes)=archive(env!("CARGO_PKG_VERSION"),lsp_version,fs::read(launcher()).unwrap(),fs::read(server).expect("build both CLI and LSP consumers before installer acceptance"));
 toolchain_install::install(&root,&release,&bytes,&launcher()).unwrap();
 assert_eq!(run(&root,"tsrust"),format!("tsrust {}",env!("CARGO_PKG_VERSION")));assert_eq!(run(&root,"tsrust-lsp"),format!("tsrust-lsp {lsp_version}"));
 let source=scratch.0.join("main.tsr");fs::write(&source,support::format_round_trip("function main(){console.log(42);}")).unwrap();
 let mut cli=Command::new(root.join("bin").join(format!("tsrust{}",std::env::consts::EXE_SUFFIX)));cli.env("PATH",scratch.0.join("no-tools")).env("RUSTUP_HOME",scratch.0.join("no-rust"));
 support::success(&support::run(cli.args(["check"]).arg(&source),20));
 let frames=[serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}),serde_json::json!({"jsonrpc":"2.0","method":"initialized","params":{}}),serde_json::json!({"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}),serde_json::json!({"jsonrpc":"2.0","method":"exit","params":null})].into_iter().map(|value|{let body=value.to_string();format!("Content-Length: {}\r\n\r\n{body}",body.len())}).collect::<String>();
 let input=scratch.0.join("requests");fs::write(&input,frames).unwrap();
 let output=support::run(Command::new(root.join("bin").join(format!("tsrust-lsp{}",std::env::consts::EXE_SUFFIX))).stdin(fs::File::open(input).unwrap()).env("PATH",scratch.0.join("no-tools")).env("RUSTUP_HOME",scratch.0.join("no-rust")),30);
 support::success(&output);let text=String::from_utf8(output.stdout).unwrap();assert!(text.contains("\"hoverProvider\":true"),"{text}");assert!(text.contains("\"id\":2"),"{text}");
 let managed_cli=root.join("bin").join(format!("tsrust{}",std::env::consts::EXE_SUFFIX));
 let doctor=||support::run(Command::new(&managed_cli).args(["doctor","--json"]).env("PATH",scratch.0.join("no-tools")).env("CARGO_HOME",scratch.0.join("no-cargo")).env("RUSTUP_HOME",scratch.0.join("no-rust")),20);
 let output=doctor();let report:serde_json::Value=serde_json::from_slice(&output.stdout).unwrap();assert_eq!(report["installation"]["status"],"ok");assert_eq!(report["toolchainReady"],false);
 fs::write(root.join("versions").join(&release.sha256).join("bin").join(format!("tsrust-lsp{}",std::env::consts::EXE_SUFFIX)),"tampered").unwrap();
 let output=doctor();assert!(!output.status.success());let report:serde_json::Value=serde_json::from_slice(&output.stdout).unwrap();assert_eq!(report["installation"]["status"],"error");
 let output=support::run(Command::new(launcher()).args(["toolchain","update"]),20);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("package manager"));
}

#[test]fn verified_pair_switches_without_replacing_running_binaries_and_rolls_back(){
 let scratch=support::TempDirectory::new();let root=scratch.0.join("installation");let (first,a)=fixture(&scratch.0,"1.0.0");let (second,b)=fixture(&scratch.0,"1.1.0");
 toolchain_install::install(&root,&first,&a,&launcher()).unwrap();
 assert_eq!(run(&root,"tsrust"),"tsrust 1.0.0");assert_eq!(run(&root,"tsrust-lsp"),"tsrust-lsp 1.0.0");
 let log=scratch.0.join("running.log");
 let child=Command::new(root.join("bin").join(format!("tsrust-lsp{}",std::env::consts::EXE_SUFFIX))).arg("--hold").stdout(fs::File::create(&log).unwrap()).stderr(Stdio::null()).spawn().unwrap();
 struct Running(std::process::Child);impl Drop for Running{fn drop(&mut self){let _=self.0.kill();let _=self.0.wait();}}
 let mut running=Running(child);let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
 while !fs::read_to_string(&log).unwrap().contains("holding"){assert!(std::time::Instant::now()<deadline,"launcher did not start its child");assert!(running.0.try_wait().unwrap().is_none());std::thread::sleep(std::time::Duration::from_millis(10));}
 let lock=toolchain_install::lock(&root).unwrap();assert!(toolchain_install::lock(&root).is_err());toolchain_install::update(&root,&second,&b).unwrap();drop(lock);
 assert!(running.0.try_wait().unwrap().is_none(),"updating must not stop the old server");
 assert_eq!(run(&root,"tsrust"),"tsrust 1.1.0");assert_eq!(run(&root,"tsrust-lsp"),"tsrust-lsp 1.1.0");
 let lock=toolchain_install::lock(&root).unwrap();toolchain_install::rollback(&root).unwrap();drop(lock);
 assert_eq!(run(&root,"tsrust"),"tsrust 1.0.0");assert_eq!(run(&root,"tsrust-lsp"),"tsrust-lsp 1.0.0");
}
#[test]fn failed_and_interrupted_updates_preserve_or_reuse_the_verified_pair(){
 let scratch=support::TempDirectory::new();let root=scratch.0.join("installation");let (first,a)=fixture(&scratch.0,"1.0.0");let (second,b)=fixture(&scratch.0,"1.1.0");
 toolchain_install::install(&root,&first,&a,&launcher()).unwrap();let original=fs::read(root.join("installation.json")).unwrap();
 let lock=toolchain_install::lock(&root).unwrap();
 assert!(toolchain_install::update(&root,&second,b"corrupt").is_err());assert_eq!(fs::read(root.join("installation.json")).unwrap(),original);
 assert!(toolchain_install::update(&root,&first,&a).is_err());
 let binary=fs::read(scratch.0.join(format!("probe-1.1.0{}",std::env::consts::EXE_SUFFIX))).unwrap();let (wrong,wrong_bytes)=archive("1.2.0","1.2.0",binary.clone(),binary);
 assert!(toolchain_install::update(&root,&wrong,&wrong_bytes).unwrap_err().contains("manifest version"));assert_eq!(fs::read(root.join("installation.json")).unwrap(),original);
 let staged=toolchain_install::stage(&root,&second,&b).unwrap();
 fs::write(root.join(".installation.pending"),"interrupted record").unwrap();
 let orphan=root.join("versions/.stage-interrupted");fs::create_dir(&orphan).unwrap();fs::write(orphan.join(".managed-stage"),"tsrust-managed-launcher-1\n").unwrap();fs::write(orphan.join("partial"),"partial").unwrap();
 toolchain_install::update(&root,&second,&b).unwrap();assert!(!orphan.exists());assert!(!root.join(".installation.pending").exists());
 assert_eq!(toolchain_install::state(&root).unwrap().current.manifest_sha256,staged.manifest_sha256);drop(lock);assert_eq!(run(&root,"tsrust"),"tsrust 1.1.0");
}
#[test]fn installer_preserves_unmanaged_directories_and_refuses_modified_executables(){
 let scratch=support::TempDirectory::new();let root=scratch.0.join("owner");fs::create_dir(&root).unwrap();fs::write(root.join("owner.txt"),"keep").unwrap();
 let (release,bytes)=fixture(&scratch.0,"1.0.0");assert!(toolchain_install::install(&root,&release,&bytes,&launcher()).is_err());assert_eq!(fs::read_to_string(root.join("owner.txt")).unwrap(),"keep");assert!(toolchain_install::lock(&root).is_err());
 let root=scratch.0.join("managed");let state=toolchain_install::install(&root,&release,&bytes,&launcher()).unwrap();
 fs::write(root.join("versions").join(state.current.release.sha256).join("bin").join(format!("tsrust{}",std::env::consts::EXE_SUFFIX)),"tampered").unwrap();
 let output=support::run(Command::new(root.join("bin").join(format!("tsrust{}",std::env::consts::EXE_SUFFIX))).arg("--version"),20);
 assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("checksum"));
}
#[test]fn archive_links_are_rejected_before_execution(){
 let scratch=support::TempDirectory::new();let (mut release,_)=fixture(&scratch.0,"1.0.0");let prefix=release.name.strip_suffix(".tar.gz").unwrap();
 let mut archive=tar::Builder::new(flate2::write::GzEncoder::new(Vec::new(),flate2::Compression::fast()));
 let mut header=tar::Header::new_gnu();header.set_entry_type(tar::EntryType::Symlink);header.set_size(0);header.set_mode(0o777);header.set_cksum();archive.append_link(&mut header,format!("{prefix}/bin/tsrust"),"../../owner").unwrap();
 let bytes=archive.into_inner().unwrap().finish().unwrap();release.sha256=digest(&bytes);release.bytes=bytes.len() as u64;
 let output=scratch.0.join("bad");assert!(toolchain_release::unpack(&output,&release,&bytes).unwrap_err().contains("links"));assert!(!output.exists());
}
