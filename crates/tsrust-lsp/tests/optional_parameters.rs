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
            "file:///optional-parameters.tsr",
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
fn native_command(
    directory: &std::path::Path,
    label: &str,
    argv: Vec<String>,
) -> std::process::Output {
    let now = || {
        u64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        )
        .unwrap()
    };
    let started = now();
    let child = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(directory)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id();
    let mut record = json!({
        "argv": argv,
        "cwd": directory,
        "pid": pid,
        "startedUnixMilliseconds": started
    });
    std::fs::write(
        directory.join(format!("{label}-started.json")),
        serde_json::to_vec_pretty(&record).unwrap(),
    )
    .unwrap();
    let output = child.wait_with_output().unwrap();
    record["finishedUnixMilliseconds"] = json!(now());
    record["exitCode"] = json!(output.status.code());
    record["success"] = json!(output.status.success());
    std::fs::write(directory.join(format!("{label}.stdout")), &output.stdout).unwrap();
    std::fs::write(directory.join(format!("{label}.stderr")), &output.stderr).unwrap();
    std::fs::write(
        directory.join(format!("{label}-terminal.json")),
        serde_json::to_vec_pretty(&record).unwrap(),
    )
    .unwrap();
    output
}
fn native(source: &str, expected: &str) {
    let analysis = tsrust_compiler::check(source).unwrap();
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/editor/optional-parameters-language-examples");
    std::fs::create_dir_all(&directory).unwrap();
    let directory = directory.canonicalize().unwrap();
    std::fs::write(directory.join("examples.tsr"), source).unwrap();
    std::fs::write(directory.join("examples.rs"), analysis.emit_rust()).unwrap();
    std::fs::write(directory.join("expected.stdout"), expected).unwrap();
    for level in [0, 3] {
        let binary = directory.join(format!("examples-o{level}"));
        let build = native_command(
            &directory,
            &format!("build-o{level}"),
            vec![
                "rustup".into(),
                "run".into(),
                "1.96.1".into(),
                "rustc".into(),
                "--edition=2024".into(),
                "-C".into(),
                format!("opt-level={level}"),
                directory.join("examples.rs").to_str().unwrap().into(),
                "-o".into(),
                binary.to_str().unwrap().into(),
            ],
        );
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let output = native_command(
            &directory,
            &format!("examples-o{level}"),
            vec![binary.to_str().unwrap().into()],
        );
        assert!(output.status.success());
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn optional_and_default_parameter_protocol_contracts_follow_checked_signatures() {
    let optional = r#"function show(value?:int):string{return String(value??7);} function main(){console.log(show(),show(undefined),show(0));}"#;
    tsrust_compiler::check(optional).unwrap();
    let help = ask(optional, "textDocument/signatureHelp", "undefined)");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("value?:"),
        "{help}"
    );
    let body = ask(optional, "textDocument/hover", "value??");
    assert!(
        text(&body).contains("int") && text(&body).contains("undefined"),
        "{body}"
    );
    let completion = ask(optional, "textDocument/completion", "value??");
    assert!(
        completion
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "value"),
        "{completion}"
    );

    let defaulted = r#"function show(value:int=7):int{return value;} function main(){console.log(show(),show(undefined),show(0));}"#;
    tsrust_compiler::check(defaulted).unwrap();
    let help = ask(defaulted, "textDocument/signatureHelp", "undefined)");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("=7"),
        "{help}"
    );
    let body = ask(defaulted, "textDocument/hover", "value;}");
    assert!(
        text(&body).contains("int") && !text(&body).contains("undefined"),
        "{body}"
    );

    for (source, needle, active) in [
        (
            "function choose(first:int=1,second?:string):int{return first;} function main(){choose(",
            "choose(",
            0,
        ),
        (
            "function choose(first:int=1,second?:string):int{return first;} function main(){choose(undefined, ",
            "undefined, ",
            1,
        ),
    ] {
        let offset = source.len();
        let help = SourceIndex::new(source)
            .request(
                "textDocument/signatureHelp",
                &json!({"position": Position::new(0, offset as u32)}),
                "file:///optional-parameters.tsr",
            )
            .unwrap();
        assert_eq!(help["activeParameter"], active, "{needle}: {help}");
        assert!(
            help["signatures"][0]["parameters"][1]["label"]
                .as_str()
                .unwrap()
                .contains("second?:"),
            "{help}"
        );
    }

    for (source, expected) in [
        (
            "/** @param value - Omit or supply an integer. */\nfunction explain(value?:int):int{return value??7;}function main(){console.log(explain(undefined));}",
            "Omit or supply an integer.",
        ),
        (
            "/** @param value - Defaults to seven when omitted. */\nfunction explain(value:int=7):int{return value;}function main(){console.log(explain(undefined));}",
            "Defaults to seven when omitted.",
        ),
    ] {
        tsrust_compiler::check(source).unwrap();
        let help = ask(source, "textDocument/signatureHelp", "undefined)");
        assert_eq!(
            help["signatures"][0]["parameters"][0]["documentation"]["value"], expected,
            "{help}"
        );
    }
    let inferred_docs = "function main(){/** @param value - Supplies a typed default. */\nconst explain=(value=7):int=>{return value;};console.log(explain());}";
    tsrust_compiler::check(inferred_docs).unwrap();
    let help = ask(inferred_docs, "textDocument/signatureHelp", "));}");
    assert_eq!(
        help["signatures"][0]["parameters"][0]["documentation"]["value"],
        "Supplies a typed default.",
        "{help}"
    );
    for unfinished in [
        "function explain(value:int=call(",
        "function main(){const explain=(value=call(",
    ] {
        let _ = SourceIndex::new(unfinished);
    }

    let initialized = r#"function use(value:int|undefined=7):int{return value;} function main(){console.log(use());}"#;
    tsrust_compiler::check(initialized).unwrap();
    let body = ask(initialized, "textDocument/hover", "value;}");
    assert!(
        text(&body).contains("Narrowed here") && text(&body).contains("int"),
        "{body}"
    );
    let declaration = ask(initialized, "textDocument/hover", "value:int|undefined");
    assert!(text(&declaration).contains("undefined"), "{declaration}");
    let inferred =
        r#"function main(){const action=(value=7):int=>{return value;};console.log(action());}"#;
    tsrust_compiler::check(inferred).unwrap();
    let body = ask(inferred, "textDocument/hover", "value;}");
    assert!(
        text(&body).contains("int") && !text(&body).contains("undefined"),
        "{body}"
    );
    let ordered_scope = r#"function pick(first:int=1,second:int=first):int{return second;} function main(){console.log(pick());}"#;
    tsrust_compiler::check(ordered_scope).unwrap();
    let hover = ask(ordered_scope, "textDocument/hover", "first):");
    assert!(text(&hover).contains("int"), "{hover}");

    let required_slot = r#"function choose(first:int=5,required:int):int{return first+required;}function main(){const action=choose;console.log(action(undefined,2));}"#;
    let analysis = tsrust_compiler::check(required_slot).unwrap();
    let hover = ask(required_slot, "textDocument/hover", "action=choose");
    assert!(!text(&hover).contains("first?:"), "{hover}");
    assert!(
        text(&hover).contains("undefined") && text(&hover).contains("required"),
        "{hover}"
    );
    let contracts = analysis
        .checked_type_facts()
        .method_results
        .into_iter()
        .filter(|fact| fact.ty.contains("required:"))
        .collect::<Vec<_>>();
    assert!(
        !contracts.is_empty(),
        "checked function-value contract must be retained"
    );
    for fact in contracts {
        assert!(!fact.ty.contains("first?:"), "{}", fact.ty);
        tsrust_compiler::type_syntax::parse(&fact.ty).unwrap();
    }

    let nested = r#"function use(operation:ReadOnly<Function<int, [value?:int]>>):int{return operation();} function step(value:int=4):int{return value+1;} function main(){console.log(use(step));}"#;
    tsrust_compiler::check(nested).unwrap();
    let help = ask(nested, "textDocument/signatureHelp", "step));");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("value?:"),
        "{help}"
    );
    let body_help = ask(nested, "textDocument/signatureHelp", ");} function step");
    assert!(
        body_help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("value?:"),
        "{body_help}"
    );

    let generic = r#"function maybe<T>(value?:T):T|undefined{return value;} function main(){console.log(String(maybe<int>(undefined)));}"#;
    tsrust_compiler::check(generic).unwrap();
    let help = ask(generic, "textDocument/signatureHelp", "undefined))");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("value?: int"),
        "{help}"
    );

    let conflict = r#"function choose<T>(value:T,fallback:T=value):T{return fallback;}function main(){console.log(choose<int>(1));console.log(choose<string>("a"));}"#;
    tsrust_compiler::check(conflict).unwrap();
    let hover = ask(conflict, "textDocument/hover", "fallback;}");
    assert!(
        text(&hover).contains("T") && !text(&hover).contains("Narrowed here"),
        "{hover}"
    );

    let array = r#"function step(value:int=4):int{return value+1;} function main(){const actions:((value?:int)=>int)[]=[step];console.log(actions[0]());}"#;
    tsrust_compiler::check(array).unwrap();
    let help = ask(array, "textDocument/signatureHelp", "));}");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("value?:"),
        "{help}"
    );

    let uri = "file:///optional-parameters/entry.tsr";
    let dependency_uri = "file:///optional-parameters/library.tsr";
    let dependency =
        r#"export type Item={id:int;};export function show(value?:Item):int{return value?.id??9;}"#;
    let source = r#"import {show,Item} from "./library.tsr";function main(){console.log(show());console.log(show({id:3}));}"#;
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, source.to_owned());
    workspace.open(source_id(dependency_uri).unwrap(), 1, dependency.to_owned());
    let checked = workspace
        .snapshot()
        .check(&source_id(uri).unwrap())
        .unwrap();
    let index = ProjectIndex::checked(&checked);
    let help = index
        .request(
            "textDocument/signatureHelp",
            &json!({"position":position(source, "));console.log(show")}),
            uri,
        )
        .unwrap()
        .unwrap();
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("value?:"),
        "{help}"
    );

    let aliased_dependency = r#"export type User={id:int;};export function choose(first:User={id:5},required:int):int{return first.id+required;}export function create():(first:ReadOnly<User|undefined>,required:int)=>int{return choose;}"#;
    let aliased_source = r#"import {User as Person,create as make} from "./library.tsr";function main(){const action=make();console.log(action(undefined,2));}"#;
    workspace.open(source_id(uri).unwrap(), 2, aliased_source.to_owned());
    workspace.open(
        source_id(dependency_uri).unwrap(),
        2,
        aliased_dependency.to_owned(),
    );
    let checked = workspace
        .snapshot()
        .check(&source_id(uri).unwrap())
        .unwrap();
    let index = ProjectIndex::checked(&checked);
    let help = index
        .request(
            "textDocument/signatureHelp",
            &json!({"position":position(aliased_source, "undefined,2")}),
            uri,
        )
        .unwrap()
        .unwrap();
    let first = help["signatures"][0]["parameters"][0]["label"]
        .as_str()
        .unwrap();
    assert!(
        first.contains("Person") && first.contains("undefined") && !first.contains("first?:"),
        "{help}"
    );

}

