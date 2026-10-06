use lsp_types::Position;
use serde_json::{Value, json};
use tsrust_lsp::intelligence::SourceIndex;
use tsrust_lsp::{
    project_diagnostics::source_id, project_intelligence::ProjectIndex, workspace::Workspace,
};

fn position(source: &str, needle: &str) -> Position {
    let offset = source.find(needle).expect(needle);
    let prefix = &source[..offset];
    Position::new(
        prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
}
fn ask(source: &str, method: &str, needle: &str) -> Value {
    SourceIndex::new(source)
        .request(
            method,
            &json!({"position": position(source, needle)}),
            "file:///nullable-callables.tsr",
        )
        .unwrap()
}
fn text(value: &Value) -> &str {
    value["contents"]["value"].as_str().expect("hover")
}
fn rejected_at(source: &str, code: &str, token: &str, message: &str) {
    let error = tsrust_compiler::check(source)
        .err()
        .expect("the specified callable contract must be rejected");
    assert_eq!(error.code, code, "{error:?}");
    assert_eq!(
        &source[error.span.start..error.span.end],
        token,
        "{error:?}"
    );
    assert!(error.message.contains(message), "{error:?}");
}
fn native(source: &str, expected: &str) {
    let analysis = tsrust_compiler::check(source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/nullable-callables-language-examples");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("examples.tsr"), source).unwrap();
    std::fs::write(directory.join("examples.rs"), analysis.emit_rust()).unwrap();
    std::fs::write(directory.join("expected.stdout"), expected).unwrap();
    for level in [0, 3] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = std::process::Command::new("rustup")
            .args(["run", "1.96.1", "rustc", "--edition=2024", "-C"])
            .arg(format!("opt-level={level}"))
            .arg(directory.join("examples.rs"))
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stdout")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-o{level}.stderr")),
            &build.stderr,
        )
        .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = std::process::Command::new(binary).output().unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stdout")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("examples-o{level}.stderr")),
            &output.stderr,
        )
        .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn nullable_callable_editor_contracts_and_language_examples_follow_checked_payloads() {
    let local = r#"function main(){const action:(Function<int, [input:int]>)|null=(value:int):int=>{return value+1;};if(action!==null){console.log(action(4));}console.log(typeof action,Boolean(action),Array.isArray(action));}"#;
    tsrust_compiler::check(local).unwrap();
    let declared_hover = ask(local, "textDocument/hover", "action:(Function");
    assert!(text(&declared_hover).contains("null"), "{declared_hover}");
    let narrowed = ask(local, "textDocument/hover", "action(4)");
    assert!(text(&narrowed).contains("Narrowed here"), "{narrowed}");
    let help = ask(local, "textDocument/signatureHelp", "4)");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("int"),
        "{help}"
    );
    let unguarded =
        r#"function main(){const action:(Function<int, [input:int]>)|null=null;action(4);}"#;
    rejected_at(
        unguarded,
        "unsupported",
        "action",
        "Only closure bindings are callable values",
    );
    assert!(ask(unguarded, "textDocument/signatureHelp", "4)").is_null());
    let field = r#"type Handler={run:(Function<string, [input:ReadOnly<string>]>)|null;};function main(){const handler:Handler={run:(text:string):string=>{return text.trim();}};if(handler.run!==null){console.log(handler.run(' Pat ').toUpperCase());}}"#;
    tsrust_compiler::check(field).unwrap();
    let help = ask(field, "textDocument/signatureHelp", "' Pat '");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("string"),
        "{help}"
    );
    assert!(!ask(field, "textDocument/hover", "toUpperCase()").is_null());
    let partial = "function use(action:(ReadOnly<Function<int, [text:ReadOnly<string>]>>)|undefined):int{if(action===undefined){return 0;}return action(";
    let help = ask(partial, "textDocument/signatureHelp", "action(");
    assert!(
        help.is_null(),
        "incomplete code must not unwrap a declared nullable callable without checked facts: {help}"
    );
    let ordinary =
        "function main(){const action=(value:int):int=>{return value;};console.log(action(7));}";
    assert!(!ask(ordinary, "textDocument/signatureHelp", "7)").is_null());
    let callable_result = "function main(){const action:(input:int)=>int|null=(value:int):int|null=>{return null;};console.log(String(action(1)));}";
    tsrust_compiler::check(callable_result).unwrap();
    assert!(!ask(callable_result, "textDocument/signatureHelp", "1)").is_null());

    let selected = r#"function main(){const action:(Function<int, [input:int]>)|null=null;const selected=action??(value:int):int=>{return value+1;};console.log(selected(4));}"#;
    tsrust_compiler::check(selected).unwrap();
    assert!(!ask(selected, "textDocument/signatureHelp", "4)").is_null());
    assert!(text(&ask(selected, "textDocument/hover", "selected=action")).contains("Function<int"));
    let optional = r#"function main(){const action:(Function<int, [value:int]>)|null=null;const result=action?.(4);console.log(String(result));}"#;
    tsrust_compiler::check(optional).unwrap();
    assert!(text(&ask(optional, "textDocument/hover", "result=action")).contains("undefined"));
    let once = r#"function main(){const label=clone('once');const action:(Once<Function<string>>)|null=():string=>{return label;};if(action!==null){console.log(action());}}"#;
    tsrust_compiler::check(once).unwrap();
    assert!(text(&ask(once, "textDocument/hover", "action());")).contains("Once<Function<string"));
    let whole_callable = r#"function main(){const action:(Function<int, [value:int]>)|null=null;console.log(String(action));}"#;
    rejected_at(
        whole_callable,
        "unsupported",
        "action",
        "String() function-source conversion",
    );
    let declared_nullable = r#"function main(){const action:(Function<int, [value:int]>)|null=(value:int):int=>{return value;};console.log(action(1));}"#;
    rejected_at(
        declared_nullable,
        "unsupported",
        "action",
        "Only closure bindings are callable values",
    );

    let nested = r#"function invoke(action:(ReadOnly<Function<string, [direct:ReadOnly<Function<string, [text:ReadOnly<string>]>>]>>)|null):string{if(action===null){return 'absent';}const normalize=(text:string):string=>{return text.trim();};return action(normalize);}function main(){const action:(Function<string, [direct:ReadOnly<Function<string, [text:ReadOnly<string>]>>]>)|null=(operation:ReadOnly<Function<string, [input:ReadOnly<string>]>>):string=>{return operation(' Pat ');};console.log(invoke(action));}"#;
    tsrust_compiler::check(nested).unwrap();
    let help = ask(nested, "textDocument/signatureHelp", "normalize);");
    let parameter = help["signatures"][0]["parameters"][0]["label"]
        .as_str()
        .expect("nested callback parameter");
    assert!(
        parameter.contains("Function<string") && parameter.contains("ReadOnly<string>"),
        "{help}"
    );
    let returning = r#"function main(){const factory:(Function<(Function<int, [value:int]>)>)|null=():(Function<int, [value:int]>)=>{return (value:int):int=>{return value+1;};};if(factory!==null){const action=factory();console.log(action(2));}}"#;
    tsrust_compiler::check(returning).unwrap();
    assert!(!ask(returning, "textDocument/signatureHelp", "2)").is_null());
    let concat_field = r#"type Handler={concat:(Function<string, [input:ReadOnly<string>]>)|null;};function main(){const handler:Handler={concat:(text:string):string=>{return text.trim();}};if(handler.concat!==null){console.log(handler.concat(' Pat '));}}"#;
    tsrust_compiler::check(concat_field).unwrap();
    let help = ask(concat_field, "textDocument/signatureHelp", "' Pat '");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("string"),
        "{help}"
    );
    assert!(
        !text(&ask(concat_field, "textDocument/hover", "concat(' Pat '")).contains("concat(...")
    );

    let mutable_alias = r#"function main(){let action:(Function<int, [value:int]>)|null=null;const selected=action??=(value:int):int=>{return value+2;};console.log(selected(5));}"#;
    rejected_at(
        mutable_alias,
        "borrowed-place",
        "selected",
        "scoped assignment result is readonly",
    );
    let guarded_null = r#"function main(){const action:(Function<int, [value:int]>)|null=null;if(action===null){console.log(String(action));}}"#;
    tsrust_compiler::check(guarded_null).unwrap();
    let whole_null = r#"function main(){const action:(Function<int, [value:int]>)|null=null;console.log(String(action));}"#;
    rejected_at(
        whole_null,
        "unsupported",
        "action",
        "String() function-source conversion",
    );
    let catalog: Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let string = catalog["globals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "String")
        .unwrap();
    assert!(
        string["documentation"]
            .as_str()
            .unwrap()
            .contains("whole nullable unions with a callable payload")
    );
    assert!(
        text(&ask(guarded_null, "textDocument/hover", "String(action)")).contains("strict guard")
    );

    let read_alias = r#"function step(input:int):int{return input+2;}function fallback():(input:int)=>int{return step;}function main(){let empty:(ReadOnly<Function<int, [input:int]>>)|null=null;const selected=empty??=fallback();console.log(selected(5));if(empty!==null){console.log(empty(7));}}"#;
    tsrust_compiler::check(read_alias).unwrap();
    let help = ask(read_alias, "textDocument/signatureHelp", "5)");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("int"),
        "{help}"
    );
    assert!(
        text(&ask(read_alias, "textDocument/hover", "selected=empty"))
            .contains("Function<int, [input: int]>")
    );

    let uri = "file:///nullable-callables/main.tsr";
    let dependency_uri = "file:///nullable-callables/dep.tsr";
    let entry = "import {User as Person,Handler as LocalHandler,use as invoke} from './dep.tsr';function main(){const action:(ReadOnly<Function<int, [person:ReadOnly<Person>]>>)|null=(person:Person):int=>{return person.name.length;};console.log(invoke<Person>(action,{name:'Pat'}));const handler:LocalHandler={run:(person:Person):int=>{return person.name.length;}};if(handler.run!==null){console.log(handler.run({name:'Sam'}));}}";
    let dependency = "export type User={name:string;};export type Handler={run:(Function<int, [person:ReadOnly<User>]>)|null;};export function use<T>(action:(ReadOnly<Function<int, [person:ReadOnly<T>]>>)|null,value:T):int{if(action===null){return 0;}return action(value);}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    workspace.open(source_id(dependency_uri).unwrap(), 1, dependency.into());
    let checked = workspace
        .snapshot()
        .check(&source_id(uri).unwrap())
        .unwrap();
    let index = ProjectIndex::checked(&checked);
    let help = index
        .request(
            "textDocument/signatureHelp",
            &json!({"position":position(entry,"action,{name")}),
            uri,
        )
        .unwrap()
        .unwrap();
    assert!(
        help["signatures"][0]["label"]
            .as_str()
            .unwrap()
            .contains("Person"),
        "{help}"
    );
    let field_help = index
        .request(
            "textDocument/signatureHelp",
            &json!({"position":position(entry,"{name:'Sam'}")}),
            uri,
        )
        .unwrap()
        .unwrap();
    assert!(
        field_help["signatures"][0]["label"]
            .as_str()
            .unwrap()
            .contains("Person"),
        "{field_help}"
    );

    let incomplete_entry = "import {Handler as LocalHandler} from './dep.tsr';function main(){const handler:LocalHandler={run:null};handler.ru";
    workspace.open(source_id(uri).unwrap(), 2, incomplete_entry.into());
    let snapshot = workspace.snapshot();
    assert!(snapshot.check(&source_id(uri).unwrap()).is_err());
    let graph =
        tsrust_compiler::project::load_project_headers(&snapshot, &source_id(uri).unwrap().0)
            .unwrap();
    let index = ProjectIndex::new(&graph);
    let completion=index.request("textDocument/completion",&json!({"position":Position::new(position(incomplete_entry,"handler.ru").line, position(incomplete_entry,"handler.ru").character+"handler.ru".len() as u32)}),uri).unwrap().unwrap();
    assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="run"&&item["detail"].as_str().unwrap().contains("null")),"{completion}");

    let callable_array = "function make():((input:int)=>int)[]{return [(value:int):int=>{return value+1;}];}function main(){const actions=make();console.log(actions[0](4));}";
    let analysis = tsrust_compiler::check(callable_array).unwrap();
    let facts = analysis.checked_type_facts();
    let array_fact = facts
        .method_results
        .iter()
        .find(|fact| &callable_array[fact.span.start..fact.span.end] == "make()")
        .expect("authoritative callable-array call result");
    assert!(
        matches!(
            tsrust_compiler::type_syntax::parse(&array_fact.ty)
                .unwrap()
                .kind,
            tsrust_compiler::type_syntax::ValueAnnotationKind::Array(_)
        ),
        "{array_fact:?}"
    );
    let help = ask(callable_array, "textDocument/signatureHelp", "4)");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("int"),
        "{help}"
    );
    assert!(text(&ask(callable_array, "textDocument/hover", "actions=make")).contains(">[]"));
    let array_uri = "file:///nullable-callables/array.tsr";
    workspace.open(source_id(array_uri).unwrap(), 1, callable_array.into());
    let checked_array = workspace
        .snapshot()
        .check(&source_id(array_uri).unwrap())
        .unwrap();
    let array_index = ProjectIndex::checked(&checked_array);
    let hover = array_index
        .request(
            "textDocument/hover",
            &json!({"position":position(callable_array,"actions=make")}),
            array_uri,
        )
        .unwrap()
        .unwrap();
    assert!(text(&hover).contains(")[]"), "{hover}");
    let help = array_index
        .request(
            "textDocument/signatureHelp",
            &json!({"position":position(callable_array,"4)")}),
            array_uri,
        )
        .unwrap()
        .unwrap();
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("int"),
        "{help}"
    );

    let examples = r#"type Handler={run:(Function<string, [input:ReadOnly<string>]>)|null;};function step(input:int):int{return input+2;}function fallback():(input:int)=>int{return step;}function use(action:(ReadOnly<Function<int, [text:ReadOnly<string>]>>)|undefined):int{if(action===undefined){return 0;}return action('abc');}function main(){
{const action:(Function<int, [input:int]>)|null=(value:int):int=>{return value+1;};if(action!==null){console.log(action(4));}}
{const action:(ReadOnly<Function<int, [text:ReadOnly<string>]>>)|undefined=(text:string):int=>{return text.length;};console.log(use(action));}
{const label=clone('once');const action:(Once<Function<string>>)|null=():string=>{return label;};if(action!==null){console.log(action());}}
{const handler:Handler={run:(text:string):string=>{return text.trim();}};if(handler.run!==null){console.log(handler.run(' Pat '));}}
{const action:(Function<int, [value:int]>)|undefined=(value:int):int=>{return value+1;};const missing:(Function<int, [value:int]>)|undefined=undefined;console.log(String(action?.(2)),String(missing?.(2)));}
{let action:(Function<int, [value:int]>)|null=null;action??=(value:int):int=>{return value+2;};if(action!==null){console.log(action(5));}}
{const action:(Function<int, [value:int]>)|null=null;if(action===null){console.log(String(action));}}
{let empty:(ReadOnly<Function<int, [input:int]>>)|null=null;const selected=empty??=fallback();console.log(selected(5));if(empty!==null){console.log(empty(7));}}
}"#;
    native(examples, "5\n3\nonce\nPat\n3 undefined\n7\nnull\n7\n9\n");
}
