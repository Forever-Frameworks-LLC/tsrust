mod support;

#[test]
fn formatted_visibility_callbacks_and_https_links_compile_and_serialize_without_effects(){
 let source=r#"import {View,Text,Link,Node,document,signal} from 'std:ui';import {strictEqual} from 'std:testing';
 function Page():Node{const count=signal(0);return <View onVisibilityChange={(visible)=>{if(visible){count.set(count.get()+1);}}}><Text>{count}</Text><Link href='https://github.com/Forever-Frameworks-LLC/tsrust/releases'>Releases</Link></View>;}
 function main(){const html=document(Page(),'Visibility');strictEqual(html.includes('>0</span>'),true);strictEqual(html.includes('https://github.com/Forever-Frameworks-LLC/tsrust/releases'),true);console.log('Visibility construction passed');}"#;
 let directory=support::TempDirectory::new();let entry=directory.0.join("main.tsrx");std::fs::write(&entry,support::format_round_trip(source)).unwrap();std::fs::write(directory.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();
 let output=support::run(support::cli().arg("run").arg(entry),120);support::success(&output);assert_eq!(output.stdout,b"Visibility construction passed\n");
}

#[test]
fn visibility_handler_rejects_incompatible_parameters_and_result(){
 let directory=support::TempDirectory::new();std::fs::write(directory.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();
 for callback in ["(visible:string)=>{}","(visible:boolean):int=>{return 1;}"]{
  let source=format!("import {{View}} from 'std:ui';function main(){{const node=<View onVisibilityChange={{{callback}}}/>;}}");let entry=directory.0.join("main.tsrx");std::fs::write(&entry,support::format_round_trip(&source)).unwrap();let output=support::run(support::cli().arg("check").arg(entry),30);assert!(!output.status.success(),"{callback}");
 }
}
