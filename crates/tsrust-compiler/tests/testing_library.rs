mod support;

#[test]
fn runner_runtime_boundary_preserves_sync_programs_and_existing_async_main() {
    for (source, asynchronous, expected) in [
        ("import {test,expect,runTests} from 'std:testing';function main(){test('boundary',():void=>{expect(7).toBe(7);});expect(runTests()).toBe(0);}", false,
         "PASS boundary\n1 passed, 0 failed, 0 skipped\n"),
        ("import {test,expect,runTests} from 'std:testing';import {delay} from 'std:futures';async function main():Task<void>{await delay(1);test('boundary',():void=>{expect(7).toBe(7);});expect(runTests()).toBe(0);}", true,
         "PASS boundary\n1 passed, 0 failed, 0 skipped\n"),
    ] {
        let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
        let checked=tsrust_compiler::project::check_graph(tsrust_compiler::project::load_project(&tsrust_compiler::project::FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap();
        let emitted=checked.emit_rust();
        assert_eq!(emitted.contains("mod tsrust_native_futures {"),asynchronous);
        assert_eq!(emitted.contains("trait TsrTestAsyncCallback"),asynchronous);
        let output=support::run(support::cli().env("TSRUST_COLOR","never").env("TSRUST_TEST_REPORTER","plain").arg("run").arg(path),120);
        support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
    }
}

#[test]
fn node_assert_strict_exports_accept_values_and_optional_messages() {
    let directory = support::TempDirectory::new();
    let path = directory.source(r#"
import { ok, strictEqual, notStrictEqual } from "std:testing";
function main() {
    ok(true);
    strictEqual(3, 3, "integer equality");
    strictEqual("é", "é");
    notStrictEqual(false, true);
    console.log("assertions passed");
}
"#);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(output.stdout, b"assertions passed\n");
}

#[test]
fn strict_assertions_preserve_nan_signed_zero_and_owned_inputs() {
    let directory = support::TempDirectory::new();
    let path = directory.source(r#"
import { ok, strictEqual, notStrictEqual } from "std:testing";
function main() {
    strictEqual(Number.NaN, Number.NaN);
    notStrictEqual(0.0, -0.0);
    strictEqual(null, null);
    strictEqual(undefined, undefined);
    const text="é".repeat(2);
    strictEqual(text, "éé");
    ok(text);
    console.log(text);
}
"#);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(output.stdout, "éé\n".as_bytes());
}

#[test]
fn failing_assertion_stops_execution_and_reports_custom_message() {
    let directory = support::TempDirectory::new();
    let path = directory.source(r#"
import { strictEqual } from "std:testing";
function main(){ strictEqual(1,2,"wrong sum"); console.log("unreachable"); }
"#);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("AssertionError: wrong sum"));
}

#[test]
fn generic_result_callable_field_can_be_invoked_directly() {
    support::execute(r#"
type Expectation<T>={toBe:Function<void,[expected:T]>;};
function expect<T>(actual:T):Expectation<T>{const saved=actual;return {toBe:(expected:T):void=>{console.log(Object.is(saved,expected));}};}
function main(){expect(3).toBe(3);}
"#, "true\n", None);
}

#[test]
fn generic_callable_read_access_copies_scalar_specializations() {
    support::execute(r#"
type Matcher<T>={match:Function<boolean,[expected:ReadOnly<T>]>;};
function matcher<T>(actual:T):Matcher<T>{return {match:(expected:T):boolean=>{return Object.is(actual,expected);}};}
function main(){console.log(matcher(3).match(3),matcher("a").match("a"));}
"#, "true true\n", None);
}

#[test]
fn concrete_scalar_callable_access_wrapper_remains_invalid() {
    let error = tsrust_compiler::check("type Matcher={match:Function<boolean,[expected:ReadOnly<int>]>;};function main(){}").err().unwrap();
    assert_eq!(error.code,"callback-access");
}

#[test]
fn strict_assertions_borrow_array_and_record_identity() {
    let directory = support::TempDirectory::new();
    let path = directory.source(r#"
import { strictEqual, notStrictEqual, ok } from "std:testing";
type Row={name:string;};
function main(){
    const values=[1,2];const other=[1,2];
    strictEqual(values,values);notStrictEqual(values,other);ok(values);
    const row:Row={name:"a"};const otherRow:Row={name:"a"};
    strictEqual(row,row);notStrictEqual(row,otherRow);ok(row);
    console.log(values.length,row.name);
}
"#);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(output.stdout,b"2 a\n");
}

#[test]
fn vitest_expect_to_be_supports_direct_and_stored_expectations() {
    let directory = support::TempDirectory::new();
    let path = directory.source(r#"
import { expect } from "std:testing";
function main(){
    expect(3).toBe(3);expect("é").toBe("é");expect(true).toBe(true);
    expect(Number.NaN).toBe(Number.NaN);expect(-0.0).toBe(-0.0);
    const assertion=expect(7);assertion.toBe(7);assertion.toBe(7);
    console.log("expectations passed");
}
"#);
    let output = support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert_eq!(output.stdout,b"expectations passed\n");
}

#[test]
fn vitest_to_be_rejects_signed_zero_mismatch_and_stops_test_body() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import { expect } from 'std:testing';function main(){expect(0.0).toBe(-0.0);console.log('unreachable');}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("AssertionError:"));
}

#[test]
fn vitest_stored_string_expectation_borrows_expected_inputs() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import { expect } from 'std:testing';function main(){const text='é'.repeat(2);const assertion=expect(clone(text));assertion.toBe(text);assertion.toBe(text);console.log(text);}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert_eq!(output.stdout,"éé\n".as_bytes());
}

#[test]
fn testing_api_rejects_invalid_arity_types_and_unknown_exports() {
    for source in [
        "import {expect} from 'std:testing';function main(){expect();}",
        "import {expect} from 'std:testing';function main(){expect(1,2);}",
        "import {expect} from 'std:testing';function main(){expect(1).toBe();}",
        "import {expect} from 'std:testing';function main(){expect(1).toBe(1,2);}",
        "import {expect} from 'std:testing';function main(){expect(1).toBe('one');}",
        "import {strictEqual} from 'std:testing';function main(){strictEqual(1,1,false);}",
        "import {unavailable} from 'std:testing';function main(){}",
    ] {
        let directory=support::TempDirectory::new();
        let path=directory.source(source);
        let output=support::run(support::cli().arg("check").arg(path),10);
        assert!(!output.status.success(),"{source}");
        let stderr=String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("error["),"{source}: {stderr}");
        assert!(!stderr.contains("panicked"),"{source}: {stderr}");
    }
}

#[test]
fn registered_tests_run_in_order_and_continue_after_assertion_failure() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"
import {test,expect,runTests} from 'std:testing';
function main(){
    test('first',():void=>{expect(1).toBe(1);console.log('first body');});
    test('failure',():void=>{expect(1).toBe(2);console.log('unreachable');});
    test('last',():void=>{console.log('last body');expect(3).toBe(3);});
    console.log('registered');
    console.log(runTests());
}
"#);
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    let stdout=String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("registered\nfirst body\n"),"{stdout}");
    assert!(stdout.contains("PASS first"),"{stdout}");
    assert!(stdout.contains("FAIL failure: AssertionError:"),"{stdout}");
    assert!(stdout.contains("last body\nPASS last"),"{stdout}");
    assert!(stdout.ends_with("2 passed, 1 failed, 0 skipped\n1\n"),"{stdout}");
    assert!(!stdout.contains("unreachable"));
    assert!(output.stderr.is_empty(),"{:?}",output.stderr);
}

#[test]
fn standard_testing_namespace_combines_assertions_expect_and_registration() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {expect,strictEqual,test,runTests} from 'std:testing';function main(){strictEqual(3,3);test('native',():void=>{expect(7).toBe(7);});console.log(runTests());}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert_eq!(output.stdout,b"PASS native\n1 passed, 0 failed, 0 skipped\n0\n");
}

#[test]
fn testing_imports_reject_unconfigured_aliases_and_private_runtime() {
    for module in ["vitest", "node:assert/strict", "node:test", "std:assert", "std:test", "std:testing-runtime"] {
        let directory = support::TempDirectory::new();
        let path = directory.source(&support::format_round_trip(&format!("import {{ok}} from {module:?};function main(){{ok(true);}}")));
        let output = support::run(support::cli().arg("check").arg(path), 60);
        assert!(!output.status.success(), "unexpected public module: {module}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        let expected = if module.starts_with("std:") { "error[module-native]" } else { "error[module-load]: No path alias matches" };
        assert!(stderr.contains(expected), "{module}: {stderr}");
        if module == "std:testing-runtime" { assert!(stderr.contains("Testing runtime bindings are private"), "{stderr}"); }
    }
}

#[test]
fn test_command_runs_top_level_registrations_and_returns_failure_status() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {test,expect} from 'std:testing';test('pass',():void=>{expect(1).toBe(1);});test('fail',():void=>{expect(1).toBe(2);});test('later',():void=>{console.log('later body');});");
    let output = support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(&path), 60);
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PASS pass"), "{stdout}");
    assert!(stdout.contains("FAIL fail: AssertionError:"), "{stdout}");
    assert!(stdout.contains("later body\nPASS later"), "{stdout}");
    assert!(stdout.ends_with("2 passed, 1 failed, 0 skipped\n"), "{stdout}");
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
}

#[test]
fn conventional_test_entry_checks_and_runs_with_original_error_locations() {
    let directory = support::TempDirectory::new();
    let path = directory.0.join("sum.test.tsr");
    std::fs::write(&path, "import {test,expect} from 'std:testing';\ntest('sum',():void=>{expect(2+3).toBe(5);});\n").unwrap();
    support::success(&support::run(support::cli().arg("check").arg(&path), 60));
    let output = support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(&path), 60);
    support::success(&output);
    assert_eq!(output.stdout, b"PASS sum\n1 passed, 0 failed, 0 skipped\n");
    std::fs::write(&path, "import {test,expect} from 'std:testing';\ntest('sum',():void=>{expect(2).toBe('wrong');});\n").unwrap();
    let output = support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(&path), 60);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("sum.test.tsr:2:"), "{stderr}");
    assert!(stderr.contains("wrong"), "{stderr}");
}

