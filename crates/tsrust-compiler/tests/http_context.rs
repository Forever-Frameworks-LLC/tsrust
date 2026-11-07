mod support;
#[test]
fn contextual_json_has_no_implicit_output_ceiling_and_keeps_explicit_encoder_bounds(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(r#"
import {Router,Client,request} from 'std:http';import {strictEqual} from 'std:testing';
type Payload={payload:string;};
async function main():Task<void>{
 const app=new Router();app.get('/large',c=>c.json({payload:'x'.repeat(1048577)}));
 const server=await app.listen('127.0.0.1:0');const client=new Client();
 const response=await client.send(request('GET','http://'+server.address()+'/large',''));strictEqual(response.status(),200);
 const text=await response.text();strictEqual(text.length,1048591);const value=JSON.parse<Payload>(text);strictEqual(value.payload,'x'.repeat(1048577));
 match(JSON.tryStringify(value,1048576)){Ok(_)=>{throw 'explicit encoder bound ignored';},Err(error)=>{strictEqual(error.code,'output-limit');}}
 await server.shutdown();console.log('unbounded contextual JSON; explicit encoder bound retained');
}
"#);let mut command=support::cli();command.arg("run").arg(directory.source(&source));for name in ["HTTP_PROXY","HTTPS_PROXY","ALL_PROXY","http_proxy","https_proxy","all_proxy"]{command.env_remove(name);}
 let output=support::run(&mut command,300);support::success(&output);assert_eq!(output.stdout,b"unbounded contextual JSON; explicit encoder bound retained\n");
}
#[test]
fn explicit_client_and_form_bounds_keep_optional_alias_contracts(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(r#"
import {Client,ClientOptions,Request,FormField,Router,request} from 'std:http';
import {strictEqual} from 'std:testing';
function input(body:string):Request{const value=request('POST','http://localhost/',body);value.setHeader('content-type','application/x-www-form-urlencoded');return value;}
async function parse(value:Request,maxFields:int|undefined):Task<FormField[]>{return await value.form(maxFields);}
type Parse=Function<Task<FormField[]>,[value:Owned<Request>,maxFields:int|undefined]>;
async function main():Task<void>{
 const options:ClientOptions={bodyLimit:4};const client=new Client(options);strictEqual(options.ca,undefined);
 const alias=parse;const stored:Parse=parse;strictEqual((await alias(input(''),0)).length,0);strictEqual((await stored(input('n=x&'.repeat(128)+'n=x'),129)).length,129);
 let invalid=false;try{new Client({bodyLimit:-1});}catch(error){strictEqual(error.code,'options');invalid=true;}strictEqual(invalid,true);
 const app=new Router();app.post('/form',async c=>{try{await c.req.form(-1);}catch(error){strictEqual(error.code,'options');}const fields=await c.req.form(1);const field=fields.pop()!;return c.text(field.value);});
 const server=await app.listen('127.0.0.1:0');
 const wire=request('POST','http://'+server.address()+'/form','n=ok');wire.setHeader('content-type','application/x-www-form-urlencoded');strictEqual(await (await client.send(wire)).text(),'ok');await server.shutdown();
 console.log('explicit HTTP bounds');
}
"#);let path=directory.source(&source);let mut command=support::cli();command.arg("run").arg(path);
 // Keep proxy observations local to this child; no process-wide environment mutation.
 for name in ["HTTP_PROXY","HTTPS_PROXY","ALL_PROXY","http_proxy","https_proxy","all_proxy"]{command.env_remove(name);}
 let output=support::run(&mut command,300);support::success(&output);assert_eq!(output.stdout,b"explicit HTTP bounds\n");
}
#[test]
fn explicit_http_bounds_reject_wrong_types_arity_and_pending_context_reuse(){
 for body in ["await input.form('1');","await input.multipart(1,2);","const pending=c.req.form(1);c.status(201);await pending;","new Client({bodyLimit:'4'});"]{
  let source=format!("import {{Request,Context,Client}} from 'std:http';async function inspect(input:Request,c:Mutable<Context>):Task<void>{{{body}}}function main(){{}}");
  let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(&source))),30);assert!(!output.status.success(),"accepted {body}");
 }
}
#[test]
fn request_json_decoding_and_context_ownership_execute(){
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(include_str!("../../../examples/http-context/json.test.tsr"));
 support::success(&support::run(support::cli().arg("test").arg(directory.source(&source)),600));
}
#[test]
fn request_json_rejects_unchecked_targets_arguments_and_reused_requests(){
 for source in [
  "import {Request} from 'std:http';async function read(input:Request):Task<void>{await input.json(1);}function main(){}",
  "import {Request} from 'std:http';async function read(input:Request):Task<void>{await input.json<string>();}function main(){}",
  "import {Request} from 'std:http';async function read(input:Request):Task<void>{const value:string=await input.json();}function main(){}",
  "import {Request} from 'std:http';async function read(input:Request):Task<void>{await input.json();input.body();}function main(){}",
  "import {Context} from 'std:http';async function read(c:Mutable<Context>):Task<void>{const pending=c.req.json();c.status(201);await pending;}function main(){}",
 ]{
  let directory=support::TempDirectory::new();let file=directory.source(&support::format_round_trip(source));
  let output=support::run(support::cli().arg("check").arg(file),30);
  assert!(!output.status.success(),"accepted {source}");
  let error=String::from_utf8_lossy(&output.stderr);
  assert!(!error.contains("panicked")&&!error.contains("error[import]")&&!error.contains("error[syntax]"),"{error}");
 }
}
#[test]
fn contextual_body_stream_preserves_metadata_and_response_helpers(){
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(include_str!("../../../examples/http-context/body-stream.test.tsr"));
 support::success(&support::run(support::cli().arg("test").arg(directory.source(&source)),600));
}
#[test]
fn hono_context_order_closures_and_short_circuit_execute(){
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(include_str!("../../../examples/http-context/main.tsr"));
 support::success(&support::run(support::cli().arg("run").arg(directory.source(&source)),300));
}
#[test]
fn hono_continuation_exclusive_loan_and_single_invocation_reject(){
 for source in [
  "import {Router} from 'std:http';function main(){const app=new Router();app.get('/');}",
  "import {Router} from 'std:http';function main(){const app=new Router();app.use();}",
  "import {Context,ContextNext} from 'std:http';async function middleware(c:Mutable<Context>,next:ContextNext):Task<void>{const pending=next();c.header('x','unsafe');await pending;}function main(){}",
  "import {Context,ContextNext} from 'std:http';async function middleware(c:Mutable<Context>,next:ContextNext):Task<void>{await next();await next();}function main(){}",
  "import {Context} from 'std:http';function invalid(c:Mutable<Context>){return clone(c);}function main(){}",
 ]{
  let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(source))),30);
  assert!(!output.status.success(),"accepted {source}");let error=String::from_utf8_lossy(&output.stderr);assert!(!error.contains("error[import]")&&!error.contains("error[syntax]"),"{error}");
 }
}

