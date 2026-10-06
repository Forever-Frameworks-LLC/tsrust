use serde_json::{Value, json};
use tsrust_lsp::intelligence::SourceIndex;

const SOURCE: &str = "type Point = { x: int; y?: int; label?: string; };\nfunction main() {\n    const point: Point = { x: 1 };\n    const fallback = point.y ?? 0;\n    const text = point.label ?? \"none\";\n    console.log(point.x, fallback, text.length);\n}\n";

fn position(source: &str, offset: usize) -> Value {
    let prefix = &source[..offset];
    json!({"line": prefix.bytes().filter(|&c| c == b'\n').count(), "character": prefix.rsplit('\n').next().unwrap().encode_utf16().count()})
}

fn hover(needle: &str) -> String {
    let offset = SOURCE.find(needle).unwrap();
    SourceIndex::new(SOURCE)
        .request("textDocument/hover", &json!({"position": position(SOURCE, offset)}), "file:///fields.tsr")
        .unwrap()["contents"]["value"].as_str().unwrap_or("").to_owned()
}

#[test]
fn optional_fields_have_types_members_and_completion() {
    assert!(tsrust_compiler::check(SOURCE).is_ok());
    let declared = hover("y?: int");
    assert!(declared.contains("y?: int"), "{declared}");
    let used = hover("y ?? 0");
    assert!(used.contains("int") && used.contains("undefined"), "{used}");
    let label = hover("label ?? ");
    assert!(label.contains("string") && label.contains("undefined"), "{label}");
    assert!(hover("fallback =").contains("fallback: int"), "{}", hover("fallback ="));
    assert!(hover("text =").contains("text: string"), "{}", hover("text ="));
    let typed = "type Point = { x: int; y?: int; label?: string; };\nfunction main() {\n    const point: Point = { x: 1 };\n    point.";
    let items = SourceIndex::new(typed)
        .request("textDocument/completion", &json!({"position": position(typed, typed.len())}), "file:///fields.tsr")
        .unwrap();
    for field in ["x", "y", "label"] {
        assert!(items.as_array().unwrap().iter().any(|item| item["label"] == field), "{field}: {items}");
    }
    let definition = {
        let offset = SOURCE.find("y ?? 0").unwrap();
        SourceIndex::new(SOURCE).request("textDocument/definition", &json!({"position": position(SOURCE, offset)}), "file:///fields.tsr").unwrap()
    };
    assert_eq!(definition["range"]["start"], position(SOURCE, SOURCE.find("y?: int").unwrap()), "{definition}");
}
