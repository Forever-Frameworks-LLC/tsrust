use serde_json::{Value, json};
use tsrust_lsp::{
    project_diagnostics::source_id, project_intelligence::ProjectIndex, workspace::Workspace,
};
fn ask(index: &ProjectIndex, source: &str, needle: &str, method: &str) -> Value {
    let offset = source.rfind(needle).unwrap();
    let prefix = &source[..offset];
    let position = json!({"line":prefix.bytes().filter(|byte| *byte == b'\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()});
    index
        .request(
            method,
            &json!({"position":position}),
            "file:///tsrust-intelligence/main.tsr",
        )
        .unwrap()
        .unwrap()
}

#[test]
fn file_api_hover_and_signature_help_preserve_consuming_close() {
    let entry = tsrust_compiler::format("import {open,read,close} from 'std:fs';function main(){match(open('input.bin')){Ok(file)=>{read(file,4);close(file);},Err(error)=>{console.log(error.osCode);}}}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let hover = ask(&index, &entry, "close(file)", "textDocument/hover");
    assert!(hover.to_string().contains("consumed even when close fails"), "{hover}");
    assert!(!hover.to_string().contains("__tsrust_private_"), "{hover}");
    let help = ask(&index, &entry, "file, 4", "textDocument/signatureHelp");
    assert!(help.to_string().contains("Mutable<File | BufferedReader>") && help.to_string().contains("67108864"), "{help}");
    let hover = ask(&index, &entry, "error.osCode", "textDocument/hover");
    assert!(!hover.is_null());
    let importing = "import { F } from 'std:fs';function main(){}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, importing.into());
    let graph = tsrust_compiler::project::load_project_headers(&workspace.snapshot(), &source_id(uri).unwrap().0).unwrap();
    let completion = ask(&ProjectIndex::new(&graph), importing, " }", "textDocument/completion");
    let labels = completion.as_array().unwrap().iter().filter_map(|item| item["label"].as_str()).collect::<Vec<_>>();
    assert_eq!(labels, ["File", "FsError"], "{completion}");
}

#[test]
fn imported_result_matches_keep_nominal_payload_help_and_pattern_navigation() {
    let entry = tsrust_compiler::format("import { User as Person, make } from './dep.tsr';function main(){const result=make();match(result){Ok(user)=>{console.log(user.name.toUpperCase());},Err(error)=>{console.log(error.length);}}}").unwrap();
    let dependency = tsrust_compiler::format("export type User={name:string;};export function make():Result<User,string>{return Ok({name:'Pat'});}").unwrap();
    for source in [&entry, &dependency] { assert_eq!(tsrust_compiler::format(source).unwrap(), *source); }
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(), 1, dependency);
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let result_hover = ask(&index, &entry, "result)", "textDocument/hover");
    let result_docs = result_hover["contents"]["value"].as_str().unwrap_or("");
    assert!(result_docs.replace(' ', "").contains("Result<Person,string>") && !result_docs.contains(" | "), "{result_hover}");
    let hover = ask(&index, &entry, "user.name", "textDocument/hover");
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("user: Person") && !docs.contains("Result#"), "{hover}");
    let hover = ask(&index, &entry, "error.length", "textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("error: string"), "{hover}");
    let completion = ask(&index, &entry, "name.toUpperCase", "textDocument/completion");
    assert!(completion.as_array().is_some_and(|items| items.iter().any(|item| item["label"] == "name")), "{completion}");
    let definition = ask(&index, &entry, "user.name", "textDocument/definition");
    assert_eq!(definition["uri"], uri, "{definition}");
    assert_eq!(definition["range"]["start"]["line"], 5, "{definition}");
}

#[test]
fn unfinished_project_match_keeps_pattern_scope_without_inventing_payload_types() {
    let complete = tsrust_compiler::format("import { make } from './dep.tsr';function main(){match(make()){Ok(user)=>{console.log(user.name);},Err(error)=>{console.log(error);}}}").unwrap();
    let stop = complete.find("user.name").unwrap() + "user.na".len();
    let entry = &complete[..stop];
    let dependency = tsrust_compiler::format("export type User={name:string;};export function make():Result<User,string>{return Ok({name:'Pat'});}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(), 1, dependency);
    let input = workspace.snapshot();
    assert!(input.check(&source_id(uri).unwrap()).is_err());
    let graph = tsrust_compiler::project::load_project_headers(&input, &source_id(uri).unwrap().0).unwrap();
    let index = ProjectIndex::new(&graph);
    let hover = ask(&index, entry, "user.na", "textDocument/hover");
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("(match) user") && !docs.contains("Function<"), "{hover}");
    assert!(!docs.contains("user: User"), "unchecked body must not invent a payload type: {hover}");
    let definition = ask(&index, entry, "user.na", "textDocument/definition");
    assert_eq!(definition["uri"], uri, "{definition}");
    let completion = ask(&index, entry, "user.na", "textDocument/completion");
    assert!(!completion.as_array().is_some_and(|items| items.iter().any(|item| item["label"] == "error")), "{completion}");
    let editing = complete.replace("user.name", "us");
    workspace.change(&source_id(uri).unwrap(), 2, editing.clone());
    let input = workspace.snapshot();
    assert!(input.check(&source_id(uri).unwrap()).is_err());
    let graph = tsrust_compiler::project::load_project_headers(&input, &source_id(uri).unwrap().0).unwrap();
    let index = ProjectIndex::new(&graph);
    let completion = ask(&index, &editing, "us);", "textDocument/completion");
    let items = completion.as_array().expect("arm completion");
    assert!(items.iter().any(|item| item["label"] == "user"), "{completion}");
    assert!(!items.iter().any(|item| item["label"] == "error"), "{completion}");
    let outside = format!("{}us;\n}}", complete.trim_end().strip_suffix('}').unwrap());
    workspace.change(&source_id(uri).unwrap(), 3, outside.clone());
    let input = workspace.snapshot();
    let graph = tsrust_compiler::project::load_project_headers(&input, &source_id(uri).unwrap().0).unwrap();
    let index = ProjectIndex::new(&graph);
    let completion = ask(&index, &outside, "us;", "textDocument/completion");
    assert!(!completion.as_array().is_some_and(|items| items.iter().any(|item| item["label"] == "user")), "{completion}");
}

#[test]
fn nested_imported_result_matches_keep_shadowed_payloads_separate() {
    let entry = tsrust_compiler::format("import {User as Person,make} from './dep.tsr';function main(){match(make()){Ok(value)=>{match(value){Ok(value)=>{console.log(value.name);},Err(value)=>{console.log(value.length);}}},Err(value)=>{console.log(value+1);}}}").unwrap();
    let dependency = tsrust_compiler::format("export type User={name:string;};export function make():Result<Result<User,string>,int>{return Ok(Ok({name:'Pat'}));}").unwrap();
    for source in [&entry, &dependency] { assert_eq!(tsrust_compiler::format(source).unwrap(), *source); }
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(), 1, dependency);
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    for (needle, ty) in [("value.name", "Person"), ("value.length", "string"), ("value + 1", "int")] {
        let hover = ask(&index, &entry, needle, "textDocument/hover");
        let docs = hover["contents"]["value"].as_str().unwrap_or("");
        assert!(docs.contains(&format!("value: {ty}")) && !docs.contains("Result#"), "{hover}");
        let definition = ask(&index, &entry, needle, "textDocument/definition");
        assert_eq!(definition["uri"], uri, "{definition}");
    }
}

#[test]
fn imported_generic_value_alias_keeps_source_contract_and_concrete_help() {
    let entry="import {identity as choose} from './dep.tsr';function main(){const action=choose;const number=action(7);const text=action('x');}";
    let dependency="/** Return the supplied value unchanged. */\nexport function identity<T>(value:T):T{return value;}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(),1,dependency.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"action=","textDocument/hover");
    let value=hover["contents"]["value"].as_str().unwrap();
    assert!(value.contains("<T>") && value.contains("Return the supplied value unchanged."),"{hover}");
    assert!(!value.contains("void"),"{hover}");
    for (needle,ty) in [("7);","int"),("'x');","string")] {
        let help=ask(&index,entry,needle,"textDocument/signatureHelp");
        let label=help["signatures"][0]["label"].as_str().expect("imported generic alias help");
        assert!(label.contains("value") && label.contains(ty),"{help}");
    }
}

#[test]
fn imported_generic_factory_alias_help_tracks_unsaved_contract_and_nominal_alias() {
    let entry="import {User as Person,make} from './dep.tsr';function main(){const action=make(0);const person:Person={name:'Pat'};const chosen=action(person,undefined);}";
    let dependency="export type User={name:string;};export function make(start:int){let calls=start;/** Select the supplied value. */function select<T>(value:T,index:int=0):T{calls+=1;return value;}return select;}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let dep=source_id("file:///tsrust-intelligence/dep.tsr").unwrap();
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    for (version,parameter) in [(1,"value"),(2,"recipient")] {
        workspace.open(dep.clone(),version,dependency.replace("value:T",&format!("{parameter}:T")).replace("return value",&format!("return {parameter}")));
        let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
        let index=ProjectIndex::checked(&checked);
        let hover=ask(&index,entry,"action=","textDocument/hover");
        let value=hover["contents"]["value"].as_str().unwrap();
        assert!(value.contains("<T>") && value.contains(parameter) && value.contains("Select the supplied value."),"{hover}");
        let help=ask(&index,entry,"undefined);","textDocument/signatureHelp");
        let label=help["signatures"][0]["label"].as_str().expect("factory alias help");
        assert!(label.contains("Owned<Person>") && label.contains(parameter) && label.contains("index?"),"{help}");
        assert!(!label.contains("__"),"{help}");
    }
}

#[test]
fn generic_alias_source_signature_remaps_fixed_nominals_without_name_capture() {
    let entry="import {User as Person,choose} from './dep.tsr';type User={wrong:int;};function main(){const action=choose;const person:Person={name:'Pat'};const chosen=action(person,7);}";
    let dependency="export type User={name:string;};export function choose<T>(user:ReadOnly<User>,value:T):T{console.log(user.name);return value;}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(),1,dependency.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"action=","textDocument/hover");
    let value=hover["contents"]["value"].as_str().unwrap();
    assert!(value.contains("<T>") && value.contains("ReadOnly<Person>"),"{hover}");
    assert!(!value.contains("ReadOnly<User>"),"{hover}");
    let help=ask(&index,entry,"person,7","textDocument/signatureHelp");
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("ReadOnly<Person>"),"{help}");
}

#[test]
fn generic_alias_source_signature_keeps_labels_defaults_and_applied_nominals() {
    let entry="import {User as Person,Box as Package,choose} from './dep.tsr';function main(){const action=choose;const person:Person={name:'Pat'};const box:Package<int>={value:1};const chosen=action(person,box,'User',7);}";
    let dependency="export type User={name:string;};export type Box<T>={value:T;};export function choose<T>(User:ReadOnly<User>,box:ReadOnly<Box<int>>,label:string='User',value:T):T{console.log(User.name,box.value,label);return value;}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(),1,dependency.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"action=","textDocument/hover");
    let value=hover["contents"]["value"].as_str().unwrap();
    assert!(value.contains("User:ReadOnly<Person>") && value.contains("ReadOnly<Package<int>>"),"{hover}");
    assert!(value.contains("label:string='User'") && value.contains("value:T"),"{hover}");
}

#[test]
fn generic_alias_source_signature_keeps_private_result_nominal_identity() {
    let entry="import {make} from './dep.tsr';type User={wrong:int;};type Box={wrong:boolean;};function main(){const action=make;const chosen=action(7);console.log(chosen.value.name);}";
    let dependency="type User={name:string;};type Box<T>={value:T;};export function make<T>(value:T):Box<User>{console.log(value);return {value:{name:'Pat'}};}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(),1,dependency.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"action=","textDocument/hover");
    let value=hover["contents"]["value"].as_str().unwrap();
    assert!(value.contains("<T>") && value.contains("Box<User>"),"{hover}");
    assert!(!value.contains("__tsrust_private_"),"{hover}");
    let definition=ask(&index,entry,"name);","textDocument/definition");
    assert_eq!(definition["uri"],"file:///tsrust-intelligence/dep.tsr","{definition}");
}

#[test]
fn checked_project_generic_callable_help_preserves_aliases_and_unsaved_names() {
    let entry = "import {User as Person,invoke as apply} from './dep.tsr';function main(){const action=(user:Person):Person=>{return user;};apply(action,{name:'Pat'});}";
    let dependency = "export type User={name:string;};export function invoke<T>(action:Function<T, [person:Owned<T>]>,value:T):T{return action(value);}";
    let uri = "file:///tsrust-intelligence/main.tsr";
    let dependency_uri = "file:///tsrust-intelligence/dep.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    workspace.open(source_id(dependency_uri).unwrap(), 1, dependency.into());
    for (version, name) in [(1, "person"), (2, "recipient")] {
        if version == 2 {
            workspace.open(source_id(dependency_uri).unwrap(), version, dependency.replace("person:", "recipient:"));
        }
        let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
        let index = ProjectIndex::checked(&checked);
        let expected = format!("Function<Person, [{name}: Owned<Person>]>");
        let hover = ask(&index, entry, "apply(action", "textDocument/hover");
        assert!(hover["contents"]["value"].as_str().unwrap().contains(&expected), "{hover}");
        let help = ask(&index, entry, "action,{name", "textDocument/signatureHelp");
        assert!(help["signatures"][0]["label"].as_str().unwrap().contains(&expected), "{help}");
    }
}

#[test]
fn checked_project_generic_callable_results_show_concrete_aliases() {
    let entry = "import {User as Person,make as build} from './dep.tsr';function main(){const seed:Person={name:'seed'};const action=build(seed);console.log(action({name:'Pat'}).name);}";
    let dependency = "export type User={name:string;};export function make<T>(seed:T):(value:Owned<T>)=>T{return (value:T):T=>{return value;};}";
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(), 1, dependency.into());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let hover = ask(&index, entry, "build(seed)", "textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Function<Person, [value: Owned<Person>]>"), "{hover}");
    let help = ask(&index, entry, "seed);", "textDocument/signatureHelp");
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("Function<Person, [value: Owned<Person>]>"), "{help}");
    let invocation = ask(&index, entry, "{name:'Pat'}", "textDocument/signatureHelp");
    assert!(invocation["signatures"][0]["label"].as_str().unwrap().contains("Person"), "{invocation}");
}

