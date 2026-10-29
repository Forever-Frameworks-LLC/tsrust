mod support;
use std::{fs,path::Path};

#[test]
fn reactive_semantics_execute_formatted_gallery_and_documented_example(){
 let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
 for relative in ["src/examples/reactive-semantics.tsrx","src/acceptance/reactive-semantics.tsr","src/acceptance/reactive-semantics-documentation.tsrx","src/components/ui/button-v3.tsrx"]{let source=fs::read_to_string(root.join(relative)).unwrap();assert_eq!(source,support::format_round_trip(&source));}
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();let entry=catalog["std:ui"].as_array().unwrap().iter().find(|entry|entry["name"]=="ElementProps").unwrap();let example=&entry["examples"][0];let documented=example["source"].as_str().unwrap().replace("function main()","export function verify()");assert_eq!(fs::read_to_string(root.join("src/acceptance/reactive-semantics-documentation.tsrx")).unwrap(),support::format_round_trip(&documented));
 let output=support::run(support::cli().arg("run").arg(root.join("src/acceptance/reactive-semantics.tsr")),180);support::success(&output);assert_eq!(output.stdout,format!("{}Reactive semantics passed\n",example["output"].as_str().unwrap()).as_bytes());
}

#[cfg(feature = "legacy-ui")]
#[test]
fn metadata_factories_release_captures_and_installed_buttons_forward_props(){
 let directory=support::TempDirectory::new();support::success(&support::run(support::cli().args(["ui","add","button","--offline","--target","native"]).current_dir(&directory.0),30));
 let source=r#"import {Node,document} from 'std:ui';import {Button} from './src/components/ui/button';import {trackedResource,resourceId,releaseCount,strictEqual,TrackedResource} from 'std:testing';
function Page(resource:TrackedResource):Node{return <Button id="retained" aria-label={{evaluate:()=>{strictEqual(releaseCount(resourceId(resource)),0);return 'Owned name';}}}>Action</Button>;}
function main(){const resource=trackedResource();const id=resourceId(resource);{const node=Page(resource);const html=document(node,'Metadata');strictEqual(html.includes('id="retained"'),true);strictEqual(html.includes('aria-label="Owned name"'),true);strictEqual(releaseCount(id),0);}strictEqual(releaseCount(id),1);console.log('Metadata ownership passed');}"#;
 let path=directory.0.join("main.tsrx");fs::write(&path,support::format_round_trip(source)).unwrap();let output=support::run(support::cli().arg("run").arg(&path),180);support::success(&output);assert_eq!(output.stdout,b"Metadata ownership passed\n");
 let source="import {Button} from 'std:ui';function main(){const node=<Button aria-label={{evaluate:()=>42}}>Wrong</Button>;}";fs::write(&path,source).unwrap();let output=support::run(support::cli().arg("check").arg(&path),30);assert!(!output.status.success());
 let lock:serde_json::Value=serde_json::from_str(&fs::read_to_string(directory.0.join("ui-components.lock.json")).unwrap()).unwrap();assert!(lock["installed"].as_array().unwrap().iter().any(|item|item["identity"]["name"]=="button"&&item["identity"]["version"]=="3.0.0"));
}
