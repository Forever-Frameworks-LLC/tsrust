mod support;
#[test]
fn hono_context_order_closures_and_short_circuit_execute(){
 let directory=support::TempDirectory::new();
 let source=support::format_round_trip(include_str!("../../../examples/http-context/main.tsr"));
 support::success(&support::run(support::cli().arg("run").arg(directory.source(&source)),300));
}
#[test]
fn hono_continuation_exclusive_loan_and_single_invocation_reject(){
 for source in [
  "import {Context,ContextNext} from 'std:http';async function middleware(c:Mutable<Context>,next:ContextNext):Task<void>{const pending=next();c.header('x','unsafe');await pending;}function main(){}",
  "import {Context,ContextNext} from 'std:http';async function middleware(c:Mutable<Context>,next:ContextNext):Task<void>{await next();await next();}function main(){}",
  "import {Context} from 'std:http';function invalid(c:Mutable<Context>){return clone(c);}function main(){}",
 ]{
  let directory=support::TempDirectory::new();let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(source))),30);
  assert!(!output.status.success(),"accepted {source}");let error=String::from_utf8_lossy(&output.stderr);assert!(!error.contains("error[import]")&&!error.contains("error[syntax]"),"{error}");
 }
}
