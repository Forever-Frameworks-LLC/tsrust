mod support;
use tsrust_compiler::project::{FileSourceProvider,check_graph,load_project};
fn checked(directory:&support::TempDirectory,source:&str)->tsrust_compiler::project::ProjectAnalysis{let path=directory.source(&support::format_round_trip(source));check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap()}
fn native(directory:&support::TempDirectory,source:&str,expected:&str){let path=directory.source(&support::format_round_trip(source));for release in[false,true]{let mut command=support::cli();command.arg("run").arg(&path);if release{command.arg("--release");}let result=support::run(&mut command,180);support::success(&result);assert_eq!(result.stdout,expected.as_bytes(),"release={release}");}}
#[test]fn owners_optional_options_union_transport_and_owned_snapshots_execute(){let directory=support::TempDirectory::new();native(&directory,r#"
import {Locale,Collator,SegmentRange,NumberFormat,NumberFormatOptions,parseLocale,formatLocale,createCollator,createSegmenter,createNumberFormat} from 'std:locale';
import {Decimal,parseDecimal} from 'std:decimal';import {strictEqual} from 'std:testing';
function snapshot():SegmentRange[]{const value=createSegmenter();const text='e\u0301🦀';return value.segment(text);}
function options(enabled:boolean):NumberFormatOptions|undefined{if(enabled){return{grouping:'never',fractionDigits:2};}return undefined;}
function render(value:int|Decimal,formatter:ReadOnly<NumberFormat>):string{return formatter.format(value);}
function compare(value:ReadOnly<Collator>):int{const left:'é'='é';return value.compare(left,'e\u0301');}
function main(){
 const saved=parseLocale;const locale=unwrap(saved('EN-us'));strictEqual(formatLocale(locale),'en-US');
 const constructor=createCollator;const collator=unwrap(constructor(locale,undefined));const method=(left:string,right:string):int=>collator.compare(left,right);strictEqual(method('a','b'),-1);strictEqual(compare(collator),0);strictEqual('é'.localeCompare('e\u0301',collator),0);
 const numeric=unwrap(parseLocale('en-u-kn-true'));const numbers=unwrap(createCollator(numeric));strictEqual(numbers.compare('2','10'),-1);
 const base=unwrap(createCollator(locale,{strength:'primary',caseLevel:false,alternateHandling:'nonIgnorable',maxVariable:'punctuation'}));strictEqual(base.compare('é','e'),0);
 const ranges=snapshot();strictEqual(ranges[0].start,0);strictEqual(ranges[0].end,3);strictEqual(ranges[1].start,3);strictEqual(ranges[1].end,7);const segmenter=createSegmenter();strictEqual(segmenter.segment('').length,0);
 const noFraction=unwrap(createNumberFormat(locale,{fractionDigits:0}));strictEqual(noFraction.format(1),'1');const named:NumberFormatOptions={grouping:'never',fractionDigits:2};const explicit=unwrap(createNumberFormat(locale,named));strictEqual(explicit.format(1),'1.00');
 const create=createNumberFormat;const formatter=unwrap(create(locale,options(true)));strictEqual(render(1,formatter),'1.00');const decimal=unwrap(parseDecimal('2.3450'));strictEqual(render(decimal,formatter),'2.34');
 const edge=unwrap(createNumberFormat(locale,{grouping:'never',fractionDigits:32768}));strictEqual(edge.format(1).length,32770);
 const plain=unwrap(createNumberFormat(locale));const normalized=unwrap(parseDecimal('12345678901234567890.1200'));strictEqual(plain.format(normalized),'12,345,678,901,234,567,890.12');
 const overridden=unwrap(parseLocale('en-u-nu-arab'));const mixed=unwrap(createNumberFormat(overridden));strictEqual(mixed.format(1234),'١,٢٣٤');
 try{const bad=unwrap(createNumberFormat(locale,{fractionDigits:32769}));console.log(bad.format(1));}catch(error){strictEqual(error.code,'Range');}
 try{const bad=unwrap(parseLocale('en_US'));console.log(formatLocale(bad));}catch(error){strictEqual(error.code,'Syntax');}
 console.log('locale owners pass');
}
"#,"locale owners pass\n");}
#[test]fn catalog_examples_execute(){let directory=support::TempDirectory::new();let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();let mut functions=String::new();let mut calls=String::new();let mut expected=String::new();let mut count=0;for family in["std:locale","Collator","Segmenter","NumberFormat","string"]{for entry in catalog[family].as_array().unwrap(){if family=="string"&&entry["name"]!="localeCompare"{continue;}for example in entry["examples"].as_array().unwrap(){let(_,body)=example["source"].as_str().unwrap().split_once("function main()").unwrap();functions.push_str(&format!("function localeExample{count}(){body}\n"));calls.push_str(&format!("localeExample{count}();"));expected.push_str(example["output"].as_str().unwrap());count+=1;}}}assert_eq!(count,17);native(&directory,&format!("import{{parseLocale,formatLocale,createCollator,createSegmenter,createNumberFormat}}from'std:locale';{functions}function main(){{{calls}}}"),&expected);}
#[test]fn opaque_owners_options_and_float_inputs_are_rejected(){for source in[
"import{Locale}from'std:locale';function main(){const value:Locale={};}",
"import{parseLocale,createNumberFormat}from'std:locale';function main(){const locale=unwrap(parseLocale('en'));const n=unwrap(createNumberFormat(locale,{currency:'USD'}));}",
"import{parseLocale,createNumberFormat}from'std:locale';function main(){const locale=unwrap(parseLocale('en'));const n=unwrap(createNumberFormat(locale,{grouping:'wrong'}));}",
"import{parseLocale,createNumberFormat}from'std:locale';function main(){const locale=unwrap(parseLocale('en'));const n=unwrap(createNumberFormat(locale));n.format(1.5);}",
"import{__compare}from'std:locale';function main(){}",
]{let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));assert!(check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).is_err(),"{source}");}}
#[test]fn locale_dependency_activation_is_optional(){let directory=support::TempDirectory::new();for(source,used)in[("import{parseLocale}from'std:locale';function main(){console.log('plain');}",false),("import{SegmentRange}from'std:locale';function main(){const r:SegmentRange={start:0,end:1};console.log(r.start);}",false),("import{parseLocale,formatLocale}from'std:locale';function main(){const l=unwrap(parseLocale('en'));console.log(formatLocale(l));}",true)]{let(rust,features)=checked(&directory,source).emit_rust_with_native_features();assert_eq!(features.contains(&"locale"),used);assert_eq!(rust.contains("mod tsrust_native_locale"),used);assert!(!rust.contains("unsafe"));}}

