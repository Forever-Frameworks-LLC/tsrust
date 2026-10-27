mod support;
#[test]
fn static_mount_completion_and_help() {
 let source="import {Router} from 'std:http';function main(){const app=new Router();app.serveStatic('/web','dist','index.html');}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 let start=source.find("app.serveStatic").unwrap();
 let hover=session.hover("main.tsr",source,start+5).unwrap_or_default();
 assert!(hover.contains("serveStatic")&&hover.contains("text/html")&&hover.contains("Ownership"),"{hover}");
 let completion=session.completion("main.tsr",source,start+4).unwrap();
 assert!(completion.iter().any(|value|value=="serveStatic"),"{completion:?}");
}
#[test]
fn hono_context_header_and_route_help(){
 let source="import {Router,Context,ContextNext} from 'std:http';async function middleware(c:Mutable<Context>,next:ContextNext):Task<void>{c.header('x-before','yes');await next();c.header('x-after','yes');}function main(){const app=new Router();app.use('/',middleware);app.get('/',async (c,next)=>{await next();},c=>c.text('ok'));}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 for (needle,expected) in [("c.header","outgoing header"),("app.use","registration order"),("c.text","response")]{let offset=source.find(needle).unwrap()+needle.find('.').unwrap()+1;let hover=session.hover("main.tsr",source,offset).unwrap_or_default();assert!(hover.contains(expected),"{needle}: {hover}");}
}

#[test]
fn router_shared_state_and_cookie_help(){
 let source="import {Router,Context,setCookie} from 'std:http';type State={name:string;};type Variables={id:string;};function probe(c:Mutable<Context<State,Variables>>){c.set('id','one');setCookie(c,'theme','dark');}function main(){const app=new Router<State,Variables>({name:'service'});app.get('/',c=>c.text('ok'));}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 for (needle,expected) in [("c.set","request"),("setCookie(c","Set-Cookie"),("app.get","callback")]{let offset=source.find(needle).unwrap()+needle.find('.').map_or(1,|i|i+1);let hover=session.hover("main.tsr",source,offset).unwrap_or_default();assert!(hover.contains(expected),"{needle}: {hover}");}
 let completion=session.completion("main.tsr",source,source.find("c.set").unwrap()+2).unwrap();
 for member in ["env","res","get","set"]{assert!(completion.iter().any(|value|value==member),"{member}: {completion:?}");}
}

#[test]
fn producer_stream_help_matches_owned_channel_contract(){
 let source="import {streamChannel} from 'std:http';import {Channel} from 'std:futures';import {Bytes} from 'std:bytes';function main(){const chunks=new Channel<Bytes>(1);streamChannel(chunks);}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 let hover=session.hover("main.tsr",source,source.rfind("streamChannel(").unwrap()+1).unwrap_or_default();
 assert!(hover.contains("Channel<Bytes>")&&hover.contains("closed"),"{hover}");
}
