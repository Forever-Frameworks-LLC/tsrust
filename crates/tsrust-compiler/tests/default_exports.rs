mod support;

#[test]
fn default_functions_and_named_exports_link_and_execute() {
 let dir=support::TempDirectory::new();
 for (name,source) in [
  ("named.tsr","export function greeting(): string {return 'hello';} export default greeting;"),
  ("declaration.tsr","export default function twice(value:int):int{return value*2;}"),
  ("anonymous.tsr","export default function(value:int):int{return value+1;}"),
  ("main.tsr","import greet, {greeting} from './named.tsr';import twice from './declaration.tsr';import {default as increment} from './anonymous.tsr';import {strictEqual} from 'std:testing';function main(){strictEqual(greet(),'hello');strictEqual(greeting(),'hello');strictEqual(twice(increment(2)),6);}")
 ] {std::fs::write(dir.0.join(name),support::format_round_trip(source)).unwrap();}
 for release in [false,true] {
  let mut command=support::cli();command.arg("run").arg(dir.0.join("main.tsr"));if release {command.arg("--release");}
  support::success(&support::run(&mut command,60));
 }
}

#[test]
fn defaults_preserve_ownership_and_reject_invalid_exports() {
 let dir=support::TempDirectory::new();
 for (dependency,main,expected) in [
 ("function one():int{return 1;} export default one; export default one;","import f from './dep.tsr';function main(){f();}","module-duplicate"),
 ("export function one():int{return 1;}","import f from './dep.tsr';function main(){f();}","does not export"),
 ("export default missing;","import f from './dep.tsr';function main(){f();}","local function or module binding"),
 ("export default 42;","import f from './dep.tsr';function main(){f();}","module-export"),
 ("export default function take(value:string):string{return value;}","import take from './dep.tsr';function main(){const value='owned';take(value);console.log(value);}","transferred"),
 ] {
  std::fs::write(dir.0.join("dep.tsr"),support::format_round_trip(dependency)).unwrap();std::fs::write(dir.0.join("main.tsr"),support::format_round_trip(main)).unwrap();
  let result=support::run(support::cli().arg("check").arg(dir.0.join("main.tsr")),20);assert!(!result.status.success());
  let message=String::from_utf8_lossy(&result.stderr);assert!(message.contains(expected),"{message}");
 }
}
