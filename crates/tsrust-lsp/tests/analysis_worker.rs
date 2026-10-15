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
