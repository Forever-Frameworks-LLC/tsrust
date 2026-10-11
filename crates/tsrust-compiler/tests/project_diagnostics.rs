mod support;
use std::fs;

#[test]
fn check_discovers_unopened_files_and_continues_after_each_error() {
    let directory=support::TempDirectory::new();
    fs::create_dir_all(directory.0.join("nested")).unwrap();
    fs::write(directory.0.join("a.tsr"),"function main(){const a:int=\"bad\";}").unwrap();
    fs::write(directory.0.join("nested/b.tsr"),"function main(){const b:boolean=42;}").unwrap();
    for excluded in ["target","node_modules",".git"] {
        fs::create_dir_all(directory.0.join(excluded)).unwrap();
        fs::write(directory.0.join(excluded).join("skip.tsr"),"this is invalid").unwrap();
    }
    fs::write(directory.0.join("not-source.txt"),"this is invalid").unwrap();
    let output=support::run(support::cli().arg("check").arg(&directory.0).arg("--json"),30);
    assert_eq!(output.status.code(),Some(1));
    let data=String::from_utf8(output.stdout.clone()).unwrap();
    assert!(data.starts_with("{\"ok\":false,\"diagnostics\":["),"{data}");
    assert_eq!(data.matches("\"code\":").count(),2,"{data}");
    assert!(data.find("/a.tsr").unwrap()<data.find("/nested/b.tsr").unwrap());
    assert!(output.stderr.is_empty());
    let default=support::run(support::cli().current_dir(&directory.0).arg("check").arg("--json"),30);
    assert_eq!(default.stdout,output.stdout);
}

#[test]
fn check_deduplicates_selected_files_and_passes_after_repair() {
    let directory=support::TempDirectory::new();
    let file=directory.source("function main(){const value:int=\"bad\";}");
    let output=support::run(support::cli().arg("check").arg(&file).arg(&directory.0).arg("--json"),30);
    let data=String::from_utf8(output.stdout).unwrap();
    assert_eq!(data.matches("\"code\":").count(),1,"{data}");
    fs::write(file,"function main(){const value:int=1;}").unwrap();
    let output=support::run(support::cli().arg("check").arg(&directory.0),30);
    support::success(&output);assert!(String::from_utf8_lossy(&output.stdout).contains("Check passed."));assert!(output.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn check_skips_symlink_cycles_and_source_aliases() {
    let directory=support::TempDirectory::new();
    let file=directory.source("function main(){}");
    std::os::unix::fs::symlink(&directory.0,directory.0.join("cycle")).unwrap();
    std::os::unix::fs::symlink(&file,directory.0.join("alias.tsr")).unwrap();
    support::success(&support::run(support::cli().arg("check").arg(&directory.0),10));
}

#[test]
fn check_reports_shared_dependency_errors_once_and_checks_library_files() {
    let directory=support::TempDirectory::new();
    fs::write(directory.0.join("library.tsr"),"export function value():int{return \"bad\";}").unwrap();
    for name in ["a.tsr","b.tsr"] {
        fs::write(directory.0.join(name),"import {value} from './library.tsr';function main(){console.log(value());}").unwrap();
    }
    let output=support::run(support::cli().arg("check").arg(&directory.0).arg("--json"),30);
    assert_eq!(output.status.code(),Some(1));
    let text=String::from_utf8(output.stdout).unwrap();
    assert_eq!(text.matches("\"code\":").count(),1,"{text}");
    assert!(text.contains("library.tsr"));
    fs::write(directory.0.join("library.tsr"),"export function value():int{return 7;}").unwrap();
    support::success(&support::run(support::cli().arg("check").arg(&directory.0),30));
}

#[test]
fn optional_main_checks_and_executes_declarations_without_calling_them() {
    for source in ["", "type Item={value:int;};", "function unused(){console.log(99);}"] {
        let checked=tsrust_compiler::check(source).unwrap();
        assert!(!checked.contracts().contains_key("main"));
        support::execute(source,"",None);
    }
    support::execute("function main(){console.log(7);}","7\n",None);
    for source in ["function main(value:int){}", "function main():int{return 7;}"] {
        assert_eq!(tsrust_compiler::check(source).err().unwrap().code,"entry");
    }
}

#[test]
fn optional_main_project_does_not_invoke_dependency_main() {
    let directory=support::TempDirectory::new();
    let entry=directory.source(&support::format_round_trip("import {value} from './library.tsr';function unused():int{return value();}"));
    fs::write(directory.0.join("library.tsr"),support::format_round_trip("export function value():int{return 7;}function main(){console.log(99);}")).unwrap();
    for command in ["check","emit","run"] {
        let output=support::run(support::cli().arg(command).arg(&entry),60);
        support::success(&output);
        if command=="run" {assert!(output.stdout.is_empty());}
    }
}

#[test]
fn check_reports_multiple_syntax_errors_in_one_file_and_rejects_build() {
    let directory=support::TempDirectory::new();
    let file=directory.source("function main(){const first=;const second=;}");
    let output=support::run(support::cli().arg("check").arg(&file).arg("--json"),30);
    assert_eq!(output.status.code(),Some(1));
    let text=String::from_utf8(output.stdout).unwrap();
    assert_eq!(text.matches("\"code\":").count(),2,"{text}");
    let output=support::run(support::cli().arg("build").arg(&file),30);
    assert_eq!(output.status.code(),Some(1));
}
