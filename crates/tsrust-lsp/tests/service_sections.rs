mod support;
#[test]
fn service_limits_forms_and_security_help(){
 let source="import {App,request} from 'std:http';import {randomBytes} from 'std:crypto';async function main():Task<void>{const app=new App();app.configure({bodyLimit:1024,maxConnections:4,maxRequests:4,requestTimeoutMs:1000,shutdownTimeoutMs:1000,handshakeTimeoutMs:1000,requestsPerSecond:0});const input=request('POST','http://localhost/','');const fields=await input.form();const bytes=unwrap(randomBytes(32));}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 for (needle,expected) in [("app.configure","requestsPerSecond"),("input.form","UTF-8"),("randomBytes(32)","operating system")]{let offset=source.find(needle).unwrap()+needle.find('.').map(|n|n+1).unwrap_or(0);let hover=session.hover("main.tsr",source,offset).unwrap_or_default();assert!(hover.contains(expected),"{needle}: expected {expected}; hover={hover:?}; counters={:?}",session.log);}
}

#[test]
fn random_bytes_range_help_agrees_with_empty_and_large_counts(){
 let source="import {randomBytes} from 'std:crypto';function main(){const empty=unwrap(randomBytes(0));const large=unwrap(randomBytes(1048577));}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 let hover=session.hover("main.tsr",source,source.find("randomBytes(0)").unwrap()).unwrap_or_default();
 for expected in ["nonnegative","Zero returns empty Bytes","no library size cap","Range","Random","owned"] {assert!(hover.contains(expected),"missing {expected}: {hover}");}
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
 let entries=catalog["std:crypto"].as_array().unwrap();
 let entry=entries.iter().find(|entry|entry["name"]=="randomBytes").unwrap();
 assert!(entry["parameters"][0]["documentation"].as_str().unwrap().contains("nonnegative"));
 assert!(!entry["documentation"].as_str().unwrap().contains("1..1048576"));
}
