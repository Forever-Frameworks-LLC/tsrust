use serde_json::json;
use tsrust_lsp::{
    project_diagnostics::source_id, project_intelligence::ProjectIndex, workspace::Workspace,
};

const ROOT: &str = "file:///tsrust-incomplete/main.tsr";
const DEP: &str = "file:///tsrust-incomplete/dep.tsr";
const LIBRARY: &str = "/** Current helper. */ export function greet(value:int):int{return value;}export type User={name:string;};";

fn hover(entry: &str, library: &str, needle: &str) -> Result<String, String> {
    let mut workspace = Workspace::default();
    workspace.open(source_id(ROOT).unwrap(), 1, entry.into());
    workspace.open(source_id(DEP).unwrap(), 1, library.into());
    let input = workspace.snapshot();
    let graph = tsrust_compiler::project::load_project_headers(&input, &source_id(ROOT).unwrap().0)
        .map_err(|error| format!("{}: {}", error.code, error.message))?;
    let index = ProjectIndex::new(&graph);
    let offset = entry.rfind(needle).unwrap();
    let line_start = entry[..offset].rfind('\n').map_or(0, |index| index + 1);
    let position = json!({"line":entry[..offset].matches('\n').count(),"character":entry[line_start..offset].encode_utf16().count()});
    let value = index
        .request("textDocument/hover", &json!({"position":position}), ROOT)
        .ok_or("no project answer")??;
    Ok(value["contents"]["value"]
        .as_str()
        .unwrap_or_default()
        .to_owned())
}

#[test]
fn imported_help_survives_unfinished_text_bodies_and_imports() {
    let import = "import { greet as hello, User } from './dep.tsr';";
    for (name, entry, library) in [
        (
            "unterminated string",
            format!("{import}function main(){{hello(2);const s=\"abc"),
            LIBRARY.to_owned(),
        ),
        (
            "unterminated template",
            format!("{import}function main(){{hello(2);const s=`abc ${{"),
            LIBRARY.to_owned(),
        ),
        (
            "unterminated comment",
            format!("{import}function main(){{hello(2);/* note"),
            LIBRARY.to_owned(),
        ),
        (
            "incomplete parameter",
            format!("{import}function main(){{hello(2);}}function next(a:"),
            LIBRARY.to_owned(),
        ),
        (
            "incomplete generic",
            format!("{import}function main(){{hello(2);}}function next<T"),
            LIBRARY.to_owned(),
        ),
        (
            "incomplete type",
            format!("{import}function main(){{hello(2);}}type Row={{a:"),
            LIBRARY.to_owned(),
        ),
        (
            "incomplete callback parameter",
            format!("{import}function main(){{hello(2);}}function next(action:(value:int)=>"),
            LIBRARY.to_owned(),
        ),
        (
            "dependency body",
            format!("{import}function main(){{hello(2);}}"),
            LIBRARY.replace("return value;}", "return value."),
        ),
        (
            "dependency string",
            format!("{import}function main(){{hello(2);}}"),
            format!("{LIBRARY}function later(){{const s=\"abc"),
        ),
        (
            "dependency later signature",
            format!("{import}function main(){{hello(2);}}"),
            format!("{LIBRARY}function later(a:"),
        ),
        (
            "stray closer",
            format!("{import}function main(){{hello(2);}}}}"),
            LIBRARY.to_owned(),
        ),
        (
            "incomplete import",
            format!("{import}import {{ gre\nfunction main(){{hello(2);}}"),
            LIBRARY.to_owned(),
        ),
        (
            "incomplete import path",
            format!("{import}import {{ greet }} from './de\nfunction main(){{hello(2);}}"),
            LIBRARY.to_owned(),
        ),
        (
            "import without from",
            format!("{import}import {{ greet as other }}\nfunction main(){{hello(2);}}"),
            LIBRARY.to_owned(),
        ),
        (
            "entry body before export",
            format!("{import}function first(){{hello(1)\nexport function main(){{hello(2);}}"),
            LIBRARY.to_owned(),
        ),
    ] {
        let text =
            hover(&entry, &library, "hello(2)").unwrap_or_else(|error| panic!("{name}: {error}"));
        assert!(text.contains("Current helper."), "{name}: {text}");
        // Assistance never replaces checking: the strict loader still rejects the project.
        let mut workspace = Workspace::default();
        workspace.open(source_id(ROOT).unwrap(), 1, entry);
        workspace.open(source_id(DEP).unwrap(), 1, library);
        assert!(
            workspace
                .snapshot()
                .check(&source_id(ROOT).unwrap())
                .is_err(),
            "{name}"
        );
    }
}

#[test]
fn header_assistance_still_requires_resolvable_import_paths() {
    for (entry, code) in [
        (
            "import { greet as hello } from './missing.tsr';function main(){hello(2);const s=\"abc",
            "module-load",
        ),
        (
            "import { greet as hello } from 'dep';function main(){hello(2);const s=\"abc",
            "module-load",
        ),
    ] {
        let mut workspace = Workspace::default();
        workspace.open(source_id(ROOT).unwrap(), 1, entry.into());
        workspace.open(source_id(DEP).unwrap(), 1, LIBRARY.into());
        let error = tsrust_compiler::project::load_project_headers(
            &workspace.snapshot(),
            &source_id(ROOT).unwrap().0,
        )
        .err()
        .unwrap();
        assert_eq!(error.code, code, "{entry}");
        if entry.contains("from 'dep'") {
            assert!(error.message.contains("No path alias matches dep"), "{error:?}");
        }
    }
}