#[test]
fn async_arrow_contracts_reject_invalid_await_and_results(){
 for source in [
  "function main(){const invalid=async (value:int):int=>value;}",
  "import {delay} from 'std:futures';async function main():Task<void>{const invalid=()=>{await delay(1);};}",
  "import {Router} from 'std:http';function main(){const app=new Router();app.use(async(c,next)=>{const pending=next();c.header('x','unsafe');await pending;});}",
  "import {Router} from 'std:http';function main(){const app=new Router();app.use(async(c,next)=>{await next();await next();});}",
 ]{
  let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(source))),30);
  assert!(!output.status.success(),"accepted {source}");
 }
}

#[test]
fn router_state_cookie_and_request_isolation_execute(){
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(include_str!("../../../examples/http-state/router.tsr"));
 support::success(&support::run(support::cli().arg("run").arg(directory.source(&source)),300));
}

#[test]
fn router_state_and_variable_ownership_reject(){
 for source in [
  "import {Router} from 'std:http';type State={name:string;};function main(){const app=new Router<State>({name:'x'});app.get('/',c=>{c.env.name='changed';return c.text('bad');});}",
  "import {Router} from 'std:http';type Variables={name:string;};function main(){const app=new Router<null,Variables>();app.get('/',c=>{c.set('unknown','x');return c.text('bad');});}",
  "import {Router} from 'std:http';type Variables={name:string;};function main(){const app=new Router<null,Variables>();app.get('/',c=>{c.set('name',1);return c.text('bad');});}",
  "import {Router} from 'std:http';type Variables={name:string;};function main(){const app=new Router<null,Variables>();app.get('/',c=>{c.set('name','one');const name=c.get('name');c.set('name','two');console.log(name);return c.text('bad');});}",
  "import {Router} from 'std:http';function main(){const app=new Router();app.get('/',c=>{return c.res;});}",
 ]{
  let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(source))),30);
  assert!(!output.status.success(),"accepted {source}");let error=String::from_utf8_lossy(&output.stderr);assert!(!error.contains("panicked")&&!error.contains("error[import]")&&!error.contains("error[syntax]"),"{error}");
 }
}