#[test]
fn test_entry_handles_empty_files_helper_functions_and_registration_state() {
    let directory = support::TempDirectory::new();
    for (source, expected) in [
        ("// empty test entry", "0 passed, 0 failed, 0 skipped\n"),
        ("import {test,expect} from 'std:testing';function sum(a:int,b:int):int{return a+b;}const seed=3;test('helper',():void=>{expect(sum(seed,4)).toBe(7);});", "PASS helper\n1 passed, 0 failed, 0 skipped\n"),
    ] {
        let path=directory.source(source);
        let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
        support::success(&output);
        assert_eq!(output.stdout,expected.as_bytes());
    }
    let path=directory.source("function main(){}");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Test entry files use top-level registration"));
}

#[test]
fn test_command_discovers_sorted_files_and_runs_one_registry() {
    let directory=support::TempDirectory::new();
    std::fs::create_dir(directory.0.join("nested")).unwrap();
    std::fs::write(directory.0.join("b.spec.tsr"),"import {test} from 'std:testing';console.log('register b');test('b',():void=>{});").unwrap();
    std::fs::write(directory.0.join("nested/a.test.tsr"),"import {test,expect} from 'std:testing';console.log('register a');test('a',():void=>{expect(2).toBe(2);});").unwrap();
    std::fs::write(directory.0.join("helper.tsr"),"not a test file").unwrap();
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").current_dir(&directory.0),60);
    support::success(&output);
    assert_eq!(output.stdout,b"register b\nregister a\nPASS b\nPASS a\n2 passed, 0 failed, 0 skipped\n");
}

