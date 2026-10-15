use serde_json::{json,Value};
use tsrust_lsp::intelligence::SourceIndex;
fn request(source:&str,needle:&str,method:&str)->Value{
 let offset=source.find(needle).unwrap()+needle.len()-usize::from(method=="textDocument/hover");
 SourceIndex::new(source).request(method,&json!({"position":{"line":0,"character":offset}}),"file:///http.tsr").unwrap()
}
#[test]
fn constructors_and_body_help_explain_defaults_and_ownership(){
 let source="import {App,Client,Request} from 'std:http';function inspect(input:Request):void{input.text();}function main(){const app=new App();const client=new Client();}";
 let app=request(source,"new App(","textDocument/signatureHelp").to_string();assert!(app.contains("new App(): App"),"{app}");
 let client=request(source,"new Client(","textDocument/signatureHelp").to_string();assert!(client.contains("30")&&client.to_lowercase().contains("redirect"),"{client}");
 let body=request(source,"input.text","textDocument/hover").to_string();assert!(body.contains("1048576")&&body.to_lowercase().contains("consum"),"{body}");
}
#[test]
fn aliased_http_members_complete(){
 let source="import {Request as Incoming} from 'std:http';function inspect(input:Incoming):void{input.;}function main(){}";
 let result=request(source,"input.","textDocument/completion").to_string();assert!(result.contains("setHeader")&&result.contains("param")&&result.contains("text"),"{result}");
}