#[test]
fn optional_and_default_parameter_native_examples_follow_checked_signatures() {
    let aggregate = r#"function show(value?:int):string{return String(value??7);}
function pick(value:int|null=5):int|null{return value;}
function choose(first:int=5,required:int):int{return first+required;}
function mark(trace:int[],value:int):int{trace.push(value);return value;}
function add(trace:int[],left:int=mark(trace,1),right:int=mark(trace,left+1)):int{return left+right;}
function word(value:string=" Pat "):string{return value.trim();}
type Item={label:string;};function rows(value:int[]=[1,2]):int{return value.length;}
function item(value:Item={label:"Pat"}):string{return value.label.trim();}
function step(value:int):int{return value+2;}
function apply(action:(ReadOnly<Function<int, [value:int]>>)=step,value:int=4):int{return action(value);}
function main(){console.log(show(),show(undefined),show(0));console.log(String(pick()),String(pick(undefined)),String(pick(null)));const trace:int[]=[];console.log(add(trace));console.log(add(trace,10));console.log(trace.join(","));console.log(word());console.log(word(" Lee "));console.log(rows());console.log(item());console.log(apply());console.log(apply(undefined,8));console.log(choose(undefined,2));}"#;
    native(
        aggregate,
        "7 7 0\n5 5 null\n3\n21\n1,2,11\nPat\nLee\n2\nPat\n6\n10\n7\n",
    );
}


