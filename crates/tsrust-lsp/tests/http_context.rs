mod support;
#[test]
fn hono_context_header_and_route_help(){
 let source="import {Hono,Context,ContextNext} from 'std:http';async function middleware(c:Mutable<Context>,next:ContextNext):Task<void>{c.header('x-before','yes');await next();c.header('x-after','yes');}function main(){const app=new Hono();app.use('/',middleware);app.get('/',async (c,next)=>{await next();},c=>c.text('ok'));}";
 let mut session=support::session::Session::start(&[]);session.open("main.tsr",source);session.settle();
 for (needle,expected) in [("c.header","outgoing header"),("app.use","registration order"),("c.text","response")]{let offset=source.find(needle).unwrap()+needle.find('.').unwrap()+1;let hover=session.hover("main.tsr",source,offset).unwrap_or_default();assert!(hover.contains(expected),"{needle}: {hover}");}
}
