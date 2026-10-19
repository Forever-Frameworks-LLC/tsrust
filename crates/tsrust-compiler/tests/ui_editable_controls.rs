mod support;
#[test]
fn editable_components_keep_value_binding_exact_numbers_and_semantic_markup() {
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-inputs");
 let output=support::run(support::cli().arg("run").arg(root.join("src/inspect.tsr")),120);support::success(&output);assert_eq!(output.stdout,b"Editable controls passed\n");
}
#[test]
fn editable_values_and_callback_boundaries_reject_unsupported_types() {
 for body in ["const node=Input({value:true});","const node=Input({type:'password'});","const node=Input({onChange:(event)=>{event.target.value='changed';}});","const node=Input({onChange:(event)=>{event.preventDefault();}});"] {
  let dir=support::TempDirectory::new();std::fs::write(dir.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();let source=support::format_round_trip(&format!("import {{Input}} from 'std:ui';function main(){{{body}}}"));let path=dir.source(&source);
  for command in ["check","emit"]{let output=support::run(support::cli().arg(command).arg(&path),30);assert!(!output.status.success(),"{body}");let error=String::from_utf8_lossy(&output.stderr);assert!(error.contains("error[") && error.contains(path.file_name().unwrap().to_str().unwrap()),"{error}");}
 }
}
#[test]
fn native_field_factories_are_private_to_the_ui_facade() {
 let dir=support::TempDirectory::new();let source=support::format_round_trip("import {input} from 'std:ui-runtime';function main(){}" );let path=dir.source(&source);let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success());
}
#[test]
fn nova_text_controls_install_offline_as_editable_source_with_exact_catalog_identity() {
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-inputs");let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/ui-catalog/components-3.0.0.json")).unwrap();let dir=support::TempDirectory::new();
 for name in ["input","textarea","label"] {
  let item=catalog.as_array().unwrap().iter().find(|item|item["name"]==name).unwrap();let source=item["files"][0]["content"].as_str().unwrap();let formatted=support::format_round_trip(source);assert_eq!(formatted,std::fs::read_to_string(root.join(format!("src/components/ui/{name}.tsrx"))).unwrap());
  let output=support::run(support::cli().args(["ui","add",&format!("{name}@3.0.0"),"--target","web","--offline"]).current_dir(&dir.0),30);support::success(&output);let installed=std::fs::read_to_string(dir.0.join(format!("src/components/ui/{name}.tsrx"))).unwrap();assert!(installed.starts_with("// License: MIT. See ../../../attributions/shadcn-MIT.txt."));assert_eq!(installed.split_once("import ").unwrap().1,formatted.split_once("import ").unwrap().1);
 }
 let lock:serde_json::Value=serde_json::from_slice(&std::fs::read(dir.0.join("ui-components.lock.json")).unwrap()).unwrap();for name in ["input","textarea","label"]{assert!(lock["installed"].as_array().unwrap().iter().any(|entry|entry["identity"]["name"]==name && entry["identity"]["version"]=="3.0.0" && entry["attribution"]["path"]=="attributions/shadcn-MIT.txt" && entry.get("licenseText").is_none()));}
 let source="import {Input} from './components/ui/input';import {Textarea} from './components/ui/textarea';import {Label} from './components/ui/label';import {Node,View,document} from 'std:ui';function Page():Node{return <View><Label htmlFor='name'>Name</Label><Input id='name' defaultValue='Ada'/><Textarea aria-label='Notes'/></View>;}function main(){const html=document(Page(),'Fields');}";
 std::fs::write(dir.0.join("src/main.tsrx"),support::format_round_trip(source)).unwrap();support::success(&support::run(support::cli().arg("check").arg(dir.0.join("src/main.tsrx")),30));
}
#[test]
fn unrelated_component_value_props_keep_their_declared_scalar_type() {
 let dir=support::TempDirectory::new();std::fs::write(dir.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();let source=support::format_round_trip("import {signal,Node,Text,document} from 'std:ui';interface Props{value:int;readOnly:boolean;}function Plain(props:Props):Node{return <Text>{String(props.value)}{String(props.readOnly)}</Text>;}function main(){const count=signal(3);const flag=signal(true);const node=<Plain value={count.get()} readOnly={flag.get()}/>;console.log(document(node,'Plain').includes('>3true</span>'));}");let path=dir.0.join("main.tsrx");std::fs::write(&path,source).unwrap();let output=support::run(support::cli().arg("run").arg(path),120);support::success(&output);assert_eq!(output.stdout,b"true\n");
}
#[test]
fn native_generic_diagnostics_preserve_checked_payload_types() {
 let dir=support::TempDirectory::new();std::fs::write(dir.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();
 for (import,ty) in [("import {Signal} from 'std:ui';","Signal<string>"),("import {LruCache} from 'std:cache';","LruCache<int, string>")] {
  let source=support::format_round_trip(&format!("{import}function probe(value:ReadOnly<{ty}>){{console.log(value.unavailable);}}function main(){{}}"));let path=dir.source(&source);let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success());let error=String::from_utf8_lossy(&output.stderr);assert!(error.contains(&format!("Type {ty:?} has no field")),"{error}");
 }
}
