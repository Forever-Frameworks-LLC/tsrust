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