#[test]
fn checked_project_generic_once_results_show_consuming_invocation() {
    let entry = "import {User as Person,make as build} from './dep.tsr';function main(){const seed:Person={name:'Pat'};const action=build(seed);console.log(action().name);}";
    let dependency = "export type User={name:string;};export function make<T>(seed:T):()=>T{return ():T=>{return seed;};}";
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(), 1, dependency.into());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let hover = ask(&index, entry, "build(seed)", "textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Once<Function<Person>>"), "{hover}");
    let help = ask(&index, entry, "seed);", "textDocument/signatureHelp");
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("Once<Function<Person>>"), "{help}");
}

#[test]
fn transitive_generic_callback_results_preserve_private_nominal_identity() {
    let entry = "import {make as build} from './middle.tsr';type Secret={wrong:int;};function main(){const action=build('Pat');const saved=action();console.log(saved.value.trim());}";
    let uri = "file:///tsrust-intelligence/main.tsr";
    let model_uri = "file:///tsrust-intelligence/model.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/middle.tsr").unwrap(), 1, "import {Secret as Hidden} from './model.tsr';export function make<T>(seed:T):()=>Hidden<T>{return ():Hidden<T> =>{return {value:seed};};}".into());
    workspace.open(source_id(model_uri).unwrap(), 1, "export type Secret<T>={value:T;};".into());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let hover = ask(&index, entry, "build('Pat')", "textDocument/hover");
    let display = hover["contents"]["value"].as_str().unwrap();
    assert!(display.contains("Once<Function<Secret<string>>>"), "{hover}");
    assert!(!display.contains("__tsrust_private_"), "{hover}");
    let definition = ask(&index, entry, "value.trim", "textDocument/definition");
    assert_eq!(definition["uri"], model_uri, "{definition}");
    assert!(!ask(&index, entry, "trim()", "textDocument/hover").is_null());
}
#[test]
fn imported_alias_docs_hover_signatures_completion_and_definitions() {
    let entry = "import { greet as hello } from './dep.tsr';function main(){hello(3);}";
    let mut workspace = Workspace::default();
    let uri = "file:///tsrust-intelligence/main.tsr";
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(),1,"/** Adds one. @param value The starting number. */ export function greet(value:int):int{return value+1;}".into());
    let snapshot = workspace.snapshot();
    let checked = snapshot.check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let hover = ask(&index, entry, "hello(3)", "textDocument/hover");
    let docs = hover["contents"]["value"].as_str().unwrap();
    assert!(docs.contains("hello"), "{hover}");
    assert!(docs.contains("Adds one"), "{hover}");
    let definition = ask(&index, entry, "hello(3)", "textDocument/definition");
    assert_eq!(definition["uri"], "file:///tsrust-intelligence/dep.tsr");
    let help = ask(&index, entry, "3)", "textDocument/signatureHelp");
    assert!(
        help["signatures"][0]["label"]
            .as_str()
            .unwrap()
            .contains("hello"),
        "{help}"
    );
    let completion = ask(&index, entry, "hello(3)", "textDocument/completion");
    assert!(
        completion
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "hello")
    );
}
#[test]
fn imported_record_fields_and_local_shadowing_have_correct_targets() {
    let entry = "import { User as Person, value } from './dep.tsr';function main(){const user:Person={name:\"Ada\"};console.log(user.name);{const value=2;console.log(value);}}";
    let mut workspace = Workspace::default();
    let uri = "file:///tsrust-intelligence/main.tsr";
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    workspace.open(
        source_id("file:///tsrust-intelligence/dep.tsr").unwrap(),
        1,
        "export type User={name:string;};export function value():int{return 1;}".into(),
    );
    let input = workspace.snapshot();
    let checked = input.check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let field = ask(&index, entry, "name);", "textDocument/definition");
    assert_eq!(
        field["uri"], "file:///tsrust-intelligence/dep.tsr",
        "{field}"
    );
    let local = ask(&index, entry, "value);", "textDocument/definition");
    assert_eq!(local["uri"], uri, "{local}");
}

#[test]
fn imported_main_is_ordinary_and_cached_help_expires_after_dependency_edit() {
    use tsrust_lsp::{Snapshot, project_session::ProjectSession};
    let root = "file:///tsrust-intelligence/main.tsr";
    let dep = "file:///tsrust-intelligence/dep.tsr";
    let source = "import { main as helper } from './dep.tsr';function main(){helper();}";
    let mut session = ProjectSession::default();
    session.open(&Snapshot {
        uri: dep.into(),
        version: 1,
        generation: 1,
        text: "/** Ordinary helper. */ export function main(){}".into(),
    });
    let entry = Snapshot {
        uri: root.into(),
        version: 1,
        generation: 2,
        text: source.into(),
    };
    session.open(&entry);
    let input = session.prepare(&entry).unwrap();
    let checked = input.check(&source_id(root).unwrap()).unwrap();
    session.cache_index(root, &input, Some(ProjectIndex::new(&checked.graph)));
    let params = json!({"position":{"line":0,"character":source.rfind("helper()").unwrap()}});
    let hover = session
        .request("textDocument/hover", &params, root)
        .unwrap()
        .unwrap();
    let docs = hover["contents"]["value"].as_str().unwrap();
    assert!(docs.contains("Ordinary helper"));
    assert!(!docs.contains("Program entry point"));
    session.change(&Snapshot {
        uri: dep.into(),
        version: 2,
        generation: 1,
        text: "export function main(){}".into(),
    });
    assert!(
        session
            .request("textDocument/hover", &params, root)
            .is_none()
    );
}

#[test]
fn imported_return_types_use_alias_identity_or_remain_unknown() {
    for import in [
        "import { User as Person, make } from './dep.tsr';",
        "import { make } from './dep.tsr';",
    ] {
        let entry = format!(
            "{import}type User={{wrong:int;}};function main(){{const user=make();console.log(user.name);}}"
        );
        let mut workspace = Workspace::default();
        let root = "file:///tsrust-intelligence/main.tsr";
        workspace.open(source_id(root).unwrap(), 1, entry.clone());
        workspace.open(
            source_id("file:///tsrust-intelligence/dep.tsr").unwrap(),
            1,
            "export type User={name:string;};export function make():User{return {name:\"Ada\"};}"
                .into(),
        );
        let input = workspace.snapshot();
        let checked = input.check(&source_id(root).unwrap()).unwrap();
        let index = ProjectIndex::new(&checked.graph);
        let completion = ask(&index, &entry, "name);", "textDocument/completion");
        assert!(
            !completion
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "wrong"),
            "{completion}"
        );
        if import.contains("Person") {
            let definition = ask(&index, &entry, "name);", "textDocument/definition");
            assert_eq!(
                definition["uri"], "file:///tsrust-intelligence/dep.tsr",
                "{definition}"
            );
        }
    }
}

#[test]
fn project_references_follow_aliases_and_exclude_shadowed_locals() {
    let root = "file:///tsrust-intelligence/main.tsr";
    let dep = "file:///tsrust-intelligence/dep.tsr";
    let other = "file:///tsrust-intelligence/other.tsr";
    let entry = "import { greet as hello } from './dep.tsr';import { call } from './other.tsr';function main(){hello(3);call();{const hello=4;console.log(hello);}}";
    let library = "export function greet(value:int):int{return value+1;}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(root).unwrap(), 1, entry.into());
    workspace.open(source_id(dep).unwrap(), 1, library.into());
    workspace.open(source_id(other).unwrap(), 1,
        "import { greet as welcome } from './dep.tsr';export function call():int{return welcome(2);}".into());
    let checked = workspace
        .snapshot()
        .check(&source_id(root).unwrap())
        .unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let refs = ask(&index, entry, "hello(3)", "textDocument/references");
    let locations = refs.as_array().unwrap();
    assert_eq!(
        locations.iter().filter(|loc| loc["uri"] == root).count(),
        3,
        "{refs}"
    );
    assert_eq!(
        locations.iter().filter(|loc| loc["uri"] == other).count(),
        3,
        "{refs}"
    );
    assert_eq!(
        locations.iter().filter(|loc| loc["uri"] == dep).count(),
        1,
        "{refs}"
    );
    let params = json!({"position":{"line":0,"character":library.find("greet").unwrap()},
        "context":{"includeDeclaration":false}});
    let uses = index
        .request("textDocument/references", &params, dep)
        .unwrap()
        .unwrap();
    assert_eq!(uses.as_array().unwrap().len(), 6, "{uses}");
    assert!(!uses.as_array().unwrap().iter().any(|loc| loc["uri"] == dep));
    let local = ask(&index, entry, "hello);", "textDocument/references");
    assert_eq!(local.as_array().unwrap().len(), 2, "{local}");
    assert!(
        local
            .as_array()
            .unwrap()
            .iter()
            .all(|loc| loc["uri"] == root)
    );
}

#[test]
fn session_navigation_preserves_open_uri_aliases() {
    use tsrust_lsp::{Snapshot, project_session::ProjectSession};
    let root = "file:///tsrust-intelligence/main.tsr";
    let dep = "file://localhost/tsrust-intelligence/dep.tsr";
    let entry = "import { value as local } from './dep.tsr';function main(){console.log(local());}";
    let mut session = ProjectSession::default();
    for (uri, text) in [
        (root, entry),
        (dep, "export function value():int{return 1;}"),
    ] {
        session.open(&Snapshot {
            uri: uri.into(),
            text: text.into(),
            version: 1,
            generation: 1,
        });
    }
    let input = session.workspace.snapshot();
    let checked = input.check(&source_id(root).unwrap()).unwrap();
    session.cache_index(root, &input, Some(ProjectIndex::new(&checked.graph)));
    let params = json!({"position":{"line":0,"character":entry.rfind("local()").unwrap()}});
    let definition = session
        .request("textDocument/definition", &params, root)
        .unwrap()
        .unwrap();
    assert_eq!(definition["uri"], dep);
    let references = session
        .request("textDocument/references", &params, root)
        .unwrap()
        .unwrap();
    assert!(
        references
            .as_array()
            .unwrap()
            .iter()
            .any(|location| location["uri"] == dep)
    );
    assert!(
        !references
            .to_string()
            .contains("file:///tsrust-intelligence/dep.tsr")
    );
}

#[test]
fn imported_nested_private_records_keep_module_identity_and_field_navigation() {
    let root = "file:///tsrust-intelligence/main.tsr";
    let dep = "file:///tsrust-intelligence/dep.tsr";
    let entry = "import { User as Person, make } from './dep.tsr';type Address={wrong:int;};function main(){const user=make();console.log(user.address.city);console.log(user.addresses[0].city);}";
    let library = "type Address={city:string;};export type User={address:Address;addresses:Address[];};export function make():User{return {address:{city:\"Ada\"},addresses:[{city:\"Rome\"}]};}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(root).unwrap(), 1, entry.into());
    workspace.open(source_id(dep).unwrap(), 1, library.into());
    let checked = workspace
        .snapshot()
        .check(&source_id(root).unwrap())
        .unwrap();
    let index = ProjectIndex::new(&checked.graph);
    for needle in ["city);", "city);console"] {
        let completion = ask(&index, entry, needle, "textDocument/completion");
        assert!(
            completion
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "city"),
            "{completion}"
        );
        assert!(
            !completion
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "wrong")
        );
        let definition = ask(&index, entry, needle, "textDocument/definition");
        assert_eq!(definition["uri"], dep, "{definition}");
        assert_eq!(
            definition["range"]["start"]["character"],
            library.find("city:string").unwrap()
        );
    }
    let references = ask(&index, entry, "city);", "textDocument/references");
    assert_eq!(references.as_array().unwrap().len(), 3, "{references}");
}

#[test]
fn nested_records_across_dependency_chains_keep_original_field_locations() {
    let root = "file:///tsrust-intelligence/main.tsr";
    let middle = "file:///tsrust-intelligence/middle.tsr";
    let leaf = "file:///tsrust-intelligence/leaf.tsr";
    let entry = "import { User as Person, make } from './middle.tsr';type Address={wrong:int;};function main(){const user=make();console.log(user.address.city);console.log(user.address.geo.code);}";
    let library = "import { Address as Location } from './leaf.tsr';export type User={address:Location;};export function make():User{return {address:{city:\"Rome\",geo:{code:7}}};}";
    let leaf_source = "type Geo={code:int;};export type Address={city:string;geo:Geo;};";
    let mut workspace = Workspace::default();
    for (uri, text) in [(root, entry), (middle, library), (leaf, leaf_source)] {
        workspace.open(source_id(uri).unwrap(), 1, text.into());
    }
    let checked = workspace
        .snapshot()
        .check(&source_id(root).unwrap())
        .unwrap();
    let index = ProjectIndex::new(&checked.graph);
    for (needle, field) in [("city);", "city"), ("code);", "code")] {
        let completion = ask(&index, entry, needle, "textDocument/completion");
        assert!(
            completion
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == field),
            "{completion}"
        );
        assert!(
            !completion
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == "wrong")
        );
        let definition = ask(&index, entry, needle, "textDocument/definition");
        assert_eq!(definition["uri"], leaf, "{definition}");
        assert_eq!(
            definition["range"]["start"]["character"],
            leaf_source.find(&format!("{field}:")).unwrap()
        );
        let references = ask(&index, entry, needle, "textDocument/references");
        assert!(
            references
                .as_array()
                .unwrap()
                .iter()
                .any(|location| location["uri"] == root)
        );
        assert!(
            references
                .as_array()
                .unwrap()
                .iter()
                .any(|location| location["uri"] == leaf)
        );
    }
}

#[test]
fn current_headers_supply_imported_help_while_bodies_are_incomplete() {
    let root = "file:///tsrust-intelligence/main.tsr";
    let dep = "file:///tsrust-intelligence/dep.tsr";
    let library = "/** Current helper. */ export function greet(value:int):int{return value;}export type User={name:string;};";
    let entry = "import { greet as hello, User } from './dep.tsr';function main(){const user:User={name:\"Ada\"};hello(2);user.na";
    let mut workspace = Workspace::default();
    workspace.open(source_id(root).unwrap(), 1, entry.into());
    workspace.open(source_id(dep).unwrap(), 1, library.into());
    let input = workspace.snapshot();
    assert!(input.check(&source_id(root).unwrap()).is_err());
    let graph = tsrust_compiler::project::load_project_headers(&input, &source_id(root).unwrap().0)
        .unwrap();
    let index = ProjectIndex::new(&graph);
    let completion = ask(&index, entry, "na", "textDocument/completion");
    assert!(
        completion
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "name"),
        "{completion}"
    );
    let hover = ask(&index, entry, "hello(2)", "textDocument/hover");
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Current helper.")
    );
    let definition = ask(&index, entry, "hello(2)", "textDocument/definition");
    assert_eq!(definition["uri"], dep);
    workspace.change(
        &source_id(dep).unwrap(),
        2,
        "function greet(value:int):int{return value;}export type User={name:string;};".into(),
    );
    // A removed export loses its imported help; the names that remain keep theirs.
    let next = workspace.snapshot();
    assert!(next.check(&source_id(root).unwrap()).is_err());
    let graph = tsrust_compiler::project::load_project_headers(&next, &source_id(root).unwrap().0)
        .unwrap();
    let index = ProjectIndex::new(&graph);
    let stale = index
        .request("textDocument/hover", &json!({"position":{"line":0,"character":entry.rfind("hello(2)").unwrap()}}), root)
        .unwrap()
        .unwrap();
    assert!(!stale.to_string().contains("Current helper."), "{stale}");
    let completion = ask(&index, entry, "na", "textDocument/completion");
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "name"), "{completion}");
}