#[test]
fn test_command_deduplicates_explicit_files_and_compiles_before_registration() {
    let directory=support::TempDirectory::new();
    let a=directory.0.join("a.tsr");let b=directory.0.join("b.tsr");
    std::fs::write(&a,"import {test} from 'std:testing';console.log('register a');test('a',():void=>{});").unwrap();
    std::fs::write(&b,"import {test} from 'std:testing';console.log('register b');test('b',():void=>{});").unwrap();
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(&b).arg(&a).arg(&b),60);
    support::success(&output);
    assert_eq!(output.stdout,b"register b\nregister a\nPASS b\nPASS a\n2 passed, 0 failed, 0 skipped\n");
    std::fs::write(&a,"import {test} from 'std:testing';test('broken',():void=>{unknown();});").unwrap();
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(&b).arg(&a),60);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("a.tsr:1:"));
}

#[test]
fn test_discovery_ignores_build_archives_hidden_directories_and_symlink_cycles() {
    let directory=support::TempDirectory::new();
    std::fs::write(directory.0.join("valid.test.tsr"),"import {test} from 'std:testing';test('valid',():void=>{});").unwrap();
    for name in ["target","archive",".cache","node_modules"] {
        let path=directory.0.join(name);std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("invalid.test.tsr"),"invalid source").unwrap();
    }
    #[cfg(unix)] std::os::unix::fs::symlink(&directory.0,directory.0.join("cycle")).unwrap();
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(&directory.0),60);
    support::success(&output);
    assert_eq!(output.stdout,b"PASS valid\n1 passed, 0 failed, 0 skipped\n");
}

#[test]
fn test_discovery_reports_empty_selection_and_invalid_options_without_execution() {
    let directory=support::TempDirectory::new();
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(&directory.0),60);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No test files found"));
    let path=directory.source("console.log('must not run');");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(&path).arg("--json"),60);
    assert!(!output.status.success());assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--json applies only to check"));
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(directory.0.join("missing.tsr")),60);
    assert!(!output.status.success());assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing.tsr"));
}

#[test]
fn describe_registers_nested_suites_before_execution_in_source_order() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {describe,test,expect} from 'std:testing';console.log('top');describe('outer',():void=>{console.log('register outer');test('first',():void=>{expect(1).toBe(1);});describe('inner',():void=>{console.log('register inner');test('second',():void=>{});});test('third',():void=>{});});test('last',():void=>{});");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
    support::success(&output);
    assert_eq!(output.stdout,b"top\nregister outer\nregister inner\nPASS outer > first\nPASS outer > inner > second\nPASS outer > third\nPASS last\n4 passed, 0 failed, 0 skipped\n");
}

#[test]
fn suite_alias_reuses_borrowed_registration_callbacks_and_preserves_names() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {suite,test} from 'std:testing';const registration=():void=>{test('same',():void=>{});};const name='group'.repeat(1);suite(name,registration);suite(name,registration);console.log(name);");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
    support::success(&output);
    assert_eq!(output.stdout,b"group\nPASS group > same\nPASS group > same\n2 passed, 0 failed, 0 skipped\n");
}

#[test]
fn suites_continue_after_failure_and_reject_registration_during_execution() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {describe,test,expect} from 'std:testing';describe('outer',():void=>{test('fail',():void=>{expect(1).toBe(2);});describe('inner',():void=>{test('later',():void=>{});});});test('invalid',():void=>{describe('nested',():void=>{});});test('last',():void=>{});");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
    assert_eq!(output.status.code(),Some(1));
    let stdout=String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FAIL outer > fail: AssertionError:"),"{stdout}");
    assert!(stdout.contains("PASS outer > inner > later"),"{stdout}");
    assert!(stdout.contains("FAIL invalid: describe() cannot register suites while a test is running"),"{stdout}");
    assert!(stdout.ends_with("PASS last\n2 passed, 2 failed, 0 skipped\n"),"{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn suite_registration_can_mutate_a_reused_callback_environment() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {suite,test} from 'std:testing';let count=0;const registration=():void=>{count+=1;console.log(count);test('same',():void=>{});};suite('first',registration);suite('second',registration);");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
    support::success(&output);
    assert_eq!(output.stdout,b"1\n2\nPASS first > same\nPASS second > same\n2 passed, 0 failed, 0 skipped\n");
}

#[test]
fn test_registration_rejects_top_level_return_without_rejecting_callback_returns() {
    let directory=support::TempDirectory::new();
    for source in ["return;", "if(true){return;}", "while(false){return;}"] {
        let path=directory.source(source);
        let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
        assert!(!output.status.success());assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("return is only valid inside a function or callback"));
    }
    let path=directory.source("import {test} from 'std:testing';test('early return',():void=>{return;});");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);support::success(&output);
    assert_eq!(output.stdout,b"PASS early return\n1 passed, 0 failed, 0 skipped\n");
}

