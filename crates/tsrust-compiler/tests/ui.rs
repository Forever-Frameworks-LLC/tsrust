mod support;
use tsrust_compiler::project::{FileSourceProvider, check_graph, load_project};
fn project(entry: &str) -> tsrust_compiler::project::ProjectAnalysis {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ui-static")
        .join(entry);
    check_graph(load_project(&FileSourceProvider, path.to_str().unwrap()).unwrap()).unwrap()
}
fn configure_ui(root: &std::path::Path) {
    std::fs::write(root.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();
}
#[test]
fn ui_generated_static_targets_share_checked_nodes() {
    let html = project("web.tsr").emit_rust();
    assert!(html.contains("tsrust_native_ui::document"));
    assert!(
        !html.contains("use vello::"),
        "HTML must not include native renderer"
    );
    let native = project("native.tsr").emit_rust();
    assert!(native.contains("use vello::"));
    assert!(native.contains("tsrust_native_ui::run"));
}
#[test]
fn ui_nodes_preserve_unique_ownership_and_private_runtime() {
    let dir = support::TempDirectory::new();
    for source in [
        "import {text} from 'std:ui-runtime';function main(){}",
        "import {Text,document} from 'std:ui';function main(){const a=<Text>hello</Text>;const b=a;console.log(document(a,'bad'));}",
        "import {Text} from 'std:ui';function main(){const a=<Text>hello</Text>;const b=clone(a);}",
    ] {
        let path = dir.0.join("main.tsrx");
        std::fs::write(&path, source).unwrap();
        let result =
            load_project(&FileSourceProvider, path.to_str().unwrap()).and_then(check_graph);
        assert!(result.is_err(), "{source}");
    }
}

#[test]
fn counter_uses_owned_handlers_and_independent_signal_owners() {
    let entry=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-counter/main.tsr");
    let checked=check_graph(load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap()).unwrap();
    let rust=checked.emit_rust();
    assert!(rust.contains("tsrust_ui_press_button"));
    assert!(rust.contains("Signal::new"));
    assert!(rust.contains("Callback::invoke(&mut registered_callback"));
}

#[test]
fn component_return_retains_state_through_checked_moves() {
    let dir=support::TempDirectory::new();let path=dir.0.join("main.tsrx");
    for tail in ["return <Text>{first} / {second}</Text>;", "if (flag) {return <Text>{first}</Text>;}return <Text>{second}</Text>;"] {
        let source=format!("import {{Text,signal,Node}} from 'std:ui';function Component(flag:boolean):Node{{const first=signal(1);const second=signal(2);{tail}}}function main(){{const node=Component(true);}}");
        std::fs::write(&path,support::format_round_trip(&source)).unwrap();let checked=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap();
        assert!(checked.emit_rust().contains(".own_state(state)"));
    }
    for source in [
        "import {Text,signal,Node} from 'std:ui';function Component():Node{const count=signal(0);return count;}function main(){}",
        "import {Text,signal,Node} from 'std:ui';function Component():Node{let count=signal(0);return <Text>Hello</Text>;}function main(){}",
        "import {add} from 'std:ui';function main(){}",
        "import {handle} from 'std:ui';function main(){}",
        "import {SignalHandle} from 'std:ui';function main(){}",
        "import {ownState} from 'std:ui';function main(){}",
    ] {std::fs::write(&path,source).unwrap();assert!(load_project(&FileSourceProvider,path.to_str().unwrap()).and_then(check_graph).is_err());}
}

#[test]
fn direct_state_bindings_and_callback_capture_preserve_scopes() {
    let dir=support::TempDirectory::new();configure_ui(&dir.0);let path=dir.0.join("main.tsrx");
    for construction in [
        "const press=()=>{count.set(count.get() + 1);};press();return (<Text>{count} / {count}</Text>);",
        "const factory=()=>{return ()=>{count.set(count.get() + 1);};};const first=factory();const second=factory();first();second();return (<View><Text>{count}</Text><Button onPress={()=>{count.set(count.get()+1);}}>Increment</Button></View>);",
        "const shadow=(count:int)=>{console.log(count);};shadow(9);return (<Text>{count}</Text>);",
        "const press:Owned<Function<void>>=flag ? ()=>{count.set(count.get() + 1);} : ()=>{count.set(count.get() + 2);};press();return (<Text>{count}</Text>);",
        "let i=0;while(i<2){const press=()=>{count.set(count.get() + 1);};press();i+=1;}return (<Text>{count}</Text>);",
    ] {
        let source=format!("import {{Text,View,Button,signal,Node,document}} from 'std:ui';function Component(flag:boolean):Node{{const count=signal(0);{construction}}}function main(){{console.log(document(Component(true),'test'));const limit=signal(9223372036854775807);try{{limit.set(limit.get()+1);}}catch(error){{console.log(limit.get());}}}}");
        let formatted=support::format_round_trip(&source);std::fs::write(&path,formatted).unwrap();
        let checked=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap_or_else(|error|panic!("{construction}: {error:?}"));
        let rust=checked.emit_rust();assert!(rust.contains(".handle()"));
        if construction.contains("const factory") {
            let output=support::run(support::cli().arg("run").arg(&path),90);support::success(&output);
            let output=String::from_utf8(output.stdout).unwrap();assert!(output.contains(">2</span>"));assert!(output.ends_with("9223372036854775807\n"));
        }
    }
    for construction in [
        "const f=()=>{const nested=signal(1);};return <Text>{count}</Text>;",
        "const f=signal;return <Text>{count}</Text>;",
        "const f=clone(count);return <Text>{count}</Text>;",
        "{const count=1;return <Text>{count}</Text>;}",
    ] {
        let source=format!("import {{Text,signal,Node}} from 'std:ui';function Component():Node{{const count=signal(0);{construction}}}function main(){{}}");
        std::fs::write(&path,source).unwrap();assert!(load_project(&FileSourceProvider,path.to_str().unwrap()).and_then(check_graph).is_err());
    }
}

#[test]
fn application_theme_local_components_and_prop_forwarding_are_checked() {
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-application");
 let entry=root.join("web.tsr");let checked=check_graph(load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap()).unwrap();assert!(checked.emit_rust().contains("with_theme"));
 let dir=support::TempDirectory::new();let entry=dir.0.join("main.tsrx");
 for source in ["import {View} from 'std:ui';function main(){const n=<View className=\"unknown\"/>;}","import {View} from 'std:ui';function main(){const n=<View className={'unknown'}/>;}"] {std::fs::write(&entry,source).unwrap();let error=load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap_err();assert_eq!(error.code,"ui-class");}
 let source="import {Button,ButtonProps,Node,document} from 'std:ui';function Forward(props:ButtonProps):Node{return <Button {...props} className=\"rounded\"/>;}function main(){const n=<Forward disabled>Done</Forward>;console.log(document(n,'Forward'));}";
 std::fs::write(&entry,support::format_round_trip(source)).unwrap();check_graph(load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap()).unwrap();
}

#[test]
fn state_dependent_disabled_condition_is_retained_and_not_a_snapshot() {
 let dir=support::TempDirectory::new();configure_ui(&dir.0);let entry=dir.0.join("main.tsrx");
 let source="import {Button,View,Node,signal,document} from 'std:ui';function Page():Node{const count=signal(0);return <View><Button onPress={()=>{count.set(count.get()+1);}}>Increment</Button><Button disabled={count.get() < 5} onPress={()=>{count.set(count.get()+100);}}>Disabled</Button></View>;}function main(){console.log(document(Page(),'Reactive'));}";
 std::fs::write(&entry,support::format_round_trip(source)).unwrap();let checked=check_graph(load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap()).unwrap();let rust=checked.emit_rust();assert!(rust.contains("tsrust_ui_bind_disabled"));assert!(rust.contains("Result<bool"));
 let output=support::run(support::cli().arg("run").arg(&entry),120);support::success(&output);assert!(String::from_utf8(output.stdout).unwrap().contains(" disabled class="));
}

#[test]
fn configured_theme_validation_runs_for_checks_and_preserves_tolerant_headers() {
 let dir=support::TempDirectory::new();std::fs::create_dir(dir.0.join("design")).unwrap();
 std::fs::write(dir.0.join("tsrust.config.json"),r#"{"ui":{"theme":"design/tokens.json","components":"local/controls","lock":"ui.lock.json"}}"#).unwrap();
 let source="import theme from './design/tokens.json';function main(){console.log(theme.schemaVersion);}";
 let entry=dir.0.join("main.tsr");std::fs::write(&entry,support::format_round_trip(source)).unwrap();
 let tokens=tsrust_compiler::ui_project::theme::DEFAULT_JSON;
 std::fs::write(dir.0.join("design/tokens.json"),tokens.replacen("#2563eb","#12345g80",1)).unwrap();
 let error=load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap_err();assert_eq!(error.code,"ui-theme");assert!(error.location.source.0.ends_with("design/tokens.json"));
 tsrust_compiler::project::load_project_headers(&FileSourceProvider,entry.to_str().unwrap()).unwrap();
 std::fs::write(dir.0.join("design/tokens.json"),tokens).unwrap();check_graph(load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap()).unwrap();
}

#[test]
fn installed_ui_compatibility_pins_block_incompatible_builds_without_rewriting() {
 let dir=support::TempDirectory::new();let entry=dir.0.join("main.tsr");
 std::fs::write(&entry,support::format_round_trip("function main(){console.log('owned source');}")).unwrap();
 std::fs::write(dir.0.join("tsrust.config.json"),r#"{"ui":{"theme":"ui.theme.json","components":"components/ui","lock":"ui.lock.json"}}"#).unwrap();
 let lock=serde_json::json!({"schemaVersion":1,"installed":[{"compilerApi":999,"runtimeAbi":"ui-application-1","rust":tsrust_compiler::RUST_BACKEND_VERSION,"compilerSha256":"a".repeat(64)}]}).to_string();
 std::fs::write(dir.0.join("ui.lock.json"),&lock).unwrap();let error=load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap_err();assert!(error.message.contains("migrate explicitly"),"{error:?}");assert_eq!(std::fs::read_to_string(dir.0.join("ui.lock.json")).unwrap(),lock);
}

#[test]
fn inline_theme_is_validated_without_importing_the_configuration() {
 let dir=support::TempDirectory::new();let entry=dir.0.join("main.tsr");std::fs::write(&entry,support::format_round_trip("function main(){}")).unwrap();
 let mut ui=serde_json::to_value(tsrust_compiler::ui_project::Config::default()).unwrap();let config=dir.0.join("tsrust.config.json");
 std::fs::write(&config,serde_json::json!({"ui":ui}).to_string()).unwrap();load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap();
 ui["theme"]["radius"]["sm"]=serde_json::json!(-1);std::fs::write(&config,serde_json::json!({"ui":ui}).to_string()).unwrap();let error=load_project(&FileSourceProvider,entry.to_str().unwrap()).unwrap_err();assert!(error.message.contains("radius.sm"),"{error:?}");tsrust_compiler::project::load_project_headers(&FileSourceProvider,entry.to_str().unwrap()).unwrap();
}

#[test]
fn exact_semantic_attributes_reach_generated_native_html() {
    let entry=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-semantics/inspect.tsr");
    let output=support::run(support::cli().arg("run").arg(entry),90);support::success(&output);assert_eq!(output.stdout,b"UI semantics passed\n");
}

#[test]
fn composed_children_and_legacy_arrays_execute_without_cloning() {
    let entry=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-composition/inspect.tsr");
    let output=support::run(support::cli().arg("run").arg(entry),120);support::success(&output);assert_eq!(output.stdout,b"UI composition passed\n");
}

#[test]
fn composed_children_reject_copying_through_unions_and_infer_native_callbacks() {
    let dir=support::TempDirectory::new();let path=dir.0.join("main.tsrx");
    let source="import {Text,Child} from 'std:ui';function main(){const child:Child=<Text>owned</Text>;const children=[child];children.forEach((item)=>{});}";
    std::fs::write(&path,support::format_round_trip(source)).unwrap();load_project(&FileSourceProvider,path.to_str().unwrap()).and_then(check_graph).unwrap();
    for operation in ["clone(child)", "clone([child])", "[child].slice()", "[child].filter((item)=>{return true;})", "clone({nested:[child]} as Holder)"] {
        let source=format!("import {{Text,Child}} from 'std:ui';type Holder={{nested:Child[];}};function main(){{const child:Child=<Text>owned</Text>;const copy={operation};}}");
        std::fs::write(&path,source).unwrap();let error=load_project(&FileSourceProvider,path.to_str().unwrap()).and_then(check_graph).err().expect("Child ownership must reject cloning");
        assert_eq!(error.code,"clone-type","{operation}: {error:?}");
    }
}

#[test]
fn shared_grid_layout_and_custom_theme_execute_in_formatted_projects() {
    let entry=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-layout/inspect.tsr");
    let output=support::run(support::cli().arg("run").arg(entry),120);support::success(&output);assert_eq!(output.stdout,b"UI layout passed\n");
    let dir=support::TempDirectory::new();let path=dir.0.join("main.tsrx");
    for class in ["grid-cols-0","grid-cols-13","col-span-13","max-w-auto","w-none","grid-cols-[2fr_1fr]"] {
        std::fs::write(&path,format!("import {{View}} from 'std:ui';function main(){{const node=<View className=\"{class}\"/>;}}")).unwrap();
        let error=load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap_err();assert_eq!(error.code,"ui-class","{class}: {error:?}");
    }
}

#[test]
fn checked_colors_rings_and_unchanged_theme_data_execute_in_formatted_projects() {
 let entry=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-colors/inspect.tsr");
 let output=support::run(support::cli().arg("run").arg(entry),120);support::success(&output);assert_eq!(output.stdout,b"UI colors passed\n");
 let dir=support::TempDirectory::new();let path=dir.0.join("main.tsrx");
 for class in ["bg-primary/101","bg-primary/-1","bg-primary/50.5","ring-1/50"] {
  std::fs::write(&path,format!("import {{View}} from 'std:ui';function main(){{const node=<View className=\"{class}\"/>;}}")).unwrap();assert_eq!(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap_err().code,"ui-class");
 }
}

#[test]
fn extended_jsx_props_move_resources_and_ignore_only_excess_spread_fields() {
    let entry=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-forwarding/inspect.tsrx");
    let output=support::run(support::cli().arg("run").arg(&entry),120);support::success(&output);assert_eq!(output.stdout,b"UI forwarding passed\n");
    let dir=support::TempDirectory::new();configure_ui(&dir.0);let path=dir.0.join("main.tsrx");
    for source in [
      "import {View,ViewProps,Node} from 'std:ui';interface Props extends ViewProps{size?:string;}function Forward(p:Props):Node{const node=<View {...p}/>;return <View {...p}/>;}function main(){}",
      "import {View,ViewProps,Node} from 'std:ui';interface Props extends ViewProps{size?:string;}function Forward(p:ReadOnly<Props>):Node{return <View {...p}/>;}function main(){}",
      "import {View,ViewProps,Node} from 'std:ui';interface Props extends ViewProps{size?:string;}function Forward(p:Props):Node{return <View {...p} size='explicit'/>;}function main(){}",
      "import {View,ViewProps,Node} from 'std:ui';interface Props extends ViewProps{size?:string;}function Forward(p:Props):Node{return View({...p});}function main(){}",
    ]{std::fs::write(&path,source).unwrap();assert!(load_project(&FileSourceProvider,path.to_str().unwrap()).and_then(check_graph).is_err(),"{source}");}
}

#[test]
fn nova_card_content_source_is_local_complete_and_native_executable() {
 let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/ui-catalog/components-3.0.0.json")).unwrap();let source=catalog[0]["files"][0]["content"].as_str().unwrap();
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-card");let installed=std::fs::read_to_string(root.join("src/components/ui/card.tsrx")).unwrap();assert_eq!(support::format_round_trip(source),installed);
 for part in ["Card","CardHeader","CardTitle","CardDescription","CardAction","CardContent","CardFooter"]{assert!(source.contains(&format!("export function {part}(")));}
 let entry=root.join("src/inspect.tsr");let output=support::run(support::cli().arg("run").arg(entry),120);support::success(&output);assert_eq!(output.stdout,b"Card content parts passed\n");
 let dir=support::TempDirectory::new();let added=support::run(support::cli().args(["ui","add","card@3.0.0","--target","native","--offline"]).current_dir(&dir.0),20);support::success(&added);
 assert_eq!(std::fs::read_to_string(dir.0.join("src/components/ui/card.tsrx")).unwrap(),installed);
 let older=support::run(support::cli().args(["ui","add","button@2.0.0","--offline"]).current_dir(&dir.0),20);support::success(&older);
 let lock:serde_json::Value=serde_json::from_slice(&std::fs::read(dir.0.join("ui-components.lock.json")).unwrap()).unwrap();let entries=lock["installed"].as_array().unwrap();
 assert!(entries.iter().any(|entry|entry["identity"]["name"]=="card" && entry["compilerApi"]==2 && entry["licenseText"].as_str().unwrap().contains("Copyright (c) 2023 shadcn")));
 assert!(entries.iter().any(|entry|entry["identity"]["name"]=="button" && entry["compilerApi"]==1));
 assert!(tsrust_compiler::ui_project::supports_compiler_api(1));assert!(tsrust_compiler::ui_project::supports_compiler_api(2));assert!(!tsrust_compiler::ui_project::supports_compiler_api(0));assert!(!tsrust_compiler::ui_project::supports_compiler_api(3));
}
