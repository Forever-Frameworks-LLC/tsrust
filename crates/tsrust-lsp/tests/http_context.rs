mod support;
#[test]
fn contextual_json_help_keeps_unbounded_default_and_explicit_encoder_policy(){
 let source="import {Router} from 'std:http';function main(){const app=new Router();app.get('/',c=>c.json({payload:'x'.repeat(1048577)}));match(JSON.tryStringify({payload:'x'},1)){Ok(_)=>{},Err(error)=>{console.log(error.code);}}}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 let hover=session.hover("main.tsr",source,source.find("c.json").unwrap()+3).unwrap();assert!(hover.contains("no implicit one-MiB output ceiling")&&hover.contains("maxBytes"),"{hover}");
 let offset=source.find("JSON.tryStringify(").unwrap()+"JSON.tryStringify(".len();let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":offset}}));assert!(help.to_string().contains("maxBytes"),"{help}");
}
#[test]
fn explicit_http_bound_help_keeps_request_and_context_signatures_equal(){
 let source="import {Request,Context,ClientOptions,Client} from 'std:http';async function inspect(input:Request,c:Mutable<Context>):Task<void>{await input.form(1);await c.req.multipart(2);}function main(){const settings:ClientOptions={bodyLimit:4};const client=new Client(settings);console.log(settings.bodyLimit);}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 for(needle,parameter)in[("input.form(","maxFields?: int"),("c.req.multipart(","maxParts?: int")]{
  let offset=source.find(needle).unwrap()+needle.len();let help=session.request("textDocument/signatureHelp",serde_json::json!({"textDocument":{"uri":session.uri("main.tsr")},"position":{"line":0,"character":offset}}));
  assert_eq!(help["signatures"][0]["parameters"][0]["label"],parameter,"{help}");
  let hover=session.hover("main.tsr",source,offset-2).unwrap();assert!(hover.contains("nonnegative")&&hover.contains("no cardinality cap"),"{hover}");
 }
 let hover=session.hover("main.tsr",source,source.rfind("settings.bodyLimit").unwrap()+10).unwrap();assert!(hover.contains("bodyLimit")&&hover.contains("byte"),"{hover}");
}
#[test]
fn contextual_stream_help_preserves_context_contract(){
 let source="import {Router} from 'std:http';function main(){const app=new Router();app.post('/',async(c)=>{const body=c.req.body();const value=await body.text();return c.text(value);});}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 let offset=source.find("c.req.body()").unwrap()+6;
 let hover=session.hover("main.tsr",source,offset).unwrap_or_default();
 for expected in ["body()", "Context response helpers", "Ownership", "configured"]{assert!(hover.contains(expected),"{hover}");}
 let completion=session.completion("main.tsr",source,offset).unwrap();
 assert!(completion.iter().any(|value|value=="body"),"{completion:?}");
}
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
