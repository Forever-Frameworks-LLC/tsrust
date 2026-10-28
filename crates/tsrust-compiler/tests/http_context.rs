mod support;
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