#[test]
fn suite_hooks_follow_nested_setup_and_reverse_teardown_order() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"
import {describe,test,beforeAll,afterAll,beforeEach,afterEach} from 'std:testing';
beforeAll(():void=>{console.log('root beforeAll');});
afterAll(():void=>{console.log('root afterAll');});
beforeEach(():void=>{console.log('root beforeEach');});
afterEach(():void=>{console.log('root afterEach');});
describe('suite',():void=>{
 beforeAll(():void=>{console.log('suite beforeAll');});
 afterAll(():void=>{console.log('suite afterAll first');});
 afterAll(():void=>{console.log('suite afterAll second');});
 beforeEach(():void=>{console.log('suite beforeEach first');});
 beforeEach(():void=>{console.log('suite beforeEach second');});
 afterEach(():void=>{console.log('suite afterEach first');});
 afterEach(():void=>{console.log('suite afterEach second');});
 describe('inner',():void=>{
  beforeEach(():void=>{console.log('inner beforeEach');});
  afterEach(():void=>{console.log('inner afterEach');});
  test('body',():void=>{console.log('body');});
 });
});
"#);
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
    support::success(&output);
    assert_eq!(output.stdout,b"root beforeAll\nsuite beforeAll\nroot beforeEach\nsuite beforeEach first\nsuite beforeEach second\ninner beforeEach\nbody\ninner afterEach\nsuite afterEach second\nsuite afterEach first\nroot afterEach\nPASS suite > inner > body\nsuite afterAll second\nsuite afterAll first\nroot afterAll\n1 passed, 0 failed, 0 skipped\n");
}

#[test]
fn hooks_teardown_after_setup_body_and_teardown_failures_without_losing_errors() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"
import {describe,test,expect,beforeEach,afterEach,afterAll} from 'std:testing';
afterAll(():void=>{console.log('file teardown');});
describe('setup fails',():void=>{
 beforeEach(():void=>{console.log('setup');expect(1).toBe(2);});
 afterEach(():void=>{console.log('teardown first');});
 afterEach(():void=>{console.log('teardown second');expect(3).toBe(4);});
 test('blocked',():void=>{console.log('unreachable');});
});
describe('body fails',():void=>{
 afterEach(():void=>{console.log('body teardown');expect(5).toBe(6);});
 test('broken',():void=>{expect(7).toBe(8);});
});
test('later',():void=>{});
"#);
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
    assert_eq!(output.status.code(),Some(1));let stdout=String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("setup\nteardown second\nteardown first\nFAIL setup fails > blocked: beforeEach: AssertionError:"),"{stdout}");
    assert!(stdout.contains("; afterEach: AssertionError:"),"{stdout}");
    assert!(stdout.contains("body teardown\nFAIL body fails > broken: AssertionError:"),"{stdout}");
    assert!(!stdout.contains("unreachable"));
    assert!(stdout.ends_with("PASS later\nfile teardown\n1 passed, 2 failed, 0 skipped\n"),"{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn top_level_hooks_stay_within_their_selected_file() {
    let directory=support::TempDirectory::new();let a=directory.0.join("a.test.tsr");let b=directory.0.join("b.test.tsr");
    std::fs::write(&a,"import {test,beforeEach,afterAll} from 'std:testing';beforeEach(():void=>{console.log('a hook');});afterAll(():void=>{console.log('a done');});test('a',():void=>{});").unwrap();
    std::fs::write(&b,"import {test,beforeEach,afterAll} from 'std:testing';beforeEach(():void=>{console.log('b hook');});afterAll(():void=>{console.log('b done');});test('b',():void=>{});").unwrap();
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(a).arg(b),60);support::success(&output);
    assert_eq!(output.stdout,b"a hook\nPASS a\na done\nb hook\nPASS b\nb done\n2 passed, 0 failed, 0 skipped\n");
}

#[test]
fn reused_each_hook_keeps_owned_mutable_capture_state() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {test,beforeEach} from 'std:testing';let count=0;beforeEach(():void=>{count+=1;console.log(count);});test('first',():void=>{});test('second',():void=>{});");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);support::success(&output);
    assert_eq!(output.stdout,b"1\nPASS first\n2\nPASS second\n2 passed, 0 failed, 0 skipped\n");
}

#[test]
fn before_all_failure_blocks_descendants_but_runs_scope_teardown_and_later_suites() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {describe,test,expect,beforeAll,afterAll} from 'std:testing';describe('broken',():void=>{beforeAll(():void=>{console.log('setup');expect(1).toBe(2);});afterAll(():void=>{console.log('teardown');});test('first',():void=>{console.log('unreachable');});describe('child',():void=>{beforeAll(():void=>{console.log('child setup unreachable');});test('second',():void=>{});});});describe('later',():void=>{test('pass',():void=>{});});");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);assert_eq!(output.status.code(),Some(1));
    let stdout=String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("HOOK FAIL broken beforeAll: AssertionError:"),"{stdout}");
    assert!(stdout.contains("FAIL broken > first: beforeAll: AssertionError:"),"{stdout}");
    assert!(stdout.contains("FAIL broken > child > second: beforeAll: AssertionError:"),"{stdout}");
    assert!(!stdout.contains("unreachable"));
    assert!(stdout.ends_with("teardown\nPASS later > pass\n1 passed, 2 failed, 0 skipped, 1 hook failures\n"),"{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn after_all_failures_run_all_remaining_teardown_and_make_command_fail() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {describe,test,expect,afterAll} from 'std:testing';describe('suite',():void=>{afterAll(():void=>{console.log('last teardown');});afterAll(():void=>{expect(1).toBe(2);});afterAll(():void=>{expect(3).toBe(4);});test('pass',():void=>{});});test('later',():void=>{});");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);assert_eq!(output.status.code(),Some(1));
    let stdout=String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("HOOK FAIL suite afterAll: AssertionError:").count(),2,"{stdout}");
    assert!(stdout.contains("last teardown\n"),"{stdout}");
    assert!(stdout.ends_with("PASS later\n2 passed, 0 failed, 0 skipped, 2 hook failures\n"),"{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn node_style_before_after_aliases_register_once_for_the_current_scope() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {test,before,after} from 'std:testing';before(():void=>{console.log('before');});after(():void=>{console.log('after');});test('first',():void=>{});test('second',():void=>{});");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);support::success(&output);
    assert_eq!(output.stdout,b"before\nPASS first\nPASS second\nafter\n2 passed, 0 failed, 0 skipped\n");
}

