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
    for tail in ["return <Text>{handle(first)} / {handle(second)}</Text>;", "if (flag) {return <Text>{handle(first)}</Text>;}return <Text>{handle(second)}</Text>;"] {
        let source=format!("import {{Text,signal,handle,Node}} from 'std:ui';function Component(flag:boolean):Node{{const first=signal(1);const second=signal(2);{tail}}}function main(){{const node=Component(true);}}");
        std::fs::write(&path,source).unwrap();let checked=check_graph(load_project(&FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap();
        assert!(checked.emit_rust().contains("tsrust_ui_own_state"));
    }
    for source in [
        "import {Text,signal,handle,Node} from 'std:ui';function Component():Node{const count=signal(0);const moved=count;return <Text>Hello</Text>;}function main(){}",
        "import {Text,signal,Node} from 'std:ui';function Component():Node{let count=signal(0);return <Text>Hello</Text>;}function main(){}",
        "import {ownState} from 'std:ui';function main(){}",
    ] {std::fs::write(&path,source).unwrap();assert!(load_project(&FileSourceProvider,path.to_str().unwrap()).and_then(check_graph).is_err());}
}