#[test]
fn std_imported_alias_help_uses_registry_and_catalog_and_respects_shadowing() {
    let root = "file:///tsrust-intelligence/main.tsr";
    let entry = "import { basename as fileName } from 'std:path';function main(){const path='/tmp/report.txt';const name=fileName(path,'.txt');console.log(name.toUpperCase());{const fileName=2;console.log(fileName);}}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(root).unwrap(), 1, entry.into());
    let checked = workspace
        .snapshot()
        .check(&source_id(root).unwrap())
        .unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let hover = ask(&index, entry, "fileName(path", "textDocument/hover");
    let docs = hover["contents"]["value"].as_str().unwrap();
    assert!(
        docs.contains("function fileName(path: string, suffix?: string): string"),
        "{hover}"
    );
    assert!(docs.contains("without reading the filesystem"));
    assert!(docs.contains("report.txt"));
    let help = ask(&index, entry, "'.txt'", "textDocument/signatureHelp");
    assert_eq!(help["activeParameter"], 1);
    assert_eq!(
        help["signatures"][0]["parameters"][1]["label"],
        "suffix?: string"
    );
    assert!(
        help["signatures"][0]["parameters"][1]["documentation"]["value"]
            .as_str()
            .unwrap()
            .contains("case-sensitive"),
        "{help}"
    );
    let completion = ask(&index, entry, "fileName(path", "textDocument/completion");
    assert!(
        completion
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "fileName"
                && item["detail"].as_str().unwrap().contains("suffix?: string"))
    );
    let result = ask(&index, entry, "toUpperCase", "textDocument/hover");
    assert!(
        result["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("toUpperCase")
    );
    let local = ask(&index, entry, "fileName);", "textDocument/hover");
    assert!(
        !local["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("basename")
    );
    let incomplete=entry.replace("const name=fileName(path,'.txt');console.log(name.toUpperCase());{const fileName=2;console.log(fileName);}","fileN");
    workspace.change(&source_id(root).unwrap(), 2, incomplete.clone());
    let input = workspace.snapshot();
    let graph = tsrust_compiler::project::load_project_headers(&input, &source_id(root).unwrap().0)
        .unwrap();
    let index = ProjectIndex::new(&graph);
    let completion = ask(&index, &incomplete, "fileN}", "textDocument/completion");
    assert!(
        completion
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "fileName"),
        "{completion}"
    );
}

#[test]
fn stored_native_values_keep_optional_signatures_docs_and_string_results() {
    let root = "file:///tsrust-intelligence/main.tsr";
    let entry = "import { basename as original } from 'std:path';function main(){const nameOf=original;const moved=nameOf;const result=moved('/tmp/report.txt','.txt');console.log(result.toUpperCase());{const original=2;console.log(original);}}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(root).unwrap(), 1, entry.into());
    let checked = workspace.snapshot().check(&source_id(root).unwrap()).unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let hover = ask(&index, entry, "moved('/tmp", "textDocument/hover");
    let docs = hover["contents"]["value"].as_str().unwrap();
    assert!(docs.contains("const moved: (path: string, suffix?: string) => string"), "{hover}");
    assert!(docs.contains("without reading the filesystem"));
    let completion = ask(&index, entry, "moved('/tmp", "textDocument/completion");
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "moved" && item["detail"].as_str().is_some_and(|detail| detail.contains("suffix?: string"))));
    let help = ask(&index, entry, "'.txt'", "textDocument/signatureHelp");
    assert_eq!(help["activeParameter"], 1);
    assert_eq!(help["signatures"][0]["parameters"][1]["label"], "suffix?: string");
    assert!(help["signatures"][0]["parameters"][1]["documentation"]["value"].as_str().unwrap().contains("case-sensitive"));
    let result = ask(&index, entry, "toUpperCase", "textDocument/hover");
    assert!(result["contents"]["value"].as_str().unwrap().contains("toUpperCase"));
    let shadow = ask(&index, entry, "original);", "textDocument/hover");
    assert!(!shadow["contents"]["value"].as_str().unwrap().contains("basename"));
}

#[test]
fn returned_native_callable_values_have_typed_hover_and_signature_help() {
    let root = "file:///tsrust-intelligence/main.tsr";
    let entry = "import { basename } from 'std:path';function factory():(path:ReadOnly<string>)=>string{return basename;}function main(){const nameOf=factory();console.log(nameOf('/tmp/report.txt').toUpperCase());}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(root).unwrap(), 1, entry.into());
    let checked = workspace.snapshot().check(&source_id(root).unwrap()).unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let hover = ask(&index, entry, "nameOf('/tmp", "textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().split_whitespace().collect::<String>().contains("=>string"), "{hover}");
    let help = ask(&index, entry, "'/tmp/report.txt'", "textDocument/signatureHelp");
    assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(), 1, "{help}");
    assert!(help["signatures"][0]["parameters"][0]["label"].as_str().unwrap().contains("path"));
    let chained = ask(&index, entry, "toUpperCase", "textDocument/hover");
    assert!(chained["contents"]["value"].as_str().unwrap().contains("toUpperCase"), "{chained}");
}

#[test]
fn imported_factory_results_remain_callable_in_current_project_metadata() {
    let root = "file:///tsrust-intelligence/main.tsr";
    let dep = "file:///tsrust-intelligence/factory.tsr";
    let entry = "import { factory as make } from './factory.tsr';function main(){const nameOf=make();console.log(nameOf('/tmp/report.txt').toUpperCase());}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(root).unwrap(), 1, entry.into());
    workspace.open(source_id(dep).unwrap(), 1, "import { basename } from 'std:path';export function factory():(path:ReadOnly<string>)=>string{return basename;}".into());
    let checked = workspace.snapshot().check(&source_id(root).unwrap()).unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let hover = ask(&index, entry, "nameOf('/tmp", "textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().split_whitespace().collect::<String>().contains("=>string"), "{hover}");
    let help = ask(&index, entry, "'/tmp/report.txt'", "textDocument/signatureHelp");
    assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(), 1, "{help}");
}

#[test]
fn imported_factory_callable_record_parameters_use_local_type_aliases() {
    let root = "file:///tsrust-intelligence/main.tsr";
    let dep = "file:///tsrust-intelligence/factory.tsr";
    let entry = "import { factory as make,User as Person } from './factory.tsr';function main(){const readName=make();const user:Person={name:'Pat'};console.log(readName(user));}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(root).unwrap(), 1, entry.into());
    workspace.open(source_id(dep).unwrap(), 1, "export type User={name:string;};export function factory():(user:ReadOnly<User>)=>string{return (user:User):string=>{return user.name.trim();};}".into());
    let checked = workspace.snapshot().check(&source_id(root).unwrap()).unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let help = ask(&index, entry, "user));", "textDocument/signatureHelp");
    assert!(help["signatures"][0]["parameters"][0]["label"].as_str().unwrap().contains("Person"), "{help}");
}

