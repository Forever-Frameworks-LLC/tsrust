mod support;
use tsrust_compiler::project::{FileSourceProvider, check_graph, load_project};
fn project(entry: &str) -> tsrust_compiler::project::ProjectAnalysis {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ui-static")
        .join(entry);
    check_graph(load_project(&FileSourceProvider, path.to_str().unwrap()).unwrap()).unwrap()
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
        assert!(checked.emit_rust().contains("tsrust_ui_own_state"));
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
    let dir=support::TempDirectory::new();let path=dir.0.join("main.tsrx");
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
        let rust=checked.emit_rust();assert!(rust.contains("tsrust_ui_handle"));
        if construction.contains("const factory") {
            let rust_path=dir.0.join("nested.rs");let binary=dir.0.join("nested");std::fs::write(&rust_path,rust).unwrap();
            let compiled=support::run(support::rustc().arg("--edition=2024").arg(&rust_path).arg("-o").arg(&binary),60);support::success(&compiled);
            let output=support::run(&mut std::process::Command::new(binary),10);support::success(&output);
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
