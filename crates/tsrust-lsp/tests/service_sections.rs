mod support;
#[test]
fn service_limits_forms_and_security_help(){
 let source="import {App,request} from 'std:http';import {randomBytes} from 'std:crypto';async function main():Task<void>{const app=new App();app.configure({bodyLimit:1024,maxConnections:4,maxRequests:4,requestTimeoutMs:1000,shutdownTimeoutMs:1000,handshakeTimeoutMs:1000,requestsPerSecond:0});const input=request('POST','http://localhost/','');const fields=await input.form();const bytes=unwrap(randomBytes(32));}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 for (needle,expected) in [("app.configure","requestsPerSecond"),("input.form","UTF-8"),("randomBytes(32)","operating system")]{let offset=source.find(needle).unwrap()+needle.find('.').map(|n|n+1).unwrap_or(0);let hover=session.hover("main.tsr",source,offset).unwrap_or_default();assert!(hover.contains(expected),"{needle}: expected {expected}; hover={hover:?}; counters={:?}",session.log);}
}