#[test]
fn conditional_native_initializers_preserve_registry_identity_and_optional_help() {
    let root = "file:///tsrust-intelligence/main.tsr";
    let entry = "import { basename as left,basename as right } from 'std:path';function main(){const original=left;const selected=(true?original:(false?left:right));console.log(selected('/tmp/report.txt','.txt'));}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(root).unwrap(), 1, entry.into());
    let checked = workspace.snapshot().check(&source_id(root).unwrap()).unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let hover = ask(&index, entry, "selected('/tmp", "textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("suffix?: string"), "{hover}");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("without reading the filesystem"));
    let help = ask(&index, entry, "'.txt'", "textDocument/signatureHelp");
    assert_eq!(help["activeParameter"], 1);
    assert!(help["signatures"][0]["parameters"][1]["documentation"]["value"].as_str().unwrap().contains("case-sensitive"));
    let mixed = entry.replace("(true?original:(false?left:right))", "(true?original:2)");
    workspace.change(&source_id(root).unwrap(), 2, mixed.clone());
    let input = workspace.snapshot();
    let graph = tsrust_compiler::project::load_project_headers(&input, &source_id(root).unwrap().0).unwrap();
    let index = ProjectIndex::new(&graph);
    let hover = ask(&index, &mixed, "selected('/tmp", "textDocument/hover");
    assert!(!hover["contents"]["value"].as_str().unwrap().contains("without reading the filesystem"), "{hover}");

}

#[test]
fn imported_generic_alias_fields_follow_unsaved_template_changes() {
    let entry = "import {Box as Package} from './dep.tsr';function main(){const item:Package<string>={value:'Pat'};console.log(item.value.trim());}";
    let mut workspace = Workspace::default();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let dependency = "file:///tsrust-intelligence/dep.tsr";
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    workspace.open(source_id(dependency).unwrap(),1,"export type Box<T>={value:T;};".into());
    let input = workspace.snapshot();
    let checked = input.check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let hover = ask(&index,entry,"value.trim","textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("value: string"),"{hover}");
    let definition = ask(&index,entry,"value.trim","textDocument/definition");
    assert_eq!(definition["uri"],dependency);
    workspace.open(source_id(dependency).unwrap(),2,"export type Box<T>={value:int;};".into());
    let input = workspace.snapshot();
    let graph = tsrust_compiler::project::load_project_headers(&input, &source_id(uri).unwrap().0).unwrap();
    let changed = ProjectIndex::new(&graph);
    let hover = ask(&changed,entry,"value.trim","textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("value: int"),"{hover}");
}

#[test]
fn imported_generic_private_records_keep_identity_and_field_navigation() {
    let entry = "import {Envelope as Package} from './dep.tsr';type Secret={wrong:int;};function main(){const item:Package<string>={inner:{value:'Pat'}};console.log(item.inner.value.trim());}";
    let mut workspace = Workspace::default();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let dependency = "file:///tsrust-intelligence/dep.tsr";
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    workspace.open(source_id(dependency).unwrap(),1,"type Secret<T>={value:T;};export type Envelope<T>={inner:Secret<T>;};".into());
    let input = workspace.snapshot();
    let checked = input.check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let hover = ask(&index,entry,"value.trim","textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("value: string"),"{hover}");
    let inner = ask(&index,entry,"inner.value","textDocument/hover");
    let display = inner["contents"]["value"].as_str().unwrap();
    assert!(display.contains("inner: Secret<string>"),"{inner}");
    assert!(!display.contains("__tsrust_private_"),"{inner}");
    let definition = ask(&index,entry,"value.trim","textDocument/definition");
    assert_eq!(definition["uri"],dependency);
    let hover = ask(&index,entry,"trim()","textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("trim"),"{hover}");
}

#[test]
fn transitive_generic_templates_preserve_original_definitions() {
    let entry = "import {Envelope as Package} from './dep.tsr';function main(){const item:Package<string>={inner:{value:'Pat'}};console.log(item.inner.value.trim());}";
    let mut workspace = Workspace::default();
    let uri = "file:///tsrust-intelligence/main.tsr";
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(),1,"import {Box as Hidden} from './model.tsr';export type Envelope<T>={inner:Hidden<T>;};".into());
    workspace.open(source_id("file:///tsrust-intelligence/model.tsr").unwrap(),1,"export type Box<T>={value:T;};".into());
    let input = workspace.snapshot();
    let checked = input.check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::new(&checked.graph);
    let hover = ask(&index,entry,"value.trim","textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("value: string"),"{hover}");
    let definition = ask(&index,entry,"value.trim","textDocument/definition");
    assert_eq!(definition["uri"],"file:///tsrust-intelligence/model.tsr");
}

#[test]
fn checked_project_generic_aliases_preserve_nominal_identity_and_unsaved_updates() {
    let entry="/* 🦀 */ import {User as Person,Box as Package,identity as keep,wrap as make} from './dep.tsr';function main(){const user:Person={name:'Pat'};const wrapped=make(user);const n=keep(7);const text=keep('Sam');console.log(wrapped.value.name,n,text.toLowerCase());}";
    let dependency="/** User data. */ export type User={name:string;};export type Box<T>={value:T;};/** Keep the supplied value. */ export function identity<T>(value:T):T{return value;}/** Wrap the supplied value. */ export function wrap<T>(value:T):Box<T>{return {value:value};}";
    let mut workspace=Workspace::default();
    let uri="file:///tsrust-intelligence/main.tsr";
    let dep="file:///tsrust-intelligence/dep.tsr";
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    workspace.open(source_id(dep).unwrap(),1,dependency.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"make(user)","textDocument/hover");
    let value=hover["contents"]["value"].as_str().unwrap();
    assert!(value.contains("make(value: Owned<Person>): Package<Person>"),"{hover}");
    assert!(value.contains("Wrap the supplied value."));
    assert!(!value.contains("__tsrust_module_"));
    let number=ask(&index,entry,"keep(7)","textDocument/hover");
    assert!(number["contents"]["value"].as_str().unwrap().contains("keep(value: int): int"));
    assert!(!ask(&index,entry,"toLowerCase()","textDocument/hover").is_null());
    assert!(!ask(&index,entry,"name,n","textDocument/hover").is_null());
    let field_definition=ask(&index,entry,"name,n","textDocument/definition");
    assert_eq!(field_definition["uri"],dep);
    let references=ask(&index,entry,"name,n","textDocument/references");
    assert!(references.as_array().unwrap().iter().any(|location|location["uri"]==uri),"{references}");
    let help=ask(&index,entry,"user);const n","textDocument/signatureHelp");
    assert_eq!(help["signatures"][0]["label"],"function make(value: Owned<Person>): Package<Person>");
    let definition=ask(&index,entry,"make(user)","textDocument/definition");
    assert_eq!(definition["uri"],dep);
    workspace.change(&source_id(dep).unwrap(),2,dependency.replace("Keep the supplied value.","Return the supplied argument unchanged."));
    let changed=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let updated=ProjectIndex::checked(&changed);
    let hover=ask(&updated,entry,"keep(7)","textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Return the supplied argument unchanged."));
    let revised=dependency.replace("function identity<T>(value:T):T{return value;}","function identity<T>(value:T):string{return 'changed';}");
    workspace.change(&source_id(dep).unwrap(),3,revised);
    let changed=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let updated=ProjectIndex::checked(&changed);
    let hover=ask(&updated,entry,"keep(7)","textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("keep(value: int): string"),"{hover}");
}

#[test]
fn checked_private_generic_results_cross_modules_without_local_name_capture() {
    let entry="import {make as build,identity as keep} from './dep.tsr';type Secret={wrong:int;};function main(){const item=build('Pat');const retained=keep(item);console.log(retained.value.trim());}";
    let mut workspace=Workspace::default();
    let uri="file:///tsrust-intelligence/main.tsr";
    let dependency="file:///tsrust-intelligence/dep.tsr";
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    workspace.open(source_id(dependency).unwrap(),1,"type Secret<T>={value:T;};export function make<T>(value:T):Secret<T>{return {value:value};}export function identity<T>(value:T):T{return value;}".into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"build('Pat')","textDocument/hover");
    let display=hover["contents"]["value"].as_str().unwrap();
    assert!(display.contains("build(value: Owned<string>): Secret<string>"),"{hover}");
    assert!(!display.contains("__tsrust_private_"));
    let field=ask(&index,entry,"value.trim","textDocument/hover");
    assert!(field["contents"]["value"].as_str().unwrap().contains("value: string"),"{field}");
    let definition=ask(&index,entry,"value.trim","textDocument/definition");
    assert_eq!(definition["uri"],dependency);
    assert!(!ask(&index,entry,"trim()","textDocument/hover").is_null());
}

#[test]
fn checked_transitive_generic_results_keep_same_named_nominals_distinct() {
    let entry="import {make as leftMake} from './left.tsr';import {make as rightMake} from './right.tsr';type Secret={wrong:int;};function main(){const left=leftMake('Pat');const right=rightMake('Sam');console.log(left.value.trim(),right.value.toLowerCase());}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let left_model="file:///tsrust-intelligence/left-model.tsr";
    let right_model="file:///tsrust-intelligence/right-model.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    for (file,model) in [("left.tsr","left-model.tsr"),("right.tsr","right-model.tsr")] {
        workspace.open(source_id(&format!("file:///tsrust-intelligence/{file}")).unwrap(),1,format!("import {{Secret as Hidden}} from './{model}';export function make<T>(value:T):Hidden<T>{{return {{value:value}};}}"));
    }
    for model in [left_model,right_model] {
        workspace.open(source_id(model).unwrap(),1,"export type Secret<T>={value:T;};".into());
    }
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    for (needle,expected) in [("value.trim",left_model),("value.toLowerCase",right_model)] {
        let hover=ask(&index,entry,needle,"textDocument/hover");
        assert!(hover["contents"]["value"].as_str().unwrap().contains("value: string"),"{hover}");
        let definition=ask(&index,entry,needle,"textDocument/definition");
        assert_eq!(definition["uri"],expected,"{definition}");
        let refs=ask(&index,entry,needle,"textDocument/references");
        assert!(refs.as_array().unwrap().iter().any(|location|location["uri"]==uri),"{refs}");
        assert!(refs.as_array().unwrap().iter().all(|location|location["uri"]!=if expected==left_model{right_model}else{left_model}),"{refs}");
    }
    let hover=ask(&index,entry,"leftMake('Pat')","textDocument/hover");
    let display=hover["contents"]["value"].as_str().unwrap();
    assert!(display.contains("leftMake(value: Owned<string>): Secret<string>"),"{hover}");
    assert!(!display.contains("__tsrust_private_"));
    assert!(!ask(&index,entry,"trim()","textDocument/hover").is_null());
    assert!(!ask(&index,entry,"toLowerCase()","textDocument/hover").is_null());
}

#[test]
fn imported_callable_field_help_tracks_unsaved_declaration_names() {
    let entry = "import {Handler as Work,make as build} from './dep.tsr';function main(){const work:Work=build();console.log(work.run('Pat').trim());}";
    let dependency = "export type Handler={run:Function<string, [input:ReadOnly<string>]>;};export function make():Handler{return {run:(text:string):string=>{return text.toLowerCase();}};}";
    let uri = "file:///tsrust-intelligence/main.tsr";
    let dependency_uri = "file:///tsrust-intelligence/dep.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    for (version, parameter) in [(1, "input"), (2, "recipient")] {
        workspace.open(source_id(dependency_uri).unwrap(), version, dependency.replace("input:", &format!("{parameter}:")));
        let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
        let index = ProjectIndex::checked(&checked);
        let hover = ask(&index, entry, "run('Pat')", "textDocument/hover");
        assert!(hover["contents"]["value"].as_str().unwrap().contains(parameter), "{hover}");
        let help = ask(&index, entry, "'Pat')", "textDocument/signatureHelp");
        assert!(help["signatures"][0]["label"].as_str().unwrap().contains(parameter), "{help}");
        assert!(!ask(&index, entry, "trim()", "textDocument/hover").is_null());
    }
}

#[test]
fn imported_callable_field_signatures_use_callers_nominal_aliases() {
    let entry = "import {User as Person,Handler as Work,make as build} from './dep.tsr';function main(){const work:Work=build();const person:Person={name:'Pat'};console.log(work.run(person).trim());}";
    let dependency = "export type User={name:string;};export type Handler={run:Function<string, [value:ReadOnly<User>]>;};export function make():Handler{return {run:(user:User):string=>{return user.name.trim();}};}";
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(), 1, dependency.into());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    for (method, needle) in [("textDocument/hover", "run(person)"), ("textDocument/signatureHelp", "person).trim")] {
        let response = ask(&index, entry, needle, method);
        let label = if method == "textDocument/hover" { &response["contents"]["value"] } else { &response["signatures"][0]["label"] };
        assert!(label.as_str().unwrap().contains("Person"), "{response}");
        assert!(!label.as_str().unwrap().contains("read User"), "{response}");
    }
}

#[test]
fn private_callable_field_parameters_use_source_type_names() {
    let entry = "import {Handler,make} from './dep.tsr';function main(){const handler:Handler=make();console.log(handler.run({name:'Pat'}));}";
    let dependency = "type Secret={name:string;};export type Handler={run:Function<string, [value:ReadOnly<Secret>]>;};export function make():Handler{return {run:(user:Secret):string=>{return user.name.trim();}};}";
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(), 1, dependency.into());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let help = ask(&index, entry, "{name:'Pat'}", "textDocument/signatureHelp");
    let parameter = help["signatures"][0]["parameters"][0]["label"].as_str().unwrap();
    assert_eq!(parameter, "value:ReadOnly<Secret>", "{help}");
    assert!(!help.to_string().contains("__tsrust_private_"), "{help}");
}

#[test]
fn imported_callable_array_fields_preserve_private_types_and_unsaved_help() {
    let entry = "import {Collection as Tasks,make,user} from './dep.tsr';function main(){const tasks:Tasks=make();const value=user();const saved=tasks.actions[0](value);console.log(saved.trim());console.log(tasks.actions[0](value).trim());}";
    let dependency = "type Hidden={name:string;};export type Collection={actions:((input:ReadOnly<Hidden>)=>string)[];};export function user():Hidden{return {name:'Pat'};}export function make():Collection{return {actions:[(value:Hidden):string=>{return value.name.trim();}]};}";
    let uri = "file:///tsrust-intelligence/main.tsr";
    let dependency_uri = "file:///tsrust-intelligence/dep.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    for (version, parameter) in [(1, "input"), (2, "recipient")] {
        workspace.open(source_id(dependency_uri).unwrap(), version, dependency.replace("input:", &format!("{parameter}:")));
        let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
        let index = ProjectIndex::checked(&checked);
        let help = ask(&index, entry, "value).trim", "textDocument/signatureHelp");
        let label = help["signatures"][0]["parameters"][0]["label"].as_str().expect("imported indexed parameter");
        assert!(label.contains(parameter), "{help}");
        assert!(label.contains("ReadOnly<Hidden>"), "{help}");
        assert!(!help.to_string().contains("__tsrust_private_"), "{help}");
        assert!(!ask(&index, entry, "trim()", "textDocument/hover").is_null());
        let saved = ask(&index, entry, "saved.trim", "textDocument/hover");
        assert!(saved["contents"]["value"].as_str().unwrap().contains("string"), "{saved}");
        let definition = ask(&index, entry, "actions[0]", "textDocument/definition");
        assert_eq!(definition["uri"], dependency_uri, "{definition}");
    }
}

#[test]
fn bundled_assertion_imports_supply_hover_and_signature_help() {
    let entry = "import {strictEqual as same} from 'std:testing';function main(){same(3,3,'sum');}";
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.into());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let hover = ask(&index, entry, "same(3", "textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("Object.is"), "{hover}");
    let help = ask(&index, entry, "3,'sum'", "textDocument/signatureHelp");
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("expected"), "{help}");
}

#[test]
fn bundled_expect_supplies_generic_method_hover_and_parameter_help() {
    let entry = "import {expect} from 'std:testing';function main(){expect(3).toBe(3);}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"toBe", "textDocument/hover");
    let completion=ask(&index,entry,"toBe","textDocument/completion");
    assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="toBe" && item["detail"].as_str().is_some_and(|label|label.contains("expected: int"))),"{completion}");
    assert!(hover["contents"]["value"].as_str().is_some_and(|value|value.contains("Object.is")),"{hover}");
    let help=ask(&index,entry,"3);", "textDocument/signatureHelp");
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("expected: int"),"{help}");
}

#[test]
fn conventional_testing_entry_supports_top_level_matcher_editor_help() {
    let entry="import {test,expect} from 'std:testing';test('sum',():void=>{expect(3).toBe(3);});";
    let uri="file:///tsrust-intelligence/sum.test.tsr";
    let id=source_id(uri).unwrap();
    let mut workspace=Workspace::default();
    workspace.open(id.clone(),1,entry.into());
    let checked=workspace.snapshot().check(&id).unwrap();
    let index=ProjectIndex::checked(&checked);
    for (needle,method,expected) in [("toBe","textDocument/hover","Object.is"),("3);","textDocument/signatureHelp","expected: int"),("test('sum'","textDocument/hover","Registers a synchronous named test")] {
        let offset=entry.rfind(needle).unwrap();
        let result=index.request(method,&json!({"position":{"line":0,"character":entry[..offset].encode_utf16().count()}}),uri).unwrap().unwrap();
        assert!(result.to_string().contains(expected),"{result}");
    }
}

#[test]
fn bundled_suite_registration_supplies_borrowed_callback_help() {
    let entry="import {describe as group,test} from 'std:testing';group('math',():void=>{test('sum',():void=>{});});";
    let uri="file:///tsrust-intelligence/sum.test.tsr";
    let id=source_id(uri).unwrap();let mut workspace=Workspace::default();workspace.open(id.clone(),1,entry.into());
    let checked=workspace.snapshot().check(&id).unwrap();let index=ProjectIndex::checked(&checked);
    for (needle,method,expected) in [("group('math'","textDocument/hover","Groups synchronous tests"),("():void=>","textDocument/signatureHelp","Mutable<Function<void>>")] {
        let offset=entry.find(needle).unwrap();
        let result=index.request(method,&json!({"position":{"line":0,"character":entry[..offset].encode_utf16().count()}}),uri).unwrap().unwrap();
        assert!(result.to_string().contains(expected),"{result}");
    }
}

#[test]
fn bundled_each_hook_editor_help_tracks_owned_reusable_callbacks() {
    let entry="import {beforeEach as setup,test} from 'std:testing';setup(():void=>{});test('body',():void=>{});";
    let uri="file:///tsrust-intelligence/hooks.test.tsr";
    let id=source_id(uri).unwrap();let mut workspace=Workspace::default();workspace.open(id.clone(),1,entry.into());
    let checked=workspace.snapshot().check(&id).unwrap();let index=ProjectIndex::checked(&checked);
    for (needle,method,expected) in [("setup(()","textDocument/hover","Registers a synchronous setup callback before every test"),("():void=>{});test","textDocument/signatureHelp","Owned<Function<void>>")] {
        let offset=entry.find(needle).unwrap();
        let result=index.request(method,&json!({"position":{"line":0,"character":entry[..offset].encode_utf16().count()}}),uri).unwrap().unwrap();
        assert!(result.to_string().contains(expected),"{result}");
    }
}

#[test]
fn bundled_truthiness_matchers_supply_hover_completion_and_zero_argument_help() {
    for name in ["toBeTruthy","toBeFalsy"] {
        let entry=format!("import {{expect}} from 'std:testing';function main(){{expect(0).{name}();}}");
        let uri="file:///tsrust-intelligence/main.tsr";
        let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.clone());
        let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
        let hover=ask(&index,&entry,name,"textDocument/hover");
        assert!(hover.to_string().contains("Boolean conversion"),"{hover}");
        let completion=ask(&index,&entry,name,"textDocument/completion");
        assert!(completion.as_array().unwrap().iter().any(|row|row["label"]==name),"{completion}");
        let help=ask(&index,&entry,");}","textDocument/signatureHelp");
        assert!(help["signatures"][0]["label"].as_str().unwrap().contains(name),"{help}");
        assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),0);
    }
}

#[test]
fn bundled_nullish_matchers_supply_documented_zero_argument_help() {
    for name in ["toBeNull","toBeUndefined","toBeDefined","toBeNaN"] {
        let entry=format!("import {{expect}} from 'std:testing';function main(){{expect(null).{name}();}}");
        let uri="file:///tsrust-intelligence/main.tsr";
        let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.clone());
        let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
        let hover=ask(&index,&entry,name,"textDocument/hover");assert!(hover.to_string().contains("Takes no arguments"),"{hover}");
        let completion=ask(&index,&entry,name,"textDocument/completion");assert!(completion.as_array().unwrap().iter().any(|row|row["label"]==name),"{completion}");
        let help=ask(&index,&entry,");}","textDocument/signatureHelp");assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),0);
    }
}

#[test]
fn nullable_expectation_exposes_predicate_completion_and_help() {
    let entry="import {expect} from 'std:testing';function inspect(value:string|null|undefined):void{expect(value).toBeDefined();}function main(){inspect(null);}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"toBeDefined","textDocument/hover");assert!(hover.to_string().contains("data unions"),"{hover}");
    let completion=ask(&index,entry,"toBeDefined","textDocument/completion");assert!(completion.as_array().unwrap().iter().any(|row|row["label"]=="toBeDefined"),"{completion}");
}

