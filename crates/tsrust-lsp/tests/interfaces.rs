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

#[test]
fn generic_inherited_members_use_substituted_compiler_types() {
    let source="interface Box<T>{value:T;} interface Item extends Box<string>{count:int;} function main(){const item:Item={value:\"ok\",count:2};console.log(item.value);}";
    assert!(tsrust_compiler::check(source).is_ok());
    let index=SourceIndex::new(source);
    let hover=index.request("textDocument/hover",&json!({"position":position(source,source.find("value);").unwrap())}),"file:///generic.tsr").unwrap();
    assert!(hover["contents"]["value"].as_str().unwrap().contains("value: string"),"{hover}");
    let typed=source.replace("console.log(item.value);}","item.");
    let items=SourceIndex::new(&typed).request("textDocument/completion",&json!({"position":position(&typed,typed.len())}),"file:///generic.tsr").unwrap();
    for field in ["value","count"] {assert!(items.as_array().unwrap().iter().any(|item|item["label"]==field),"{items}");}
}

#[test]
fn imported_generic_inheritance_has_project_member_types() {
    use tsrust_lsp::{project_diagnostics::source_id,project_intelligence::ProjectIndex,workspace::Workspace};
    let uri="file:///generic-inheritance/main.tsr";
    let source="import {Base as Imported} from './base.tsr'; interface Item extends Imported<string>{count:int;} function main(){const item:Item={value:\"ok\",count:2};console.log(item.value);}";
    let mut workspace=Workspace::default();
    workspace.open(source_id("file:///generic-inheritance/base.tsr").unwrap(),1,"export interface Base<T>{value:T;}".into());
    let entry=source_id(uri).unwrap();workspace.open(entry.clone(),1,source.into());
    let checked=workspace.snapshot().check(&entry).unwrap();let index=ProjectIndex::checked(&checked);
    let hover=index.request("textDocument/hover",&json!({"position":position(source,source.find("value);").unwrap())}),uri).unwrap().unwrap();
    assert!(hover["contents"]["value"].as_str().unwrap().contains("value: string"),"{hover}");
    let completion=index.request("textDocument/completion",&json!({"position":position(source,source.find("value);").unwrap())}),uri).unwrap().unwrap();
    for name in ["value","count"]{assert!(completion.as_array().unwrap().iter().any(|item|item["label"]==name),"{completion}");}
}
