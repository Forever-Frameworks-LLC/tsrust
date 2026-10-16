mod support;
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