fn import_names(entry: &str, cursor: &str) -> Vec<String> {
    let mut workspace = Workspace::default();
    workspace.open(source_id(ROOT).unwrap(), 1, entry.into());
    workspace.open(source_id(DEP).unwrap(), 1, LIBRARY.into());
    let graph = tsrust_compiler::project::load_project_headers(
        &workspace.snapshot(),
        &source_id(ROOT).unwrap().0,
    )
    .unwrap();
    let offset = entry.rfind(cursor).unwrap() + cursor.len();
    let line_start = entry[..offset].rfind('\n').map_or(0, |index| index + 1);
    let position = json!({"line":entry[..offset].matches('\n').count(),"character":entry[line_start..offset].encode_utf16().count()});
    let value = ProjectIndex::new(&graph)
        .request(
            "textDocument/completion",
            &json!({"position":position}),
            ROOT,
        )
        .unwrap()
        .unwrap();
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            assert_eq!(item["textEdit"]["newText"], item["label"]);
            item["label"].as_str().unwrap().to_owned()
        })
        .collect()
}

#[test]
fn import_braces_complete_the_exports_of_the_named_module() {
    assert_eq!(
        import_names("import { } from './dep.tsr';function main(){}", "import {"),
        ["User", "greet"]
    );
    assert_eq!(
        import_names("import { gre } from './dep.tsr';function main(){}", "gre"),
        ["greet"]
    );
    // A name that is already imported is not offered again.
    assert_eq!(
        import_names(
            "import { greet, } from './dep.tsr';function main(){greet(1);}",
            "greet,"
        ),
        ["User"]
    );
    assert_eq!(
        import_names(
            "import { greet as hello, Us } from './dep.tsr';function main(){hello(1);}",
            "Us"
        ),
        ["User"]
    );
    // A local alias is a new name.
    assert!(
        import_names(
            "import { greet as he } from './dep.tsr';function main(){}",
            "as he"
        )
        .is_empty()
    );
    // Native modules offer their documented members.
    let native = import_names("import { str } from 'std:testing';function main(){}", "str");
    assert!(native.contains(&"strictEqual".to_owned()), "{native:?}");
    // Outside the braces the ordinary completion applies.
    assert!(
        !import_names(
            "import { greet } from './dep.tsr';function main(){gre}",
            "){gre"
        )
        .contains(&"User".to_owned())
    );
}

#[test]
fn a_name_that_is_not_exported_has_no_imported_help_and_keeps_the_other_names() {
    let entry = "import { absent as hello, User } from './dep.tsr';function main(){hello(2);const user:User={name:\"Ada\"};user.name;";
    assert!(
        !hover(entry, LIBRARY, "hello(2)")
            .unwrap_or_default()
            .contains("Current helper.")
    );
    assert!(hover(entry, LIBRARY, "name;").unwrap().contains("string"));
}

#[test]
fn unfinished_text_keeps_a_module_buffer_in_project_mode() {
    let session = tsrust_lsp::project_session::ProjectSession::default();
    for (text, project) in [
        (
            "import { greet } from './dep.tsr';function main(){const s=\"abc",
            true,
        ),
        (
            "import { greet } from './dep.tsr';function main(){/* note",
            true,
        ),
        (
            "function helper(){const s=`abc ${\nexport function main(){}",
            true,
        ),
        (
            "function main(){const s=\"import { x } from './dep.tsr';",
            false,
        ),
        ("function main(){const s=\"abc", false),
    ] {
        let snapshot = tsrust_lsp::Snapshot {
            uri: ROOT.into(),
            version: 1,
            generation: 1,
            text: text.into(),
        };
        assert_eq!(session.prepare(&snapshot).is_some(), project, "{text}");
    }
}

#[test]
fn guide_example_gives_signature_help_in_an_unfinished_call() {
    let entry = "import { greet } from './dep.tsr';\n\nfunction main() {\n    greet(";
    let mut workspace = Workspace::default();
    workspace.open(source_id(ROOT).unwrap(), 1, entry.into());
    workspace.open(source_id(DEP).unwrap(), 1, LIBRARY.into());
    let graph = tsrust_compiler::project::load_project_headers(
        &workspace.snapshot(),
        &source_id(ROOT).unwrap().0,
    )
    .unwrap();
    let help = ProjectIndex::new(&graph)
        .request(
            "textDocument/signatureHelp",
            &json!({"position":{"line":3,"character":10}}),
            ROOT,
        )
        .unwrap()
        .unwrap();
    let label = help["signatures"][0]["label"].as_str().unwrap_or_default();
    assert!(label.contains("greet") && label.contains("value"), "{help}");
}
