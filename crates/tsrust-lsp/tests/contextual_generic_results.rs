use serde_json::json;
use tsrust_compiler::project::SourceId;
use tsrust_lsp::{project_diagnostics::{file_uri, source_id}, project_intelligence::ProjectIndex, workspace::Workspace};

#[test]
fn native_values_returned_from_generic_identity_keep_concrete_help() {
    let source=tsrust_compiler::format("import {copyFile,File,seek,position} from 'std:fs';function identity<T>(value:T):T{return value;}function inspect(file:Mutable<File>){const copied=identity(copyFile);const move=identity(seek);const query=identity(position);copied('a','b');move(file,0,'start');query(file);}function main(){}").unwrap();
    let id=SourceId(std::env::current_dir().unwrap().join("generic-native-values.tsr").to_str().unwrap().into());
    let uri=file_uri(&id).unwrap();let id=source_id(uri.as_str()).unwrap();
    let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.clone());
    let checked=workspace.snapshot().check(&id).unwrap();let index=ProjectIndex::checked(&checked);
    let position=|offset:usize| {let prefix=&source[..offset];json!({"position":{"line":prefix.matches('\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()}})};
    for (call,required) in [("copied(",&["string","int","FsError"][..]),("move(",&["File","offset","origin","int","FsError"][..]),("query(",&["File","int","FsError"][..])] {
        let offset=source.find(call).unwrap();
        let hover=index.request("textDocument/hover",&position(offset+1),uri.as_str()).unwrap().unwrap().to_string();
        assert!(required.iter().all(|fragment|hover.contains(fragment)),"{call}: {hover}");
        let help=index.request("textDocument/signatureHelp",&position(offset+call.len()),uri.as_str()).unwrap().unwrap().to_string();
        assert!(required.iter().all(|fragment|help.contains(fragment)),"{call}: {help}");
        for text in [&hover,&help] {assert!(!["@native:","NativeCallable","Closure","identity#"].iter().any(|private|text.contains(private)),"{text}");}
    }
}

#[test]
fn union_result_context_keeps_checked_completion_and_concrete_signature() {
    let source = tsrust_compiler::format("type First={kind:'first';value:int;};type Second={kind:'second';text:string;};type Box={variant:First|Second;};function choose<T>(first:boolean):T{if(first){return {kind:'first',value:7};}return {kind:'second',text:'ready'};}function main(){const box:Box={variant:choose(true)};if(box.variant.kind==='first'){console.log(box.variant.value);}}").unwrap();
    assert_eq!(tsrust_compiler::format(&source).unwrap(), source);
    let path = std::env::current_dir().unwrap().join("contextual-generic-results.tsr");
    let id = SourceId(path.to_str().unwrap().into());
    let uri = file_uri(&id).unwrap();
    let id = source_id(uri.as_str()).unwrap();
    let mut workspace = Workspace::default();
    workspace.open(id.clone(), 1, source.clone());
    let checked = workspace.snapshot().check(&id).unwrap();
    let index = ProjectIndex::checked(&checked);
    let position = |offset: usize| {
        let prefix = &source[..offset];
        json!({"position":{"line":prefix.matches('\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()}})
    };
    let offset = source.find("box.variant.value").unwrap() + "box.variant.".len();
    let items = index.request("textDocument/completion", &position(offset), uri.as_str()).unwrap().unwrap();
    assert!(items.as_array().unwrap().iter().any(|item| item["label"] == "value"), "{items}");
    assert!(!items.as_array().unwrap().iter().any(|item| item["label"] == "text"), "{items}");
    let offset = source.find("choose(true)").unwrap() + 2;
    let hover = index.request("textDocument/hover", &position(offset), uri.as_str()).unwrap().unwrap().to_string();
    assert!(hover.contains("First") && hover.contains("Second"), "{hover}");
}
