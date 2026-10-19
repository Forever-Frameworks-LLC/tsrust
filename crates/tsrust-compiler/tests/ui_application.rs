mod support;

#[test]
fn composite_generic_results_use_unique_union_members_without_guessing_between_payloads() {
    let source=support::format_round_trip("interface Box<T>{value:T;}function box<T>(value:T):Box<T>{return {value};}function accept(value:string|Box<int>):int{if(typeof value==='string'){return 0;}return value.value;}function either(value:Box<int>|Box<string>):boolean{return true;}function main(){console.log(accept(box(7)),either(box('text')));}");
    support::execute(&source,"7 true\n",None);
    let rejected=support::format_round_trip("interface Box<T>{value:T;}function empty<T>():Box<T>{throw 'unused';}function main(){const value:Box<int>|Box<string> = empty();}");
    let dir=support::TempDirectory::new();let path=dir.source(&rejected);let output=support::run(support::cli().arg("check").arg(&path),30);assert!(!output.status.success());assert!(String::from_utf8_lossy(&output.stderr).contains("generic-inference"));
}

#[test]
fn application_modes_visibility_and_options_check_emit_and_execute() {
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-application-state");
    for file in ["src/app/page.tsrx","src/main.tsr","src/inspect.tsr"] {
        let path=root.join(file);assert_eq!(std::fs::read_to_string(&path).unwrap(),support::format_round_trip(&std::fs::read_to_string(&path).unwrap()));
    }
    let output=support::run(support::cli().arg("run").arg(root.join("src/inspect.tsr")),120);support::success(&output);assert_eq!(output.stdout,b"Application contracts passed\n");
    let output=support::run(support::cli().arg("emit").arg(root.join("src/main.tsr")),40);support::success(&output);
    let rust=String::from_utf8(output.stdout).unwrap();assert!(rust.contains("run_configured"));assert!(rust.contains("with_theme_binding"));assert!(rust.contains("with_hidden"));
}

#[test]
fn application_props_reject_unsupported_types_and_do_not_change_unrelated_hidden_props() {
    let dir=support::TempDirectory::new();std::fs::write(dir.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();
    for body in ["const n=<Text tabIndex={0}>Wrong element</Text>;","const n=<View tabIndex='zero'/>;","const n=<Text hidden>Wrong element</Text>;","const n=<View windowDragRegion={1}/>;","const n=<View hidden='false'/>;","const n=withTheme(<View/>,config.ui.theme,signal(1));","const result=run(<View/>,'Demo',{titlebar:'frameless'});"] {
        let path=dir.0.join("main.tsrx");let source=support::format_round_trip(&format!("import {{View,Text,signal,withTheme,run}} from 'std:ui';import config from './tsrust.config.json';function main(){{{body}}}"));std::fs::write(&path,source).unwrap();
        let output=support::run(support::cli().arg("check").arg(&path),30);assert!(!output.status.success(),"{body}");assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    }
    let source=support::format_round_trip("import {signal} from 'std:ui';interface Props{hidden:boolean;}function Ordinary(props:Props):boolean{return props.hidden;}function main(){const show=signal(true);console.log(<Ordinary hidden={show.get()}/>);}");
    let path=dir.0.join("ordinary.tsrx");std::fs::write(&path,source).unwrap();let output=support::run(support::cli().arg("check").arg(&path),30);support::success(&output);
}

#[test]
fn documentation_foundation_formats_checks_emits_and_executes() {
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    for file in ["src/app/page.tsrx","src/examples/counter.tsrx","src/main.tsr","src/inspect.tsr"] {
        let path=root.join(file);assert_eq!(std::fs::read_to_string(&path).unwrap(),support::format_round_trip(&std::fs::read_to_string(&path).unwrap()));
    }
    let output=support::run(support::cli().arg("run").arg(root.join("src/inspect.tsr")),120);support::success(&output);assert_eq!(output.stdout,b"Documentation foundation passed\n");
    let output=support::run(support::cli().arg("emit").arg(root.join("src/main.tsr")),40);support::success(&output);
    let rust=String::from_utf8(output.stdout).unwrap();assert!(rust.contains("configure_scrolls"));assert!(!rust.contains("#[path = \"native_scrolling.rs\"]"));
}
