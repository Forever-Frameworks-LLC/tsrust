use serde_json::{Value, json};
use tsrust_lsp::intelligence::SourceIndex;

const SOURCE: &str = "interface Named {\n  name: string;\n}\ninterface Player extends Named {\n  score: int,\n  nickname?: string\n}\nfunction main() {\n  const player: Player = { name: \"Pat\", score: 3 };\n  console.log(player.name.length, player.score, player.nickname ?? \"none\");\n}\n";

fn position(source: &str, offset: usize) -> Value {
    let prefix = &source[..offset];
    json!({"line": prefix.bytes().filter(|&c| c == b'\n').count(), "character": prefix.rsplit('\n').next().unwrap().encode_utf16().count()})
}

fn ask(method: &str, needle: &str) -> Value {
    SourceIndex::new(SOURCE)
        .request(method, &json!({"position": position(SOURCE, SOURCE.find(needle).unwrap())}), "file:///interfaces.tsr")
        .unwrap()
}

fn hover(needle: &str) -> String {
    ask("textDocument/hover", needle)["contents"]["value"].as_str().unwrap_or("").to_owned()
}

#[test]
fn interfaces_have_symbols_fields_and_inherited_members() {
    assert!(tsrust_compiler::check(SOURCE).is_ok());
    assert!(hover("Player extends").contains("interface Player"), "{}", hover("Player extends"));
    assert!(hover("Player = {").contains("interface Player"), "{}", hover("Player = {"));
    assert!(hover("score, player").contains("score: int"), "{}", hover("score, player"));
    assert!(hover("name.length").contains("name: string"), "inherited field: {}", hover("name.length"));
    assert!(hover("nickname ?? ").contains("nickname?: string"), "{}", hover("nickname ?? "));
    let definition = ask("textDocument/definition", "Named {\n  score");
    assert_eq!(definition["range"]["start"], position(SOURCE, SOURCE.find("Named {").unwrap()), "{definition}");
    let symbols = ask("textDocument/documentSymbol", "interface");
    let names = symbols.as_array().unwrap().iter().map(|symbol| symbol["name"].as_str().unwrap_or("").to_owned()).collect::<Vec<_>>();
    for name in ["Named", "Player", "main"] {
        assert!(names.iter().any(|found| found == name), "{name}: {names:?}");
    }
    let typed = "interface Named { name: string; }\ninterface Player extends Named { score: int; }\nfunction main() {\n  const player: Player = { name: \"Pat\", score: 3 };\n  player.";
    let items = SourceIndex::new(typed)
        .request("textDocument/completion", &json!({"position": position(typed, typed.len())}), "file:///interfaces.tsr")
        .unwrap();
    for field in ["name", "score"] {
        assert!(items.as_array().unwrap().iter().any(|item| item["label"] == field), "{field}: {items}");
    }
}
