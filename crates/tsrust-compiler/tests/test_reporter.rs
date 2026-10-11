mod support;
#[test]
fn pretty_report_groups_files_shows_assertion_values_and_totals_without_ansi_when_disabled() {
 let dir=support::TempDirectory::new();
 let source=support::format_round_trip("import {test,strictEqual} from 'std:testing';test('greeting',():void=>{strictEqual('hello','goodbye');});test('passes',():void=>{strictEqual(1,1);});");
 let file=dir.0.join("greeting.test.tsr");std::fs::write(&file,source).unwrap();
 let output=support::run(support::cli().args(["test","--color","never"]).arg(&file).current_dir(&dir.0).env("TSRUST_TEST_REPORTER","pretty"),60);
 assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));
 assert_eq!(output.status.code(),Some(1));let text=String::from_utf8_lossy(&output.stdout);
 for expected in ["greeting.test.tsr","✓ passes","× greeting","Failed Tests","Expected:","\"goodbye\"","Received:","\"hello\"","Test Files","Tests","Duration"]{assert!(text.contains(expected),"missing {expected}: {text}");}
 assert!(!text.contains(dir.0.to_str().unwrap()),"{text}");
 assert!(!text.contains('\x1b'));assert!(output.stderr.is_empty());
}

#[test]
fn default_report_counts_multiple_files_and_empty_files_and_supports_forced_color() {
 let dir=support::TempDirectory::new();
 for name in ["one","two"] {std::fs::write(dir.0.join(format!("{name}.test.tsr")),support::format_round_trip("import {test,strictEqual} from 'std:testing';test('same name',():void=>{strictEqual(1,1);});")).unwrap();}
 std::fs::write(dir.0.join("empty.test.tsr"),"// no tests\n").unwrap();
 let output=support::run(support::cli().args(["test","--color","always"]).arg(&dir.0).env_remove("TSRUST_TEST_REPORTER"),60);
 support::success(&output);let text=String::from_utf8_lossy(&output.stdout);
 assert!(text.contains("one.test.tsr")&&text.contains("two.test.tsr"),"{text}");
 assert!(text.contains("Test Files  2 passed | 0 failed (2)"),"{text}");assert!(text.contains("\x1b["));
}

#[test]
fn pretty_report_renders_record_and_array_failures_without_changing_identity_semantics() {
 let dir=support::TempDirectory::new();
 let file=dir.0.join("identity.test.tsr");
 std::fs::write(&file,support::format_round_trip("import {test,strictEqual} from 'std:testing';type Item={value:int;};test('array',():void=>{strictEqual([1,2],[1,2]);});test('record',():void=>{const a:Item={value:1};const b:Item={value:2};strictEqual(a,b);});")).unwrap();
 let output=support::run(support::cli().arg("test").arg(file).env_remove("TSRUST_TEST_REPORTER"),60);
 assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));assert_eq!(output.status.code(),Some(1));
 let text=String::from_utf8_lossy(&output.stdout);assert!(text.contains("Expected: [ 1, 2 ]"),"{text}");assert!(text.contains("value: 2"),"{text}");assert!(text.contains("1 failed (1)"),"{text}");
}

#[test]
fn pretty_console_logs_show_test_and_original_module_coordinates() {
 let dir=support::TempDirectory::new();std::fs::create_dir(dir.0.join("src")).unwrap();
 std::fs::write(dir.0.join("src/greeting.tsr"),support::format_round_trip("export function greeting(): void {\n  console.log(\"hello\\nworld\");\n  console.warn(\"warning\");\n}\n")).unwrap();
 let file=dir.0.join("greeting.test.tsr");std::fs::write(&file,support::format_round_trip("import {test} from 'std:testing';\nimport {greeting} from './src/greeting.tsr';\ntest('greets',():void=>{greeting();});\n")).unwrap();
 let output=support::run(support::cli().args(["test","--color","never"]).arg(&file).current_dir(&dir.0).env("TSRUST_TEST_REPORTER","pretty"),60);support::success(&output);
 let stdout=String::from_utf8_lossy(&output.stdout);let stderr=String::from_utf8_lossy(&output.stderr);
 for expected in ["     stdout\n       at src/greeting.tsr:2:3\n         hello\n         world","src/greeting.tsr:2:3","         hello\n         world","✓ greets"] {assert!(stdout.contains(expected),"missing {expected}: {stdout}");}
 for expected in ["     stderr\n       at src/greeting.tsr:3:3\n         warning","src/greeting.tsr:3:3","         warning"] {assert!(stderr.contains(expected),"missing {expected}: {stderr}");}
 assert!(stdout.find("✓ greets").unwrap()<stdout.find("     stdout\n       at src/greeting.tsr:2:3\n         hello\n         world").unwrap());
 assert!(!stdout.contains("stdout | greets"));assert!(!stderr.contains("stderr | greets"));
 assert!(!stdout.contains(dir.0.to_str().unwrap()));
 let plain=support::run(support::cli().arg("test").arg(file).current_dir(&dir.0).env("TSRUST_TEST_REPORTER","plain"),60);support::success(&plain);
 assert_eq!(String::from_utf8_lossy(&plain.stderr),"warning\n");
 assert!(String::from_utf8_lossy(&plain.stdout).starts_with("hello\nworld\nPASS greets\n"));
}

#[test]
fn pretty_console_keeps_hooks_failed_tests_and_later_test_logs_separate() {
 let dir=support::TempDirectory::new();let file=dir.0.join("hooks.test.tsr");
 std::fs::write(&file,support::format_round_trip("import {test,beforeAll,afterAll,beforeEach,strictEqual} from 'std:testing';beforeAll(():void=>{console.log('setup');});afterAll(():void=>{console.log('teardown');});beforeEach(():void=>{console.log('each');});test('fails',():void=>{console.log(42);strictEqual(1,2);});test('later',():void=>{console.log('survived');});")).unwrap();
 let output=support::run(support::cli().args(["test","--color","never"]).arg(file).current_dir(&dir.0).env("TSRUST_TEST_REPORTER","pretty"),60);assert_eq!(output.status.code(),Some(1));
 let text=String::from_utf8_lossy(&output.stdout);
 for expected in ["beforeAll","afterAll","× fails","✓ later","         42","         survived","1 passed | 1 failed"]{assert!(text.contains(expected),"{text}");}
 assert_eq!(text.matches("         each\n").count(),2);
 assert!(output.stderr.is_empty());
}
