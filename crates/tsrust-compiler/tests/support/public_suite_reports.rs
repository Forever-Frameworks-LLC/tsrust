//! Existing native suite assertions shared by the ordinary batch and portable export.
use serde_json::Value;
#[path="iterator_catalog_examples.rs"] mod iterator_catalog_examples;
use std::collections::BTreeSet;
pub const LOGGING_NAMES:&[&str]=&["structured logging > request context and owned child cleanup", "structured logging > disabled factory drops its owned capture without invocation", "structured logging > aliases and stored calls retain children after parent close", "structured logging > invalid sink options remain explicit Result errors", "structured logging > enabled factory preserves its checked rejection"];
pub const REALTIME_NAMES:&[&str]=&["SSE backend bytes and ownership > framing preserves empty and CRLF backend semantics", "SSE backend bytes and ownership > owned producer completes and drains buffered events", "SSE backend bytes and ownership > unpolled response drop settles owned producer", "SSE backend bytes and ownership > partial body drop settles owned producer", "SSE backend bytes and ownership > producer failure is a body failure and drops typed cause", "SSE backend bytes and ownership > invalid event fields fail the body", "WebSocket owned transport > text and binary tags survive full socket and split halves", "explicit realtime policies > keepalive is opt in and preserves comment bytes", "explicit realtime policies > authentication and Origin middleware reject before upgrade", "explicit realtime policies > explicit frame and message limits reject an oversized echo", "explicit realtime policies > invalid close frames fail before affecting the connection", "explicit realtime policies > explicit shutdown deadline settles an active upgraded handler", "explicit realtime policies > representative binary sizes retain exact bytes", "WebSocket TLS verification > explicit private CA verifies a matching hostname", "WebSocket TLS verification > default public roots reject the private CA", "WebSocket TLS verification > custom trust still rejects a wrong hostname", "WebSocket TLS verification > custom trust still rejects an expired certificate", "WebSocket TLS verification > custom trust still rejects a future certificate"];
pub const ITERATOR_REGRESSION_NAMES:&[&str]=&["iterator carrier regressions > physical carriers", "iterator carrier regressions > conditional closure results", "iterator carrier regressions > borrowed records first", "iterator carrier regressions > owned records first", "iterator carrier regressions > temporary discriminators", "iterator carrier regressions > callback and task payloads", "iterator carrier regressions > flatmap mixed alternatives", "iterator carrier regressions > nested receivers", "iterator carrier regressions > borrowed nullable capture", "iterator carrier regressions > nullable flatmap state", "iterator carrier regressions > nullable counter reinitialization", "iterator carrier regressions > mixed union returns", "iterator carrier regressions > readonly union observations", "iterator carrier regressions > generic legacy callback records", "iterator carrier regressions > concrete nullable record", "iterator carrier regressions > named nullable consumers", "iterator carrier regressions > started job transport"];
pub const ITERATOR_NAMES:&[&str]=&["owned iterator contracts"];
pub fn parse(output:std::process::Output)->Vec<Value>{
 super::support::success(&output);assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));let stdout=String::from_utf8(output.stdout).unwrap();print!("{stdout}");assert!(!stdout.contains("never-log-this"));stdout.lines().map(|line|serde_json::from_str(line).unwrap_or_else(|error|panic!("invalid JSON {line}: {error}"))).collect()
}
fn cases(rows:&[Value],expected:&[&str]){
 let expected=expected.iter().copied().collect::<BTreeSet<_>>();let selected=rows.iter().filter(|row|row["event"]=="test"&&row["name"].as_str().is_some_and(|name|expected.contains(name))).collect::<Vec<_>>();assert_eq!(selected.len(),expected.len(),"missing or duplicate cases: {rows:?}");let names=selected.iter().map(|row|row["name"].as_str().unwrap()).collect::<BTreeSet<_>>();assert_eq!(names,expected);
 for row in selected{assert_eq!(row["status"],"passed","{row}");let duration=row["durationMs"].as_f64().unwrap();assert!(duration.is_finite()&&duration>=0.0,"{row}");}
}
pub fn summary(rows:&[Value],passed:u64,files:u64){
 let summaries=rows.iter().filter(|row|row["event"]=="summary").collect::<Vec<_>>();assert_eq!(summaries.len(),1);assert_eq!(rows.last().unwrap()["event"],"summary");let summary=summaries[0];for(field,value)in[("passed",passed),("failed",0),("hookFailures",0),("skipped",0),("filesPassed",files),("filesFailed",0)]{assert_eq!(summary[field],value,"{field}: {summary}");}
}
pub fn logging(rows:&[Value]){cases(rows,LOGGING_NAMES);
 let logs=rows.iter().filter(|row|row["fields"]["message"].is_string()).collect::<Vec<_>>();assert_eq!(logs.len(),10);let mut requests=std::collections::BTreeMap::<u64,Vec<String>>::new();for row in logs{let message=row["fields"]["message"].as_str().unwrap();let id=row["spans"][0]["request_id"].as_u64().unwrap();requests.entry(id).or_default().push(message.into());assert!(row["timestamp"].is_string());if message=="request"{assert_eq!(row["fields"]["status"],200);assert_eq!(row["fields"]["method"],"GET");assert_eq!(row["fields"]["route"],"/items/:id");assert!(row["fields"]["elapsed_ms"].as_f64().unwrap()>=0.0);}else{assert_eq!(row["fields"]["ok"],true);assert_eq!(row["spans"][1]["service"],"api");assert_eq!(row["spans"][1]["worker"],7);}}
 assert_eq!(requests.len(),2);for messages in requests.values(){assert_eq!(messages.as_slice(),["before","worker","cleanup","after","request"]);}
}
pub fn realtime(rows:&[Value]){cases(rows,REALTIME_NAMES);}
pub fn iterator(rows:&[Value]){cases(rows,ITERATOR_NAMES);assert!(rows.iter().any(|row|row["event"]=="output"&&row["stream"]=="stdout"&&row["text"]=="owned iterators passed\n"));}
pub fn iterator_catalog_sources(){iterator_catalog_examples::assert_sources();}
pub fn iterator_catalog(rows:&[Value]){
 let names=iterator_catalog_examples::case_names();assert_eq!(names.len(),14);cases(rows,&names);
 let catalog=iterator_catalog_examples::catalog();let mut expected=std::collections::BTreeMap::new();
 for &(receiver,name,index,case,_) in iterator_catalog_examples::REFERENCES{let output=iterator_catalog_examples::example(&catalog,receiver,name,index)["output"].as_str().unwrap();if let Some(previous)=expected.insert(case,output){assert_eq!(previous,output,"shared example outputs disagree: {case}");}}
 assert_eq!(expected.len(),14);for(case,output)in expected{let mut actual=String::new();for row in rows.iter().filter(|row|row["event"]=="output"&&row["name"]==case){assert_eq!(row["stream"],"stdout","catalog emitted stderr: {row}");actual.push_str(row["text"].as_str().unwrap());}assert_eq!(actual,output,"catalog output differs: {case}");}
}
pub fn combined(rows:&[Value]){
 let mut expected=LOGGING_NAMES.iter().chain(REALTIME_NAMES).chain(ITERATOR_NAMES).chain(ITERATOR_REGRESSION_NAMES).copied().collect::<Vec<_>>();expected.extend(iterator_catalog_examples::case_names());assert_eq!(expected.len(),55);assert_eq!(rows.iter().filter(|row|row["event"]=="test").count(),55);cases(rows,&expected);logging(rows);realtime(rows);iterator(rows);iterator_catalog(rows);summary(rows,55,5);
 let builds=rows.iter().filter(|row|row["event"]=="build").collect::<Vec<_>>();assert_eq!(builds.len(),1);assert_eq!(builds[0]["testFiles"],5);for field in ["durationMs","checkMs","emitMs","backendMs","cacheMs","nativeBuildMs"]{let duration=builds[0][field].as_f64().unwrap();assert!(duration.is_finite()&&duration>=0.0,"{field}");}
}

