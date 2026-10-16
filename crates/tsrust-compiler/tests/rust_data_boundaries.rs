mod support;
#[test]
fn checked_rust_adapters_preserve_widths_bytes_optional_and_owned_pairs(){
 let dir=support::TempDirectory::new();std::fs::write(dir.0.join("adapter.rs"),include_str!("../../../examples/rust-data-boundaries/adapter.rs")).unwrap();
 let input=dir.source(&support::format_round_trip(include_str!("../../../examples/rust-data-boundaries/checked.tsr")));
 support::success(&support::run(support::cli().arg("run").arg(input),180));
}
#[test]
fn unsupported_direct_rust_widths_and_containers_are_explicit_rejections(){
 for ty in ["u8","u64","i128","u128","f32","Vec<u8>","Option<i64>","(i64,String)"]{
  let dir=support::TempDirectory::new();std::fs::write(dir.0.join("adapter.rs"),format!("pub fn identity(value:{ty})->{ty}{{value}}")).unwrap();
  let source=dir.source("import {identity} from './adapter.rs';function main(){}");let result=support::run(support::cli().arg("check").arg(source),30);
  assert!(!result.status.success(),"unexpected direct support: {ty}");assert!(String::from_utf8_lossy(&result.stderr).contains("rust-bridge"));
 }
}
