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
 let client=request(source,"new Client(","textDocument/signatureHelp").to_string();assert!(client.contains("no total request/body timeout")&&client.to_lowercase().contains("redirect"),"{client}");
 let body=request(source,"input.text","textDocument/hover").to_string();assert!(body.contains("configured")&&body.to_lowercase().contains("consum"),"{body}");
}
#[test]
fn aliased_http_members_complete(){
 let source="import {Request as Incoming} from 'std:http';function inspect(input:Incoming):void{input.;}function main(){}";
 let result=request(source,"input.","textDocument/completion").to_string();assert!(result.contains("setHeader")&&result.contains("param")&&result.contains("text"),"{result}");
}
#[test]
fn request_json_help_and_signature_keep_response_json_distinct(){
 let source="import {Request,Context} from 'std:http';async function inspect(input:Request,c:Mutable<Context>):Task<void>{const value=await input.json();c.json(value);}";
 let hover=request(source,"input.json","textDocument/hover").to_string();
 for expected in ["Task<JsonValue>","schema.parse", "JsonError", "Ownership"]{assert!(hover.contains(expected),"{hover}");}
 let signature=request(source,"input.json(","textDocument/signatureHelp");
 assert_eq!(signature["signatures"][0]["parameters"].as_array().unwrap().len(),0);
 let response=request(source,"c.json","textDocument/hover").to_string();
 assert!(response.contains("Response"),"{response}");
}
#[test]
fn tls_options_and_certificate_checks_are_documented(){
 let source="import {App,Client,TlsOptions} from 'std:http';async function main():Task<void>{const app=new App();const client=new Client({ca:'pem'});const tls:TlsOptions={cert:'pem',key:'pem'};const server=await app.listen('127.0.0.1:0',tls);await server.shutdown();}";
 let client=request(source,"new Client(","textDocument/signatureHelp");assert!(client["signatures"][0]["parameters"][0]["label"].as_str().unwrap().contains("ClientOptions"));
 let listen=request(source,"app.listen","textDocument/hover").to_string();assert!(listen.contains("TlsOptions")&&listen.contains("handshake")&&listen.contains("hostname"),"{listen}");
 let completion=request("import {TlsOptions} from 'std:http';function inspect(tls:TlsOptions):void{tls.;}","tls.","textDocument/completion").to_string();assert!(completion.contains("cert")&&completion.contains("key"),"{completion}");
}

#[test]
fn client_transport_options_and_upload_help_keep_owned_contracts(){
 let source="import {Client,ClientOptions,requestBytes,requestStream,requestChannel} from 'std:http';function inspect(options:ClientOptions){options.;}function main(){const client=new Client({noProxy:true,timeoutMs:0});}";
 let result=request(source,"options.","textDocument/completion");let labels=result.as_array().unwrap().iter().filter_map(|item|item["label"].as_str()).collect::<Vec<_>>();for name in ["ca","bodyLimit","timeoutMs","connectTimeoutMs","followRedirects","maxRedirects","proxy","noProxy"]{assert!(labels.contains(&name),"{result}");}
 for name in ["requestBytes","requestStream","requestChannel"]{let hover=request(source,name,"textDocument/hover").to_string();assert!(hover.contains("Ownership")&&hover.contains("consumes")&&hover.contains("Request"),"{hover}");}
 let constructor=request(source,"new Client(","textDocument/signatureHelp").to_string();for text in ["timeoutMs","connectTimeoutMs","maxRedirects","noProxy","TLS"]{assert!(constructor.contains(text),"{constructor}");}
}

#[test]
fn form_factories_document_borrowed_fields_owned_parts_and_backend_limitations(){let source="import {requestForm,requestMultipart} from 'std:http';function main(){}";let form=request(source,"requestForm","textDocument/hover").to_string();assert!(form.contains("ReadOnly")&&form.contains("duplicate")&&form.contains("Ownership"),"{form}");let multipart=request(source,"requestMultipart","textDocument/hover").to_string();assert!(multipart.contains("Owned")&&multipart.contains("ASCII")&&multipart.contains("Content-Length")&&multipart.contains("multipart"),"{multipart}");}

#[test]
fn source_policy_help_explains_closed_defaults_source_order_and_backend_limits(){let source="import {cors,secureHeaders,compress,Middleware,CorsOptions} from 'std:http';function main(){}";for(name,terms)in[("cors",vec!["closed","credentials","maxAgeSeconds","Ownership"]),("secureHeaders",vec!["absent","SAMEORIGIN","HSTS"]),("compress",vec!["gzip","deflate","identity","Content-Length"])]{let hover=request(source,name,"textDocument/hover").to_string();for term in terms{assert!(hover.contains(term),"{name} missing {term}: {hover}");}}}