const REALTIME_TLS_FIXTURES:&[(&str,&str)]=&[("ca.pem",include_str!("../fixtures/tls/ca.pem")),("server.pem",include_str!("../fixtures/tls/server.pem")),("server.key",include_str!("../fixtures/tls/server.key")),("wrong-host.pem",include_str!("../fixtures/tls/wrong-host.pem")),("expired.pem",include_str!("../fixtures/tls/expired.pem")),("future.pem",include_str!("../fixtures/tls/future.pem"))];
/// Keep runtime certificate reads outside protected project folders and exported source paths.
pub fn realtime_tls_fixtures(directory:&std::path::Path)->std::path::PathBuf{
 let fixtures=directory.join("tls");std::fs::create_dir_all(&fixtures).unwrap();
 let lock=std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(fixtures.join(".lock")).unwrap();lock.lock().unwrap();
 for &(name,source)in REALTIME_TLS_FIXTURES{let path=fixtures.join(name);if std::fs::read(&path).ok().as_deref()!=Some(source.as_bytes()){std::fs::write(path,source).unwrap();}}
 fixtures
}
pub fn realtime_tls_fixture_cache()->std::path::PathBuf{
 use std::hash::{Hash,Hasher};let mut identity=std::collections::hash_map::DefaultHasher::new();env!("CARGO_MANIFEST_DIR").hash(&mut identity);REALTIME_TLS_FIXTURES.hash(&mut identity);
 // Runtime fixture paths enter the compiler's build-environment identity too.
 realtime_tls_fixtures(&std::env::temp_dir().join(format!("tsrust-realtime-fixtures-{:016x}",identity.finish())))
}
