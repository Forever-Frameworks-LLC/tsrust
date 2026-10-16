use serde_json::json;
use tsrust_compiler::project::SourceId;
use tsrust_lsp::{workspace::Workspace,project_intelligence::ProjectIndex};
#[test]
fn empty_rest_is_plain_data_even_when_opaque_resources_are_imported(){
 let entry=SourceId("/record-rest/main.tsr".into());
 let source="import {trackedResource} from 'std:testing';type Row={value:int;};function main(){const row:Row={value:2};const {value,...empty}=row;const saved=clone(empty);}";
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.into());let checked=workspace.snapshot().check(&entry).unwrap();let index=ProjectIndex::checked(&checked);
 let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":source.rfind("empty").unwrap()}}),"file:///record-rest/main.tsr").unwrap().unwrap().to_string();assert!(!hover.contains("TrackedResource"),"{hover}");
 workspace.open(entry.clone(),2,"import {trackedResource} from 'std:testing';function main(){const value=trackedResource();const {...rest}=value;}".into());
 let error=workspace.snapshot().check(&entry).err().expect("opaque rest must reject");assert!(format!("{error:?}").contains("native-capability"));
}