#[test]
fn hook_registration_errors_fail_current_work_and_empty_scopes_do_not_run_hooks() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {describe,test,beforeAll,beforeEach,afterEach} from 'std:testing';describe('empty',():void=>{beforeAll(():void=>{console.log('empty unreachable');});});beforeEach(():void=>{beforeAll(():void=>{});});afterEach(():void=>{console.log('cleanup');});test('blocked',():void=>{console.log('body unreachable');});");
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);assert_eq!(output.status.code(),Some(1));
    let stdout=String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cleanup\nFAIL blocked: beforeEach: Hooks cannot register while tests or hooks are running"),"{stdout}");
    assert!(!stdout.contains("unreachable"));
    assert!(stdout.ends_with("0 passed, 1 failed, 0 skipped\n"));assert!(output.stderr.is_empty());
}

#[test]
fn hook_calls_reject_wrong_arity_callback_results_and_escaped_borrowed_captures() {
    let directory=support::TempDirectory::new();
    for source in [
        "import {beforeEach} from 'std:testing';beforeEach();",
        "import {beforeEach} from 'std:testing';beforeEach(():void=>{},1000);",
        "import {beforeEach} from 'std:testing';beforeEach(():int=>{return 3;});",
        "import {beforeEach} from 'std:testing';beforeEach((value:int):void=>{});",
        "import {beforeEach} from 'std:testing';function install(value:ReadOnly<string>):void{beforeEach(():void=>{console.log(value);});}install('text');",
    ] {
        let path=directory.source(source);
        let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
        assert!(!output.status.success(),"{source}");assert!(output.stdout.is_empty());
        let stderr=String::from_utf8_lossy(&output.stderr);assert!(stderr.contains("error["),"{source}: {stderr}");assert!(!stderr.contains("Rust rejected"),"{source}: {stderr}");
    }
}

#[test]
fn bundled_testing_implementation_uses_only_rust_source_files() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/stdlib/testing");
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            assert_eq!(path.extension().and_then(|value| value.to_str()), Some("rs"), "{}", path.display());
        }
    }
}

#[test]
fn expectation_truthiness_matches_values_and_reuses_owned_expectation() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"
import {expect} from "std:testing";
function main(){
 expect(false).toBeFalsy();expect(0).toBeFalsy();expect(-0.0).toBeFalsy();
 expect(Number.NaN).toBeFalsy();expect("").toBeFalsy();expect(null).toBeFalsy();expect(undefined).toBeFalsy();
 expect(1).toBeTruthy();expect("é").toBeTruthy();expect([1]).toBeTruthy();
 const text=expect("saved");text.toBeTruthy();text.toBe("saved");text.toBeTruthy();
 console.log("passed");
}
"#);
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);assert_eq!(output.stdout,b"passed\n");
}

#[test]
fn truthiness_matcher_failure_stops_body_and_later_test_continues() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"import {test,expect} from "std:testing";
test("falsy",():void=>{expect(0).toBeTruthy();console.log("unreachable");});
test("truthy",():void=>{expect("x").toBeFalsy();});test("later",():void=>{expect(true).toBeTruthy();});"#);
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
    assert!(!output.status.success());
    let stdout=String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("FAIL falsy: AssertionError: Expected a truthy value"));
    assert!(stdout.contains("FAIL truthy: AssertionError: Expected a falsy value"));
    assert!(stdout.contains("PASS later"));assert!(!stdout.contains("unreachable"));
}

#[test]
fn nullish_matchers_distinguish_null_undefined_and_defined_falsy_values() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"import {expect} from "std:testing";
function main(){expect(null).toBeNull();expect(null).toBeDefined();expect(undefined).toBeUndefined();
expect(false).toBeDefined();expect(0).toBeDefined();expect("").toBeDefined();expect(Number.NaN).toBeDefined();
const retained=expect("saved");retained.toBeDefined();retained.toBe("saved");retained.toBeTruthy();
console.log("passed");}"#);
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert_eq!(output.stdout,b"passed\n");
}

#[test]
fn nullish_failures_are_recoverable_and_zero_argument_signatures_are_enforced() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"import {test,expect} from "std:testing";
test("null",():void=>{expect(undefined).toBeNull();});test("undefined",():void=>{expect(null).toBeUndefined();});
test("defined",():void=>{expect(undefined).toBeDefined();});test("later",():void=>{expect(null).toBeDefined();});"#);
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(&path),60);assert!(!output.status.success());
    let text=String::from_utf8(output.stdout).unwrap();assert!(text.contains("3 failed"));assert!(text.contains("PASS later"));
    for name in ["toBeNull","toBeUndefined","toBeDefined"] {
        std::fs::write(&path,format!("import {{expect}} from 'std:testing';function main(){{expect(null).{name}(1);}}")).unwrap();
        let output=support::run(support::cli().arg("check").arg(&path),10);assert!(!output.status.success());
    }
}

#[test]
fn nullable_expectations_support_predicates_and_identity_without_cloning_data() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"import {expect,strictEqual} from "std:testing";
function inspect(value:string|null|undefined):void {
 const isNull=value===null;const isUndefined=value===undefined;
 const assertion=expect(value);
 if(isNull){assertion.toBeNull();assertion.toBeDefined();}
 else if(isUndefined){assertion.toBeUndefined();assertion.toBeFalsy();}
 else {assertion.toBeDefined();assertion.toBeTruthy();}
}
function same(value:number|null|undefined):void{strictEqual(value,value);}
function main(){inspect(null);inspect(undefined);inspect("é");same(Number.NaN);same(null);console.log("passed");}"#);
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert_eq!(output.stdout,b"passed\n");
}

