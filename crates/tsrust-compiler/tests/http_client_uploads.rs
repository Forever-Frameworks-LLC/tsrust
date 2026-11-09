mod support;
#[test]
fn client_policies_and_owned_upload_factories_execute_through_aliases_and_stored_calls(){
 let source=support::format_round_trip(include_str!("../../../examples/http-client-uploads/main.tsr"));let directory=support::TempDirectory::new();let path=directory.source(&source);
 for release in [false,true]{let mut command=support::cli();command.arg("run").arg(&path);if release{command.arg("--release");}let output=support::run(&mut command,180);support::success(&output);assert_eq!(output.stdout,b"binary, stream and bounded channel uploads\n");assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));}
}
#[test]
fn upload_factories_keep_exact_argument_types_and_single_owner_rules(){
 let prefix="import {Client,requestBytes,requestStream,requestChannel,request} from 'std:http';import {alloc,length,view,Bytes} from 'std:bytes';import {Channel} from 'std:futures';";
 for body in [
  "const bytes=unwrap(alloc(2));requestBytes('POST','http://localhost/',bytes);console.log(length(bytes));",
  "const body=request('POST','http://localhost/','data').body();requestStream('POST','http://localhost/',body);await body.next();",
  "const channel=new Channel<Bytes>(1);requestChannel('POST','http://localhost/',channel);channel.sender();",
  "requestBytes('POST','http://localhost/','text');",
  "const bytes=unwrap(alloc(2));requestBytes('POST','http://localhost/',unwrap(view(bytes,0,2)));",
  "requestStream('POST','http://localhost/',unwrap(alloc(1)));",
  "const channel=new Channel<int>(1);requestChannel('POST','http://localhost/',channel);",
  "new Client({timeoutMs:'1'});","new Client({connectTimeoutMs:false});","new Client({followRedirects:1});","new Client({proxy:1});","new Client({noProxy:'yes'});",
 ]{let source=support::format_round_trip(&format!("{prefix}async function main():Task<void>{{{body}}}"));let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&source)),30);assert!(!output.status.success(),"accepted {body}");}
}