#[test]
fn copy_union_expectation_signature_uses_copy_parameter_contract() {
    let entry="import {expect} from 'std:testing';function inspect(value:number|null|undefined):void{expect(value).toBe(value);}function main(){inspect(null);}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
    let help=ask(&index,entry,"value);}","textDocument/signatureHelp");
    let label=help["signatures"][0]["label"].as_str().unwrap();
    assert!(label.contains("expected:"),"{help}");assert!(!label.contains("ReadOnly"),"{help}");
}

#[test]
fn negated_expectation_exposes_completion_hover_and_parameter_help() {
    let entry="import {expect} from 'std:testing';function main(){const assertion=expect('saved');assertion.not.toBe('different');}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"toBe(","textDocument/hover");
    assert!(hover.to_string().contains("false"),"{hover}");
    let completion=ask(&index,entry,"toBe(","textDocument/completion");
    for name in ["toBe","toBeTruthy","toBeFalsy","toBeNull","toBeUndefined","toBeDefined","toBeNaN"] {
        assert!(completion.as_array().unwrap().iter().any(|row|row["label"]==name),"{name}: {completion}");
    }
    let help=ask(&index,entry,"'different'","textDocument/signatureHelp");
    assert!(help.to_string().contains("expected"),"{help}");
}

#[test]
fn typeof_matcher_completion_hover_and_parameter_help_cover_both_views() {
    for view in ["", "not."] {
        let entry=format!("import {{expect}} from 'std:testing';function main(){{expect(null).{view}toBeTypeOf('object');}}");
        let uri="file:///tsrust-intelligence/main.tsr";
        let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.clone());
        let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
        let hover=ask(&index,&entry,"toBeTypeOf(","textDocument/hover");
        assert!(hover.to_string().contains("No value coercion"),"{hover}");
        let completion=ask(&index,&entry,"toBeTypeOf(","textDocument/completion");
        assert!(completion.as_array().unwrap().iter().any(|row|row["label"]=="toBeTypeOf"),"{completion}");
        let help=ask(&index,&entry,"'object'","textDocument/signatureHelp");
        assert!(help.to_string().contains("expected"),"{help}");
    }
}

#[test]
fn numeric_comparison_completion_hover_and_parameter_help_cover_both_views() {
    for view in ["", "not."] {
        for method in ["toBeGreaterThan","toBeGreaterThanOrEqual","toBeLessThan","toBeLessThanOrEqual"] {
            let entry=format!("import {{expect}} from 'std:testing';function main(){{expect(3).{view}{method}(2.0);}}");
            let uri="file:///tsrust-intelligence/main.tsr";let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.clone());
            let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
            let hover=ask(&index,&entry,&format!("{method}("),"textDocument/hover");assert!(hover.to_string().contains("without coercion"),"{hover}");
            let completion=ask(&index,&entry,&format!("{method}("),"textDocument/completion");assert!(completion.as_array().unwrap().iter().any(|row|row["label"]==method),"{completion}");
            let help=ask(&index,&entry,"2.0","textDocument/signatureHelp");assert!(help.to_string().contains("expected"),"{help}");
        }
    }
}

#[test]
fn close_to_matcher_optional_precision_help_and_completion_cover_both_views() {
    for view in ["", "not."] {
        let entry=format!("import {{expect}} from 'std:testing';function main(){{expect(0.1+0.2).{view}toBeCloseTo(0.3,5.0);}}");
        let uri="file:///tsrust-intelligence/main.tsr";let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.clone());
        let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
        let hover=ask(&index,&entry,"toBeCloseTo(","textDocument/hover");assert!(hover.to_string().contains("default 2"),"{hover}");
        let completion=ask(&index,&entry,"toBeCloseTo(","textDocument/completion");assert!(completion.as_array().unwrap().iter().any(|row|row["label"]=="toBeCloseTo"),"{completion}");
        let help=ask(&index,&entry,"5.0","textDocument/signatureHelp");assert!(help.to_string().contains("numDigits"),"{help}");assert!(help.to_string().contains("?"),"{help}");
    }
}

#[test]
fn length_matcher_completion_hover_and_parameter_help_cover_both_views() {
    for view in ["", "not."] {
        let entry=format!("import {{expect}} from 'std:testing';function main(){{expect('abc').{view}toHaveLength(3.0);}}");
        let uri="file:///tsrust-intelligence/main.tsr";let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.clone());
        let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
        let hover=ask(&index,&entry,"toHaveLength(","textDocument/hover");assert!(hover.to_string().contains("UTF-16"),"{hover}");
        let completion=ask(&index,&entry,"toHaveLength(","textDocument/completion");assert!(completion.as_array().unwrap().iter().any(|row|row["label"]=="toHaveLength"),"{completion}");
        let help=ask(&index,&entry,"3.0","textDocument/signatureHelp");assert!(help.to_string().contains("expected"),"{help}");
    }
}

#[test]
fn checked_project_labels_keep_local_definitions_after_linking() {
 let entry="import {count} from './dep.tsr';function main(){const outer=count();outer:while(true){break outer;}console.log(outer);}";
 let mut workspace=Workspace::default();let uri="file:///tsrust-intelligence/main.tsr";
 workspace.open(source_id(uri).unwrap(),1,entry.into());
 workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(),1,"export function count():int{return 4;}".into());
 let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
 let definition=ask(&index,entry,"outer;","textDocument/definition");
 assert_eq!(definition["range"]["start"]["character"],json!(entry.find("outer:while").unwrap()),"{definition}");
 let hover=ask(&index,entry,"outer;","textDocument/hover");assert!(hover.to_string().contains("label outer"),"{hover}");
 let value=ask(&index,entry,"outer);","textDocument/hover");assert!(value.to_string().contains("int"),"{value}");
 let labels=ask(&index,entry,"outer;","textDocument/references");assert_eq!(labels.as_array().unwrap().len(),2,"{labels}");
 let values=ask(&index,entry,"outer);","textDocument/references");assert_eq!(values.as_array().unwrap().len(),2,"{values}");
}

#[test]
fn checked_project_local_declarations_keep_source_names_and_imported_result_identity() {
    let entry="import {User as Person} from './dep.tsr';function main(){function identity(user:Person):Person{return user;} function report(){console.log(1);} const user=identity({name:'Pat'});report();console.log(user.name);}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let dependency_uri="file:///tsrust-intelligence/dep.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    workspace.open(source_id(dependency_uri).unwrap(),1,"export type User={name:string;};".into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let facts=checked.checked_local_functions();
    assert_eq!(facts.len(),2);
    let identity=facts.iter().find(|fact|fact.name=="identity").unwrap();
    assert_eq!(identity.declaration.source,source_id(uri).unwrap());
    assert_eq!(&entry[identity.declaration.span.start..identity.declaration.span.end],"identity");
    assert!(matches!(&identity.result, Some(tsrust_compiler::project::ProjectType::Named { declaration: Some(location), .. })
        if location.source == source_id(dependency_uri).unwrap()));
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"identity({name","textDocument/hover");
    let text=hover["contents"]["value"].as_str().unwrap();
    assert!(text.contains("Person"),"{text}");assert!(!text.contains("local#"),"{text}");
    let report=ask(&index,entry,"report();","textDocument/hover");
    assert!(report["contents"]["value"].as_str().unwrap().contains(": void"));
}

#[test]
fn selected_generic_factories_keep_concrete_calls_and_array_results() {
    for source in [
        include_str!("../../tsrust-compiler/tests/fixtures/generic-value-adapters/selected-factories.tsr"),
        include_str!("../../tsrust-compiler/tests/fixtures/generic-value-adapters/nested-selected-factories.tsr"),
    ] {
        let uri = "file:///tsrust-intelligence/main.tsr";
        let mut workspace = Workspace::default();
        workspace.open(source_id(uri).unwrap(), 1, source.into());
        let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
        let index = ProjectIndex::checked(&checked);
        let request = |method: &str, needle: &str| {
            let offset = source.find(needle).unwrap();
            let prefix = &source[..offset];
            let position = json!({"line":prefix.bytes().filter(|&c| c == b'\n').count(),
                "character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()});
            index.request(method, &json!({"position":position}), uri).unwrap().unwrap()
        };
        let help = request("textDocument/signatureHelp", "7), 7");
        assert!(help["signatures"][0]["label"].as_str().unwrap().contains("int"), "{help}");
        let hover = request("textDocument/hover", "mapped =");
        assert!(hover["contents"]["value"].as_str().unwrap().contains("int[]"), "{hover}");
        let definition = request("textDocument/definition", "right)(++arguments)");
        let offset = source.find("right(start").unwrap();
        let prefix = &source[..offset];
        assert_eq!(definition["range"]["start"], json!({"line":prefix.bytes().filter(|&c| c == b'\n').count(),
            "character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()}), "{definition}");
    }
}

#[test]
fn lexical_path_imported_editor_contracts() {
    for name in ["dirname", "extname", "isAbsolute", "normalize"] {
        let entry=format!("import {{{name} as operation}} from 'std:path';function main(){{const path='a.txt';const result=operation(path);}}");
        let mut workspace=Workspace::default();let uri=source_id("file:///tsrust-intelligence/main.tsr").unwrap();
        workspace.open(uri.clone(),1,entry.clone());
        let checked=workspace.snapshot().check(&uri).unwrap();let index=ProjectIndex::checked(&checked);
        let hover=ask(&index,&entry,"operation(path", "textDocument/hover");
        let docs=hover["contents"]["value"].as_str().unwrap();
        assert!(docs.contains(&format!("operation(path: string): {}",if name=="isAbsolute" {"boolean"} else {"string"})),"{hover}");
        assert!(docs.contains("Ownership:") && docs.contains("```tsrust"),"{hover}");
        let help=ask(&index,&entry,"path);", "textDocument/signatureHelp");
        assert_eq!(help["signatures"][0]["parameters"][0]["label"], "path: string");
        let completion=ask(&index,&entry,"operation(path", "textDocument/completion");
        assert!(completion.as_array().unwrap().iter().any(|item| item["label"]=="operation" && item["detail"].as_str().unwrap().contains("path: string")),"{completion}");
    }
}

#[test]
fn os_identity_imported_editor_contracts() {
    for name in ["platform", "arch", "endianness"] {
        let entry=format!("import {{{name} as operation}} from 'std:os';function main(){{const result=operation();}}");
        let mut workspace=Workspace::default();let uri=source_id("file:///tsrust-intelligence/main.tsr").unwrap();
        workspace.open(uri.clone(),1,entry.clone());
        let checked=workspace.snapshot().check(&uri).unwrap();let index=ProjectIndex::checked(&checked);
        let hover=ask(&index,&entry,"operation();", "textDocument/hover");
        let docs=hover["contents"]["value"].as_str().unwrap();
        assert!(docs.contains("operation(): string") && docs.contains("Ownership:") && docs.contains("```tsrust"),"{hover}");
        let help=ask(&index,&entry,");}", "textDocument/signatureHelp");
        assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),0,"{help}");
        let completion=ask(&index,&entry,"operation();", "textDocument/completion");
        assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="operation"),"{completion}");
    }
}

#[test]
fn path_join_imported_rest_editor_contract() {
    let entry="import {join as compose} from 'std:path';function main(){const stored=compose;const result=stored('tmp','report');const direct=compose('a','b','c');}";
    let mut workspace=Workspace::default();let uri=source_id("file:///tsrust-intelligence/main.tsr").unwrap();
    workspace.open(uri.clone(),1,entry.into());
    let checked=workspace.snapshot().check(&uri).unwrap();let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"compose('a'", "textDocument/hover");
    let docs=hover["contents"]["value"].as_str().unwrap();
    assert!(docs.contains("...paths") && docs.contains("string[]") && docs.contains("Ownership:") && docs.contains("```tsrust"),"{hover}");
    let help=ask(&index,entry,"'c');", "textDocument/signatureHelp");
    assert!(help["signatures"][0]["parameters"][0]["label"].as_str().unwrap().contains("...paths"),"{help}");
    assert_eq!(help["activeParameter"],0,"{help}");
    let completion=ask(&index,entry,"compose('a'", "textDocument/completion");
    assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="compose"),"{completion}");
    let stored=ask(&index,entry,"stored('tmp'", "textDocument/hover");
    assert!(stored["contents"]["value"].as_str().unwrap().contains("...paths"),"{stored}");
}

#[test]
fn path_relative_imported_borrowed_pair_editor_contract() {
    let entry="import {relative as between} from 'std:path';function main(){const saved=between;const direct=between('reports','reports/2026');const stored=saved('a','b');}";
    let mut workspace=Workspace::default();let uri=source_id("file:///tsrust-intelligence/main.tsr").unwrap();workspace.open(uri.clone(),1,entry.into());let checked=workspace.snapshot().check(&uri).unwrap();let index=ProjectIndex::checked(&checked);
    for site in ["between('reports'","saved('a'"] {
        let hover=ask(&index,entry,site,"textDocument/hover");let docs=hover["contents"]["value"].as_str().unwrap();assert!(docs.contains("from: string, to: string") && docs.contains("Ownership:") && docs.contains("path-relative") && docs.contains("```tsrust"),"{hover}");
        let completion=ask(&index,entry,site,"textDocument/completion");assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="between" && item["detail"].as_str().unwrap().contains("from: string, to: string")),"{completion}");
    }
    let help=ask(&index,entry,"'reports/2026');","textDocument/signatureHelp");assert_eq!(help["signatures"][0]["parameters"][0]["label"],"from: string");assert_eq!(help["signatures"][0]["parameters"][1]["label"],"to: string");assert_eq!(help["activeParameter"],1,"{help}");
}

#[test]
fn imported_assertions_and_native_aliases_keep_independent_factory_signatures() {
    let entry=r#"import {join} from "std:path";import {strictEqual} from "std:testing";function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function factory<A>(initial:A){let action=initial;return(restore:boolean)=>{const previous=action;if(restore){action=one;}else{action=two;}return previous;};}function wrap<A>(initial:A){let action=factory(initial);return(restore:boolean)=>{return action(restore);};}function main(){const saved=join;strictEqual(saved("a","b"),"a/b");const instances=[wrap(one)];for(let index=0;index<2;index+=1){instances.push(wrap(one));}const seed=instances.shift()!;const first=instances.shift()!;const second=instances.shift()!;const original=first(false);const independent=second(true);const changed=first(true);strictEqual(original<int>(1),1);strictEqual(independent<int>(2),2);strictEqual(changed<int>(3,9),9);}"#;
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let independent=ask(&index,entry,"2),2","textDocument/signatureHelp");
    assert!(independent.to_string().contains("value: int"),"{independent}");
    assert!(!independent.to_string().contains("other: int"),"{independent}");
    let changed=ask(&index,entry,"3,9","textDocument/signatureHelp");
    assert!(changed.to_string().contains("other: int"),"{changed}");
}

