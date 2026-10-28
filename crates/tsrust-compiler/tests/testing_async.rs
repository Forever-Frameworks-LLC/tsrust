mod support;
use tsrust_compiler::project::{FileSourceProvider,load_project,check_graph};

const ARROW:&str="import {test,runTests,strictEqual} from 'std:testing';import {delay} from 'std:futures';function main(){test('arrow',async()=>{await delay(0);strictEqual(7,7);});strictEqual(runTests(),0);}";
const NAMED:&str="import {test,runTests,strictEqual} from 'std:testing';import {delay} from 'std:futures';async function work():Task<void>{await delay(0);strictEqual(7,7);}function main(){const alias=work;test('named',work);test('alias',alias);strictEqual(runTests(),0);}";
const TYPED:&str="import {test,runTests,strictEqual} from 'std:testing';import {delay} from 'std:futures';function main(){const action:Function<Task<void>>=async():Task<void>=>{await delay(0);strictEqual(7,7);};test('typed',action);strictEqual(runTests(),0);}";
const ALIAS:&str="import {test,runTests,strictEqual} from 'std:testing';import {delay} from 'std:futures';async function main():Task<void>{const action=async()=>{await delay(0);strictEqual(7,7);};await action();test('alias',action);strictEqual(runTests(),0);}";
const OWNED_HOOK:&str="import {test,runTests,strictEqual,beforeEach} from 'std:testing';import {delay} from 'std:futures';function main(){const items=[1];let calls=1;beforeEach(async()=>{await delay(0);calls+=1;items.push(2);strictEqual(items.length,calls);});test('first',async()=>{});test('second',async()=>{});strictEqual(runTests(),0);}";
const EVALUATED_FACTORY:&str="import {test,runTests,strictEqual} from 'std:testing';import {delay} from 'std:futures';let constructions=0;function make():Function<Task<void>>{constructions+=1;return async()=>{await delay(0);};}function main(){test('factory',make());strictEqual(constructions,1);strictEqual(runTests(),0);strictEqual(constructions,1);}";
const GLOBAL_FACTORY:&str="import {test,runTests,strictEqual} from 'std:testing';import {delay} from 'std:futures';let value=0;async function main():Task<void>{const action=async()=>{await delay(0);value+=1;};await action();test('reuse',action);strictEqual(runTests(),0);strictEqual(value,2);}";
const ALIASES:&str="import {before,after,it,runTests,strictEqual} from 'std:testing';import {delay} from 'std:futures';let count=0;function main(){before(async()=>{await delay(0);count=1;});after(async()=>{await delay(0);strictEqual(count,2);count=3;});it('aliases',async()=>{await delay(0);strictEqual(count,1);count=2;});strictEqual(runTests(),0);strictEqual(count,3);}";
const COPIED_SCALAR:&str="import {beforeEach,test,runTests,strictEqual} from 'std:testing';import {delay} from 'std:futures';function main(){let count=0;beforeEach(async()=>{await delay(0);count+=1;if(count===2){console.log('second count');}});test('one',async()=>{});test('two',async()=>{});strictEqual(runTests(),0);}";
const MIXED_STATE:&str="import {beforeEach,test,runTests,strictEqual} from 'std:testing';import {delay} from 'std:futures';let total=0;function main(){let count=0;beforeEach(async()=>{await delay(0);count+=1;total+=1;strictEqual(count,total);});test('one',async()=>{});test('two',async()=>{});strictEqual(runTests(),0);strictEqual(total,2);}";
const SHARED_STATE:&str=r#"
import {test,runTests,strictEqual,beforeAll,beforeEach,afterEach,afterAll} from 'std:testing';
import {delay} from 'std:futures';
let count=0;
function main(){
 beforeAll(async()=>{await delay(0);count=1;});
 beforeEach(async()=>{await delay(0);count+=1;});
 afterEach(async()=>{await delay(0);count+=1;});
 afterAll(async()=>{await delay(0);count+=1;});
 test('state',async()=>{await delay(0);strictEqual(count,2);});
 strictEqual(runTests(),0);strictEqual(count,4);
}
"#;

fn checked(source:&str)->String {
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap();
    let checked=check_graph(graph).unwrap_or_else(|error|panic!("{}: {} at {:?}",error.code,error.message,error.location));
    assert!(checked.warnings().is_empty());
    checked.emit_rust()
}

