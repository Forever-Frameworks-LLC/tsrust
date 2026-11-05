mod support;
use tsrust_compiler::{native_runtime,project::{FileSourceProvider,check_graph,load_project}};
#[test]
fn only_used_search_requests_memchr_while_std_operations_stay_dependency_free(){
 let directory=support::TempDirectory::new();
 for (source,search) in [
  ("import {indexOf} from 'std:bytes';function main(){console.log('unused');}",false),
  ("import {Bytes,equals,compare} from 'std:bytes';function same(left:ReadOnly<Bytes>,right:ReadOnly<Bytes>):boolean{return equals(left,right)&&compare(left,right)===0;}function main(){}",false),
  ("import {alloc,writeInt,readInt} from 'std:bytes';function main(){const bytes=unwrap(alloc(4));unwrap(writeInt(bytes,1,-2,2,'big'));console.log(unwrap(readInt(bytes,1,2,'big')));}",false),
  ("import {fromArray,concat,compare} from 'std:bytes';function main(){const bytes=unwrap(fromArray([0]));console.log(compare(unwrap(concat(bytes,bytes)),bytes));}",false),
  ("import {fromArray,indexOf as find} from 'std:bytes';function main(){const bytes=unwrap(fromArray([0,255]));const saved=find;console.log(unwrap(saved(bytes,bytes,0)));}",true),
 ]{
  let path=directory.source(&support::format_round_trip(source));let program=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap();let emitted=program.emit_rust();
  assert!(!emitted.contains("unsafe"));assert_eq!(native_runtime::separate(&emitted).dependencies,search,"{source}");assert_eq!(native_runtime::requires_byte_search(&emitted),search);assert!(!native_runtime::requires_byte_codecs(&emitted));
 }
}
#[test]
fn combined_codec_search_packet_export_builds_offline_after_relocation_at_o0_and_o3(){
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip("import {alloc,fromUtf8,toBase64,fromUtf16,toUtf16,indexOf,writeInt,readInt} from 'std:bytes';function main(){const text=unwrap(fromUtf8('f'));const packet=unwrap(alloc(4));unwrap(writeInt(packet,1,-2,2,'big'));console.log(unwrap(toBase64(text)),unwrap(toUtf16(unwrap(fromUtf16('A😀','big')),'big')),unwrap(indexOf(text,text,0)),unwrap(readInt(packet,1,2,'big')));}"));
 support::success(&support::run(support::cli().current_dir(&directory.0).args(["setup","--portable","--offline"]),180));let package=directory.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&path).args(["--package","-o"]).arg(&package),60));std::fs::remove_file(path).unwrap();let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 assert!(relocated.join("vendor/memchr-2.8.3/LICENSE-MIT").is_file());assert!(relocated.join("vendor/base64-0.23.1/LICENSE-MIT").is_file());let manifest=std::fs::read_to_string(relocated.join("Cargo.toml")).unwrap();let defaults=manifest.lines().find(|line|line.starts_with("default = [")).unwrap();assert!(defaults.contains("byte-search")&&defaults.contains("byte-codecs"));
 let source=relocated.join("src/main.rs");let emitted=std::fs::read_to_string(&source).unwrap();assert!(!emitted.contains("unsafe"));std::fs::write(source,format!("#![forbid(unsafe_code)]\n{emitted}")).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));assert!(target.is_absolute());
 for release in [false,true]{let mut build=std::process::Command::new("rustup");build.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",directory.0.join("empty-cargo-home")).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1");if release{build.arg("--release");}support::success(&support::run(&mut build,180));let executable=target.join(if release{"release"}else{"debug"}).join(format!("program{}",std::env::consts::EXE_SUFFIX));let output=support::run(std::process::Command::new(executable).current_dir(&relocated),30);support::success(&output);assert_eq!(output.stdout,"Zg== A😀 0 -2\n".as_bytes());}
}