#[test]
fn expectation_copy_union_specialization_keeps_identity_matcher_invocable() {
    let directory=support::TempDirectory::new();let path=directory.source(r#"import {expect} from 'std:testing';
function inspect(value:number|null|undefined):void{const assertion=expect(value);assertion.toBe(value);}
function main(){inspect(Number.NaN);inspect(-0.0);inspect(null);inspect(undefined);console.log("passed");}"#);
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert_eq!(output.stdout,b"passed\n");
}

#[test]
fn nan_matcher_checks_actual_value_without_coercion_and_recovers_failures() {
    let directory=support::TempDirectory::new();let path=directory.source(r#"import {expect,test} from 'std:testing';type Row={value:int;};
test('nan',():void=>{const saved=expect(Number.NaN);saved.toBeNaN();saved.toBe(Number.NaN);});
test('integer',():void=>{expect(0).toBeNaN();});test('text',():void=>{expect('NaN').toBeNaN();});
test('null',():void=>{expect(null).toBeNaN();});
test('undefined',():void=>{expect(undefined).toBeNaN();});test('infinity',():void=>{expect(Number.POSITIVE_INFINITY).toBeNaN();});
test('array',():void=>{expect([1]).toBeNaN();});test('record',():void=>{const row:Row={value:1};expect(row).toBeNaN();});test('later',():void=>{expect(Number.NaN).toBeNaN();});"#);
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);assert!(!output.status.success());
    let text=String::from_utf8(output.stdout).unwrap();assert!(text.contains("PASS nan"),"{text}");assert!(text.contains("PASS later"),"{text}");assert!(text.contains("2 passed, 7 failed"),"{text}");
}

#[test]
fn negated_expectation_shares_owned_actual_and_preserves_repeated_positive_checks() {
    let directory = support::TempDirectory::new();
    let path = directory.source(r#"
import { expect } from "std:testing";
function main(){const assertion=expect("retained");assertion.toBe("retained");assertion.not.toBe("different");assertion.toBe("retained");assertion.not.toBeNull();expect(0).not.toBeTruthy();expect(7).not.toBeFalsy();expect(undefined).not.toBeDefined();expect(null).not.toBeUndefined();expect(undefined).not.toBeNull();expect(4).not.toBeNaN();console.log("passed");}
"#);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(output.stdout, b"passed\n");
}

#[test]
fn negated_matcher_failures_stop_body_and_later_tests_continue() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"import {test,expect} from "std:testing";
test("identity",():void=>{expect(Number.NaN).not.toBe(Number.NaN);console.log("unreachable");});
test("truthy",():void=>{expect(1).not.toBeTruthy();});
test("falsy",():void=>{expect(0).not.toBeFalsy();});
test("null",():void=>{expect(null).not.toBeNull();});
test("undefined",():void=>{expect(undefined).not.toBeUndefined();});
test("defined",():void=>{expect(null).not.toBeDefined();});
test("nan",():void=>{expect(Number.NaN).not.toBeNaN();});
test("later",():void=>{expect(0.0).not.toBe(-0.0);});"#);
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
    assert!(!output.status.success());
    let stdout=String::from_utf8(output.stdout).unwrap();
    for name in ["identity","truthy","falsy","null","undefined","defined","nan"] {
        assert!(stdout.contains(&format!("FAIL {name}: AssertionError:")),"{stdout}");
    }
    assert!(stdout.contains("PASS later"),"{stdout}");assert!(!stdout.contains("unreachable"));
}

#[test]
fn negated_identity_borrows_expected_arrays_records_and_nullable_values() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"import {expect} from 'std:testing';
type Row={value:int;};
function inspect(value:string|null|undefined):void{const nil=value===null;const absent=value===undefined;const assertion=expect(value);if(nil){assertion.not.toBeUndefined();}else if(absent){assertion.not.toBeNull();}else{assertion.not.toBeNull();assertion.not.toBeUndefined();}}
function main(){const first=[1,2];const expected=[1,2];const assertion=expect(first);assertion.not.toBe(expected);assertion.not.toBe(expected);console.log(expected.length);const row:Row={value:3};const other:Row={value:3};const recordAssertion=expect(row);recordAssertion.not.toBe(other);recordAssertion.not.toBe(other);console.log(other.value);inspect(null);inspect(undefined);inspect('saved');console.log('passed');}"#);
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(output.stdout,b"2\n3\npassed\n");
}

#[test]
fn negated_matchers_reject_bad_arity_types_and_actual_owner_reuse() {
    let directory=support::TempDirectory::new();
    for body in ["expect(1).not.toBe();", "expect(1).not.toBe('wrong');", "expect(1).not.toBeNull(1);", "const actual=[1];const assertion=expect(actual);console.log(actual.length);", "expect(1).not.not.toBe(1);"] {
        let path=directory.source(&format!("import {{expect}} from 'std:testing';function main(){{{body}}}"));
        let output=support::run(support::cli().arg("check").arg(path),10);
        assert!(!output.status.success(),"unexpected acceptance: {body}");
    }
}

#[test]
fn typeof_matchers_support_all_data_categories_and_negation() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"import {expect} from 'std:testing';type Row={value:int;};
function inspect(value:string|null|undefined):void{const kind=typeof value;const assertion=expect(value);assertion.toBeTypeOf(kind);assertion.not.toBeTypeOf('boolean');}
function main(){const text=expect('saved');const kind='string';text.toBeTypeOf(kind);text.not.toBeTypeOf('number');text.toBe('saved');console.log(kind);expect(1).toBeTypeOf('number');expect(Number.NaN).toBeTypeOf('number');expect(false).toBeTypeOf('boolean');expect(null).toBeTypeOf('object');expect(undefined).toBeTypeOf('undefined');expect([1]).toBeTypeOf('object');const row:Row={value:3};expect(row).toBeTypeOf('object');inspect(null);inspect(undefined);inspect('x');console.log('passed');}"#);
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(output.stdout,b"string\npassed\n");
}