#[test]
fn async_registration_checks_arrows_named_aliases_and_typed_factories(){
    for source in [
        ARROW,
        NAMED,
        TYPED,
        ALIAS,
    ] {
        let rust=checked(source);
        assert!(rust.contains("TsrTestAsyncCallback for Closure"));
    }
}

#[test]
fn async_hooks_and_tests_borrow_module_state_until_await_completes(){
    checked(SHARED_STATE);
}

#[test]
fn async_hook_factories_retain_owned_captures_for_repeated_invocation(){
    checked(OWNED_HOOK);
}

#[test]
fn registration_evaluates_factory_expressions_before_test_execution(){
    checked(EVALUATED_FACTORY);
}

#[test]
fn async_global_factory_keeps_its_task_result_at_other_calls(){
    checked(GLOBAL_FACTORY);
}

#[test]
fn async_registration_keeps_result_suite_and_borrowing_rejections(){
    for source in [
        "import {test} from 'std:testing';function main(){test('wrong',async()=>{return 7;});}",
        "import {describe} from 'std:testing';function main(){describe('wrong',async()=>{});}",
        "import {test} from 'std:testing';async function work():Task<void>{}function main(){test('task',work());}",
        "import {describe} from 'std:testing';describe('early',()=>{console.log(value);});let value=1;",
        "import {test} from 'std:testing';import {delay} from 'std:futures';function register(value:ReadOnly<string>):void{test('borrow',async()=>{await delay(0);console.log(value);});}function main(){const value='owned'.repeat(2);register(value);}",
        "import {test,runTests} from 'std:testing';import {delay} from 'std:futures';let value=0;async function main():Task<void>{const action=async()=>{await delay(0);value+=1;};const pending=action();value+=1;await pending;test('reuse',action);runTests();}",
    ] {
        let directory=support::TempDirectory::new();let formatted=support::format_round_trip(source);let path=directory.source(&formatted);
        let graph=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap();
        let error=check_graph(graph).err().unwrap_or_else(||panic!("unexpected admission: {source}"));
        if source.contains("const pending=action()") {
            let pending=formatted.find("const pending").unwrap();
            let write=pending+formatted[pending..].find("value").unwrap();
            assert!(error.location.span.start<=write+5 && error.location.span.end>=write,"loan rejection must identify the write while the task is pending: {error:?}");
        }
    }
}

#[test]
fn native_async_registration_preserves_factories_state_and_hook_environments(){
    let directory=support::TempDirectory::new();
    let mut rust=String::new();let mut calls=String::new();let mut expected=String::new();
    let mut cases=vec![
        (ARROW,"PASS arrow\n1 passed, 0 failed, 0 skipped\n"),
        (NAMED,"PASS named\nPASS alias\n2 passed, 0 failed, 0 skipped\n"),
        (TYPED,"PASS typed\n1 passed, 0 failed, 0 skipped\n"),
        (ALIAS,"PASS alias\n1 passed, 0 failed, 0 skipped\n"),
        (SHARED_STATE,"PASS state\n1 passed, 0 failed, 0 skipped\n"),
        (OWNED_HOOK,"PASS first\nPASS second\n2 passed, 0 failed, 0 skipped\n"),
        (EVALUATED_FACTORY,"PASS factory\n1 passed, 0 failed, 0 skipped\n"),
        (GLOBAL_FACTORY,"PASS reuse\n1 passed, 0 failed, 0 skipped\n"),
        (ALIASES,"PASS aliases\n1 passed, 0 failed, 0 skipped\n"),
        (COPIED_SCALAR,"PASS one\nsecond count\nPASS two\n2 passed, 0 failed, 0 skipped\n"),
        (MIXED_STATE,"PASS one\nPASS two\n2 passed, 0 failed, 0 skipped\n"),
    ];
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    for entry in catalog["std:testing"].as_array().unwrap() {
        if matches!(entry["name"].as_str(),Some("test"|"it"|"beforeAll"|"afterAll"|"beforeEach"|"afterEach"|"before"|"after"|"runTests")) {
            let example=entry["examples"].as_array().unwrap().last().unwrap();
            cases.push((example["source"].as_str().unwrap(),example["output"].as_str().unwrap()));
        }
    }
    for (index,(source,output)) in cases.into_iter().enumerate(){
        rust.push_str(&format!("mod native_{index}{{{}\npub fn verify(){{main();}}\n}}\n",checked(source)));
        calls.push_str(&format!("native_{index}::verify();"));expected.push_str(output);
    }
    rust.push_str(&format!("fn main(){{{calls}}}"));
    let input=directory.0.join("async-batch.rs");let executable=directory.0.join(format!("async-batch{}",std::env::consts::EXE_SUFFIX));
    std::fs::write(&input,rust).unwrap();
    support::success(&support::run(support::rustc().arg("--edition=2024").arg("--crate-name=testing_async_batch").arg(&input).arg("-o").arg(&executable),180));
    let output=support::run(std::process::Command::new(executable).env("TSRUST_COLOR","never").env("TSRUST_TEST_REPORTER","plain"),30);
    support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
}

