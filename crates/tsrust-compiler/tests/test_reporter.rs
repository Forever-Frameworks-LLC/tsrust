mod support;
#[test]
fn pretty_report_groups_files_shows_assertion_values_and_totals_without_ansi_when_disabled() {
 let dir=support::TempDirectory::new();
 let source=support::format_round_trip("import {test,strictEqual} from 'std:testing';test('greeting',():void=>{strictEqual('hello','goodbye');});test('passes',():void=>{strictEqual(1,1);});");
 let file=dir.0.join("greeting.test.tsr");std::fs::write(&file,source).unwrap();
 let output=support::run(support::cli().args(["test","--color","never"]).arg(&file).env("TSRUST_TEST_REPORTER","pretty"),60);
 assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));
 assert_eq!(output.status.code(),Some(1));let text=String::from_utf8_lossy(&output.stdout);
 for expected in ["greeting.test.tsr","✓ passes","× greeting","Failed Tests","Expected:","\"goodbye\"","Received:","\"hello\"","Test Files","Tests","Duration"]{assert!(text.contains(expected),"missing {expected}: {text}");}
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