#[test]
fn catch_and_finally_declarations_keep_checked_navigation_and_signatures() {
    let entry=r#"import {strictEqual} from "std:testing";function main(){let total=0;try{const value=enter(2);function enter(value:int):int{total+=value;return total;}strictEqual(value,2);throw 3;}catch(error){const value=recover(4);function recover(value:int):int{total+=value;return total;}strictEqual(value,6);strictEqual(error===3,true);}finally{finish();function finish():void{total+=8;}}strictEqual(total,14);}"#;
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    for (needle,name) in [("recover(4)","recover"),("finish();","finish")] {
        let hover=ask(&index,entry,needle,"textDocument/hover");
        assert!(hover.to_string().contains(name),"{hover}");
        let definition=ask(&index,entry,needle,"textDocument/definition");
        assert_eq!(definition["uri"],uri,"{definition}");
    }
    let help=ask(&index,entry,"4);function recover","textDocument/signatureHelp");
    assert!(help["signatures"][0]["label"].as_str().unwrap().replace(' ',"").contains("recover(value:int):int"),"{help}");
}

#[test]
fn caught_values_keep_checked_generic_assertion_help() {
    let entry="import {strictEqual} from 'std:testing';function main(){try{strictEqual(1,1);throw 3;}catch(error){strictEqual(error,3);}}";
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,entry,"strictEqual(error","textDocument/hover");
    assert!(hover["contents"]["value"].as_str().unwrap().contains("strictEqual"),"{hover}");
    let help=ask(&index,entry,"3);}}","textDocument/signatureHelp");
    assert!(help["signatures"][0]["label"].as_str().unwrap().contains("expected"),"{help}");
}

#[test]
fn relative_native_aliases_keep_point_sensitive_factory_editor_contracts() {
    let entry=r#"import {relative as between} from "std:path";import {strictEqual} from "std:testing";
function one<T>(value:T):T{return value;}
function two<T>(value:T,other:T):T{return other;}
function factory<A>(initial:A){let action=initial;return(restore:boolean)=>{const previous=action;if(restore){action=one;}else{action=two;}return previous;};}
function wrap<A>(initial:A){let action=factory(initial);return(restore:boolean)=>{return action(restore);};}
function main(){const saved=between;const instances=[wrap(one)];strictEqual(between("reports","reports/2026"),"2026");for(let index=0;index<2;index+=1){instances.push(wrap(one));}const seed=instances.shift()!;const first=instances.shift()!;const second=instances.shift()!;const original=first(false);strictEqual(saved("reports","reports/2026"),"2026");const independent=second(true);const changed=first(true);strictEqual(original<int>(1),1);strictEqual(independent<int>(2),2);strictEqual(changed<int>(3,9),9);console.log("relative factory accepted");}"#;
    let entry=entry.replace('\n'," ");let entry=entry.as_str();
    let uri="file:///tsrust-intelligence/main.tsr";let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
    let independent=ask(&index,entry,"2),2","textDocument/signatureHelp");assert!(independent.to_string().contains("value: int"),"{independent}");assert!(!independent.to_string().contains("other: int"),"{independent}");
    let changed=ask(&index,entry,"3,9","textDocument/signatureHelp");assert!(changed.to_string().contains("other: int"),"{changed}");
    let hover=ask(&index,entry,"saved(\"reports\"","textDocument/hover");assert!(hover.to_string().contains("path-relative"),"{hover}");
}

#[test]
fn native_type_aliases_have_documented_hover_and_public_fields() {
    let entry = tsrust_compiler::format("import {Bytes as Buffer, ByteError as Failure} from 'std:bytes';function borrow(value:ReadOnly<Buffer>):void{} function report(error:ReadOnly<Failure>):void{console.log(error.message);} function main(){const error:Failure={code:'range',message:'bad'};console.log(error.offset);}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    for (mode, index) in [ProjectIndex::checked(&checked), ProjectIndex::new(&checked.graph)].into_iter().enumerate() {
        let hover = ask(&index, &entry, "Buffer>", "textDocument/hover");
        let docs = hover["contents"]["value"].as_str().unwrap_or("");
        assert!(docs.contains("type Buffer") && docs.contains("byte buffer") && !docs.contains("__tsrust_module_"), "{hover}");
        let hover = ask(&index, &entry, "Failure>", "textDocument/hover");
        assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("offset"), "{hover}");
        let completion = ask(&index, &entry, "message);", "textDocument/completion");
        let fields = completion.as_array().unwrap();
        for name in ["code", "message", "offset"] {
            assert!(fields.iter().any(|field| field["label"] == name), "mode {mode}: {completion}; receiver {}", ask(&index, &entry, "error.message", "textDocument/hover"));
        }
    }
    let stop = entry.find("error.message").unwrap() + "error.me".len();
    let editing = &entry[..stop];
    workspace.change(&source_id(uri).unwrap(), 2, editing.into());
    let input = workspace.snapshot();
    assert!(input.check(&source_id(uri).unwrap()).is_err());
    let graph = tsrust_compiler::project::load_project_headers(&input, &source_id(uri).unwrap().0).unwrap();
    let index = ProjectIndex::new(&graph);
    let completion = ask(&index, editing, "me", "textDocument/completion");
    assert!(completion.as_array().unwrap().iter().any(|field| field["label"] == "message"), "{completion}");
    let importing = "import { ByteError, } from 'std:bytes';function main(){}";
    workspace.change(&source_id(uri).unwrap(), 3, importing.into());
    let input = workspace.snapshot();
    let graph = tsrust_compiler::project::load_project_headers(&input, &source_id(uri).unwrap().0).unwrap();
    let index = ProjectIndex::new(&graph);
    let completion = ask(&index, importing, "} from", "textDocument/completion");
    assert!(completion.as_array().unwrap().iter().any(|item| item["label"] == "Bytes" && item["kind"] == 22), "{completion}");
}

#[test]
fn reexported_native_type_aliases_keep_docs_fields_and_definition() {
    let entry = tsrust_compiler::format("import { Buffer, Failure } from './dep.tsr';function borrow(value:ReadOnly<Buffer>):void{} function report(error:ReadOnly<Failure>):void{console.log(error.message);}function main(){const error:Failure={code:'range',message:'bad'};report(error);}").unwrap();
    let dependency = tsrust_compiler::format("import {Bytes, ByteError} from 'std:bytes';export type Buffer=Bytes;export type Failure=ByteError;").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(), 1, dependency);
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    for index in [ProjectIndex::checked(&checked), ProjectIndex::new(&checked.graph)] {
        let hover = ask(&index, &entry, "Buffer>", "textDocument/hover");
        assert!(hover["contents"]["value"].as_str().unwrap_or("").contains("byte buffer"), "{hover}");
        let completion = ask(&index, &entry, "message);", "textDocument/completion");
        assert!(completion.as_array().unwrap().iter().any(|field| field["label"] == "offset"), "{completion}");
        let definition = ask(&index, &entry, "Failure>", "textDocument/definition");
        assert_eq!(definition["uri"], "file:///tsrust-intelligence/dep.tsr", "{definition}");
    }
}

#[test]
fn native_byte_calls_document_recursive_signatures_and_hidden_result_payloads() {
    let entry = tsrust_compiler::format("import {fromArray as build, length} from 'std:bytes';function main(){match(build([65])){Ok(bytes)=>{console.log(length(bytes));},Err(error)=>{console.log(error.message);}}}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    for (mode, index) in [ProjectIndex::checked(&checked), ProjectIndex::new(&checked.graph)].into_iter().enumerate() {
        let hover = ask(&index, &entry, "build([", "textDocument/hover");
        let docs = hover["contents"]["value"].as_str().unwrap_or("");
        assert!(docs.contains("ReadOnly<int[]>") && docs.contains("Result<Bytes, ByteError>") && docs.contains("byte-range"), "{hover}");
        assert!(!docs.contains("__tsrust_private_"), "{hover}");
        let hover = ask(&index, &entry, "length(bytes)", "textDocument/hover");
        assert!(hover.to_string().contains("number of bytes"), "{hover}");
        if mode == 0 {
        let hover = ask(&index, &entry, "message);", "textDocument/hover");
        assert!(hover.to_string().contains("Explains the failure"), "mode {mode}: {hover}; receiver {}", ask(&index, &entry, "error.message", "textDocument/hover"));
        let completion = ask(&index, &entry, "message);", "textDocument/completion");
        assert!(completion.as_array().unwrap().iter().any(|field| field["label"] == "offset"), "{completion}");
        }
        let help = ask(&index, &entry, "[65]", "textDocument/signatureHelp");
        assert!(help.to_string().contains("0..255"), "{help}");
    }
}

#[test]
fn structured_exported_aliases_keep_hidden_native_fields_without_name_collisions() {
    let entry = tsrust_compiler::format("import {Failures as Problems, Outcome as Batch} from './dep.tsr';type ByteError={fake:int;};function accept(result:ReadOnly<Batch>):void{}function report(errors:ReadOnly<Problems>):void{console.log(errors[0].message);}function main(){report([{code:'range',message:'bad'}]);}").unwrap();
    let dependency = tsrust_compiler::format("import {ByteError as Failure} from 'std:bytes';export type Failures=Failure[];export type Outcome=Result<Failure[],int>;").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    workspace.open(source_id("file:///tsrust-intelligence/dep.tsr").unwrap(), 1, dependency);
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    for (mode, index) in [ProjectIndex::checked(&checked), ProjectIndex::new(&checked.graph)].into_iter().enumerate() {
        let hover = ask(&index, &entry, "Batch>", "textDocument/hover");
        assert!(hover.to_string().replace(' ', "").contains("Result<ByteError[],int>"), "{hover}");
        assert!(!hover.to_string().contains("__tsrust_private_"), "{hover}");
        let globals = ask(&index, &entry, "report([{", "textDocument/completion");
        assert!(!globals.to_string().contains("__tsrust_private_"), "private type leaked in global completion");
        let hover = ask(&index, &entry, "Problems>", "textDocument/hover");
        assert!(hover.to_string().contains("type Problems"), "{hover}");
        assert!(!hover.to_string().contains("__tsrust_private_"), "{hover}");
        let hover = ask(&index, &entry, "message);", "textDocument/hover");
        assert!(hover.to_string().contains("Explains the failure"), "mode {mode}: {hover}; receiver {}", ask(&index, &entry, "errors[0]", "textDocument/hover"));
        let completion = ask(&index, &entry, "message);", "textDocument/completion");
        let fields = completion.as_array().unwrap();
        assert!(fields.iter().any(|field| field["label"] == "offset"), "{completion}");
        assert!(!fields.iter().any(|field| field["label"] == "fake"), "{completion}");
        let definition = ask(&index, &entry, "Problems>", "textDocument/definition");
        assert_eq!(definition["uri"], "file:///tsrust-intelligence/dep.tsr", "{definition}");
    }
}

