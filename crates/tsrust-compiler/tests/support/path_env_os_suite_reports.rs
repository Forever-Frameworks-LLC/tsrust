//! Exact catalog-source and private-output correspondence for the stable suite.
use super::support;
pub const CASES:&[(&str,&str,&str)]=&[
    ("env-envvar-0",include_str!("../fixtures/path-env-os-runtime/catalog/env-envvar-0.tsr"),"APP_MODE 0\n"),
    ("os-oserror-1",include_str!("../fixtures/path-env-os-runtime/catalog/os-oserror-1.tsr"),"Io\n"),
    ("path-patherror-2",include_str!("../fixtures/path-env-os-runtime/catalog/path-patherror-2.tsr"),"Io\n"),
    ("path-pathformatoptions-3",include_str!("../fixtures/path-env-os-runtime/catalog/path-pathformatoptions-3.tsr"),"notes.txt\n"),
    ("os-availableparallelism-4",include_str!("../fixtures/path-env-os-runtime/catalog/os-availableparallelism-4.tsr"),"OS observation completed\n"),
    ("env-currentdir-5",include_str!("../fixtures/path-env-os-runtime/catalog/env-currentdir-5.tsr"),"directory observed\n"),
    ("env-currentexe-6",include_str!("../fixtures/path-env-os-runtime/catalog/env-currentexe-6.tsr"),"executable observed\n"),
    ("path-delimiter-7",include_str!("../fixtures/path-env-os-runtime/catalog/path-delimiter-7.tsr"),"true\n"),
    ("env-homedir-8",include_str!("../fixtures/path-env-os-runtime/catalog/env-homedir-8.tsr"),"home directory observed\n"),
    ("os-hostname-9",include_str!("../fixtures/path-env-os-runtime/catalog/os-hostname-9.tsr"),"OS observation completed\n"),
    ("path-pathparts-10",include_str!("../fixtures/path-env-os-runtime/catalog/path-pathparts-10.tsr"),"  file.tar.gz .gz file.tar\n"),
    ("path-resolve-11",include_str!("../fixtures/path-env-os-runtime/catalog/path-resolve-11.tsr"),"true\n"),
    ("path-separator-12",include_str!("../fixtures/path-env-os-runtime/catalog/path-separator-12.tsr"),"true\n"),
    ("env-tempdir-13",include_str!("../fixtures/path-env-os-runtime/catalog/env-tempdir-13.tsr"),"temporary directory observed\n"),
    ("env-vars-14",include_str!("../fixtures/path-env-os-runtime/catalog/env-vars-14.tsr"),"snapshot observed\n"),
];
pub fn assert_registered(source:&str,expected:&str){
 let (original,exported)=if source.contains("async function main("){("async function main(","export async function run(")}else{("function main(","export function run(")};
 assert_eq!(source.matches(original).count(),1);
 let source=support::format_round_trip(&source.replacen(original,exported,1));
 assert!(CASES.iter().any(|(_,registered,output)|support::format_round_trip(registered)==source && *output==expected),"Original path/environment/OS catalog program or exact output lost from registered suite");
}
pub fn execute(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
 let suite=root.join("crates/tsrust-compiler/tests/fixtures/path-env-os-runtime/catalog.test.tsr");
 let mut expected_suite=String::from("import {describe,test} from 'std:testing';\n");
 for(index,(name,registered,_))in CASES.iter().enumerate(){
  expected_suite.push_str(&format!("import {{run as case{index}}} from './catalog/{name}.tsr';\n"));
  let module=suite.parent().unwrap().join("catalog").join(format!("{name}.tsr"));
  assert_eq!(support::format_round_trip(&std::fs::read_to_string(module).unwrap()),support::format_round_trip(registered),"registered module source changed: {name}");
 }
 expected_suite.push_str("\ndescribe('path environment OS catalog',()=>{\n");
 for(index,(name,_,_))in CASES.iter().enumerate(){expected_suite.push_str(&format!("test('{name}',case{index});\n"));}
 expected_suite.push_str("});\n");
 let actual_suite=std::fs::read_to_string(&suite).unwrap();
 assert_eq!(support::format_round_trip(&actual_suite),actual_suite);
 assert_eq!(support::format_round_trip(&expected_suite),actual_suite,"suite imports/registrations must match exact catalog module identities");
 for release in [false]{
  let mut command=support::cli();command.current_dir(&root).arg("test").arg(&suite).args(["--reporter=json","--profile"]);if release{command.arg("--release");}
  let output=support::run(&mut command,600);support::success(&output);assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));
  let text=String::from_utf8(output.stdout).unwrap();print!("{text}");let rows=text.lines().map(|line|serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();
  for row in &rows{match row["event"].as_str().expect("event name"){
   "test"|"output"|"build"|"file"|"summary"=>{},
   other=>panic!("unexpected reporter event: {other}: {row}"),
  }}
  assert_eq!(rows.last().unwrap()["event"],"summary");
  let expected=CASES.iter().map(|(name,_,_)|format!("path environment OS catalog > {name}")).collect::<std::collections::BTreeSet<_>>();assert_eq!(expected.len(),CASES.len());
  let tests=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();assert_eq!(tests.len(),expected.len());
  let actual=tests.iter().map(|row|row["name"].as_str().unwrap().to_owned()).collect::<std::collections::BTreeSet<_>>();assert_eq!(actual,expected);
  for row in rows.iter().filter(|row|row["event"]=="output"){assert!(expected.contains(row["name"].as_str().unwrap()));assert_eq!(row["stream"],"stdout");}
  for(name,_,stdout)in CASES{let name=format!("path environment OS catalog > {name}");let test=tests.iter().find(|row|row["name"]==name).unwrap();assert_eq!(test["status"],"passed","{name}");let actual=rows.iter().filter(|row|row["event"]=="output"&&row["name"]==name).map(|row|row["text"].as_str().unwrap()).collect::<String>();assert_eq!(actual,*stdout,"{name}");}
  let summary=rows.iter().filter(|row|row["event"]=="summary").collect::<Vec<_>>();assert_eq!(summary.len(),1);assert_eq!(summary[0]["passed"],CASES.len());for key in ["failed","skipped","hookFailures","filesFailed"]{assert_eq!(summary[0][key],0);}assert_eq!(summary[0]["filesPassed"],1);
  let builds=rows.iter().filter(|row|row["event"]=="build").collect::<Vec<_>>();assert_eq!(builds.len(),1);assert_eq!(builds[0]["testFiles"],1);
 }
}