#[test]fn locale_package_relocates_and_builds_offline_without_unsafe(){
 let directory=support::TempDirectory::new();let source="import{parseLocale,formatLocale,createCollator,createSegmenter,createNumberFormat}from'std:locale';import{parseDecimal}from'std:decimal';function main(){const locale=unwrap(parseLocale('EN-us'));const collator=unwrap(createCollator(locale));const segmenter=createSegmenter();const number=unwrap(createNumberFormat(locale,{grouping:'never',fractionDigits:2}));const decimal=unwrap(parseDecimal('2.3450'));console.log(formatLocale(locale),collator.compare('a','b'),segmenter.segment('é🦀').length,number.format(decimal));}";
 let path=directory.source(&support::format_round_trip(source));support::success(&support::run(support::cli().current_dir(&directory.0).args(["setup","--portable","--offline"]),180));
 let package=directory.0.join("package");support::success(&support::run(support::cli().arg("emit").arg(&path).args(["--package","-o"]).arg(&package),60));std::fs::remove_file(path).unwrap();let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let manifest=std::fs::read_to_string(relocated.join("Cargo.toml")).unwrap();let doc=manifest.parse::<toml_edit::DocumentMut>().unwrap();assert!(doc["features"]["default"].as_array().unwrap().iter().any(|feature|feature.as_str()==Some("locale")));
 assert!(relocated.join("vendor/icu_collator-2.3.1/Cargo.toml").is_file());let file=relocated.join("src/main.rs");let emitted=std::fs::read_to_string(&file).unwrap();assert!(!emitted.contains("unsafe"));std::fs::write(file,format!("#![forbid(unsafe_code)]\n{emitted}")).unwrap();
 let target=std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));assert!(target.is_absolute());let home=directory.0.join("empty-home");std::fs::create_dir(&home).unwrap();
 for release in[false,true]{let mut command=std::process::Command::new("rustup");command.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked"]).current_dir(&relocated).env("CARGO_HOME",&home).env("CARGO_TARGET_DIR",&target).env("CARGO_INCREMENTAL","0").env("CARGO_BUILD_JOBS","1");if release{command.arg("--release");}support::success(&support::run(&mut command,180));let executable=target.join(if release{"release"}else{"debug"}).join(format!("program{}",std::env::consts::EXE_SUFFIX));let result=support::run(&mut std::process::Command::new(executable),30);support::success(&result);assert_eq!(result.stdout,b"en-US -1 2 2.34\n");}
}

#[test]fn local_formatter_suspension_executes_in_started_and_awaited_tasks(){
 let directory=support::TempDirectory::new();
 for source in[include_str!("fixtures/explicit-locale/local-formatter-awaited.tsr"),include_str!("fixtures/explicit-locale/formatter-scope-before-await.tsr")]{native(&directory,source,"1\n");}
 native(&directory,include_str!("fixtures/explicit-locale/local-formatter-started.tsr"),"1\n");
}

#[test]fn formatter_owner_moves_to_worker_and_back(){let directory=support::TempDirectory::new();native(&directory,r#"
import{NumberFormat,parseLocale,createNumberFormat}from'std:locale';import{spawn,join}from'std:threads';
function main(){const locale=unwrap(parseLocale('en'));const value=unwrap(createNumberFormat(locale,{fractionDigits:2}));const worker=unwrap(spawn(():string=>value.format(1)));console.log(unwrap(join(worker)));const builder=unwrap(spawn(():NumberFormat=>{const en=unwrap(parseLocale('en'));return unwrap(createNumberFormat(en,{fractionDigits:2}));}));const returned=unwrap(join(builder));console.log(returned.format(2));}
"#,"1.00\n2.00\n");}
