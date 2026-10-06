use serde_json::{Value, json};
use tsrust_lsp::intelligence::SourceIndex;

const SOURCE: &str = "function initial(word: string): string { return word[0]; }\nfunction main() {\n    const text = \"abc\";\n    const first = text[0];\n    const upper = text[1].toUpperCase();\n    const same = text[2] === \"c\";\n    console.log(first.length, upper, same, initial(text));\n}\n";

fn ask(method: &str, needle: &str) -> Value {
    let offset = SOURCE.find(needle).unwrap();
    let prefix = &SOURCE[..offset];
    SourceIndex::new(SOURCE)
        .request(method, &json!({"position": {"line": prefix.bytes().filter(|&c| c == b'\n').count(), "character": prefix.rsplit('\n').next().unwrap().encode_utf16().count()}}), "file:///index.tsr")
        .unwrap()
}

fn hover(needle: &str) -> String {
    ask("textDocument/hover", needle)["contents"]["value"].as_str().unwrap_or("").to_owned()
}

#[test]
fn string_positions_are_strings_in_the_editor() {
    assert!(tsrust_compiler::check(SOURCE).is_ok());
    assert!(hover("first =").contains("first: string"), "{}", hover("first ="));
    assert!(hover("upper =").contains("upper: string"), "{}", hover("upper ="));
    assert!(hover("same =").contains("same: boolean"), "{}", hover("same ="));
    assert!(!hover("toUpperCase").is_empty(), "string member after a string position");
    assert!(!hover("length, upper").is_empty(), "member of a binding from a string position");
    let typed = "function main() {\n    const text = \"abc\";\n    text[0].";
    let prefix = typed;
    let items = SourceIndex::new(typed)
        .request("textDocument/completion", &json!({"position": {"line": prefix.bytes().filter(|&c| c == b'\n').count(), "character": prefix.rsplit('\n').next().unwrap().encode_utf16().count()}}), "file:///index.tsr")
        .unwrap();
    assert!(items.as_array().unwrap().iter().any(|item| item["label"] == "toUpperCase"), "{items}");
}
