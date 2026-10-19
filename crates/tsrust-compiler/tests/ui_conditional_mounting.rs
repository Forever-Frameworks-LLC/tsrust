mod support;
#[test]
fn conditional_mounting_executes_the_formatted_native_project_and_transparent_root() {
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-conditional-mounting");let output=support::run(support::cli().arg("run").arg(root.join("src/inspect.tsr")),120);support::success(&output);assert_eq!(output.stdout,b"Conditional mounting passed\n");
}
#[test]
fn lazy_branches_reject_consumed_captures_and_private_factories() {
 let dir=support::TempDirectory::new();std::fs::write(dir.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();
 for (source,wanted) in [
  ("import {View,Text,Node,signal} from 'std:ui';function Page():Node{const visible=signal(true);const owned=<Text>Owned</Text>;return <View>{visible.get()?owned:null}</View>;}function main(){const node=Page();}","error[callback-once]"),
  ("import {View,Node,signal} from 'std:ui';function Page():Node{const visible=signal(true);return <View>{visible.get()?{value:1}:null}</View>;}function main(){const node=Page();}","error[type]"),
  ("import {bindBranch} from 'std:ui';function main(){}","export"),
 ] {let path=dir.0.join("main.tsrx");std::fs::write(&path,support::format_round_trip(source)).unwrap();for command in ["check","emit"]{let output=support::run(support::cli().arg(command).arg(&path),30);assert!(!output.status.success(),"{source}");let error=String::from_utf8_lossy(&output.stderr);assert!(error.contains(wanted)&&!error.contains("panicked")&&!error.contains("Generic type applications are not yet"),"{source}: {error}");eprintln!("rejection {wanted}: {}",error.lines().next().unwrap_or(""));}}
}
