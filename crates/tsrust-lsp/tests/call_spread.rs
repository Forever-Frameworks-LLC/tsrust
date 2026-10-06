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
            "file:///call-spread.tsr",
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
        .join("../../target/editor/call-spread-language-examples");
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

fn help_at(source: &str, needle: &str) -> Value {
    ask(source, "textDocument/signatureHelp", needle)
}
fn active(source: &str, needle: &str, expected: Option<usize>) {
    let help = help_at(source, needle);
    assert!(
        !help["signatures"][0]["label"].as_str().unwrap().is_empty(),
        "{help}"
    );
    match expected {
        Some(index) => assert_eq!(help["activeParameter"], index, "{source}: {help}"),
        None => assert!(
            help.get("activeParameter").is_none(),
            "ambiguous expansion must not select a slot: {source}: {help}"
        ),
    }
}

#[test]
fn call_spread_protocol_preserves_types_and_conservative_supplied_positions() {
    let declaration =
        "function pair(first:int,second:int,third:int=3):int{return first+second+third;}";
    for (call, needle, slot) in [
        ("pair(...[10,20],30)", "30)", Some(2)),
        ("pair(10,...[20],30)", "30)", Some(2)),
        ("pair(...[],...([10]),...[20],30)", "30)", Some(2)),
        ("pair(...[10,20,],30)", "30)", Some(2)),
        ("pair(...[10,20])", "...", None),
        ("pair(...[],10,20,30)", "...", None),
    ] {
        let source = format!("{declaration}function main(){{console.log({call});}}");
        tsrust_compiler::check(&source).unwrap();
        let needle = if needle == "..." { "[" } else { needle };
        active(&source, needle, slot);
    }
    let nested = format!(
        "{declaration}function add(a:int,b:int):int{{return a+b;}}function main(){{console.log(pair(...[add(1,2),20],30));}}"
    );
    tsrust_compiler::check(&nested).unwrap();
    active(&nested, "30)", Some(2));

    let dynamic = format!(
        "{declaration}function main(){{const values:int[]=[10,20];console.log(pair(...values,30));}}"
    );
    tsrust_compiler::check(&dynamic).unwrap();
    active(&dynamic, "30)", None);
    let hover = ask(&dynamic, "textDocument/hover", "values,30");
    assert!(
        text(&hover).contains("values: int[]"),
        "spread source stays an array, not selected element: {hover}"
    );
    let complete = ask(&dynamic, "textDocument/completion", "values,30");
    assert!(
        complete
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "values"),
        "{complete}"
    );
    let rest = "/** @param tail - Repeated integers. */\nfunction total(first:int,...tail:int[]):int{let result=first;for(const value of tail){result+=value;}return result;}function main(){const values:int[]=[20];console.log(total(10,...values,30));}";
    tsrust_compiler::check(rest).unwrap();
    active(rest, "30)", Some(1));
    let help = help_at(rest, "30)");
    assert!(
        help["signatures"][0]["parameters"][1]["label"]
            .as_str()
            .unwrap()
            .contains("...tail:"),
        "{help}"
    );
    assert!(
        help["signatures"][0]["parameters"][1]["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("fresh owning array"),
        "{help}"
    );

    for (suffix, slot) in [
        ("pair(...[10,20], ", Some(2)),
        ("pair(10,...values, ", None),
        ("pair(...values, ", None),
        ("pair(10,...[20, ", None),
    ] {
        let source = format!("{declaration}function main(){{const values:int[]=[20];{suffix}");
        let help = SourceIndex::new(&source)
            .request(
                "textDocument/signatureHelp",
                &json!({"position":Position::new(0,source.len() as u32)}),
                "file:///call-spread.tsr",
            )
            .unwrap();
        match slot {
            Some(slot) => assert_eq!(help["activeParameter"], slot, "{help}"),
            None => assert!(help.get("activeParameter").is_none(), "{help}"),
        }
    }
    // Array-literal spread is still a separate feature; do not invent its width.
    let nested_spread =
        format!("{declaration}function main(){{const values:int[]=[10,20];pair(...[...values], ");
    let help = SourceIndex::new(&nested_spread)
        .request(
            "textDocument/signatureHelp",
            &json!({"position":Position::new(0,nested_spread.len() as u32)}),
            "file:///call-spread.tsr",
        )
        .unwrap();
    assert!(help.get("activeParameter").is_none(), "{help}");
    let methods = "function main(){const values:int[]=[1,2];const items:int[]=[3,4];values.splice(0,1,...items);}";
    tsrust_compiler::check(methods).unwrap();
    let help = help_at(methods, "...items");
    assert_eq!(help["activeParameter"], 2, "{help}");
    let builtin = "function main(){console.log(Math.max(...[1.0,2.0]));}";
    tsrust_compiler::check(builtin).unwrap();
    let help = help_at(builtin, "[1.0");
    assert_eq!(help["activeParameter"], 0, "{help}");

    let generic = "function pack<T>(...values:T[]):T[]{return values;}function main(){const values:int[]=[1,2];const rows=pack<int>(...values);console.log(rows.length);}";
    tsrust_compiler::check(generic).unwrap();
    let help = help_at(generic, "...values);");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("...values: int[]"),
        "{help}"
    );
    assert!(text(&ask(generic, "textDocument/hover", "rows=pack")).contains("int[]"));
    let nullable = "function count(...values:int[]):int{return values.length;}function main(){let action:(ReadOnly<Function<int, [...values:int[]]>>)|null=count;const values:int[]=[1,2];console.log(String(action?.(...values)));}";
    tsrust_compiler::check(nullable).unwrap();
    let help = help_at(nullable, "...values)");
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("...values:"),
        "{help}"
    );
    assert_eq!(help["activeParameter"], 0, "{help}");

    let uri = "file:///call-spread/main.tsr";
    let dependency_uri = "file:///call-spread/library.tsr";
    let dependency = "export type Item={id:int;};export function pack<T>(...values:T[]):T[]{return values;}export function count(first:Item,...tail:Item[]):int{return first.id+tail.length;}";
    let current = r#"import {Item as Entry,pack,count} from "./library.tsr";function main(){const rows=pack<Entry>(...Array.of<Entry>({id:2},{id:3}));console.log(count(...rows));}"#;
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, current.to_owned());
    workspace.open(source_id(dependency_uri).unwrap(), 1, dependency.to_owned());
    let checked = workspace
        .snapshot()
        .check(&source_id(uri).unwrap())
        .unwrap();
    let index = ProjectIndex::checked(&checked);
    let help = index
        .request(
            "textDocument/signatureHelp",
            &json!({"position":position(current,"...Array")}),
            uri,
        )
        .unwrap()
        .unwrap();
    assert!(
        help["signatures"][0]["parameters"][0]["label"]
            .as_str()
            .unwrap()
            .contains("Entry[]"),
        "{help}"
    );
    let help = index
        .request(
            "textDocument/signatureHelp",
            &json!({"position":position(current,"...rows")}),
            uri,
        )
        .unwrap()
        .unwrap();
    assert!(
        help["signatures"][0]["parameters"][1]["label"]
            .as_str()
            .unwrap()
            .contains("...tail:Entry[]"),
        "{help}"
    );
    assert!(help.get("activeParameter").is_none(), "{help}");
}

#[test]
fn call_spread_native_examples_follow_checked_segment_order_and_ownership() {
    let aggregate = r#"function total(first:int=5,...tail:int[]):int{let result=first;for(const value of tail){result+=value;}return result;}
function words(...values:string[]):string[]{return values;}
function count(...values:int[]):int{return values.length;}
function fill(trace:int[]):int[]{trace.push(1);return [7];}
function main(){console.log(total(0,...[1,2],3),total(...[]));const source=["é".repeat(1),"x".repeat(1)];const result=words(...clone(source));console.log(result.join("|"),source.join("|"));let optional:(ReadOnly<Function<int, [...values:int[]]>>)|null=null;const trace:int[]=[];console.log(String(optional?.(...fill(trace))),trace.length);console.log(Math.max(...[1.0,3.0]));console.log("é".concat(...["x".repeat(1),"y".repeat(1)]));console.log(Array.of<int>(...[1,2]).join(","));}"#;
    native(aggregate, "6 5\né|x é|x\nundefined 0\n3\néxy\n1,2\n");
}