#[test]
fn native_byte_access_stored_values_keep_borrow_modes_and_result_help() {
    let entry = tsrust_compiler::format("import {fromArray,at as inspect,set as write} from 'std:bytes';function main(){match(fromArray([65])){Ok(bytes)=>{const read=inspect;const update=write;match(update(bytes,0,90)){Ok(_)=>{match(read(bytes,0)){Ok(value)=>{console.log(value);},Err(error)=>{console.log(error.message);}}},Err(error)=>{console.log(error.message);}}},Err(error)=>{console.log(error.message);}}}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let hover = ask(&index, &entry, "update(bytes", "textDocument/hover");
    let docs = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(docs.contains("Mutable<Bytes>") && docs.contains("Result<void, ByteError>") && docs.contains("leaves all bytes unchanged"), "{hover}");
    assert!(!docs.contains("__tsrust_private_"), "{hover}");
    let hover = ask(&index, &entry, "read(bytes", "textDocument/hover");
    assert!(hover.to_string().contains("ReadOnly<Bytes>") && hover.to_string().contains("Result<int, ByteError>"), "{hover}");
    let help = ask(&index, &entry, "90)", "textDocument/signatureHelp");
    assert_eq!(help["activeParameter"], 2, "{help}");
    assert!(help.to_string().contains("0..255"), "{help}");
}

#[test]
fn native_byte_growth_stored_values_keep_optional_fill_help() {
    let entry = tsrust_compiler::format("import {alloc,resize} from 'std:bytes';function main(){const create=alloc;match(create(1)){Ok(bytes)=>{const change=resize;match(change(bytes,3,65)){Ok(_)=>{},Err(error)=>{console.log(error.message);}}},Err(error)=>{console.log(error.message);}}}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let hover = ask(&index, &entry, "create(1)", "textDocument/hover");
    assert!(hover.to_string().contains("fill?: int") && hover.to_string().contains("defaults to 0"), "{hover}");
    let hover = ask(&index, &entry, "change(bytes", "textDocument/hover");
    assert!(hover.to_string().contains("Mutable<Bytes>") && hover.to_string().contains("Every failure leaves length and contents unchanged"), "{hover}");
    assert!(!hover.to_string().contains("__tsrust_private_"), "{hover}");
    let help = ask(&index, &entry, "65)", "textDocument/signatureHelp");
    assert_eq!(help["activeParameter"], 2, "{help}");
    assert!(help.to_string().contains("fill?: int") && help.to_string().contains("0..255"), "{help}");
}

#[test]
fn native_byte_append_copy_stored_values_document_independence_and_bounds() {
    let entry = tsrust_compiler::format("import {fromArray,append,copy} from 'std:bytes';function main(){match(fromArray([65])){Ok(bytes)=>{const add=append;const duplicate=copy;match(add(bytes,[255])){Ok(_)=>{match(duplicate(bytes,0,2)){Ok(part)=>{},Err(error)=>{console.log(error.message);}}},Err(error)=>{console.log(error.message);}}},Err(error)=>{console.log(error.message);}}}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let hover = ask(&index, &entry, "add(bytes", "textDocument/hover");
    assert!(hover.to_string().contains("Mutable<Bytes>") && hover.to_string().contains("ReadOnly<int[]>") && hover.to_string().contains("original length and contents"), "{hover}");
    let hover = ask(&index, &entry, "duplicate(bytes", "textDocument/hover");
    assert!(hover.to_string().contains("independent owned buffer") && hover.to_string().contains("ReadOnly<Bytes>"), "{hover}");
    assert!(!hover.to_string().contains("__tsrust_private_"), "{hover}");
    let help = ask(&index, &entry, "2))", "textDocument/signatureHelp");
    assert_eq!(help["activeParameter"], 2, "{help}");
    assert!(help.to_string().contains("Exclusive ending byte index"), "{help}");
}

#[test]
fn native_byte_clone_help_describes_independent_storage() {
    let entry = tsrust_compiler::format("import {alloc,length} from 'std:bytes';function main(){match(alloc(1)){Ok(bytes)=>{const other=clone(bytes);console.log(length(other));},Err(error)=>{console.log(error.message);}}}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let hover = ask(&index, &entry, "clone(bytes)", "textDocument/hover");
    assert!(hover.to_string().contains("independent byte storage") && hover.to_string().contains("allocation failure") && hover.to_string().contains("65"), "{hover}");
    let help = ask(&index, &entry, "bytes);", "textDocument/signatureHelp");
    assert!(help.to_string().contains("value: T") && help.to_string().contains("without consuming"), "{help}");
}

#[test]
fn native_utf8_stored_values_document_strict_offsets_and_byte_counts() {
    let entry=tsrust_compiler::format("import {fromUtf8,toUtf8} from 'std:bytes';function main(){const encode=fromUtf8;const decode=toUtf8;match(encode('é😀')){Ok(bytes)=>{match(decode(bytes)){Ok(text)=>{console.log(text);},Err(error)=>{console.log(error.offset);}}},Err(error)=>{console.log(error.message);}}}").unwrap();
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.clone());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,&entry,"encode(","textDocument/hover");
    assert!(hover.to_string().contains("Result<Bytes, ByteError>")&&hover.to_string().contains("NUL bytes"),"{hover}");
    let hover=ask(&index,&entry,"decode(bytes)","textDocument/hover");
    assert!(hover.to_string().contains("Result<string, ByteError>")&&hover.to_string().contains("first invalid sequence start"),"{hover}");
    assert!(!hover.to_string().contains("__tsrust_private_"),"{hover}");
    let help=ask(&index,&entry,"bytes))","textDocument/signatureHelp");
    assert!(help.to_string().contains("ReadOnly<Bytes>")&&help.to_string().contains("invalid bytes are never replaced"),"{help}");
}

#[test]
fn byte_view_factories_types_and_shared_observers_have_editor_help() {
    let entry = tsrust_compiler::format("import {ByteView, fromUtf8, view, viewMut, length, set} from 'std:bytes';function size(part:ReadOnly<ByteView>):int{return length(part);}function main(){match(fromUtf8(\"hello\")){Ok(bytes)=>{match(view(bytes,1,4)){Ok(part)=>{console.log(size(part));},Err(error)=>{console.log(error.message);}}match(viewMut(bytes,0,1)){Ok(edit)=>{match(set(edit,0,72)){Ok(_)=>{},Err(error)=>{console.log(error.message);}}},Err(error)=>{console.log(error.message);}}},Err(error)=>{console.log(error.message);}}}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    for index in [ProjectIndex::checked(&checked), ProjectIndex::new(&checked.graph)] {
        let hover = ask(&index, &entry, "view(bytes", "textDocument/hover");
        let docs = hover["contents"]["value"].as_str().unwrap_or("");
        assert!(docs.contains("Result<ByteView, ByteError>") && docs.contains("without copying") && docs.contains("Bytes | ByteView | ByteViewMut"), "{hover}");
        assert!(!docs.contains("__tsrust"), "{hover}");
        let hover = ask(&index, &entry, "viewMut(bytes", "textDocument/hover");
        assert!(hover.to_string().contains("exclusive") && hover.to_string().contains("Mutable<Bytes | ByteViewMut>"), "{hover}");
        let hover = ask(&index, &entry, "ByteView>", "textDocument/hover");
        assert!(hover.to_string().contains("readonly window"), "{hover}");
        let help = ask(&index, &entry, "1, 4)", "textDocument/signatureHelp");
        assert!(help.to_string().contains("Inclusive starting byte index"), "{help}");
        let hover = ask(&index, &entry, "length(part)", "textDocument/hover");
        assert!(hover.to_string().contains("ByteView"), "{hover}");
        let hover = ask(&index, &entry, "set(edit", "textDocument/hover");
        assert!(hover.to_string().contains("ByteViewMut"), "{hover}");
    }
    let importing = "import { vi } from 'std:bytes';function main(){}";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, importing.into());
    let graph = tsrust_compiler::project::load_project_headers(&workspace.snapshot(), &source_id(uri).unwrap().0).unwrap();
    let completion = ask(&ProjectIndex::new(&graph), importing, " }", "textDocument/completion");
    let labels = completion.as_array().unwrap().iter().filter_map(|item| item["label"].as_str()).collect::<Vec<_>>();
    assert_eq!(labels, ["view", "viewMut"], "{completion}");
}

#[test]
fn native_deque_callback_payload_keeps_field_signature() {
    let entry = tsrust_compiler::format("import {Deque as Queue} from 'std:collections';type Reader={read:ReadOnly<Function<int>>;};function main(){const text='value';const values=new Queue<Reader>();values.pushBack({read:()=>text.length});for(const item of values){console.log(item.read());}}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    for index in [ProjectIndex::checked(&checked), ProjectIndex::new(&checked.graph)] {
        let hover = ask(&index, &entry, "read());", "textDocument/hover");
        assert!(hover.to_string().contains("Function<int>"), "{hover}");
    }
}

#[test]
fn native_heap_iteration_has_element_hover() {
    let entry = tsrust_compiler::format("import {Heap as Queue} from 'std:collections';function main(){const values=new Queue<string>();values.push('a');for(const item of values){console.log(item);}}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    for index in [ProjectIndex::checked(&checked), ProjectIndex::new(&checked.graph)] {
        let hover = ask(&index, &entry, "item);", "textDocument/hover");
        assert!(hover.to_string().contains("item: string"), "{hover}");
    }
}

#[test]
fn native_deque_iteration_has_element_hover() {
    let entry = tsrust_compiler::format("import {Deque as Queue} from 'std:collections';function main(){const values=new Queue<string>();values.pushBack('a');for(const item of values){console.log(item);}}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    for index in [ProjectIndex::checked(&checked), ProjectIndex::new(&checked.graph)] {
        let hover = ask(&index, &entry, "item);", "textDocument/hover");
        assert!(hover.to_string().contains("item: string"), "{hover}");
    }
}

#[test]
fn native_heap_alias_methods_have_typed_editor_help() {
    let entry = tsrust_compiler::format("import {Heap as Queue} from 'std:collections';function inspect(input:ReadOnly<Queue<int>>){return input.peek();}function main(){const values=new Queue<int>();values.push(7);const first=values.peek();const count=values.size;console.log(count,first);}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    for index in [ProjectIndex::checked(&checked), ProjectIndex::new(&checked.graph)] {
        let borrowed = ask(&index, &entry, "peek();", "textDocument/hover");
        assert!(borrowed.to_string().contains("peek(): int | undefined"), "{borrowed}");
        let hover = ask(&index, &entry, "push(7)", "textDocument/hover");
        assert!(hover.to_string().contains("push(value: int): void"), "{hover}");
        let help = ask(&index, &entry, "7);", "textDocument/signatureHelp");
        assert!(help.to_string().contains("value: int"), "{help}");
        let items = ask(&index, &entry, "peek();", "textDocument/completion");
        assert!(items.to_string().contains("pop"), "{items}");
        let count = ask(&index, &entry, "count,", "textDocument/hover");
        assert!(count.to_string().contains("count: int"), "{count}");
        let first = ask(&index, &entry, "first);", "textDocument/hover");
        assert!(first.to_string().contains("int | undefined"), "{first}");
    }
}

#[test]
fn native_deque_alias_methods_have_typed_editor_help() {
    let entry = tsrust_compiler::format("import {Deque as Queue} from 'std:collections';function inspect(input:ReadOnly<Queue<int>>){return input.back();}function main(){const values=new Queue<int>();values.pushBack(7);const first=values.front();const count=values.size;console.log(count,first);}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    for index in [ProjectIndex::checked(&checked), ProjectIndex::new(&checked.graph)] {
        let borrowed = ask(&index, &entry, "back();", "textDocument/hover");
        assert!(borrowed.to_string().contains("back(): int | undefined"), "{borrowed}");
        let hover = ask(&index, &entry, "pushBack(7)", "textDocument/hover");
        assert!(hover.to_string().contains("pushBack(value: int): void"), "{hover}");
        let help = ask(&index, &entry, "7);", "textDocument/signatureHelp");
        assert!(help.to_string().contains("value: int"), "{help}");
        let items = ask(&index, &entry, "front();", "textDocument/completion");
        assert!(items.to_string().contains("popFront"), "{items}");
        let count = ask(&index, &entry, "count,", "textDocument/hover");
        assert!(count.to_string().contains("count: int"), "{count}");
        let first = ask(&index, &entry, "first);", "textDocument/hover");
        assert!(first.to_string().contains("int | undefined"), "{first}");
    }
}

#[test]
fn native_deque_constructor_alias_has_generic_hover() {
    let entry = tsrust_compiler::format("import {Deque as Queue} from 'std:collections';function accept(value:Queue<int>){}function main(){const values=new Queue<int>();accept(values);}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    for index in [ProjectIndex::checked(&checked), ProjectIndex::new(&checked.graph)] {
        let hover = ask(&index, &entry, "Queue<int>", "textDocument/hover");
        let docs = hover["contents"]["value"].as_str().unwrap_or("");
        assert!(docs.contains("type Queue<T>") && docs.contains("empty queue"), "{hover}");
        let help = ask(&index, &entry, ");\n  accept", "textDocument/signatureHelp");
        assert!(help.to_string().contains("new Queue<int>(): Queue<int>"), "{help}");
        let value = ask(&index, &entry, "values);", "textDocument/hover");
        assert!(value.to_string().contains("Queue<int>"), "{value}");
    }
}

#[test]
fn native_collection_spread_has_element_hover() {
    for (kind,push) in [("Deque","pushBack"),("Heap","push")] {
        let entry=tsrust_compiler::format(&format!("import {{{kind} as Queue}} from 'std:collections';function sum(...items:int[]):int{{let total=0;for(const item of items){{total+=item;}}return total;}}function main(){{const values=new Queue<int>();values.{push}(1);const copied=[...values];console.log(sum(...values));for(const item of copied){{console.log(item);}}}}" )).unwrap();
        let uri="file:///tsrust-intelligence/main.tsr";
        let mut workspace=Workspace::default();
        workspace.open(source_id(uri).unwrap(),1,entry.clone());
        let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
        for (mode,index) in [ProjectIndex::checked(&checked),ProjectIndex::new(&checked.graph)].into_iter().enumerate() {
            let hover=ask(&index,&entry,"item);","textDocument/hover");
            assert!(hover.to_string().contains("item: int"),"{kind} index {mode}: {hover}");
        }
    }
}

#[test]
fn native_deque_nested_index_has_element_hover() {
 let entry=tsrust_compiler::format("import {Deque} from 'std:collections';function main(){const values=new Deque<int[]>();values.pushBack([4]);const first=values.front()![0];console.log(first);}").unwrap();
 let uri="file:///tsrust-intelligence/main.tsr";let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.clone());let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
 for index in [ProjectIndex::checked(&checked),ProjectIndex::new(&checked.graph)] {let hover=ask(&index,&entry,"first);","textDocument/hover");assert!(hover.to_string().contains("first: int"),"{hover}");}
}

#[test]
fn native_ordered_set_alias_has_typed_editor_help() {
 let entry=tsrust_compiler::format("import {OrderedSet as Tree} from 'std:collections';function main(){const values=new Tree<int>();values.add(7);const found=values.has(7);const count=values.size;for(const item of values){console.log(item);}console.log(count,found);}").unwrap();
 let uri="file:///tsrust-intelligence/main.tsr";let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.clone());let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
 for index in [ProjectIndex::checked(&checked),ProjectIndex::new(&checked.graph)]{
  for (needle,expected) in [("add(7)","add(value: int): void"),("has(7)","has(value: int): boolean"),("item);","item: int"),("found);","found: boolean"),("count,","count: int")]{let hover=ask(&index,&entry,needle,"textDocument/hover");assert!(hover.to_string().contains(expected),"{hover}");}
  let help=ask(&index,&entry,"7);","textDocument/signatureHelp");assert!(help.to_string().contains("value: int"),"{help}");
  let items=ask(&index,&entry,"has(7)","textDocument/completion");assert!(items.to_string().contains("delete"),"{items}");
 }
}

#[test]
fn native_ordered_map_alias_has_typed_editor_help() {
 let entry=tsrust_compiler::format("import {OrderedMap as Tree} from 'std:collections';function main(){const values=new Tree<string,int>();values.set('a',7);const found=values.get('a');const count=values.size;for(const item of values){console.log(item[0]);}console.log(count,found);}").unwrap();
 let uri="file:///tsrust-intelligence/main.tsr";let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.clone());let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
 for index in [ProjectIndex::checked(&checked),ProjectIndex::new(&checked.graph)]{
  for (needle,expected) in [("set('a'","set(key: string, value: int): void"),("get('a'","get(key: string): int | undefined"),("item[0]","item: [string, int]"),("count,","count: int")]{let hover=ask(&index,&entry,needle,"textDocument/hover");assert!(hover.to_string().contains(expected),"{hover}");}
  let help=ask(&index,&entry,"7);","textDocument/signatureHelp");assert!(help.to_string().contains("value: int"),"{help}");
  let items=ask(&index,&entry,"get('a'","textDocument/completion");assert!(items.to_string().contains("delete"),"{items}");
 }
}

#[test]
fn file_write_signature_help_documents_borrowed_views_and_partial_progress() {
    let entry = tsrust_compiler::format("import {writeAll,File} from 'std:fs';import {ByteView} from 'std:bytes';function save(file:Mutable<File>,bytes:ReadOnly<ByteView>){writeAll(file,bytes);}function main(){}").unwrap();
    let uri = "file:///tsrust-intelligence/main.tsr";
    let mut workspace = Workspace::default();
    workspace.open(source_id(uri).unwrap(), 1, entry.clone());
    let checked = workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index = ProjectIndex::checked(&checked);
    let help = ask(&index, &entry, "file, bytes", "textDocument/signatureHelp");
    for phrase in ["ByteView", "ByteViewMut", "67108864"] { assert!(help.to_string().contains(phrase), "{help}"); }
    let hover = ask(&index, &entry, "writeAll(file", "textDocument/hover");
    assert!(hover.to_string().contains("not rolled back"), "{hover}");
    assert!(!hover.to_string().contains("__tsrust_private_"), "{hover}");
}

