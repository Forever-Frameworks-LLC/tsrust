mod support;

#[test]
fn suite_registration_throw_restores_scope() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {describe,test,strictEqual,runTests} from 'std:testing';function main(){let caught=0;try{describe('broken',():void=>{throw 'registration';});}catch(error){if(typeof error==='string'){strictEqual(error,'registration');caught=1;}else{throw error;}}strictEqual(caught,1);test('root',():void=>{strictEqual(3,3);});console.log(runTests());}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    let stdout=String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS root"),"{stdout}");
    assert!(!stdout.contains("PASS broken > root"),"{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn throwing_hooks_run_cleanup_and_later_suites() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {describe,test,beforeEach,afterEach,strictEqual,runTests} from 'std:testing';function main(){describe('broken',():void=>{beforeEach(():void=>{try{throw 'hook';}finally{console.log('hook cleanup');}});afterEach(():void=>{console.log('after');});test('blocked',():void=>{console.log('must not run');});});describe('later',():void=>{test('pass',():void=>{strictEqual(3,3);});});console.log(runTests());}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    let stdout=String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hook cleanup"),"{stdout}");
    assert!(stdout.contains("after"),"{stdout}");
    assert!(stdout.contains("PASS later > pass"),"{stdout}");
    assert!(!stdout.contains("must not run"),"{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn nested_runner_execution_error_is_caught_in_source() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {test,strictEqual,runTests} from 'std:testing';function main(){test('guard',():void=>{let caught=0;try{runTests();}catch(error){strictEqual(error.code,'testing');caught=1;}strictEqual(caught,1);});console.log(runTests());}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("PASS guard"));
    assert!(output.stderr.is_empty());
}

#[test]
fn runner_guards_are_recoverable_in_registered_source_tests() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {test,strictEqual,runTests} from 'std:testing';function main(){test('guards',():void=>{let count=0;try{runTests();}catch(error){strictEqual(error.code,'testing');count=1;}try{test('invalid',():void=>{throw 'must not run';});}catch(error){strictEqual(error.code,'testing');count=count+1;}strictEqual(count,2);});test('later',():void=>{strictEqual(3,3);});console.log(runTests());}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    let stdout=String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS guards"),"{stdout}");
    assert!(stdout.contains("PASS later"),"{stdout}");
    assert!(stdout.contains("2 passed, 0 failed"),"{stdout}");
    assert!(!stdout.contains("PASS invalid"),"{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn runner_result_entry_executes_through_source_completion() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {runTests} from 'std:testing';function main(){try{console.log(runTests());}finally{console.log('finished');}}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("0\nfinished\n"));
    assert!(output.stderr.is_empty());
}

