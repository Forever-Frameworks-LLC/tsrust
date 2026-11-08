mod support;
use std::{fs,path::Path};
fn config(root:&Path){fs::write(root.join("words.bindings.json"),r#"{"schemaVersion":2,"source":"./words.rs","dependencies":{"shlex":{"version":"=2.0.1","features":[],"defaultFeatures":true}}}"#).unwrap();}
fn generate(root:&Path,source:&Path,names:&str)->std::process::Output{support::run(support::cli().args(["bindings","generate"]).arg(root.join("words.bindings.json")).args(["--crate","shlex","--source"]).arg(source).args(["--select",names]),30)}
#[test]
fn selected_generation_is_deterministic_and_preserves_customized_files(){
 let dir=support::TempDirectory::new();config(&dir.0);let source=dir.0.join("upstream.rs");fs::write(&source,"/// Split shell words.\npub fn split(value:&str)->Option<Vec<String>>{None}pub fn generic<T>(value:T)->T{value}").unwrap();
 support::success(&generate(&dir.0,&source,"split,generic"));
 let paths=["words.rs","words.api.tsr","words.stubs.rs","words.bindings.generation.json"];
 let original=paths.map(|path|fs::read(dir.0.join(path)).unwrap());
 support::success(&generate(&dir.0,&source,"generic,split"));for(index,path)in paths.iter().enumerate(){assert_eq!(fs::read(dir.0.join(path)).unwrap(),original[index],"{path}");}
 let report:serde_json::Value=serde_json::from_slice(&original[3]).unwrap();assert_eq!(report["selected"],serde_json::json!(["split"]));assert_eq!(report["skipped"][0]["name"],"generic");assert!(!dir.0.join("words.bindings.lock").exists(),"generation must not resolve dependencies");
 fs::write(dir.0.join("words.stubs.rs"),"pub fn adapt_generic(){let chosen=7;}\n").unwrap();support::success(&generate(&dir.0,&source,"split,generic"));assert_eq!(fs::read_to_string(dir.0.join("words.stubs.rs")).unwrap(),"pub fn adapt_generic(){let chosen=7;}\n");
 fs::write(dir.0.join("words.rs"),"pub fn custom()->i64{7}\n").unwrap();let custom=fs::read(dir.0.join("words.rs")).unwrap();let output=generate(&dir.0,&source,"split");assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("Customized adapter preserved"));assert_eq!(fs::read(dir.0.join("words.rs")).unwrap(),custom);assert_eq!(fs::read(dir.0.join("words.api.tsr")).unwrap(),original[1]);
}
fn shlex_source()->std::path::PathBuf{
 let home=std::env::var_os("CARGO_HOME").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".cargo"));
 let mut paths=fs::read_dir(home.join("registry/src")).unwrap().map(|entry|entry.unwrap().path().join("shlex-2.0.1/src/lib.rs")).filter(|path|path.is_file()).collect::<Vec<_>>();paths.sort();assert_eq!(paths.len(),1,"expected the single admitted cached shlex-2.0.1 source, no download");paths.remove(0)
}
#[test]
fn real_generated_shlex_moves_values_and_relocated_export_builds_offline(){
 let dir=support::TempDirectory::new();config(&dir.0);support::success(&generate(&dir.0,&shlex_source(),"split,try_join,try_quote"));
 let report:serde_json::Value=serde_json::from_str(&fs::read_to_string(dir.0.join("words.bindings.generation.json")).unwrap()).unwrap();assert_eq!(report["selected"],serde_json::json!(["split"]));assert_eq!(report["skipped"].as_array().unwrap().len(),2);
 let json=dir.0.join("shlex-rustdoc.json");fs::write(&json,include_str!("../src/project/fixtures/shlex-rustdoc-1.96.1.json")).unwrap();
 let generate_json=||support::run(support::cli().args(["bindings","generate"]).arg(dir.0.join("words.bindings.json")).args(["--crate","shlex","--source"]).arg(shlex_source()).args(["--select","split,try_join,try_quote","--rustdoc"]).arg(&json),30);
 support::success(&generate_json());let adapter=fs::read(dir.0.join("words.rs")).unwrap();let mut wrong:serde_json::Value=serde_json::from_str(include_str!("../src/project/fixtures/shlex-rustdoc-1.96.1.json")).unwrap();wrong["crate_version"]=serde_json::json!("0.0.0");fs::write(&json,wrong.to_string()).unwrap();let failed=generate_json();assert!(!failed.status.success());assert!(String::from_utf8_lossy(&failed.stderr).contains("exact binding dependency pin"));assert_eq!(fs::read(dir.0.join("words.rs")).unwrap(),adapter);

 // Export only copies a verified pack. Prepare the combined runtime+binding
 // source closure explicitly, offline, in the same inherited Cargo home/pack dir.
 support::success(&support::run(support::cli().args(["bindings","resolve"]).arg(dir.0.join("words.bindings.json")).args(["--offline","--portable"]),120));
 let source=r#"import {split} from './words.bindings.json';import {strictEqual} from 'std:testing';function main(){const valid=split("alpha 'two words' 世界");if(valid.kind==='some'){strictEqual(valid.value.length,3);strictEqual(valid.value[1],'two words');strictEqual(valid.value[2],'世界');}else{throw 'expected words';}strictEqual(split("'unfinished").kind,'none');const empty=split('');if(empty.kind==='some'){strictEqual(empty.value.length,0);}else{throw 'empty input is Some empty';}console.log('words preserved');}"#;
 let input=dir.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("run").arg(&input),120);support::success(&output);assert_eq!(output.stdout,b"words preserved\n");
 let package=dir.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&input).args(["--package","-o"]).arg(&package),120));
 fs::remove_file(&input).unwrap();
 for name in ["words.bindings.json","words.bindings.lock","words.rs"]{let path=dir.0.join(name);if path.exists(){fs::remove_file(path).unwrap();}}
 let relocated=dir.0.join("relocated");fs::rename(package,&relocated).unwrap();let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
 for release in [false,true]{let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",dir.0.join("empty-cargo-home")).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1");if release{command.arg("--release");}support::success(&support::run(&mut command,180));let binary=target.join(if release{"release"}else{"debug"}).join(format!("program{}",std::env::consts::EXE_SUFFIX));let output=support::run(std::process::Command::new(binary).current_dir(&relocated),30);support::success(&output);assert_eq!(output.stdout,b"words preserved\n");}
}

