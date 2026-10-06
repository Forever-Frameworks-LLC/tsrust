use serde_json::{Value, json};
use tsrust_lsp::intelligence::SourceIndex;

const SOURCE: &str = "type Id = int | string;\ntype Ids = Id[];\ntype Point = { x: int; label: string; };\ntype Points = Point[];\ntype Name = string;\nfunction first(points: Points): Point { return clone(points[0]); }\nfunction main() {\n    const ids: Ids = [1, \"b\"];\n    const name: Name = \"pat\";\n    const points: Points = [{ x: 1, label: \"a\" }];\n    for (const point of points) { console.log(point.label.length); }\n    console.log(ids.length, name.toUpperCase(), first(points).x);\n}\n";

fn position(needle: &str, n: usize) -> Value {
    let offset = SOURCE.match_indices(needle).nth(n).unwrap().0;
    let prefix = &SOURCE[..offset];
    json!({"line": prefix.bytes().filter(|&c| c == b'\n').count(), "character": prefix.rsplit('\n').next().unwrap().encode_utf16().count()})
}

fn ask(method: &str, needle: &str, n: usize) -> Value {
    SourceIndex::new(SOURCE)
        .request(method, &json!({"position": position(needle, n), "context": {"includeDeclaration": true}}), "file:///aliases.tsr")
        .unwrap()
}

fn hover(needle: &str, n: usize) -> String {
    ask("textDocument/hover", needle, n)["contents"]["value"].as_str().unwrap_or("").to_owned()
}

#[test]
fn alias_declarations_and_uses_have_hover_and_definition() {
    assert!(tsrust_compiler::check(SOURCE).is_ok());
    let declared = hover("Ids = ", 0);
    assert!(declared.contains("type Ids = Id[]"), "{declared}");
    let used = hover("Ids = [1", 0);
    assert!(used.contains("type Ids = Id[]"), "{used}");
    let definition = ask("textDocument/definition", "Ids = [1", 0);
    assert_eq!(definition["range"]["start"], position("Ids = Id", 0), "{definition}");
    let nested = hover("Id[]", 0);
    assert!(nested.contains("type Id = int | string"), "{nested}");
    let symbols = ask("textDocument/documentSymbol", "type", 0);
    let names = symbols.as_array().unwrap().iter().map(|symbol| symbol["name"].as_str().unwrap_or("").to_owned()).collect::<Vec<_>>();
    for name in ["Id", "Ids", "Point", "Points", "Name", "first", "main"] {
        assert!(names.iter().any(|found| found == name), "{name}: {names:?}");
    }
}

#[test]
fn alias_typed_values_keep_members_and_element_types() {
    let binding = hover("ids: Ids", 0);
    assert!(binding.contains("ids: (int|string)[]") || binding.contains("ids: (int | string)[]"), "{binding}");
    assert!(!hover("length, name", 0).is_empty(), "array member through an alias");
    assert!(!hover("toUpperCase", 0).is_empty(), "string member through an alias");
    let element = hover("point of", 0);
    assert!(element.contains("point: Point"), "{element}");
    assert!(!hover("label.length", 0).is_empty(), "record field through an alias element");
    let field = hover("x);", 0);
    assert!(field.contains("x: int"), "{field}");
    let typed = "type Words = string[];\nfunction main() {\n    const words: Words = [\"a\"];\n    words.";
    let offset = typed.len();
    let prefix = &typed[..offset];
    let items = SourceIndex::new(typed)
        .request("textDocument/completion", &json!({"position": {"line": prefix.bytes().filter(|&c| c == b'\n').count(), "character": prefix.rsplit('\n').next().unwrap().encode_utf16().count()}}), "file:///aliases.tsr")
        .unwrap();
    assert!(items.as_array().unwrap().iter().any(|item| item["label"] == "push"), "{items}");
}
