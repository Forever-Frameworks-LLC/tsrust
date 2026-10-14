mod support;

fn format_project(directory: &support::TempDirectory) {
    support::success(&support::run(support::cli().arg("format").arg(&directory.0), 30));
    let first = std::fs::read_to_string(directory.0.join("input.tsr")).unwrap();
    support::success(&support::run(support::cli().arg("format").arg(&directory.0), 30));
    assert_eq!(first, std::fs::read_to_string(directory.0.join("input.tsr")).unwrap());
}

#[test]
fn imported_program_state_is_borrowed_by_hooks_and_tests_at_invocation() {
    let directory = support::TempDirectory::new();
    std::fs::write(directory.0.join("library.tsr"), "const greeting='hello';let count=0;export function size():int{return greeting.length;}export function reset():void{count=10;}export function increment():void{count=count+1;}export function current():int{return count;}").unwrap();
    let path = directory.source("import {test,describe,beforeAll,afterAll,strictEqual} from 'std:testing';import {size,reset,increment,current} from './library.tsr';describe('program',():void=>{beforeAll(():void=>{reset();});test('first',():void=>{strictEqual(size(),5);strictEqual(current(),10);increment();});describe('nested',():void=>{test('second',():void=>{strictEqual(current(),11);increment();});});afterAll(():void=>{strictEqual(current(),12);});});");
    format_project(&directory);
    let output = support::run(support::cli().arg("test").arg(path), 90);
    support::success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 passed"), "{stdout}");
    assert!(stdout.contains("0 failed"), "{stdout}");
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn program_state_remains_available_between_registry_runs() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {test,runTests,strictEqual} from 'std:testing';let value=0;function read():int{return value;}function write(next:int):void{value=next;}function main():void{test('first',():void=>{write(1);});strictEqual(runTests(),0);strictEqual(read(),1);write(2);test('second',():void=>{strictEqual(read(),2);write(3);});strictEqual(runTests(),0);strictEqual(read(),3);}");
    format_project(&directory);
    let output = support::run(support::cli().arg("run").arg(path), 90);
    support::success(&output);
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn registered_program_callbacks_keep_reentrant_runner_errors_recoverable() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {test,runTests,strictEqual} from 'std:testing';let count=0;function main():void{test('guards',():void=>{try{runTests();}catch(error){strictEqual(error.code,'testing');count=count+1;}try{test('invalid',():void=>{count=99;});}catch(error){strictEqual(error.code,'testing');count=count+1;}});test('later',():void=>{strictEqual(count,2);});strictEqual(runTests(),0);strictEqual(count,2);}");
    format_project(&directory);
    let output = support::run(support::cli().arg("run").arg(path), 90);
    support::success(&output);
    assert!(!String::from_utf8_lossy(&output.stdout).contains("PASS invalid"));
    assert!(output.stderr.is_empty(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn registry_does_not_admit_borrowed_inputs_or_hide_execution_conflicts() {
    let directory = support::TempDirectory::new();
    for (source, code) in [
        ("import {test} from 'std:testing';function install(value:ReadOnly<string>):void{test('bad',():void=>{console.log(value);});}function main(){install('text');}", "callback-escape"),
        ("import {test} from 'std:testing';function install(callback:Mutable<Function<void>>):void{test('bad',():void=>{callback();});}function main(){install(():void=>{});}", "callback-escape"),
        ("const value='hello';function own(callback:Owned<Function<void>>):void{callback();}function main(){own(():void=>{console.log(value);});}", "callback-escape"),
        ("import {test,runTests} from 'std:testing';const values=[1];function inspect(input:ReadOnly<int[]>):void{runTests();console.log(input.length);}function main(){test('write',():void=>{values.push(2);});inspect(values);}", "conflicting-access"),
    ] {
        let path = directory.source(source);
        format_project(&directory);
        let output = support::run(support::cli().arg("check").arg(&path), 30);
        assert!(!output.status.success(), "{source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(&format!("error[{code}]")), "{stderr}");
        assert!(stderr.contains(path.to_str().unwrap()), "{stderr}");
        assert!(!stderr.contains(" --> std:testing:"), "{stderr}");
    }
}

#[test]
fn immediate_suite_borrows_locals_and_registered_callback_owns_local_values() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {describe,test,runTests,strictEqual} from 'std:testing';function main(){let value=0;describe('local',():void=>{value=2;const owned=[7];test('owned',():void=>{strictEqual(owned[0],7);});});strictEqual(value,2);strictEqual(runTests(),0);}");
    format_project(&directory);
    support::success(&support::run(support::cli().arg("run").arg(path), 90));
}

#[test]
fn registry_catalog_examples_and_program_state_example_execute() {
    let catalog: serde_json::Value = serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    for entry in catalog["std:testing"].as_array().unwrap().iter().filter(|entry| matches!(entry["name"].as_str(), Some("test" | "it" | "beforeAll" | "afterAll" | "beforeEach" | "afterEach" | "before" | "after" | "describe" | "suite"))) {
        for example in entry["examples"].as_array().unwrap() {
            let directory = support::TempDirectory::new();
            let path = directory.source(example["source"].as_str().unwrap());
            format_project(&directory);
            let output = support::run(support::cli().arg("run").arg(path), 90);
            support::success(&output);
            assert_eq!(String::from_utf8(output.stdout).unwrap(), example["output"].as_str().unwrap(), "{}", entry["name"]);
        }
    }
    let directory = support::TempDirectory::new();
    std::fs::write(directory.0.join("program-state.tsr"), include_str!("../../../examples/testing/program-state.tsr")).unwrap();
    let path = directory.source(include_str!("../../../examples/testing/program-state.test.tsr"));
    format_project(&directory);
    support::success(&support::run(support::cli().arg("test").arg(path), 90));
}

#[test]
fn named_registration_callback_supports_direct_and_suite_invocation() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {suite,test,runTests,strictEqual} from 'std:testing';function main(){let count=0;{const registration=():void=>{count+=1;test('same',():void=>{});};registration();suite('second',registration);strictEqual(runTests(),0);}strictEqual(count,2);}");
    format_project(&directory);
    support::success(&support::run(support::cli().arg("run").arg(path), 90));
}
