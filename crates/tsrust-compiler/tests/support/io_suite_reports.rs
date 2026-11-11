//! Exact authored I/O programs and registered runner output; no execution receipts.
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
const REFERENCES:&[(&str,usize,Option<(&str,&str)>)]=&[
 ("reader",0,Some(("async io runtime > catalog-00",include_str!("../fixtures/async-io/catalog-00.tsr")))),
 ("writer",0,Some(("async io runtime > catalog-01",include_str!("../fixtures/async-io/catalog-01.tsr")))),
 ("stdout",0,None),
 ("stderr",0,None),
 ("stdin",0,Some(("async io runtime > catalog-04",include_str!("../fixtures/async-io/catalog-04.tsr")))),
 ("read",0,Some(("async io runtime > catalog-05",include_str!("../fixtures/async-io/catalog-05.tsr")))),
 ("read",1,Some(("async io runtime > catalog-06",include_str!("../fixtures/async-io/catalog-06.tsr")))),
 ("read",2,Some(("async io runtime > catalog-07",include_str!("../fixtures/async-io/catalog-07.tsr")))),
 ("readInto",0,Some(("async io runtime > catalog-08",include_str!("../fixtures/async-io/catalog-08.tsr")))),
 ("readInto",1,Some(("async io runtime > catalog-06",include_str!("../fixtures/async-io/catalog-06.tsr")))),
 ("readInto",2,Some(("async io runtime > catalog-07",include_str!("../fixtures/async-io/catalog-07.tsr")))),
 ("readExact",0,Some(("async io runtime > catalog-09",include_str!("../fixtures/async-io/catalog-09.tsr")))),
 ("readExact",1,Some(("async io runtime > catalog-06",include_str!("../fixtures/async-io/catalog-06.tsr")))),
 ("readExact",2,Some(("async io runtime > catalog-07",include_str!("../fixtures/async-io/catalog-07.tsr")))),
 ("readAll",0,Some(("async io runtime > catalog-10",include_str!("../fixtures/async-io/catalog-10.tsr")))),
 ("readAll",1,Some(("async io runtime > catalog-06",include_str!("../fixtures/async-io/catalog-06.tsr")))),
 ("readAll",2,Some(("async io runtime > catalog-07",include_str!("../fixtures/async-io/catalog-07.tsr")))),
 ("write",0,Some(("async io runtime > catalog-11",include_str!("../fixtures/async-io/catalog-11.tsr")))),
 ("write",1,Some(("async io runtime > catalog-06",include_str!("../fixtures/async-io/catalog-06.tsr")))),
 ("write",2,Some(("async io runtime > catalog-07",include_str!("../fixtures/async-io/catalog-07.tsr")))),
 ("writeAll",0,Some(("async io runtime > catalog-01",include_str!("../fixtures/async-io/catalog-01.tsr")))),
 ("writeAll",1,Some(("async io runtime > catalog-06",include_str!("../fixtures/async-io/catalog-06.tsr")))),
 ("writeAll",2,Some(("async io runtime > catalog-07",include_str!("../fixtures/async-io/catalog-07.tsr")))),
 ("flush",0,Some(("async io runtime > catalog-01",include_str!("../fixtures/async-io/catalog-01.tsr")))),
 ("flush",1,Some(("async io runtime > catalog-06",include_str!("../fixtures/async-io/catalog-06.tsr")))),
 ("flush",2,Some(("async io runtime > catalog-07",include_str!("../fixtures/async-io/catalog-07.tsr")))),
 ("copy",0,Some(("async io runtime > catalog-12",include_str!("../fixtures/async-io/catalog-12.tsr")))),
 ("copy",1,Some(("async io runtime > catalog-06",include_str!("../fixtures/async-io/catalog-06.tsr")))),
 ("copy",2,Some(("async io runtime > catalog-07",include_str!("../fixtures/async-io/catalog-07.tsr")))),
 ("lines",0,Some(("async io runtime > catalog-00",include_str!("../fixtures/async-io/catalog-00.tsr")))),
 ("readLine",0,Some(("async io runtime > catalog-00",include_str!("../fixtures/async-io/catalog-00.tsr")))),
 ("close",0,Some(("async io runtime > catalog-00",include_str!("../fixtures/async-io/catalog-00.tsr")))),
 ("close",1,Some(("async io runtime > catalog-07",include_str!("../fixtures/async-io/catalog-07.tsr")))),
 ("readStdin",0,Some(("async io runtime > catalog-04",include_str!("../fixtures/async-io/catalog-04.tsr")))),
 ("readLineStdin",0,Some(("async io runtime > catalog-13",include_str!("../fixtures/async-io/catalog-13.tsr")))),
 ("body",0,Some(("async io runtime > catalog-14",include_str!("../fixtures/async-io/catalog-14.tsr")))),
 ("Reader",0,Some(("async io runtime > catalog-00",include_str!("../fixtures/async-io/catalog-00.tsr")))),
 ("Writer",0,Some(("async io runtime > catalog-01",include_str!("../fixtures/async-io/catalog-01.tsr")))),
 ("LineReader",0,Some(("async io runtime > catalog-00",include_str!("../fixtures/async-io/catalog-00.tsr")))),
 ("Stdin",0,Some(("async io runtime > catalog-04",include_str!("../fixtures/async-io/catalog-04.tsr")))),
 ("IoError",0,Some(("async io runtime > catalog-15",include_str!("../fixtures/async-io/catalog-15.tsr")))),
];
const REGRESSIONS:&[(&str,&str,&str,&str)]=&[
 ("consuming_buffered_adapters_preserves_unread_and_pending_bytes","async io runtime > buffered-adapters","fi\nrst-last\n",include_str!("../fixtures/async-io/buffered-adapters.tsr")),
 ("file_to_http_body_to_file_streams_without_whole_body_accumulation","async io runtime > http-files","20\nsplit-é-lines\r\nlast\n",include_str!("../fixtures/async-io/http-files.tsr")),
 ("started_io_tasks_reject_caller_loans_and_owned_readers_transfer","async io runtime > started-reader","3\n",include_str!("../fixtures/async-io/started-reader.tsr")),
 ("consumed_io_task_relays_preserve_outer_and_invariant_view_lifetimes_at_o0_and_o3","async io runtime > task-relay","3\nxbc\n",include_str!("../fixtures/async-io/task-relay.tsr")),
 ];
