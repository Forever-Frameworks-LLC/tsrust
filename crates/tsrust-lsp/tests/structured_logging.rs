use serde_json::json;
use tsrust_compiler::project::SourceId;
use tsrust_lsp::{workspace::Workspace,project_diagnostics::file_uri,project_intelligence::ProjectIndex};
#[test]
fn local_logger_help_preserves_defaults_lazy_ownership_and_sink_error_contract(){
 let source="import {createLogger,LoggerOptions} from 'std:log';function main(){const logger=unwrap(createLogger({sink:'stdout'}));logger.info(()=>({message:'ok',count:3}));}";let id=SourceId(std::fs::canonicalize(std::env::temp_dir()).unwrap().join("structured-logging-help/main.tsr").to_string_lossy().into_owned());let uri=file_uri(&id).unwrap().to_string();let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.into());let checked=workspace.snapshot().check_with_diagnostics(&id).expect("logging example must check");let index=ProjectIndex::checked(&checked);
 for(needle,terms)in[("createLogger",vec!["info","append","global","Ownership"]),("logger.info",vec!["once","enabled","callback","Ownership"]) ]{let offset=source.rfind(needle).unwrap()+needle.len()-1;let value=index.request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),&uri).unwrap().unwrap().to_string();for term in terms{assert!(value.contains(term),"{needle} missing {term}: {value}");}}
}