#[test]
fn typeof_matcher_failures_recover_and_invalid_arguments_reject() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"import {test,expect} from 'std:testing';
test('positive',():void=>{expect(null).toBeTypeOf('undefined');console.log('unreachable');});
test('negative',():void=>{expect(3).not.toBeTypeOf('number');});
test('later',():void=>{expect(undefined).toBeTypeOf('undefined');});"#);
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
    assert!(!output.status.success());let stdout=String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("FAIL positive: AssertionError:"),"{stdout}");
    assert!(stdout.contains("FAIL negative: AssertionError:"),"{stdout}");
    assert!(stdout.contains("PASS later"),"{stdout}");assert!(!stdout.contains("unreachable"));
    for call in ["expect(1).toBeTypeOf()", "expect(1).toBeTypeOf(1)", "expect(1).not.toBeTypeOf('number','string')"] {
        let path=directory.source(&format!("import {{expect}} from 'std:testing';function main(){{{call};}}"));
        let output=support::run(support::cli().arg("check").arg(path),10);
        assert!(!output.status.success(),"unexpected acceptance: {call}");
    }
}

#[test]
fn numeric_relational_matchers_cover_boundaries_nan_and_negation() {
    let directory=support::TempDirectory::new();
    let path=directory.source(r#"import {expect} from 'std:testing';
function inspect(value:number|null|undefined):void{const numeric=typeof value==='number';const assertion=expect(value);if(numeric){assertion.toBeGreaterThan(1.0);assertion.not.toBeLessThanOrEqual(1.0);}}
function main(){const assertion=expect(3);assertion.toBeGreaterThan(2.0);assertion.toBeGreaterThanOrEqual(3.0);assertion.toBeLessThan(4.0);assertion.toBeLessThanOrEqual(3.0);assertion.not.toBeGreaterThan(3.0);assertion.not.toBeGreaterThanOrEqual(4.0);assertion.not.toBeLessThan(3.0);assertion.not.toBeLessThanOrEqual(2.0);expect(Number.NaN).not.toBeGreaterThan(0.0);expect(Number.POSITIVE_INFINITY).toBeGreaterThan(7.0);expect(-0.0).toBeGreaterThanOrEqual(0.0);inspect(2.5);inspect(null);console.log('passed');}"#);
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert_eq!(output.stdout,b"passed\n");
}

#[test]
fn numeric_comparison_failures_recover_without_coercing_actuals() {
    let directory=support::TempDirectory::new();
    for method in ["toBeGreaterThan","toBeGreaterThanOrEqual","toBeLessThan","toBeLessThanOrEqual"] {
        let source=format!("import {{test,expect}} from 'std:testing';test('nan',():void=>{{expect(Number.NaN).{method}(0.0);console.log('unreachable');}});test('string',():void=>{{expect('42').not.{method}(0.0);}});test('boolean',():void=>{{expect(true).{method}(0.0);}});test('null',():void=>{{expect(null).not.{method}(0.0);}});test('later',():void=>{{expect(Number.NaN).not.{method}(0.0);}});");
        let path=directory.source(&source);let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);
        assert!(!output.status.success());let stdout=String::from_utf8(output.stdout).unwrap();
        for name in ["nan","string","boolean","null"] {assert!(stdout.contains(&format!("FAIL {name}: AssertionError:")),"{method}: {stdout}");}
        assert!(stdout.contains("PASS later"),"{stdout}");assert!(!stdout.contains("unreachable"));
        for call in [format!("expect(1).{method}()"),format!("expect(1).not.{method}('wrong')"),format!("expect(1).{method}(1.0,2.0)")] {
            let path=directory.source(&format!("import {{expect}} from 'std:testing';function main(){{{call};}}"));
            let output=support::run(support::cli().arg("check").arg(path),10);assert!(!output.status.success(),"unexpected acceptance: {call}");
        }
    }
}