#[test]
fn native_runner_reports_result_errors_and_continues() {
    let directory = support::TempDirectory::new();
    let source = directory.0.join("runner.rs");
    let binary = directory.0.join("runner");
    let runtime = concat!(include_str!("../src/runtime/number_tie.rs"),include_str!("../src/runtime/console.rs"),include_str!("../src/stdlib/testing/runtime/mod.rs"));
    let registration = include_str!("../src/stdlib/testing/runtime/registration.rs");
    std::fs::write(&source, format!(r#"
type String=std::borrow::Cow<'static,str>;
struct TsrTestContext<'p>(std::marker::PhantomData<&'p ()>);
trait ProgramCallback<A,R>{{fn invoke_program(&mut self,args:A,program:&mut TsrTestContext<'_>,registry:&mut TsrTestRegistry)->R;}}
fn tsrust_testing_run_in_scope<R>(body:impl FnOnce()->R)->R {{body()}}
{runtime}
{registration}
fn main(){{
    let mut registry=TsrTestRegistry::default();
    let mut program=TsrTestContext(std::marker::PhantomData);
    TSR_TEST_RUNNING.with(|running|running.set(true));
    assert!(tsrust_testing_register_test_result(&mut registry,&String::Borrowed("invalid"),Box::new(|_,_|panic!("must not invoke"))).is_err());
    assert!(tsrust_testing_run_tests_result(&mut registry,&mut program).is_err());
    TSR_TEST_RUNNING.with(|running|running.set(false));
    tsrust_testing_begin_suite(&mut registry,&String::Borrowed("active"));
    assert!(tsrust_testing_run_tests_result(&mut registry,&mut program).is_err());
    tsrust_testing_end_suite(&mut registry);
    registry.suites[0].entries.push(TsrTestEntry::Test{{name:"typed".into(),callback:Box::new(|_,_|Err("typed failure".to_owned()))}});
    registry.suites[0].entries.push(TsrTestEntry::Test{{name:"later".into(),callback:Box::new(|_,_|Ok(()))}});
    assert_eq!(tsrust_testing_run_tests(&mut registry,&mut program),1);
    // The production registry owns pending callbacks even when never run.
    struct Disposal(std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>, &'static str);
    impl Drop for Disposal {{ fn drop(&mut self) {{ self.0.borrow_mut().push(self.1); }} }}
    let events=std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    {{
        let _global=Disposal(events.clone(), "program");
        let mut pending=TsrTestRegistry::default();
        let capture=Disposal(events.clone(), "callback");
        tsrust_testing_register_test_result(&mut pending,&String::Borrowed("never run"),Box::new(move |_,_| {{ let _=&capture; Ok(()) }})).unwrap();
    }}
    assert_eq!(*events.borrow(),["callback","program"]);
}}
"#)).unwrap();
    let compiled = support::run(support::rustc().arg("--edition=2024").arg(&source).arg("-o").arg(&binary), 60);
    support::success(&compiled);
    let output = support::run(&mut std::process::Command::new(binary), 30);
    support::success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FAIL typed: typed failure"), "{stdout}");
    assert!(stdout.contains("PASS later"), "{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn registered_test_catches_errors_and_runner_continues_after_throw() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {test,strictEqual,runTests} from 'std:testing';function main(){test('handled',():void=>{try{throw 'handled';}catch(error){strictEqual(error,'handled');}});test('failure',():void=>{try{throw 'runner';}finally{console.log('cleanup');}});test('later',():void=>{strictEqual(3,3);});console.log(runTests());}");
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS handled"), "{stdout}");
    assert!(stdout.contains("cleanup"), "{stdout}");
    assert!(stdout.contains("FAIL failure"), "{stdout}");
    assert!(stdout.contains("PASS later"), "{stdout}");
    assert!(stdout.contains("2 passed, 1 failed"), "{stdout}");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
}

#[test]
fn source_callback_errors_cross_abstract_invocation() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/exception-handling/callbacks.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn uncaught_source_error_runs_finally_and_exits_without_panic() {
    let directory = support::TempDirectory::new();
    let path = directory.source("function zero():int{return 0;} function main(){try{const value=7/zero();console.log(value);}finally{console.log(42);}}");
    let output = support::run(support::cli().arg("run").arg(path), 60);
    assert!(!output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "42\n");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("division"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}

#[test]
fn source_runtime_failures_are_caught_and_finally_runs() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/exception-handling/runtime-recovery.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn explicit_errors_catch_finally_and_propagation_match_node() {
    let source = include_str!("fixtures/exception-handling/basic.tsr");
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn finally_preserves_targets_and_overrides_pending_completions() {
    let source = include_str!("fixtures/exception-handling/completions.tsr");
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn test_command_reports_typed_failure_after_finally() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {test} from 'std:testing';test('failure',():void=>{try{throw 'failed';}finally{console.log('finish cleanup');}});");
    let output = support::run(support::cli().arg("test").arg(path), 60);
    assert!(!output.status.success());
    let stdout=String::from_utf8_lossy(&output.stdout);
    let stderr=String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("× failure") && stdout.contains("input.tsr > failure") && stdout.contains("\n   failed\n"),"stdout: {stdout}; stderr: {stderr}");
    assert!(stdout.contains("Tests  0 passed | 1 failed (1)"),"{stdout}");
    assert!(stdout.contains("finish cleanup"),"{stdout}");
    assert!(!stderr.contains("panicked"),"{stderr}");
}

#[test]
fn runtime_error_uses_original_module_coordinates() {
    let directory=support::TempDirectory::new();
    let helper="export function divide():int{return 7/zero();}function zero():int{return 0;}";
    std::fs::write(directory.0.join("helper.tsr"),helper).unwrap();
    let start=helper.find("/zero()").unwrap();
    let source=format!("import {{divide}} from './helper.tsr';import {{strictEqual}} from 'std:testing';function main(){{try{{divide();}}catch(error){{strictEqual(error.source.endsWith('/helper.tsr'),true);strictEqual(error.start,{start});strictEqual(error.end,{});}}}}",start+1);
    let path=directory.source(&source);
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn stored_registration_callback_reports_error_and_continues() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {test,runTests,strictEqual} from 'std:testing';function main(){let callback=():void=>{throw 'stored failure';};test('stored',callback);test('later',():void=>{strictEqual(1,1);});strictEqual(runTests(),1);}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    let stdout=String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FAIL stored: stored failure"),"{stdout}");
    assert!(stdout.contains("PASS later"),"{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn array_registration_callback_reports_error_and_continues() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {test,runTests,strictEqual} from 'std:testing';function main(){const callbacks:(()=>void)[]=[():void=>{},():void=>{throw 'stored failure';}];const callback=callbacks.remove(1);test('stored',callback);test('later',():void=>{strictEqual(1,1);});strictEqual(runTests(),1);}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    let stdout=String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FAIL stored: stored failure"),"{stdout}");
    assert!(stdout.contains("PASS later"),"{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn registered_stored_callback_cannot_be_used_after_transfer() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {test} from 'std:testing';function main(){const callbacks:(()=>void)[]=[():void=>{}];const callback=callbacks.remove(0);test('owned',callback);callback();}");
    let output=support::run(support::cli().arg("check").arg(path),60);
    assert!(!output.status.success());
    let stderr=String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("consumed") || stderr.contains("moved"),"{stderr}");
}

#[test]
fn single_use_stored_callback_rejects_reusable_test_registration() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {test} from 'std:testing';function main(){const text='owned';const callbacks:(Once<Function<string>>)[]=[():string=>{return text;}];const callback=callbacks.remove(0);test('once',callback);}");
    let output=support::run(support::cli().arg("check").arg(path),60);
    assert!(!output.status.success());
    let stderr=String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("callback-contract"),"{stderr}");
    assert!(stderr.contains("single-use") || stderr.contains("Once<Function"),"{stderr}");
}

#[test]
fn prepared_test_entry_runs_cleanup_and_stops_registration() {
    let directory=support::TempDirectory::new();
    let path=directory.0.join("a.test.tsr");
    std::fs::write(&path,support::format_round_trip("try{throw 'registration failed';}finally{console.log('file cleanup');}")).unwrap();
    let emitted=support::run(support::cli().arg("emit").arg(&path),60);
    support::success(&emitted);
    for multiple in [false,true] {
        if multiple {
            std::fs::write(directory.0.join("b.test.tsr"),support::format_round_trip("import {test,strictEqual} from 'std:testing';console.log('later registration');test('later',()=>{strictEqual(1,1);});")).unwrap();
        }
        let output=support::run(support::cli().arg("test").arg(if multiple {&directory.0} else {&path}),60);
        assert!(!output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout),"file cleanup\n");
        let stderr=String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Uncaught registration failed"),"{stderr}");
        assert!(!stderr.contains("panicked"),"{stderr}");
    }
}

#[test]
fn generated_entry_tail_does_not_hide_unreachable_user_statements() {
    let directory=support::TempDirectory::new();
    let path=directory.0.join("unreachable.test.tsr");
    std::fs::write(&path,support::format_round_trip("throw 'registration failed';console.log('unreachable user code');")).unwrap();
    let output=support::run(support::cli().arg("check").arg(&path),30);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unreachable"));
}

#[test]
fn expectation_matcher_failure_is_caught_in_source() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {expect,strictEqual} from 'std:testing';function main(){let seen=0;try{expect(1).toBe(2);}catch(error){strictEqual(error.name,'AssertionError');seen=1;}strictEqual(seen,1);}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn partial_return_cleanup_fixture_preserves_nested_cleanup_and_siblings() {
    let source=support::format_round_trip(include_str!("fixtures/exception-handling/owned-returns.tsr"));
    let directory=support::TempDirectory::new();let path=directory.source(&source);
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
