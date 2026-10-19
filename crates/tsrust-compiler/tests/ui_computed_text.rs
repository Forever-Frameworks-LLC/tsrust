mod support;
#[test]
fn computed_text_and_numeric_children_execute_with_exact_values_after_formatting() {
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-computed-text");let output=support::run(support::cli().arg("run").arg(root.join("src/inspect.tsr")),120);support::success(&output);assert_eq!(output.stdout,b"Computed text passed\n");
}
#[test]
fn unrelated_children_and_node_helpers_keep_snapshot_types_and_binding_handles_stay_private() {
 let dir=support::TempDirectory::new();std::fs::write(dir.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();
 let source="import {Node,Text,View,signal,document} from 'std:ui';interface Props{children?:int[];}function Plain(props:Props):Node{return Text({children:props.children??[]});}function node(value:int):Node{return <Text>{value}</Text>;}function Page():Node{const count=signal(3);return <View><Plain>{count.get()}</Plain>{node(count.get())}</View>;}function main(){const html=document(Page(),'Snapshots');console.log(html.includes('>3</span>'));}";
 let path=dir.0.join("main.tsrx");std::fs::write(&path,support::format_round_trip(source)).unwrap();let output=support::run(support::cli().arg("run").arg(&path),120);support::success(&output);assert_eq!(output.stdout,b"true\n");
 for body in ["const binding:TextBinding={};","const values:TextChild[]=['x'];const copy=clone(values);","const node=<Text>{false}</Text>;"] {
  std::fs::write(&path,support::format_round_trip(&format!("import {{Text,TextBinding,TextChild}} from 'std:ui';function main(){{{body}}}"))).unwrap();for command in ["check","emit"]{let output=support::run(support::cli().arg(command).arg(&path),30);assert!(!output.status.success(),"{body}");}
 }
}
