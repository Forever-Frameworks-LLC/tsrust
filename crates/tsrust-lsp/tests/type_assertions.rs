use serde_json::json;
use tsrust_lsp::intelligence::SourceIndex;

#[test]
fn assertions_format_and_expose_checked_types() {
    let source = "function main(){const values=[] as int[];values.push(7);const widened=1 as number;const kept=3 satisfies number;console.log(widened,kept);}";
    tsrust_compiler::check(source).unwrap();
    let formatted = tsrust_compiler::format(source).unwrap();
    tsrust_compiler::check(&formatted).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
    let index = SourceIndex::new(source);
    for (name, ty) in [("values", "int[]"), ("widened", "number"), ("kept", "int")] {
        let position = source.find(name).unwrap();
        let hover = index.request("textDocument/hover", &json!({"position":{"line":0,"character":position}}), "file:///assertions.tsr").unwrap();
        assert!(hover["contents"]["value"].as_str().unwrap_or("").contains(&format!("{name}: {ty}")), "{hover}");
    }
}

#[test]
fn assertion_operators_have_syntax_highlighting() {
    let grammar: serde_json::Value = serde_json::from_str(include_str!("../../../editors/vscode/syntaxes/tsrust.tmLanguage.json")).unwrap();
    let rule = grammar["patterns"].as_array().unwrap().iter().find(|row| row["name"] == "keyword.operator.expression.tsrust").unwrap();
    assert!(rule["match"].as_str().unwrap().contains("as|satisfies"));
}

#[test]
fn generic_alias_assignment_formats_without_a_comparison_operator() {
    let source = "type Box<T >= {value:T;};function main(){const box:Box<int>={value:7};console.log(box.value);}";
    let formatted = tsrust_compiler::format(source).unwrap();
    assert!(formatted.starts_with("type Box<T> = {"), "{formatted}");
    tsrust_compiler::check(&formatted).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
    let index = SourceIndex::new(&formatted);
    let hover = index.request("textDocument/hover", &json!({"position":{"line":0,"character":5}}), "file:///alias.tsr").unwrap();
    assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("Box"), "{hover}");
}