#[test]
fn test_command_awaits_http_setup_test_and_teardown(){
    let directory=support::TempDirectory::new();let path=directory.0.join("http.test.tsr");
    std::fs::write(&path,support::format_round_trip(include_str!("fixtures/testing-async/http.test.tsr"))).unwrap();
    let output=support::run(support::cli().env("TSRUST_COLOR","never").env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),600);
    support::success(&output);assert_eq!(output.stdout,b"PASS native HTTP\n1 passed, 0 failed, 0 skipped\n");
}

#[test]
fn test_command_reuses_its_default_project_cache(){
    let directory=support::TempDirectory::new();let path=directory.0.join("cache.test.tsr");
    std::fs::write(&path,support::format_round_trip("import {test} from 'std:testing';test('cached',()=>{});")).unwrap();
    for expected in ["cache=miss","cache=hit"] {
        let output=support::run(support::cli().current_dir(&directory.0).env_remove("TSRUST_CACHE_DIR")
            .env("TSRUST_BUILD_TIMINGS","1").env("TSRUST_COLOR","never").env("TSRUST_TEST_REPORTER","plain").arg("test").arg(&path),60);
        support::success(&output);assert_eq!(output.stdout,b"PASS cached\n1 passed, 0 failed, 0 skipped\n");
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected),"{}",String::from_utf8_lossy(&output.stderr));
    }
    assert!(directory.0.join("target/tsrust-native").is_dir());
}

#[test]
fn test_command_reports_awaited_failures_and_finishes_teardown(){
    let directory=support::TempDirectory::new();let path=directory.0.join("failures.test.tsr");
    std::fs::write(&path,support::format_round_trip(include_str!("fixtures/testing-async/failures.test.tsr"))).unwrap();
    let output=support::run(support::cli().env("TSRUST_COLOR","never").env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),180);
    assert_eq!(output.status.code(),Some(1));let text=String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("FAIL throw: awaited failure"),"{text}\n{}",String::from_utf8_lossy(&output.stderr));
    assert!(text.contains("FAIL assertion: AssertionError:"),"{text}");
    assert!(text.contains("later body\nPASS later"),"{text}");assert!(text.contains("cleanup done"),"{text}");
    assert!(text.ends_with("1 passed, 2 failed, 0 skipped\n"),"{text}");
}

#[test]
fn test_files_keep_shared_owners_and_requested_initialization_order(){
    let directory=support::TempDirectory::new();let mut paths=Vec::new();
    for (name,initial) in [("second",2),("first",10)] {
        let source=format!("import {{describe,beforeAll,test,strictEqual}} from 'std:testing';import {{delay}} from 'std:futures';function initial():int{{console.log('{name} init');return {initial};}}let shared=initial();try{{describe('failed-registration',():void=>{{throw 'registration';}});}}catch{{}}describe('group-{name}',()=>{{strictEqual(shared,{initial});console.log('{name} register');beforeAll(async()=>{{await delay(0);shared+=1;}});test('{name}',async()=>{{await delay(0);strictEqual(shared,{});}});}});",initial+1);
        let path=directory.0.join(format!("{name}.test.tsr"));std::fs::write(&path,support::format_round_trip(&source)).unwrap();paths.push(path);
    }
    let output=support::run(support::cli().env("TSRUST_COLOR","never").env("TSRUST_TEST_REPORTER","plain").arg("test").args(paths),180);
    support::success(&output);assert_eq!(output.stdout,b"second init\nsecond register\nfirst init\nfirst register\nPASS group-second > second\nPASS group-first > first\n2 passed, 0 failed, 0 skipped\n");
}
