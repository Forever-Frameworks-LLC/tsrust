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
            "file:///rest-parameters.tsr",
        )
        .unwrap()
}
fn text(value: &Value) -> &str {
    value["contents"]["value"].as_str().expect("hover")
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
        .join("../../target/editor/rest-parameters-language-examples");
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
fn rest_parameter_protocol_facts_preserve_array_bodies_and_repeated_tail_help() {
    let source = r#"/** @param values - Fresh owning tail in supplied order. */
function total(first:int=7,...values:int[]):int{let sum=first;for(const value of values){sum+=value;}return sum;}
function main(){console.log(total(),total(undefined,1,2,3));}"#;
    tsrust_compiler::check(source).unwrap();
    let help = ask(source, "textDocument/signatureHelp", "1,2,3)");
    assert_eq!(
        help["signatures"][0]["parameters"]
            .as_array()
            .unwrap()
            .len(),
        2,
        "{help}"
    );
    assert!(
        help["signatures"][0]["parameters"][1]["label"]
            .as_str()
            .unwrap()
            .contains("...values:"),
        "{help}"
    );
    assert!(
        !help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("first:int|undefined"),
        "rest is not a later required slot: {help}"
    );
    assert!(
        help["signatures"][0]["parameters"][1]["documentation"]["value"]
            .as_str()
            .unwrap()
            .starts_with("Fresh owning tail in supplied order.\n\n"),
        "{help}"
    );
    let body = ask(source, "textDocument/hover", "values){");
    assert!(text(&body).contains("values: int[]"), "{body}");
    let completion = ask(source, "textDocument/completion", "values){");
    assert!(
        completion
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "values"),
        "{completion}"
    );
    for suffix in ["total(", "total(undefined, ", "total(undefined,1,2,3, "] {
        let prefix = source.split("function main()").next().unwrap();
        let incomplete = format!("{prefix}function main(){{{suffix}");
        let help = SourceIndex::new(&incomplete)
            .request(
                "textDocument/signatureHelp",
                &json!({"position":position_end(&incomplete)}),
                "file:///rest-parameters.tsr",
            )
            .unwrap();
        assert_eq!(
            help["activeParameter"],
            if suffix == "total(" { 0 } else { 1 },
            "{help}"
        );
        assert!(
            help["signatures"][0]["parameters"][1]["label"]
                .as_str()
                .unwrap()
                .contains("...values:"),
            "{help}"
        );
    }

    for (array, element) in [
        ("int[]", "int[]"),
        ("number[]", "number[]"),
        ("boolean[]", "boolean[]"),
        ("null[]", "null[]"),
        ("undefined[]", "undefined[]"),
        ("string[]", "string[]"),
        ("Item[]", "Item[]"),
        ("int[][]", "int[][]"),
        ("Box<int>[]", "Box<int>[]"),
        ("((input:int)=>int)[]", "Function<int, [input: int]>[]"),
    ] {
        let source = format!(
            "type Item={{id:int;}};type Box<T>={{value:T;}};function size(...values:{array}):int{{return values.length;}}function main(){{console.log(size());}}"
        );
        tsrust_compiler::check(&source).unwrap();
        let body = ask(&source, "textDocument/hover", "values.length");
        assert!(text(&body).contains(element), "{array}: {body}");
        let help = ask(&source, "textDocument/signatureHelp", "));}");
        let label = help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap();
        assert!(
            // Declaration help keeps the source spelling; hover shows the canonical one.
            label.contains("...values:") && (label.contains(element) || label.contains(array)),
            "{array}: {help}"
        );
    }

    let field = r#"type Handler={concat:(Function<int, [...values:int[]]>);};function count(...values:int[]):int{return values.length;}function main(){const holder:Handler={concat:count};console.log(holder.concat(1,2,3));}"#;
    tsrust_compiler::check(field).unwrap();
    let help = ask(field, "textDocument/signatureHelp", "1,2,3)");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("...values:"),
        "{help}"
    );

    let contextual = r#"function main(){const action:(ReadOnly<Function<int, [...values:int[]]>>)=(...values):int=>{return values.length;};console.log(action(1,2,3));}"#;
    tsrust_compiler::check(contextual).unwrap();
    assert!(
        text(&ask(contextual, "textDocument/hover", "values.length")).contains("values: int[]")
    );
    let help = ask(contextual, "textDocument/signatureHelp", "1,2,3)");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("...values:"),
        "{help}"
    );
    assert_eq!(help["activeParameter"], 0, "{help}");

    let generic = r#"function pack<T>(...values:T[]):T[]{return values;}function main(){const numbers=pack<int>(1,2);const words=pack<string>(clone("x"));console.log(numbers.length,words.length);}"#;
    tsrust_compiler::check(generic).unwrap();
    for (needle, ty) in [("1,2)", "int[]"), ("clone(\"x\")", "string[]")] {
        let help = ask(generic, "textDocument/signatureHelp", needle);
        assert!(
            help["signatures"][0]["parameters"][0]["label"]
                .as_str()
                .unwrap()
                .contains("...values:"),
            "{help}"
        );
        assert!(
            help["signatures"][0]["parameters"][0]["label"]
                .as_str()
                .unwrap()
                .contains(ty),
            "{help}"
        );
    }

    let nullable = r#"function count(...values:int[]):int{return values.length;}function main(){let action:(ReadOnly<Function<int, [...values:int[]]>>)|null=count;console.log(String(action?.(1,2)));if(action!==null){console.log(action(3,4,5));}}"#;
    tsrust_compiler::check(nullable).unwrap();
    for needle in ["1,2)", "3,4,5)"] {
        let help = ask(nullable, "textDocument/signatureHelp", needle);
        assert!(
            help["signatures"][0]["parameters"][0]["label"]
                .as_str()
                .unwrap()
                .contains("...values:"),
            "{help}"
        );
    }

    let uri = "file:///rest-parameters/main.tsr";
    let dependency_uri = "file:///rest-parameters/library.tsr";
    let dependency = r#"export type Item={id:int;};export function pack<T>(...values:T[]):T[]{return values;}export function count(...values:Item[]):int{return values.length;}"#;
    let current = r#"import {Item as Entry,pack,count} from "./library.tsr";function main(){const rows=pack<Entry>({id:1},{id:2});console.log(count({id:3},{id:4}));console.log(rows.length);}"#;
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, current.to_owned());
    workspace.open(source_id(dependency_uri).unwrap(), 1, dependency.to_owned());
    let checked = workspace
        .snapshot()
        .check(&source_id(uri).unwrap())
        .unwrap();
    let index = ProjectIndex::checked(&checked);
    for needle in ["{id:1},{id:2}", "{id:3},{id:4}"] {
        let help = index
            .request(
                "textDocument/signatureHelp",
                &json!({"position":position(current,needle)}),
                uri,
            )
            .unwrap()
            .unwrap();
        let label = help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap();
        assert!(
            label.contains("...values:") && label.contains("Entry[]"),
            "{help}"
        );
    }

    // Invalid/incomplete headers retain TDZ blockers; declaration and body hover remain useful.
    let blocked = "function main(){const tail=9;const action=(first:int=tail,...tail:int[]):int=>{return tail.length;};}";
    assert!(ask(blocked, "textDocument/hover", "tail,...").is_null());
    assert!(text(&ask(blocked, "textDocument/hover", "tail:int[]")).contains("tail: int[]"));
    assert!(text(&ask(blocked, "textDocument/hover", "tail.length")).contains("tail: int[]"));
}
fn position_end(source: &str) -> Position {
    let prefix = source;
    Position::new(
        prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
}

#[test]
fn rest_parameter_native_examples_follow_declared_owning_pack_contracts() {
    let aggregate = r#"function sum(first:int=5,...values:int[]):int{let result=first;for(const value of values){result+=value;}return result;}
function words(...values:string[]):string[]{return values;}
function pack<T>(...values:T[]):T[]{return values;}
function count(...values:int[]):int{return values.length;}
function main(){console.log(sum(),sum(undefined,1,2),sum(0,3));console.log(words().length);const text=words(clone("é"),clone("x"));console.log(text.join("|"));const rows=pack<int[]>([1],[2,3]);console.log(rows.length,rows[1].length);const action:(ReadOnly<Function<int, [...values:int[]]>>)=count;console.log(action(1,2,3));let optional:(ReadOnly<Function<int, [...values:int[]]>>)|null=count;console.log(String(optional?.(4,5)));}"#;
    native(aggregate, "5 8 3\n0\né|x\n2 2\n3\n2\n");
}

#[test]
fn rest_parameter_help_explains_owning_pack_after_authored_docs() {
    let source = "/** @param first - Fixed input.\n * @param values - Additional integers. */\nfunction describe(first:int,...values:int[]):int{return first+values.length;}function main(){console.log(describe(1,2));}";
    tsrust_compiler::check(source).unwrap();
    let help = ask(source, "textDocument/signatureHelp", "2));");
    assert_eq!(
        help["signatures"][0]["parameters"][0]["documentation"]["value"], "Fixed input.",
        "{help}"
    );
    let rest = help["signatures"][0]["parameters"][1]["documentation"]["value"]
        .as_str()
        .unwrap();
    assert!(rest.starts_with("Additional integers.\n\n"), "{help}");
    for expected in [
        "fresh owning array",
        "Copy elements copy",
        "owned elements transfer",
        "Fixed parameters retain",
        "padding adds no tail elements",
    ] {
        assert!(rest.contains(expected), "{help}");
    }
}