fn catalog()->Value{serde_json::from_str(include_str!("../../../tsrust-lsp/src/builtins.json")).unwrap()}
fn example<'a>(catalog:&'a Value,name:&str,index:usize)->&'a Value{&catalog["std:io"].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap()["examples"][index]}
// Compare exported forms: export can affect formatter separators. Token offsets
// ensure comments and quoted function spellings are never rewritten.
fn exported_main(source:&str)->String{super::support::exported_main(source)}
fn same(source:&str,fixture:&str){assert_eq!(super::support::format_round_trip(&exported_main(source)),super::support::format_round_trip(fixture));}
pub fn assert_stored_stdio_source(source:&str){same(source,include_str!("../fixtures/async-io/stored-operations.tsr"));}
pub fn assert_registered_source(owner:&str,source:&str){
 if owner=="memory_reader_example_executes_with_explicit_results"{same(source,include_str!("../fixtures/async-io/catalog-00.tsr"));return;}
 let matches=REGRESSIONS.iter().filter(|(name,_,_,_)|*name==owner).collect::<Vec<_>>();assert_eq!(matches.len(),1);same(source,matches[0].3);
}
pub fn assert_sources(){
 let mut modules=Vec::new();let mut seen=BTreeSet::new();
 for &(_,_,registered)in REFERENCES{if let Some((case,source))=registered{if seen.insert(case){modules.push((case,source));}}}
 for &(_,case,_,source)in REGRESSIONS{assert!(seen.insert(case));modules.push((case,source));}
 assert_eq!(modules.len(),18);
 let mut suite=String::from("import {describe,test} from 'std:testing';\n");
 for(index,(case,_))in modules.iter().enumerate(){let name=case.strip_prefix("async io runtime > ").unwrap();suite.push_str(&format!("import {{main as example{index}}} from './{name}.tsr';\n"));}
 suite.push_str("describe('async io runtime',()=>{\n");
 for(index,(case,source))in modules.iter().enumerate(){let name=case.strip_prefix("async io runtime > ").unwrap();let asynchronous=source.contains("export async function main(");suite.push_str(&format!("test('{name}',{}()=>{{{}example{index}();}});\n",if asynchronous{"async "}else{""},if asynchronous{"await "}else{""}));}
 suite.push_str("});\n");
 let tokens=|source:&str|tsrust_compiler::source_tokens(&super::support::format_round_trip(source)).unwrap().into_iter().map(|(_,text)|text).collect::<Vec<_>>();
 assert_eq!(tokens(&suite),tokens(include_str!("../fixtures/async-io/runtime.test.tsr")),"exact suite imports/registrations changed");
 let catalog=catalog();let actual=catalog["std:io"].as_array().unwrap().iter().flat_map(|entry|entry["examples"].as_array().unwrap().iter().enumerate().map(move |(index,_)|(entry["name"].as_str().unwrap(),index))).collect::<BTreeSet<_>>();
 assert_eq!(actual,REFERENCES.iter().map(|(name,index,_)|(*name,*index)).collect());assert_eq!(actual.len(),41);
 let mut unique=BTreeSet::new();for &(name,index,registered)in REFERENCES{let value=example(&catalog,name,index);unique.insert(value["source"].as_str().unwrap());if let Some((_,fixture))=registered{same(value["source"].as_str().unwrap(),fixture);}else{assert!(matches!(name,"stdout"|"stderr"));}}
 assert_eq!(example(&catalog,"reader",0)["output"],"2 he\nll o\na  last\ntrue\n");
 assert_eq!(unique.len(),16);same(include_str!("../../../../examples/async-io/main.tsr"),include_str!("../fixtures/async-io/catalog-00.tsr"));
 assert_registered_source("file_to_http_body_to_file_streams_without_whole_body_accumulation",include_str!("../../../../examples/async-io/http-files.tsr"));
}
pub fn assert_report(output:std::process::Output){
 super::support::success(&output);assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));let text=String::from_utf8(output.stdout).unwrap();print!("{text}");let rows=text.lines().map(|line|serde_json::from_str::<Value>(line).unwrap_or_else(|error|panic!("invalid I/O reporter JSON: {error}\nfull stdout:\n{text}"))).collect::<Vec<_>>();
 for row in &rows{match row["event"].as_str().expect("event name"){"test"|"output"|"build"|"file"|"summary"=>{},other=>panic!("unexpected reporter event: {other}: {row}"),}}
 let catalog=catalog();let mut expected=BTreeMap::new();for &(name,index,registered)in REFERENCES{if let Some((case,_))=registered{let value=example(&catalog,name,index);assert!(value.get("stderr").and_then(Value::as_str).unwrap_or("").is_empty());let output=value["output"].as_str().unwrap();if let Some(previous)=expected.insert(case,output){assert_eq!(previous,output);}}}
 for &(_,case,output,_)in REGRESSIONS{assert!(expected.insert(case,output).is_none());}assert_eq!(expected.len(),18);
 let tests=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();assert_eq!(tests.len(),18);assert_eq!(tests.iter().map(|row|row["name"].as_str().unwrap()).collect::<BTreeSet<_>>(),expected.keys().copied().collect());
 for row in tests{assert_eq!(row["status"],"passed","{row}");let duration=row["durationMs"].as_f64().unwrap();assert!(duration.is_finite()&&duration>=0.0);}
 for row in rows.iter().filter(|row|row["event"]=="output"){assert_eq!(row["stream"],"stdout");assert!(expected.contains_key(row["name"].as_str().unwrap()));}
 for(case,output)in expected{let actual=rows.iter().filter(|row|row["event"]=="output"&&row["name"]==case).map(|row|row["text"].as_str().unwrap()).collect::<String>();assert_eq!(actual,output,"{case}");}
 let builds=rows.iter().filter(|row|row["event"]=="build").collect::<Vec<_>>();assert_eq!(builds.len(),1);assert_eq!(builds[0]["testFiles"],1);for field in ["durationMs","checkMs","emitMs","backendMs","cacheMs","nativeBuildMs"]{let duration=builds[0][field].as_f64().unwrap();assert!(duration.is_finite()&&duration>=0.0);}
 let summaries=rows.iter().filter(|row|row["event"]=="summary").collect::<Vec<_>>();assert_eq!(summaries.len(),1);assert_eq!(rows.last().unwrap()["event"],"summary");for(field,value)in[("passed",18),("failed",0),("hookFailures",0),("skipped",0),("filesPassed",1),("filesFailed",0)]{assert_eq!(summaries[0][field],value);}
}
