use serde_json::{json, Value};
use tsrust_lsp::{project_diagnostics::source_id, project_intelligence::ProjectIndex, workspace::Workspace};
fn uri() -> &'static str { if cfg!(windows) { "file:///C:/tsrust-codecs/main.tsr" } else { "file:///tsrust-codecs/main.tsr" } }
fn ask(index: &ProjectIndex, source: &str, needle: &str, method: &str) -> Value {
    let offset = source.rfind(needle).unwrap(); let prefix = &source[..offset];
    let position = json!({"line":prefix.bytes().filter(|b| *b==b'\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()});
    index.request(method, &json!({"position":position}), uri()).unwrap().unwrap()
}
#[test]
fn codec_examples_alias_hover_and_order_parameter_help_agree_with_compiler() {
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let names = ["toHex", "fromHex", "toBase64", "fromBase64", "toBase64Url", "fromBase64Url", "toUtf8Lossy", "fromUtf16", "toUtf16"];
    for name in names {
        let entry = catalog["std:bytes"].as_array().unwrap().iter().find(|entry| entry["name"]==name).unwrap();
        let source = tsrust_compiler::format(entry["examples"][0]["source"].as_str().unwrap()).unwrap();
        let mut workspace = Workspace::default(); workspace.open(source_id(uri()).unwrap(), 1, source.clone());
        let checked = workspace.snapshot().check(&source_id(uri()).unwrap()).unwrap();
        let index = ProjectIndex::checked(&checked);
        let hover = ask(&index, &source, &format!("{name}("), "textDocument/hover");
        assert!(hover.to_string().contains("ByteError") && hover.to_string().contains("Ownership"), "{name}: {hover}");
        if matches!(name, "fromUtf16" | "toUtf16") {
            let help = ask(&index, &source, "\"little\"", "textDocument/signatureHelp");
            assert_eq!(help["activeParameter"], 1, "{help}");
            assert!(help.to_string().contains("byteOrder") && help.to_string().contains("big"), "{help}");
        }
    }
    let source = tsrust_compiler::format("import {fromUtf8,toBase64 as encode} from 'std:bytes';function main(){const saved=encode;const bytes=unwrap(fromUtf8('f'));saved(bytes);}").unwrap();
    let mut workspace = Workspace::default(); workspace.open(source_id(uri()).unwrap(), 1, source.clone());
    let checked = workspace.snapshot().check(&source_id(uri()).unwrap()).unwrap(); let index = ProjectIndex::checked(&checked);
    let hover = ask(&index, &source, "saved(bytes)", "textDocument/hover");
    assert!(hover.to_string().contains("canonical") && hover.to_string().contains("padding"), "{hover}");
}

#[test]
fn codec_exports_are_discoverable_from_partial_imports() {
    for (prefix, expected) in [
        ("toB", vec!["toBase64", "toBase64Url"]),
        ("fromUtf", vec!["fromUtf8", "fromUtf16"]),
        ("toUtf", vec!["toUtf8", "toUtf8Lossy", "toUtf16"]),
    ] {
        let source = format!("import {{ {prefix} }} from 'std:bytes';function main(){{}}");
        let mut workspace = Workspace::default(); workspace.open(source_id(uri()).unwrap(), 1, source.clone());
        let graph = tsrust_compiler::project::load_project_headers(&workspace.snapshot(), &source_id(uri()).unwrap().0).unwrap();
        let completion = ask(&ProjectIndex::new(&graph), &source, " }", "textDocument/completion");
        let labels = completion.as_array().unwrap().iter().map(|item| item["label"].as_str().unwrap()).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(labels, expected.into_iter().collect(), "{completion}");
    }
}
