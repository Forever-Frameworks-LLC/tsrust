use serde_json::json;
use tsrust_compiler::project::SourceId;
use tsrust_lsp::{workspace::Workspace,project_intelligence::ProjectIndex};
#[test]
fn explicit_sql_decimal_encoder_retains_checked_borrowed_signature(){
 let entry=SourceId("/sql-domain/main.tsr".into());let source="import {decimal38} from './encoding.tsr';import {parseDecimal} from 'std:decimal';function main(){decimal38(unwrap(parseDecimal('1.25')));}";
 let mut workspace=Workspace::default();workspace.open(entry.clone(),1,source.into());workspace.open(SourceId("/sql-domain/encoding.tsr".into()),1,include_str!("../../../examples/sql-domain-values/encoding.tsr").into());
 let checked=workspace.snapshot().check(&entry).unwrap();let index=ProjectIndex::checked(&checked);
 let answer=index.request("textDocument/hover",&json!({"position":{"line":0,"character":source.rfind("decimal38").unwrap()}}),"file:///sql-domain/main.tsr").unwrap().unwrap().to_string();
 assert!(answer.contains("ReadOnly<Decimal>")&&answer.contains("string"),"{answer}");
}
