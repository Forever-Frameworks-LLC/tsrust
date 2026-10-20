mod support;
use std::fs;

#[test]
fn text_imports_preserve_utf8_bytes_and_invalidate_native_cache_without_source_changes(){
 let directory=support::TempDirectory::new();let text="\u{feff}Not executable {\r\n\"café 🦀\"\0\r\n";fs::write(directory.0.join("payload.txt"),text).unwrap();
 let source=format!("import text from './payload.txt';import {{strictEqual}} from 'std:testing';function main(){{strictEqual(text,{});strictEqual(text.length,{});const copy=clone(text);strictEqual(copy,text);}}",tsrust_compiler::json_string(text),text.len());let entry=directory.0.join("main.tsr");fs::write(&entry,support::format_round_trip(&source)).unwrap();support::success(&support::run(support::cli().arg("run").arg(&entry),180));
 let source="import text from './payload.txt';function main(){console.log(text.startsWith('Second'));}";fs::write(&entry,support::format_round_trip(source)).unwrap();let first=support::run(support::cli().arg("run").arg(&entry),180);support::success(&first);assert_eq!(first.stdout,b"false\n");
 fs::write(directory.0.join("payload.txt"),"Second version").unwrap();let second=support::run(support::cli().arg("run").arg(&entry),180);support::success(&second);assert_eq!(second.stdout,b"true\n");
 fs::write(directory.0.join("payload.txt"),[0xff]).unwrap();let invalid=support::run(support::cli().arg("check").arg(&entry),30);assert!(!invalid.status.success());assert!(String::from_utf8_lossy(&invalid.stderr).contains("module-load"));
}

#[test]
fn text_imports_are_readonly_default_values_and_named_exports_reject(){
 let directory=support::TempDirectory::new();fs::write(directory.0.join("payload.txt"),"Hello").unwrap();let entry=directory.0.join("main.tsr");
 for (source,code) in [("import text from './payload.txt';function main(){const moved=text;}","data-readonly"),("import {text} from './payload.txt';function main(){}","module-private")]{fs::write(&entry,support::format_round_trip(source)).unwrap();let output=support::run(support::cli().arg("check").arg(&entry),30);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains(code),"{}",String::from_utf8_lossy(&output.stderr));}
}
