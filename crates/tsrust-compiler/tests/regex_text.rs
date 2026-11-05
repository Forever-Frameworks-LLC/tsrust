mod support;
use tsrust_compiler::project::{FileSourceProvider,check_graph,load_project};
fn checked(directory:&support::TempDirectory,source:&str)->tsrust_compiler::project::ProjectAnalysis{
 let path=directory.source(&support::format_round_trip(source));check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap()
}
fn native(directory:&support::TempDirectory,source:&str,expected:&str){
 let path=directory.source(&support::format_round_trip(source));
 for release in [false,true]{let mut command=support::cli();command.arg("run").arg(&path);if release{command.arg("--release");}let result=support::run(&mut command,180);support::success(&result);assert_eq!(result.stdout,expected.as_bytes(),"release={release}");}
}
#[test]
fn regex_owners_snapshots_optional_options_and_stored_functions_execute(){
 let directory=support::TempDirectory::new();
 native(&directory,r#"
import {Regex,RegexOptions,RegexCaptures,compile as create,escape} from 'std:regex';
import {strictEqual} from 'std:testing';
function snapshot():RegexCaptures[]{const expression=unwrap(create('(é)(x)?'));return expression.capturesAll('aé éx');}
function options(enabled:boolean):RegexOptions|undefined{if(enabled){return {dfaSizeLimit:0};}return undefined;}
function read(value:ReadOnly<Regex>):boolean{return value.isMatch('é');}
function choose(flag:boolean):'a'|'b'{if(flag){return 'a';}return 'b';}
function main(){
 const saved=create;const expression=unwrap(saved('(é)(x)?',options(true)));
 const letters=unwrap(create('b'));const literal:'abc'='abc';strictEqual(literal.search(letters),1);
 strictEqual(choose(false).search(letters),0);strictEqual(letters.isMatch(choose(false)),true);strictEqual(escape(choose(false)),'b');const quote=escape;strictEqual(quote(choose(false)),'b');
 strictEqual(read(expression),true);strictEqual(expression.isMatch('no'),false);
 const found=expression.find('aé');if(found!==undefined){strictEqual(found.start,1);strictEqual(found.end,3);}else{throw 'missing';}
 strictEqual(expression.find('no')===undefined,true);
 const rows=snapshot();strictEqual(rows.length,2);strictEqual(rows[0].match.end,3);strictEqual(rows[0].groups[1]===undefined,true);
 const first=expression.captures('é');if(first!==undefined){strictEqual(first.groups.length,2);}else{throw 'missing captures';}
 strictEqual('aé'.search(expression),1);strictEqual('no'.search(expression),-1);
 strictEqual('é é'.matchAll(expression).length,2);const matched='é'.match(expression);if(matched!==undefined){strictEqual(matched.match.end,2);}else{throw 'missing match';}
 strictEqual(expression.findAll('é é').length,2);
 strictEqual(expression.replace('é éx','${2}:$1 $$'),':é $ éx');strictEqual(expression.replaceAll('é éx','${2}:$1 $$'),':é $ x:é $');
 const empty=unwrap(create(''));strictEqual(empty.findAll('é🦀').length,3);strictEqual(empty.split('é🦀').length,4);
 const separator=unwrap(create(','));strictEqual(separator.split(',a,,').length,4);
 strictEqual(unwrap(create(escape('a+b'))).isMatch('a+b'),true);
 strictEqual(unwrap(create('a',undefined)).isMatch('a'),true);strictEqual(unwrap(create('a',options(false))).isMatch('a'),true);
 let caught=false;try{unwrap(create('('));}catch(error){strictEqual(error.code,'Syntax');caught=true;}strictEqual(caught,true);
 caught=false;try{unwrap(saved('('));}catch(error){strictEqual(error.code,'Syntax');caught=true;}strictEqual(caught,true);
 console.log('regex complete');
}
"#,"regex complete\n");
}
#[test]
fn regex_catalog_examples_execute_at_o0_and_o3(){
 let directory=support::TempDirectory::new();let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let mut functions=String::new();let mut calls=String::new();let mut expected=String::new();let mut count=0;
 for family in ["std:regex","Regex","string"]{for entry in catalog[family].as_array().unwrap(){if family=="string"&&!matches!(entry["name"].as_str(),Some("match"|"matchAll"|"search")){continue;}
  for example in entry["examples"].as_array().unwrap(){let(_,body)=example["source"].as_str().unwrap().split_once("function main()").unwrap();functions.push_str(&format!("function regexExample{count}(){body}\n"));calls.push_str(&format!("regexExample{count}();"));expected.push_str(example["output"].as_str().unwrap());count+=1;}
 }}assert_eq!(count,18);
 native(&directory,&format!("import {{compile,escape}} from 'std:regex';{functions}function main(){{{calls}}}"),&expected);
}
#[test]
fn regex_feature_selection_follows_used_owners_and_calls(){
 let directory=support::TempDirectory::new();
 for name in ["compile","escape","__isMatch","__find","__captures","__split","__stringMatch"] {tsrust_compiler::native_bindings::lookup("std:regex",name).unwrap();}
 assert!(tsrust_compiler::native_types::declaration(tsrust_compiler::native_signatures::NativeTypeId::Regex).type_parameters.is_empty());
 for(source,used)in[
 ("import {compile,Regex} from 'std:regex';function main(){console.log('plain');}",false),
 ("import {RegexMatch} from 'std:regex';function main(){const value:RegexMatch={start:0,end:0};console.log(value.start);}",false),
 ("import {escape} from 'std:regex';function main(){console.log(escape('a+b'));}",true),
 ("import {Regex} from 'std:regex';export function probe(value:ReadOnly<Regex>):boolean{return value.isMatch('a');}function main(){}",true),
 ]{
  let(rust,features)=checked(&directory,source).emit_rust_with_native_features();assert_eq!(features,if used{vec!["regex"]}else{vec![]});assert_eq!(rust.contains("mod tsrust_native_regex"),used);
  assert!(!rust.contains("unsafe"));let units=tsrust_compiler::native_runtime::separate(&rust);assert_eq!(units.dependencies,used);
  if used{assert!(units.runtime.unwrap().contains("pub mod tsrust_native_regex"));assert!(!units.application.contains("mod tsrust_native_regex"));}
 }
 for manifest in [include_str!("../backend-dependencies/Cargo.toml"),include_str!("../native-dependencies/Cargo.toml")]{let doc=manifest.parse::<toml_edit::DocumentMut>().unwrap();assert_eq!(doc["dependencies"]["regex"]["version"].as_str(),Some("=1.13.1"));assert_eq!(doc["dependencies"]["regex"]["optional"].as_bool(),Some(true));assert_eq!(doc["features"]["regex"].as_array().unwrap().len(),1);}
}
#[test]
fn regex_hidden_intrinsics_and_opaque_construction_are_rejected(){
 let directory=support::TempDirectory::new();
 for source in ["import {__find} from 'std:regex';function main(){}","import {Regex} from 'std:regex';function main(){const value:Regex={};}"]{
  let path=directory.source(source);assert!(load_project(&FileSourceProvider,path.to_str().unwrap()).and_then(check_graph).is_err());
 }
}
#[test]
fn regex_package_relocates_and_builds_offline_without_unsafe(){
 let directory=support::TempDirectory::new();let source="import {compile} from 'std:regex';function main(){const value=unwrap(compile('(é)'));console.log(value.findAll('é é').length,'aé'.search(value));}";
 let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().current_dir(&directory.0).args(["setup","--portable","--offline"]),180));
 let package=directory.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&path).args(["--package","-o"]).arg(&package),60));
 std::fs::remove_file(path).unwrap();let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let manifest=std::fs::read_to_string(relocated.join("Cargo.toml")).unwrap();let doc=manifest.parse::<toml_edit::DocumentMut>().unwrap();assert!(doc["features"]["default"].as_array().unwrap().iter().any(|feature|feature.as_str()==Some("regex")));
 assert!(relocated.join("vendor/regex-1.13.1/LICENSE-MIT").is_file());let file=relocated.join("src/main.rs");let emitted=std::fs::read_to_string(&file).unwrap();assert!(!emitted.contains("unsafe"));std::fs::write(file,format!("#![forbid(unsafe_code)]\n{emitted}")).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));assert!(target.is_absolute());
 let home=directory.0.join("empty-home");std::fs::create_dir(&home).unwrap();
 for release in [false,true]{let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",&home).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1");if release{command.arg("--release");}support::success(&support::run(&mut command,180));
  let executable=target.join(if release{"release"}else{"debug"}).join(format!("program{}",std::env::consts::EXE_SUFFIX));let result=support::run(&mut std::process::Command::new(executable),30);support::success(&result);assert_eq!(result.stdout,b"2 1\n");
 }
}