#[test]
fn full_file_helpers_document_limits_defaults_and_view_inputs() {
 let entry=tsrust_compiler::format("import {readFile,writeFile} from 'std:fs';import {ByteView} from 'std:bytes';function save(bytes:ReadOnly<ByteView>){writeFile('output.bin',bytes);readFile('input.bin',4);}function main(){}").unwrap();
 let uri="file:///tsrust-intelligence/main.tsr";let mut workspace=Workspace::default();
 workspace.open(source_id(uri).unwrap(),1,entry.clone());
 let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
 let help=ask(&index,&entry,"4);","textDocument/signatureHelp");
 assert!(help.to_string().contains("67108864")&&help.to_string().contains("maxBytes"),"{help}");
 let hover=ask(&index,&entry,"writeFile(","textDocument/hover");
 assert!(hover.to_string().contains("ByteViewMut")&&hover.to_string().contains("before the destination changes"),"{hover}");
}

#[test]
fn buffered_file_help_documents_transfer_capacity_and_retry() {
 let entry=tsrust_compiler::format("import {File,BufferedWriter,bufferWriter,flush,writeAll} from 'std:fs';import {ByteView} from 'std:bytes';function make(file:File){bufferWriter(file,8);}function save(writer:Mutable<BufferedWriter>,bytes:ReadOnly<ByteView>){writeAll(writer,bytes);flush(writer);}").unwrap();
 let uri="file:///tsrust-intelligence/main.tsr";let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.clone());
 let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
 let help=ask(&index,&entry,"8);","textDocument/signatureHelp");
 for phrase in ["8192","1048576","Consumed"] {assert!(help.to_string().contains(phrase),"{help}");}
 let help=ask(&index,&entry,"writer, bytes","textDocument/signatureHelp");assert!(help.to_string().contains("BufferedWriter"),"{help}");
 let hover=ask(&index,&entry,"flush(writer","textDocument/hover");assert!(hover.to_string().contains("unwritten suffix"),"{hover}");
}

#[test]
fn default_function_imports_keep_alias_help_definitions_and_completion() {
 for dependency in ["/** Adds one. */ export default function add(value:int):int{return value+1;}","/** Adds one. */ export default function(value:int):int{return value+1;}","/** Adds one. */ export function add(value:int):int{return value+1;} export default add;"] {
  let entry="import hello from './dep.tsr';function main(){hello(3);}";
  let uri="file:///tsrust-intelligence/main.tsr";let dep="file:///tsrust-intelligence/dep.tsr";
  let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.into());workspace.open(source_id(dep).unwrap(),1,dependency.into());
  let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
  let hover=ask(&index,entry,"hello(3)","textDocument/hover");assert!(hover.to_string().contains("hello")&&hover.to_string().contains("Adds one"),"{hover}");assert!(!hover.to_string().contains("__d0"),"{hover}");
  let definition=ask(&index,entry,"hello(3)","textDocument/definition");assert_eq!(definition["uri"],dep,"{definition}");
  let help=ask(&index,entry,"3)","textDocument/signatureHelp");assert!(help.to_string().contains("hello"),"{help}");
  let importing="import { default as hello } from './dep.tsr';function main(){}";
  workspace.open(source_id(uri).unwrap(),2,importing.into());let graph=tsrust_compiler::project::load_project_headers(&workspace.snapshot(),&source_id(uri).unwrap().0).unwrap();
  let completion=ask(&ProjectIndex::new(&graph),importing,"default as","textDocument/completion");assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="default"),"{completion}");
 }
}

#[test]
fn environment_lookup_editor_contract() {
 let entry="import {get as lookup,EnvError} from 'std:env';function main(){const result=lookup('HOME');}";
 let uri=source_id("file:///tsrust-intelligence/main.tsr").unwrap();let mut workspace=Workspace::default();workspace.open(uri.clone(),1,entry.into());
 let checked=workspace.snapshot().check(&uri).unwrap();let index=ProjectIndex::checked(&checked);
 let hover=ask(&index,entry,"lookup('HOME'","textDocument/hover");let docs=hover["contents"]["value"].as_str().unwrap();
 for text in ["EnvError","undefined","InvalidName","NotUnicode","Ownership:","```tsrust"] {assert!(docs.contains(text),"{hover}");}
 let help=ask(&index,entry,"'HOME'","textDocument/signatureHelp");assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),1,"{help}");
 let importing="import { } from 'std:env';";let mut workspace=Workspace::default();workspace.open(uri.clone(),1,importing.into());
 let graph=tsrust_compiler::project::load_project_headers(&workspace.snapshot(),&uri.0).unwrap();let completion=ask(&ProjectIndex::new(&graph),importing," }","textDocument/completion");
 for name in ["get","EnvError"] {assert!(completion.as_array().unwrap().iter().any(|item|item["label"]==name),"{completion}");}
}

#[test]
fn monotonic_clock_editor_contract() {
 let entry="import {monotonicNow as now} from 'std:time';function main(){const value=now();}";
 let uri=source_id("file:///tsrust-intelligence/main.tsr").unwrap();let mut workspace=Workspace::default();workspace.open(uri.clone(),1,entry.into());
 let checked=workspace.snapshot().check(&uri).unwrap();let index=ProjectIndex::checked(&checked);
 let hover=ask(&index,entry,"now()","textDocument/hover");let docs=hover["contents"]["value"].as_str().unwrap();
 for text in ["number","milliseconds","process","Ownership:","```tsrust"] {assert!(docs.contains(text),"{hover}");}
 let help=ask(&index,entry,");}","textDocument/signatureHelp");assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),0,"{help}");
 let importing="import { } from 'std:time';";let mut workspace=Workspace::default();workspace.open(uri.clone(),1,importing.into());
 let graph=tsrust_compiler::project::load_project_headers(&workspace.snapshot(),&uri.0).unwrap();let completion=ask(&ProjectIndex::new(&graph),importing," }","textDocument/completion");
 assert!(completion.as_array().unwrap().iter().any(|item|item["label"]=="monotonicNow"),"{completion}");
}

#[test]
fn json_editor_contract() {
 let entry="import {parse as decode,JsonValue,JsonLimits} from 'std:json';function main(){const result=decode('null');}";
 let uri=source_id("file:///tsrust-intelligence/main.tsr").unwrap();let mut workspace=Workspace::default();workspace.open(uri.clone(),1,entry.into());
 let checked=workspace.snapshot().check(&uri).unwrap();let index=ProjectIndex::checked(&checked);
 let hover=ask(&index,entry,"decode('null'","textDocument/hover");let docs=hover["contents"]["value"].as_str().unwrap();
 for text in ["JsonValue","JsonError","JsonLimits","16777216","Ownership:","```tsrust"] {assert!(docs.contains(text),"{hover}");}
 let help=ask(&index,entry,"'null'","textDocument/signatureHelp");assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),2,"{help}");
 let importing="import { } from 'std:json';";let mut workspace=Workspace::default();workspace.open(uri.clone(),1,importing.into());
 let graph=tsrust_compiler::project::load_project_headers(&workspace.snapshot(),&uri.0).unwrap();let completion=ask(&ProjectIndex::new(&graph),importing," }","textDocument/completion");
 for name in ["parse","parseJsonc","stringify","kind","JsonValue","JsonError","JsonLimits","openJsonl","readJsonl","closeJsonl","JsonlReader","JsonlError"] {assert!(completion.as_array().unwrap().iter().any(|item|item["label"]==name),"{completion}");}
}

#[test]
fn jsonc_editor_contract() {
 let entry="import {parseJsonc as decode} from 'std:json';function main(){const result=decode('null',true);}";
 let uri=source_id("file:///tsrust-intelligence/main.tsr").unwrap();let mut workspace=Workspace::default();workspace.open(uri.clone(),1,entry.into());
 let checked=workspace.snapshot().check(&uri).unwrap();let index=ProjectIndex::checked(&checked);
 let hover=ask(&index,entry,"decode('null'","textDocument/hover");let docs=hover["contents"]["value"].as_str().unwrap();
 for text in ["allowTrailingComma","unterminated-comment","original UTF-8","Ownership:","```tsrust"] {assert!(docs.contains(text),"{hover}");}
 let help=ask(&index,entry,"'null'","textDocument/signatureHelp");assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),3,"{help}");
}

#[test]
fn imported_documentation_preserves_multiline_tags_and_parameter_help() {
    let entry=tsrust_compiler::format("import {greet as greeting} from './greeting.tsr';function main(){console.log(greeting(\"Pat\"));}").unwrap();
    let dependency=tsrust_compiler::format("/** First line.\n * Second line.\n * @param name - Person to greet.\n *   Continuation.\n * @returns Greeting text.\n */\nexport function greet(name:string):string{return name.trim();}").unwrap();
    assert_eq!(tsrust_compiler::format(&entry).unwrap(),entry);
    assert_eq!(tsrust_compiler::format(&dependency).unwrap(),dependency);
    let uri="file:///tsrust-intelligence/main.tsr";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.clone());
    workspace.open(source_id("file:///tsrust-intelligence/greeting.tsr").unwrap(),1,dependency);
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,&entry,"greeting(\"Pat\")","textDocument/hover");
    let docs=hover["contents"]["value"].as_str().unwrap();
    assert!(docs.contains("First line.  \nSecond line.") && docs.contains("**Returns**"),"{hover}");
    let help=ask(&index,&entry,"\"Pat\"","textDocument/signatureHelp");
    assert!(help["signatures"][0]["parameters"][0]["documentation"]["value"].as_str().unwrap().contains("Continuation."),"{help}");
}

#[test]
fn jsonl_editor_contract_exposes_ownership_limits_and_error_locations() {
 let entry="import {openJsonl as open,JsonlReader,readJsonl,closeJsonl} from 'std:json';function main(){const result=open('records.jsonl');match(result){Ok(reader)=>{closeJsonl(reader);},Err(error)=>{console.log(error.line);}}}";
 let uri="file:///tsrust-intelligence/main.tsr";let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,entry.into());
 let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
 let hover=ask(&index,entry,"open('records","textDocument/hover");let docs=hover["contents"]["value"].as_str().unwrap();
 for text in ["JsonlReader","JsonlError","1048576","Ownership:","```tsrust"]{assert!(docs.contains(text),"{hover}");}
 let field=ask(&index,entry,"line);","textDocument/hover");assert!(field.to_string().contains("One-based JSON Lines record"),"{field}");
 let help=ask(&index,entry,"'records.jsonl'","textDocument/signatureHelp");assert_eq!(help["signatures"][0]["parameters"].as_array().unwrap().len(),3);
}

#[test]
fn configured_alias_default_import_has_checked_completion_hover_and_navigation() {
 let entry=tsrust_compiler::format("import greet from '@/components';function main(){console.log(greet('Pat'));}").unwrap();
 let dependency=tsrust_compiler::format("/** Builds a greeting. */ function greeting(name:string):string{return 'Hello '+name;} export default greeting;").unwrap();
 let uri="file:///tsrust-intelligence/main.tsr";
 let target="file:///tsrust-intelligence/src/components/index.tsr";
 let mut workspace=Workspace::default();
 workspace.open(source_id("file:///tsrust-intelligence/tsrust.config.json").unwrap(),1,r#"{"paths":{"@/*":"./src/*"}}"#.into());
 workspace.open(source_id(uri).unwrap(),1,entry.clone());workspace.open(source_id(target).unwrap(),1,dependency);
 let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();let index=ProjectIndex::checked(&checked);
 let hover=ask(&index,&entry,"greet(","textDocument/hover");assert!(hover.to_string().contains("string")&&hover.to_string().contains("Builds a greeting"),"{hover}");
 let definition=ask(&index,&entry,"greet(","textDocument/definition");assert_eq!(definition["uri"],target,"{definition}");
 let completion=ask(&index,&entry,"greet(","textDocument/completion");assert!(completion.as_array().is_some_and(|items|items.iter().any(|item|item["label"]=="greet")),"{completion}");
}

#[test]
fn typed_json_import_fields_have_hover_completion_and_source_navigation() {
    let entry=tsrust_compiler::format("import settings from './settings.jsonc';function main(){console.log(settings.server.host);}").unwrap();
    let uri="file:///tsrust-intelligence/main.tsr";
    let target="file:///tsrust-intelligence/settings.jsonc";
    let mut workspace=Workspace::default();
    workspace.open(source_id(uri).unwrap(),1,entry.clone());
    workspace.open(source_id(target).unwrap(),1,"{\n  // server configuration\n  \"server\": {\"host\": \"local\", \"port\": 8080},\n}".into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let hover=ask(&index,&entry,"host);","textDocument/hover");
    assert!(hover.to_string().contains("string"),"{hover}");
    let completion=ask(&index,&entry,"host);","textDocument/completion");
    assert!(completion.as_array().is_some_and(|items|items.iter().any(|item|item["label"]=="host")),"{completion}");
    let definition=ask(&index,&entry,"host);","textDocument/definition");
    assert_eq!(definition["uri"],target,"{definition}");
    assert_eq!(definition["range"]["start"]["line"],2,"{definition}");
    workspace.change(&source_id(target).unwrap(),2,"{\"server\":{\"host\":42}}".into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let hover=ask(&ProjectIndex::checked(&checked),&entry,"host);","textDocument/hover");
    assert!(hover.to_string().contains("int"),"{hover}");
}

#[test]
fn typed_json_import_completion_survives_incomplete_source() {
    let root="file:///tsrust-intelligence/main.tsr";
    let entry="import settings from './settings.json';function main(){settings.server.ho";
    let mut workspace=Workspace::default();
    workspace.open(source_id(root).unwrap(),1,entry.into());
    workspace.open(source_id("file:///tsrust-intelligence/settings.json").unwrap(),1,r#"{"server":{"host":"local","port":8080}}"#.into());
    let input=workspace.snapshot();
    assert!(input.check(&source_id(root).unwrap()).is_err());
    let graph=tsrust_compiler::project::load_project_headers(&input,&source_id(root).unwrap().0).unwrap();
    let completion=ask(&ProjectIndex::new(&graph),entry,"ho","textDocument/completion");
    assert!(completion.as_array().is_some_and(|items|items.iter().any(|item|item["label"]=="host")),"{completion}");
}