#[test]
fn generated_nominal_record_projections_move_nested_string_buffers_without_clone(){
 let backend=r#"
pub struct Row{pub words:Vec<String>,pub pointer:i64,pub capacity:i64}
pub fn seed()->Row{let mut value=String::with_capacity(128);value.push_str("retained");let pointer=value.as_ptr() as i64;let capacity=value.capacity() as i64;Row{words:vec![value],pointer,capacity}}
pub fn echo(value:Option<Vec<Row>>)->Option<Vec<Row>>{if let Some(rows)=&value{for row in rows{assert_eq!(row.words[0].as_ptr() as i64,row.pointer);assert_eq!(row.words[0].capacity() as i64,row.capacity);}}value}
"#;
 let generated=tsrust_compiler::project::rust::generation::source("backend",backend,&["echo".into(),"seed".into()]).unwrap();assert_eq!(generated.selected.len(),2);assert!(generated.skipped.is_empty());assert!(!generated.adapter.contains(".clone("));
 let dir=support::TempDirectory::new();fs::write(dir.0.join("backend.rs"),backend).unwrap();fs::write(dir.0.join("owned.rs"),format!("use backend;\n{}",generated.adapter)).unwrap();
 let source=r#"import {Row,RustAdapterOption,echo,seed} from './owned.rs';import {strictEqual} from 'std:testing';function main(){const input:RustAdapterOption<Row[]>={kind:'some',value:[seed()]};const output=echo(input);if(output.kind==='some'){strictEqual(output.value[0].words[0],'retained');}else{throw 'lost rows';}console.log('nominal moves');}"#;
 let entry=dir.source(&support::format_round_trip(source));let checked=tsrust_compiler::project::check_graph(tsrust_compiler::project::load_project(&tsrust_compiler::project::FileSourceProvider,entry.to_str().unwrap()).unwrap()).unwrap();let rust=checked.emit_rust();
 let emitted=support::run(support::cli().arg("emit").arg(&entry),30);support::success(&emitted);assert_eq!(String::from_utf8(emitted.stdout).unwrap(),rust);let path=dir.0.join("application.rs");fs::write(&path,&rust).unwrap();
 // The compiler already depends on this pinned Rayon. Reuse the same test
 // dependency directory, as support::rustc does for the other runtime crates.
 let executable=std::env::current_exe().unwrap();let deps=executable.parent().unwrap().to_path_buf();
 // The pinned Cargo test fingerprint identifies the actual direct dependency.
 let stem=executable.file_stem().unwrap().to_str().unwrap();let (_,unit)=stem.rsplit_once('-').unwrap();let fingerprints=deps.parent().unwrap().join(".fingerprint");
 let fingerprint:serde_json::Value=serde_json::from_str(&fs::read_to_string(fingerprints.join(format!("tsrust-compiler-{unit}/test-integration-test-rust_adapter_generation.json"))).unwrap()).unwrap();
 let wanted=fingerprint["deps"].as_array().unwrap().iter().find(|dependency|dependency[1]=="rayon").unwrap()[3].as_u64().unwrap();
 let candidates=fs::read_dir(&fingerprints).unwrap().map(|entry|entry.unwrap().path()).filter(|path|path.file_name().unwrap().to_string_lossy().starts_with("rayon-")).filter_map(|path|{
  let saved=fs::read_to_string(path.join("lib-rayon")).ok()?;let actual=u64::from_str_radix(saved.trim(),16).ok()?.swap_bytes();
  if actual!=wanted{return None;}let name=path.file_name().unwrap().to_str().unwrap();let library=deps.join(format!("lib{name}.rlib"));library.is_file().then_some(library)
 }).collect::<Vec<_>>();
 let [rayon]=candidates.as_slice()else{panic!("Expected one exact Rayon dependency from the selected Cargo build: {candidates:?}")};
 for optimization in ["0","3"]{
  let library=dir.0.join(format!("libbackend{optimization}.rlib"));support::success(&support::run(support::rustc().args(["--edition=2024","--crate-type=rlib","--crate-name=backend","-C"]).arg(format!("opt-level={optimization}")).args(["-C","overflow-checks=on"]).arg(dir.0.join("backend.rs")).arg("-o").arg(&library),60));
  let binary=dir.0.join(format!("nominal{optimization}{}",std::env::consts::EXE_SUFFIX));let compiled=support::run(support::rustc().args(["--edition=2024","-C"]).arg(format!("opt-level={optimization}")).args(["-C","overflow-checks=on","--extern"]).arg(format!("backend={}",library.display())).arg("--extern").arg(format!("rayon={}",rayon.display())).arg("-L").arg(format!("dependency={}",deps.display())).arg(&path).arg("-o").arg(&binary),60);support::success(&compiled);let output=support::run(&mut std::process::Command::new(binary),20);support::success(&output);assert_eq!(output.stdout,b"nominal moves\n");
 }

}
