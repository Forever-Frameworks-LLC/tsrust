//! Exact source and output correspondence for the stable native network suite.
use super::support;
pub const CASES:&[(&str,&str,&str)]=&[
    ("address_values_std_grammar_scope_and_native_forwarding-0",include_str!("../fixtures/network-runtime/runtime-cases/address-values-std-grammar-scope-and-native-forwarding-0.tsr"),"127.0.0.1 true false\n[2001:db8::1]:8080 [2001:db8::1]:8080\nsyntax parseAddress\n"),
    ("address_values_std_grammar_scope_and_native_forwarding-1",include_str!("../fixtures/network-runtime/runtime-cases/address-values-std-grammar-scope-and-native-forwarding-1.tsr"),"2001:db8::1 true\n[fe80::1%2]:80\n"),
    ("malformed_addresses_reject_without_dns_or_implicit_whitespace-2",include_str!("../fixtures/network-runtime/runtime-cases/malformed-addresses-reject-without-dns-or-implicit-whitespace-2.tsr"),"syntax\nsyntax\nsyntax\nsyntax\nsyntax\nsyntax\nsyntax\nsyntax\n"),
    ("dns_literal_default_optional_and_stored_calls_keep_explicit_results-3",include_str!("../fixtures/network-runtime/runtime-cases/dns-literal-default-optional-and-stored-calls-keep-explicit-results-3.tsr"),"1 127.0.0.1\n0\n"),
    ("dns_literal_default_optional_and_stored_calls_keep_explicit_results-4",include_str!("../fixtures/network-runtime/runtime-cases/dns-literal-default-optional-and-stored-calls-keep-explicit-results-4.tsr"),"1 ::1\n"),
    ("dns_host_loans_settle_and_owned_factories_can_start-5",include_str!("../fixtures/network-runtime/runtime-cases/dns-host-loans-settle-and-owned-factories-can-start-5.tsr"),"1\n"),
    ("borrowed_union_task_factories_keep_descriptor_and_other_input_lifetimes-6",include_str!("../fixtures/network-runtime/runtime-cases/borrowed-union-task-factories-keep-descriptor-and-other-input-lifetimes-6.tsr"),"5\n5\n5\n5\n"),
    ("awaited_forwarded_task_keeps_temporary_argument_owner_through_settlement-7",include_str!("../fixtures/network-runtime/runtime-cases/awaited-forwarded-task-keeps-temporary-argument-owner-through-settlement-7.tsr"),"4\n5\n"),
    ("full_stream_and_split_half_close_examples_execute-0",include_str!("../fixtures/network-runtime/runtime-cases/full-stream-and-split-half-close-examples-execute-0.tsr"),"ping\n"),
    ("full_stream_and_split_half_close_examples_execute-1",include_str!("../fixtures/network-runtime/runtime-cases/full-stream-and-split-half-close-examples-execute-1.tsr"),"reply after EOF\n"),
    ("udp_binary_empty_and_remote_endpoint_snapshots_execute-2",include_str!("../fixtures/network-runtime/runtime-cases/udp-binary-empty-and-remote-endpoint-snapshots-execute-2.tsr"),"5\nhello\n0\n0\n"),
    ("udp_binary_empty_and_remote_endpoint_snapshots_execute-3",include_str!("../fixtures/network-runtime/runtime-cases/udp-binary-empty-and-remote-endpoint-snapshots-execute-3.tsr"),"3 255 true\n"),
    ("full_stream_short_reads_exact_destination_and_getter_aliases-4",include_str!("../fixtures/network-runtime/runtime-cases/full-stream-short-reads-exact-destination-and-getter-aliases-4.tsr"),"true true\nabcd\n0\n"),
    ("copy_supports_memory_reader_to_full_stream-5",include_str!("../fixtures/network-runtime/runtime-cases/copy-supports-memory-reader-to-full-stream-5.tsr"),"10\ncopy bytes\n"),
    ("tcp_copy_to_full_stream_and_owned_writer_half-full",include_str!("../fixtures/network-runtime/runtime-cases/tcp-copy-to-full-stream-and-owned-writer-half-full.tsr"),"forward\n"),
    ("tcp_copy_to_full_stream_and_owned_writer_half-half",include_str!("../fixtures/network-runtime/runtime-cases/tcp-copy-to-full-stream-and-owned-writer-half-half.tsr"),"forward\n"),
    ("handler_owner_move_before_rethrow_preserves_normal_continuation-6",include_str!("../fixtures/network-runtime/runtime-cases/handler-owner-move-before-rethrow-preserves-normal-continuation-6.tsr"),"3\n3\n1\n"),
    ("catalog-net-IpAddress-0",include_str!("../fixtures/network-runtime/runtime-cases/catalog-net-ipaddress-0.tsr"),"127.0.0.1 true false\n[2001:db8::1]:8080 [2001:db8::1]:8080\nsyntax parseAddress\n"),
];
pub fn assert_registered(source:&str,expected:&str){
 let (original,exported)=if source.contains("async function main("){("async function main(","export async function run(")}else{("function main(","export function run(")};
 assert_eq!(source.matches(original).count(),1);
 let source=support::format_round_trip(&source.replacen(original,exported,1));
 assert!(CASES.iter().any(|(_,registered,output)|support::format_round_trip(registered)==source && *output==expected),"Original network program or exact output lost from registered suite");
}
pub fn execute(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
 let suite=root.join("crates/tsrust-compiler/tests/fixtures/network-runtime/runtime.test.tsr");
 assert_eq!(support::format_round_trip(&std::fs::read_to_string(&suite).unwrap()),std::fs::read_to_string(&suite).unwrap());
 for release in [false,true]{
  let mut command=support::cli();command.current_dir(&root).arg("test").arg(&suite).args(["--reporter=json","--profile","--timeout=10000"]);if release{command.arg("--release");}
  let output=support::run(&mut command,600);support::success(&output);assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));
  let text=String::from_utf8(output.stdout).unwrap();print!("{text}");let rows=text.lines().map(|line|serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();
  let expected=CASES.iter().map(|(name,_,_)|format!("network runtime > {name}")).collect::<std::collections::BTreeSet<_>>();assert_eq!(expected.len(),CASES.len());
  let tests=rows.iter().filter(|row|row["event"]=="test").collect::<Vec<_>>();assert_eq!(tests.len(),expected.len());
  let actual=tests.iter().map(|row|row["name"].as_str().unwrap().to_owned()).collect::<std::collections::BTreeSet<_>>();assert_eq!(actual,expected);
  for row in rows.iter().filter(|row|row["event"]=="output"){assert!(expected.contains(row["name"].as_str().unwrap()));assert_eq!(row["stream"],"stdout");}
  for(name,_,stdout)in CASES{let name=format!("network runtime > {name}");let test=tests.iter().find(|row|row["name"]==name).unwrap();assert_eq!(test["status"],"passed","{name}");let actual=rows.iter().filter(|row|row["event"]=="output"&&row["name"]==name).map(|row|row["text"].as_str().unwrap()).collect::<String>();assert_eq!(actual,*stdout,"{name}");}
  let summary=rows.iter().filter(|row|row["event"]=="summary").collect::<Vec<_>>();assert_eq!(summary.len(),1);assert_eq!(summary[0]["passed"],CASES.len());for key in ["failed","skipped","hookFailures","filesFailed"]{assert_eq!(summary[0][key],0);}assert_eq!(summary[0]["filesPassed"],1);
  let builds=rows.iter().filter(|row|row["event"]=="build").collect::<Vec<_>>();assert_eq!(builds.len(),1);assert_eq!(builds[0]["testFiles"],1);
 }
}