#[test]
fn close_to_matcher_supports_default_precision_fractional_precision_and_infinities() {
    let directory=support::TempDirectory::new();let path=directory.source(r#"import {expect} from 'std:testing';
function main(){const assertion=expect(0.1+0.2);assertion.toBeCloseTo(0.3);assertion.toBeCloseTo(0.3,5.0);assertion.not.toBeCloseTo(0.3,50.0);expect(0.0).toBeCloseTo(0.1,0.5);expect(0.0).toBeCloseTo(2.0,-1.0);expect(0.0).not.toBeCloseTo(0.5,0.0);expect(Number.POSITIVE_INFINITY).toBeCloseTo(Number.POSITIVE_INFINITY);expect(Number.NEGATIVE_INFINITY).toBeCloseTo(Number.NEGATIVE_INFINITY);expect(Number.POSITIVE_INFINITY).not.toBeCloseTo(Number.NEGATIVE_INFINITY);expect(Number.NaN).not.toBeCloseTo(0.0);expect(-0.0).toBeCloseTo(0.0);console.log('passed');}"#);
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert_eq!(output.stdout,b"passed\n");
}

#[test]
fn close_to_precision_protocol_and_failures_preserve_runner_recovery() {
    let directory=support::TempDirectory::new();let path=directory.source(r#"import {test,expect} from 'std:testing';
test('defaults',():void=>{const assertion=expect(0.1+0.2);assertion.toBeCloseTo(0.3,undefined);assertion.toBeCloseTo(0.3);assertion.not.toBeCloseTo(0.3,Number.NaN);assertion.not.toBeCloseTo(0.3,Number.POSITIVE_INFINITY);assertion.toBeCloseTo(0.3,Number.NEGATIVE_INFINITY);});
test('boundary',():void=>{expect(0.0).toBeCloseTo(0.5,0.0);console.log('unreachable');});
test('negated',():void=>{expect(0.1+0.2).not.toBeCloseTo(0.3);});
test('nonnumeric',():void=>{expect('0.3').not.toBeCloseTo(0.3);});
test('nan',():void=>{expect(Number.NaN).toBeCloseTo(0.0);});
test('later',():void=>{expect(Number.NaN).not.toBeCloseTo(0.0);});"#);
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);assert!(!output.status.success());let stdout=String::from_utf8(output.stdout).unwrap();
    for name in ["boundary","negated","nonnumeric","nan"] {assert!(stdout.contains(&format!("FAIL {name}: AssertionError:")),"{stdout}");}
    assert!(stdout.contains("PASS defaults"),"{stdout}");assert!(stdout.contains("PASS later"),"{stdout}");assert!(!stdout.contains("unreachable"));
    for call in ["expect(1).toBeCloseTo()","expect(1).toBeCloseTo('wrong')","expect(1).not.toBeCloseTo(1.0,'wrong')","expect(1).toBeCloseTo(1.0,2.0,3.0)"] {
        let path=directory.source(&format!("import {{expect}} from 'std:testing';function main(){{{call};}}"));let output=support::run(support::cli().arg("check").arg(path),10);assert!(!output.status.success(),"unexpected acceptance: {call}");
    }
}

#[test]
fn length_matcher_observes_utf16_arrays_tuples_and_numeric_record_properties() {
    let directory=support::TempDirectory::new();let path=directory.source(r#"import {expect} from 'std:testing';type Row={length:number;};
function inspect(value:string|null):void{const present=value!==null;const assertion=expect(value);if(present){assertion.toHaveLength(2.0);assertion.not.toHaveLength(1.0);}}
function main(){const string=expect('😀');string.toHaveLength(2.0);string.not.toHaveLength(1.0);string.toBe('😀');const array=expect([1,2]);array.toHaveLength(2.0);array.not.toHaveLength(3.0);const tuple:[int,string]=[1,'x'];expect(tuple).toHaveLength(2.0);const row:Row={length:2.5};expect(row).toHaveLength(2.5);inspect('ab');inspect(null);console.log('passed');}"#);
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert_eq!(output.stdout,b"passed\n");
}

#[test]
fn length_matcher_failures_recover_and_invalid_arguments_reject() {
    let directory=support::TempDirectory::new();let path=directory.source(r#"import {test,expect} from 'std:testing';type Row={value:int;};
test('wrong',():void=>{expect('😀').toHaveLength(1.0);console.log('unreachable');});
test('negative',():void=>{expect([1]).not.toHaveLength(1.0);});
test('scalar',():void=>{expect(3).not.toHaveLength(3.0);});
test('null',():void=>{expect(null).toHaveLength(0.0);});
test('record',():void=>{const row:Row={value:1};expect(row).not.toHaveLength(0.0);});
test('later',():void=>{expect('').toHaveLength(0.0);});"#);
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);assert!(!output.status.success());let stdout=String::from_utf8(output.stdout).unwrap();
    for name in ["wrong","negative","scalar","null","record"] {assert!(stdout.contains(&format!("FAIL {name}: AssertionError:")),"{stdout}");}
    assert!(stdout.contains("PASS later"),"{stdout}");assert!(!stdout.contains("unreachable"));
    for call in ["expect('x').toHaveLength()","expect('x').toHaveLength('1')","expect('x').not.toHaveLength(1.0,2.0)"] {
        let path=directory.source(&format!("import {{expect}} from 'std:testing';function main(){{{call};}}"));let output=support::run(support::cli().arg("check").arg(path),10);assert!(!output.status.success(),"unexpected acceptance: {call}");
    }
}

#[test]
fn length_matcher_handles_active_union_typed_record_properties() {
    let directory=support::TempDirectory::new();let path=directory.source(r#"import {expect,test} from 'std:testing';type Row={length:number|string|undefined;};
function inspect(value:number|string|undefined):void{const row:Row={length:value};const assertion=expect(row);assertion.toHaveLength(3.0);assertion.not.toHaveLength(4.0);}
test('numeric',():void=>{inspect(3.0);});test('string',():void=>{inspect('3');});test('undefined',():void=>{inspect(undefined);});test('later',():void=>{const row:Row={length:Number.NaN};expect(row).not.toHaveLength(3.0);});"#);
    let output=support::run(support::cli().env("TSRUST_TEST_REPORTER","plain").arg("test").arg(path),60);assert!(!output.status.success());let stdout=String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("PASS numeric"),"{stdout}");assert!(stdout.contains("FAIL string: AssertionError:"),"{stdout}");assert!(stdout.contains("FAIL undefined: AssertionError:"),"{stdout}");assert!(stdout.contains("PASS later"),"{stdout}");
}

#[test]
fn runner_formats_typed_record_errors_without_moving_payloads() {
    let directory = support::TempDirectory::new();
    let path = directory.source(r#"
import { test } from 'std:testing';
interface Failure { message: string; }
test('typed failure', (): void => {
  const failure: Failure = { message: 'record error preserved' };
  throw failure;
});
"#);
    let output = support::run(support::cli().arg("test").arg(path), 60);
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("typed failure"), "{stdout}");
    assert!(stdout.contains("record error preserved"), "{stdout}");
}

#[test]
fn throws_invokes_once_reports_normal_return_and_releases_owned_resources(){
 let source=include_str!("fixtures/testing-throws/assertions.tsr");
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
 let output=support::run(support::cli().arg("run").arg(&path),60);support::success(&output);assert_eq!(output.stdout,b"throws passed\n");
 for source in ["import {throws} from 'std:testing';function main(){throws((value:int)=>{});}","import {throws} from 'std:testing';function main(){const action=()=>{};throws(action);action();}"]{
  let path=directory.source(&support::format_round_trip(source));let output=support::run(support::cli().arg("check").arg(path),10);assert!(!output.status.success(),"{source}");
 }
}

#[test]
fn documented_throw_assertion_example_runs_formatted(){
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
 let example=&catalog["std:testing"].as_array().unwrap().iter().find(|entry|entry["name"]=="throws").unwrap()["examples"][0];
 let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(example["source"].as_str().unwrap()));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),example["output"].as_str().unwrap());
}