#[test]
fn parameter_defaults_expose_only_earlier_initialized_parameters() {
    let own = "function main(){const first=99;const later=99;const build=(first:int=first,later:int=7):int=>{return first+later;};}";
    let hover = ask(own,"textDocument/hover","first,later");
    assert!(hover.is_null(), "current parameter is not initialized: {hover}");
    let completion = ask(own,"textDocument/completion","first,later");
    assert!(!completion.as_array().unwrap().iter().any(|item| item["label"]=="first"), "{completion}");
    let declaration = ask(own,"textDocument/hover","first:int");
    assert!(text(&declaration).contains("first: int"),"{declaration}");

    let later = "function main(){const later=99;const build=(first:int=later,later:int=7):int=>{return first+later;};}";
    let hover = ask(later,"textDocument/hover","later,later");
    assert!(hover.is_null(), "later parameter shadows outer name before initialization: {hover}");
    let completion = ask(later,"textDocument/completion","later,later");
    assert!(!completion.as_array().unwrap().iter().any(|item| item["label"]=="later"),"{completion}");

    let valid = "function main(){const build=(first:int=1,later:int=first):int=>{return first+later;};console.log(build());}";
    tsrust_compiler::check(valid).unwrap();
    let earlier = ask(valid,"textDocument/hover","first):");
    assert!(text(&earlier).contains("first: int"),"{earlier}");
    let body = ask(valid,"textDocument/hover","later;}");
    assert!(text(&body).contains("later: int"),"{body}");
    let completion = ask(valid,"textDocument/completion","later;}");
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"]=="first"),"{completion}");
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"]=="later"),"{completion}");
}
