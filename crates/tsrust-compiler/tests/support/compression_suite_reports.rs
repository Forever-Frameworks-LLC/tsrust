//! Exact public compression programs and independent registered output assertions.
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
const REFERENCES:&[(&str,&str,usize,&str,&str)]=&[
 ("CompressionOptions","",0,"compression runtime > catalog-00",include_str!("../fixtures/compression/catalog-00.tsr")),
 ("CompressionOptions","level",0,"compression runtime > catalog-00",include_str!("../fixtures/compression/catalog-00.tsr")),
 ("DecompressionOptions","",0,"compression runtime > catalog-01",include_str!("../fixtures/compression/catalog-01.tsr")),
 ("DecompressionOptions","maxOutputBytes",0,"compression runtime > catalog-01",include_str!("../fixtures/compression/catalog-01.tsr")),
 ("CompressionError","",0,"compression runtime > catalog-02",include_str!("../fixtures/compression/catalog-02.tsr")),
 ("CompressionError","code",0,"compression runtime > catalog-02",include_str!("../fixtures/compression/catalog-02.tsr")),
 ("CompressionError","operation",0,"compression runtime > catalog-02",include_str!("../fixtures/compression/catalog-02.tsr")),
 ("CompressionError","message",0,"compression runtime > catalog-02",include_str!("../fixtures/compression/catalog-02.tsr")),
 ("gzip","",0,"compression runtime > catalog-03",include_str!("../fixtures/compression/catalog-03.tsr")),
 ("gunzip","",0,"compression runtime > catalog-03",include_str!("../fixtures/compression/catalog-03.tsr")),
 ("deflate","",0,"compression runtime > catalog-04",include_str!("../fixtures/compression/catalog-04.tsr")),
 ("inflate","",0,"compression runtime > catalog-04",include_str!("../fixtures/compression/catalog-04.tsr")),
 ("deflateRaw","",0,"compression runtime > catalog-05",include_str!("../fixtures/compression/catalog-05.tsr")),
 ("inflateRaw","",0,"compression runtime > catalog-05",include_str!("../fixtures/compression/catalog-05.tsr")),
 ("CompressionFormat","",0,"compression runtime > catalog-06",include_str!("../fixtures/compression/catalog-06.tsr")),
 ("CompressionReader","",0,"compression runtime > catalog-07",include_str!("../fixtures/compression/catalog-07.tsr")),
 ("CompressionWriter","",0,"compression runtime > catalog-08",include_str!("../fixtures/compression/catalog-08.tsr")),
 ("compressReader","",0,"compression runtime > catalog-07",include_str!("../fixtures/compression/catalog-07.tsr")),
 ("decompressReader","",0,"compression runtime > catalog-07",include_str!("../fixtures/compression/catalog-07.tsr")),
 ("compressWriter","",0,"compression runtime > catalog-08",include_str!("../fixtures/compression/catalog-08.tsr")),
 ("decompressWriter","",0,"compression runtime > catalog-08",include_str!("../fixtures/compression/catalog-08.tsr")),
];
const REGRESSIONS:&[(&str,&str,&str,&str)]=&[
 ("six_buffer_operations_preserve_results_and_execute","compression runtime > buffers","hello compression\nzlib\nraw\n",include_str!("../fixtures/compression/buffers.tsr")),
 ("typed_stream_examples_execute_with_one_pool_and_explicit_finish","compression runtime > readers","0\nhello\n0 0\n",include_str!("../fixtures/compression/readers.tsr")),
 ("typed_stream_examples_execute_with_one_pool_and_explicit_finish","compression runtime > writers","2\nhello\n",include_str!("../fixtures/compression/writers.tsr")),
 ("typed_stream_examples_execute_with_one_pool_and_explicit_finish","compression runtime > pipe","17\none pool pipeline\n",include_str!("../fixtures/compression/pipe.tsr")),
 ("all_b05_copy_endpoint_pairs_keep_explicit_finish_and_error_types","compression runtime > copy-endpoints","4\n4\ncopy\n",include_str!("../fixtures/compression/copy-endpoints.tsr")),
 ("tcp_and_compression_copy_preserve_typed_errors_and_execute","compression runtime > tcp-pipeline","7\n5\nreply request\n",include_str!("../fixtures/compression/tcp-pipeline.tsr")),
 ("stream_constructor_reads_format_descriptor_without_consuming_it","compression runtime > format-descriptor","descriptor\n",include_str!("../fixtures/compression/format-descriptor.tsr")),
];
fn exported_main(source:&str)->String{super::support::exported_main(source)}
fn same(source:&str,fixture:&str){assert_eq!(super::support::format_round_trip(&exported_main(source)),super::support::format_round_trip(fixture));}
fn catalog()->Value{serde_json::from_str(include_str!("../../../tsrust-lsp/src/builtins.json")).unwrap()}
fn example<'a>(catalog:&'a Value,name:&str,field:&str,index:usize)->&'a Value{let entry=catalog["std:compression"].as_array().unwrap().iter().find(|entry|entry["name"]==name).unwrap();if field.is_empty(){&entry["examples"][index]}else{&entry["fields"][field]["examples"][index]}}
pub fn assert_registered_source(owner:&str,source:&str)->&'static str{let candidates=REGRESSIONS.iter().filter(|(name,_,_,_)|*name==owner).collect::<Vec<_>>();assert!(!candidates.is_empty(),"missing {owner}");let exported=super::support::format_round_trip(&exported_main(source));let selected=candidates.iter().find(|(_,_,_,fixture)|super::support::format_round_trip(fixture)==exported).unwrap_or_else(||panic!("authored source changed: {owner}"));selected.2}
pub fn assert_sources(){
 let catalog=catalog();let mut actual=BTreeSet::new();for entry in catalog["std:compression"].as_array().unwrap(){let name=entry["name"].as_str().unwrap();for(index,_)in entry["examples"].as_array().into_iter().flatten().enumerate(){actual.insert((name,"",index));}if let Some(fields)=entry["fields"].as_object(){for(field,value)in fields{for(index,_)in value["examples"].as_array().into_iter().flatten().enumerate(){actual.insert((name,field.as_str(),index));}}}}
 assert_eq!(actual,REFERENCES.iter().map(|(name,field,index,_,_)|(*name,*field,*index)).collect());assert_eq!(actual.len(),21);
 let mut unique=BTreeSet::new();for &(name,field,index,_,fixture)in REFERENCES{let value=example(&catalog,name,field,index);let raw=value["source"].as_str().unwrap();same(raw,fixture);unique.insert(raw);}assert_eq!(unique.len(),9);
 assert_registered_source("six_buffer_operations_preserve_results_and_execute",include_str!("../../../../examples/compression/buffers.tsr"));
 assert_registered_source("typed_stream_examples_execute_with_one_pool_and_explicit_finish",include_str!("../../../../examples/compression/readers.tsr"));
 assert_registered_source("typed_stream_examples_execute_with_one_pool_and_explicit_finish",include_str!("../../../../examples/compression/writers.tsr"));
 assert_registered_source("typed_stream_examples_execute_with_one_pool_and_explicit_finish",include_str!("../../../../examples/compression/pipe.tsr"));
 assert_registered_source("tcp_and_compression_copy_preserve_typed_errors_and_execute",include_str!("../../../../examples/compression/tcp-pipeline.tsr"));
}
pub fn assert_report(output:std::process::Output){
 super::support::success(&output);assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));let text=String::from_utf8(output.stdout).unwrap();print!("{text}");let rows=text.lines().map(|line|serde_json::from_str::<Value>(line).unwrap()).collect::<Vec<_>>();
 let catalog=catalog();let mut expected=BTreeMap::new();for &(name,field,index,case,_)in REFERENCES{let value=example(&catalog,name,field,index);assert!(value.get("stderr").and_then(Value::as_str).unwrap_or("").is_empty());let output=value["output"].as_str().unwrap();if let Some(previous)=expected.insert(case,output){assert_eq!(previous,output);}}
 for &(_,case,output,_)in REGRESSIONS{assert!(expected.insert(case,output).is_none());}assert_eq!(expected.len(),16);
 let tests=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();assert_eq!(tests.len(),16);assert_eq!(tests.iter().map(|row|row["name"].as_str().unwrap()).collect::<BTreeSet<_>>(),expected.keys().copied().collect());
 for row in tests{assert_eq!(row["status"],"passed","{row}");let duration=row["durationMs"].as_f64().unwrap();assert!(duration.is_finite()&&duration>=0.0);}
 for row in rows.iter().filter(|row|row["event"]=="output"){assert_eq!(row["stream"],"stdout");assert!(expected.contains_key(row["name"].as_str().unwrap()));}
 for(case,output)in expected{let actual=rows.iter().filter(|row|row["event"]=="output"&&row["name"]==case).map(|row|row["text"].as_str().unwrap()).collect::<String>();assert_eq!(actual,output,"{case}");}
 let builds=rows.iter().filter(|row|row["event"]=="build").collect::<Vec<_>>();assert_eq!(builds.len(),1);assert_eq!(builds[0]["testFiles"],1);for field in ["durationMs","checkMs","emitMs","backendMs","cacheMs","nativeBuildMs"]{let duration=builds[0][field].as_f64().unwrap();assert!(duration.is_finite()&&duration>=0.0);}
 let summaries=rows.iter().filter(|row|row["event"]=="summary").collect::<Vec<_>>();assert_eq!(summaries.len(),1);assert_eq!(rows.last().unwrap()["event"],"summary");for(field,value)in[("passed",16),("failed",0),("hookFailures",0),("skipped",0),("filesPassed",1),("filesFailed",0)]{assert_eq!(summaries[0][field],value);}
}
