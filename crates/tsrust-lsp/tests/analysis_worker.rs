mod support;
use support::session::Session;
#[test]
fn complete_numeric_examples_keep_the_analysis_worker_alive(){
 let checked=include_str!("../../../examples/numeric-boundaries/checked.tsr");
 let workload=include_str!("../../../benchmarks/numeric/numeric-boundaries.tsr");
 let mut session=Session::start(&[]);
 for (name,source) in [("checked.tsr",checked),("workload.tsr",workload)]{
 session.open(name,source);session.settle();
 let offset=source.find("toInt(").unwrap()+1;
 assert!(session.hover(name,source,offset).unwrap_or_default().contains("whole number"));
 }
 session.request("tsrust/counters",serde_json::json!({}));
}
#[test]
fn http_examples_keep_the_analysis_worker_alive(){
 for (name,source) in [
 ("https-benchmark.tsr",include_str!("../../../benchmarks/http/https.tsr")),
 ("loopback-benchmark.tsr",include_str!("../../../benchmarks/http/loopback.tsr")),
 ("https.tsr",include_str!("../../../examples/http/https.tsr")),
 ("loopback.tsr",include_str!("../../../examples/http/loopback.tsr")),
 ("validated-body.tsr",include_str!("../../../examples/http/validated-body.tsr"))]{
 eprintln!("checking {name}");
 let mut session=Session::start(&[]);session.open(name,source);session.settle();session.request("tsrust/counters",serde_json::json!({}));
 }
}

#[test]
fn background_scan_checks_nested_http_before_publishing_closed_file_errors(){
 let mut session=Session::start(&[
 ("http.tsr",include_str!("../../../examples/http/loopback.tsr")),
 ("numeric.tsr",include_str!("../../../examples/numeric-boundaries/checked.tsr")),
 ("zz-sentinel.tsr","function main(){const value:int='wrong';}")]);
 let diagnostics=session.wait_for_diagnostics("zz-sentinel.tsr");
 assert!(!diagnostics.as_array().unwrap().is_empty(),"{diagnostics}");
 session.request("tsrust/counters",serde_json::json!({}));
}

#[test]
fn foreground_imported_members_remain_current_during_a_workspace_scan(){
 let workload=include_str!("../../../examples/http/loopback.tsr");
 let names=(0..24).map(|index|format!("background-{index:02}.tsr")).collect::<Vec<_>>();
 let mut files=names.iter().map(|name|(name.as_str(),workload)).collect::<Vec<_>>();files.push(("values.tsr","export interface Value { title:string; }"));
 let mut session=Session::start(&files);std::thread::sleep(std::time::Duration::from_millis(50));
 let source="import {Value} from './values.tsr';function read(value:ReadOnly<Value>){console.log(value.title);}";
 session.open("foreground.tsr",source);let offset=source.find("value.title").unwrap()+"value.".len();
 let hover=session.hover("foreground.tsr",source,offset+1).unwrap_or_default();assert!(hover.contains("string"),"{hover}");
 let bad=source.replace("value.title","value.missing");session.change("foreground.tsr",&bad);
 let mut diagnostics=session.wait_for_diagnostics("foreground.tsr");for _ in 0..3{if diagnostics.to_string().contains("missing"){break;}diagnostics=session.wait_for_diagnostics("foreground.tsr");}assert!(diagnostics.to_string().contains("missing"),"{diagnostics}");
 let counters=session.request("tsrust/counters",serde_json::Value::Null);assert_eq!(counters["staleAnswers"],0,"{counters}");
}
