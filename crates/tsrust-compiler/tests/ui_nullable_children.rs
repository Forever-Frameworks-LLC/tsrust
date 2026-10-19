mod support;
#[test]
fn nullable_components_preserve_order_state_and_independent_resource_release() {
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-nullable-children");
    let output=support::run(support::cli().arg("run").arg(root.join("src/inspect.tsr")),120);
    support::success(&output);assert_eq!(output.stdout,b"Nullable children passed\n");
}
#[test]
fn nullable_children_do_not_relax_owned_storage_or_supported_payloads() {
    for body in [
      "const children:Child[]=[null,<Text>owned</Text>,undefined];const copied=clone(children);",
      "const children:Child[]=[null,<Text>owned</Text>];const borrow=(items:ReadOnly<Child[]>):Node=>{return View({children:items});};const node=borrow(children);",
      "const node=<View>{false}</View>;",
      "const node=<Text>{123}</Text>;",
    ] {
      let dir=support::TempDirectory::new();std::fs::write(dir.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();
      let source=support::format_round_trip(&format!("import {{Child,Node,Text,View,signal}} from 'std:ui';function main(){{{body}}}"));let path=dir.0.join("main.tsrx");std::fs::write(&path,source).unwrap();
      for command in ["check","emit"] {let output=support::run(support::cli().arg(command).arg(&path),30);assert!(!output.status.success(),"{body}");let error=String::from_utf8_lossy(&output.stderr);assert!(error.contains("error[")&&!error.contains("panicked"),"{error}");}
    }
}

#[test]
fn nullable_state_ownership_still_rejects_non_node_results() {
 let dir=support::TempDirectory::new();std::fs::write(dir.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();let source=support::format_round_trip("import {signal} from 'std:ui';function Wrong():int{const count=signal(0);return 42;}function main(){Wrong();}");let path=dir.source(&source);
 for command in ["check","emit"]{let output=support::run(support::cli().arg(command).arg(&path),30);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("ui-state-return"),"{}",String::from_utf8_lossy(&output.stderr));}
}