#[test]
fn context_json_accepts_inferred_objects_and_preserves_shared_env() {
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(r#"
 import {Router,Client,request} from 'std:http';
 import {strictEqual} from 'std:testing';
 type State={greeting:string;};
 async function main():Task<void>{
   const app=new Router<State>({greeting:"hello".repeat(2)});
   app.use(async(c,next)=>{c.header("x-before","entry");await next();c.header("x-after","exit");});
   app.get("/",c=>{return c.json({greeting:clone(c.env.greeting),nested:{ready:true}});});
   app.use('/private/*',async(c,next)=>{const authorization=c.req.header('authorization');if(authorization!=='Bearer development-only'){return c.json({error:'Unauthorized'},401);}await next();});
   app.use('/private/*',async(c,next)=>{const nested=():int=>{return 7;};c.header('x-nested',String(nested()));if(c.req.header('x-stop')==='yes'){return;}return await next();});
   app.get('/private/profile',c=>c.json({name:'Ada'}));
   const server=await app.listen("127.0.0.1:0");
   const client=new Client();
   const response=await client.send(request("GET","http://"+server.address()+"/",""));
   strictEqual(response.status(),200,"root status");
   strictEqual(response.header("x-before"),"entry","before header");strictEqual(response.header("x-after"),"exit","after header");
   strictEqual(await response.text(),'{"greeting":"hellohello","nested":{"ready":true}}');
   const denied=await client.send(request('GET','http://'+server.address()+'/private/profile',''));strictEqual(denied.status(),401,'denied status');strictEqual(await denied.text(),'{"error":"Unauthorized"}');
   const allowed=request('GET','http://'+server.address()+'/private/profile','');allowed.setHeader('authorization','Bearer development-only');
   const profile=await client.send(allowed);strictEqual(profile.status(),200,'profile status');strictEqual(profile.header('x-nested'),'7','nested closure header');strictEqual(await profile.text(),'{"name":"Ada"}');
   const stopped=request('GET','http://'+server.address()+'/private/profile','');stopped.setHeader('authorization','Bearer development-only');stopped.setHeader('x-stop','yes');const stopped_response=await client.send(stopped);strictEqual(stopped_response.status(),500,'unfinished middleware status');await stopped_response.text();
   await server.shutdown();console.log("inferred JSON passed");
 }
 "#);
 let output=support::run(support::cli().arg("run").arg(directory.source(&source)),300);
 support::success(&output);assert_eq!(output.stdout,b"inferred JSON passed\n");
}

#[test]
fn http_optional_arguments_keep_defaults_values_and_evaluation_order(){
 let source=r#"
import {Router,Client,ClientOptions,TlsOptions,CookieOptions,request,setCookie,deleteCookie} from 'std:http';
import {strictEqual} from 'std:testing';
function absent(trace:int[]):undefined{trace.push(1);return undefined;}
function address(trace:int[]):string{trace.push(2);return '127.0.0.1:0';}
function directory():string{return '.';}
function cookieName():string{return 'theme';}
async function main():Task<void>{
 const trace:int[]=[];const ignored=new Client(absent(trace));strictEqual(trace.length,1);
 for(const supplied of [false,true]){
  const options:ClientOptions|undefined=undefined;const client=new Client(options);
  const app=new Router();const missing:CookieOptions|undefined=supplied?{path:'/'}:undefined;
  const status:int|undefined=supplied?201:undefined;
  app.get('/text',c=>{c.status(202);setCookie(c,cookieName(),'light',missing);deleteCookie(c,'old',undefined);return c.text('text',status);});
  app.get('/json',c=>c.json({ready:true},undefined));app.get('/html',c=>c.html('<b>ready</b>',undefined));
  const fallback:string|undefined=supplied?'index.html':undefined;app.serveStatic('/static',directory(),fallback);
  const tls:TlsOptions|undefined=undefined;const server=await app.listen('127.0.0.1:0',tls);
  const base='http://'+server.address();
  const text=await client.send(request('GET',base+'/text',''));strictEqual(text.status(),supplied?201:202);strictEqual(text.header('set-cookie'),'theme=light; Path=/');strictEqual(await text.text(),'text');
  const json=await client.send(request('GET',base+'/json',''));strictEqual(json.status(),200);strictEqual(await json.text(),'{"ready":true}');
  const html=await client.send(request('GET',base+'/html',''));strictEqual(html.status(),200);strictEqual(await html.text(),'<b>ready</b>');
  const input=request('GET',base+'/static/missing','');input.setHeader('accept','text/html');const file=await client.send(input);strictEqual(file.status(),supplied?200:404);
  await server.shutdown();
 }
 const app=new Router();app.get('/',c=>c.text('ok'));const server=await app.listen(address(trace),absent(trace));strictEqual(trace.join(','),'1,2,1');await server.shutdown();
}
"#;
 let dir=support::TempDirectory::new();std::fs::write(dir.0.join("index.html"),"static").unwrap();let path=dir.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("run").arg(path).current_dir(&dir.0),300);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn http_optional_arguments_reject_invalid_present_values(){
 for body in [
  "const client=new Client(null);",
  "const client=new Client(1);",
  "const app=new Router();app.get('/',c=>c.text('bad',null));",
  "const app=new Router();app.get('/',c=>c.html('bad','201'));",
  "const app=new Router();app.get('/',c=>c.json({ready:true},null));",
  "const app=new Router();app.serveStatic('/files','.',null);",
  "const app=new Router();await app.listen('127.0.0.1:0',null);",
  "const app=new Router();app.get('/',c=>{setCookie(c,'theme','x',null);return c.text('bad');});",
 ]{
  let source=format!("import {{Router,Client,setCookie}} from 'std:http';async function main():Task<void>{{{body}}}");
  let dir=support::TempDirectory::new();let path=dir.source(&support::format_round_trip(&source));let output=support::run(support::cli().arg("check").arg(path),30);assert!(!output.status.success(),"accepted {source}");assert!(String::from_utf8_lossy(&output.stderr).contains("error[type]"),"{}",String::from_utf8_lossy(&output.stderr));
 }
}
