use serde_json::{Value, json};
use tsrust_lsp::intelligence::SourceIndex;

const SOURCE: &str = "function parse(text: string): Checked<int, string> {\n    if (text.length === 0) {\n        throw 'empty';\n    }\n    return text.length;\n}\nfunction show(result: Result<int, RuntimeError | string>): int {\n    return 1;\n}\nfunction main() {\n    const parsed = parse('a');\n    console.log(show(parsed));\n}\n";

fn position(source: &str, offset: usize) -> Value {
    let prefix = &source[..offset];
    json!({"line": prefix.bytes().filter(|&c| c == b'\n').count(), "character": prefix.rsplit('\n').next().unwrap().encode_utf16().count()})
}

fn hover(needle: &str) -> String {
    let offset = SOURCE.find(needle).unwrap();
    SourceIndex::new(SOURCE)
        .request("textDocument/hover", &json!({"position": position(SOURCE, offset)}), "file:///checked.tsr")
        .unwrap()["contents"]["value"].as_str().unwrap_or("").to_owned()
}

#[test]
fn checked_and_runtime_error_have_documentation_completion_and_result_types() {
    assert!(tsrust_compiler::check(SOURCE).is_ok());
    let checked = hover("Checked<int");
    assert!(checked.contains("type Checked<T, E = never>") && checked.contains("catches every failure"), "{checked}");
    let runtime = hover("RuntimeError |");
    assert!(runtime.contains("type RuntimeError") && runtime.contains("`line`, `column`"), "{runtime}");
    let parsed = hover("parsed =");
    assert!(parsed.contains("Result<int, RuntimeError | string>"), "{parsed}");
    let typed = "function parse(text: string): Che";
    let items = SourceIndex::new(typed)
        .request("textDocument/completion", &json!({"position": position(typed, typed.len())}), "file:///checked.tsr")
        .unwrap();
    let item = items.as_array().unwrap().iter().find(|item| item["label"] == "Checked").unwrap_or_else(|| panic!("{items}"));
    assert_eq!(item["detail"], "type Checked<T, E = never>");
}
