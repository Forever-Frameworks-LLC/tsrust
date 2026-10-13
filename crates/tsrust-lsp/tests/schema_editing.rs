use serde_json::{json,Value};
use tsrust_lsp::intelligence::SourceIndex;

fn request(source:&str,needle:&str,method:&str)->Value {
    let offset=source.find(needle).unwrap()+needle.len()-usize::from(method=="textDocument/hover");
    let prefix=&source[..offset];
    SourceIndex::new(source).request(method,&json!({"position":{"line":prefix.bytes().filter(|b|*b==b'\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()}}),"file:///schema-editing.tsr").unwrap()
}
#[test]
fn unfinished_schema_members_complete_before_following_statements() {
    for (expression,expected) in [("s.","string"),("schema.","safeParse"),("schema.s","safeParse")] {
        let source=format!("import {{s}} from 'std:schema';\nconst schema=s.string();\nfunction main(){{\n{expression}\nconsole.log('next');\n}}");
        let result=request(&source,&format!("\n{expression}"),"textDocument/completion");
        assert!(result.as_array().unwrap().iter().any(|item|item["label"]==expected),"{expression}: {result}");
    }
}
#[test]
fn result_hover_explains_safe_access() {
    let source="import {s} from 'std:schema';const schema=s.string();function main(){const parsed=schema.safeParse('hello');console.log(parsed);}";
    let result=request(source,"console.log(parsed","textDocument/hover");
    let text=result.to_string();
    assert!(text.contains("Ok(")&&text.contains("Err(")&&text.contains("issues"),"{text}");
}
#[test]
fn result_match_payloads_complete_during_incomplete_editing() {
    for (arm,expected) in [("Ok(value)","toUpperCase"),("Err(value)","issues")] {
        let source=format!("import {{s}} from 'std:schema';const schema=s.string();function main(){{const parsed=schema.safeParse('hello');match(parsed){{{arm}=>{{\nvalue.\nconsole.log('next');\n}}}}}}");
        let result=request(&source,"\nvalue.","textDocument/completion");
        assert!(result.as_array().unwrap().iter().any(|item|item["label"]==expected),"{arm}: {result}");
    }
}
#[test]
fn native_recovery_respects_aliases_and_local_shadowing() {
    let source="import {s as rules} from 'std:schema';function main(){rules.\nconsole.log('next');}";
    assert!(request(source,"rules.","textDocument/completion").as_array().unwrap().iter().any(|item|item["label"]=="string"));
    let source="import {s} from 'std:schema';type Local={local:int;};function main(){const s:Local={local:1};s.\nconsole.log('next');}";
    let items=request(source,";s.","textDocument/completion");
    assert!(items.as_array().unwrap().iter().any(|item|item["label"]=="local"));
    assert!(!items.as_array().unwrap().iter().any(|item|item["label"]=="string"));
}
#[test]
fn chained_global_schema_keeps_completion_with_options_and_constraints() {
    let source="import {s} from 'std:schema';const greeting=s.string({message:'Expected a greeting'}).min(1);function main(){\ngreeting.\nmatch(greeting.safeParse('Hello')){Ok(value)=>{console.log(value);},Err(error)=>{console.log(error.issues);}}}";
    let result=request(source,"\ngreeting.","textDocument/completion");
    assert!(result.as_array().unwrap().iter().any(|item|item["label"]=="safeParse"),"{result}");
    use tsrust_lsp::{workspace::Workspace,project_diagnostics::source_id,project_intelligence::ProjectIndex};
    let uri="file:///schema-editing.tsr";
    let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,source.into());
    let graph=tsrust_compiler::project::load_project_headers(&workspace.snapshot(),&source_id(uri).unwrap().0).unwrap();
    let offset=source.find("\ngreeting.").unwrap()+"\ngreeting.".len();let prefix=&source[..offset];
    let result=ProjectIndex::new(&graph).request("textDocument/completion",&json!({"position":{"line":prefix.matches('\n').count(),"character":prefix.rsplit('\n').next().unwrap().len()}}),uri).unwrap().unwrap();
    assert!(result.as_array().unwrap().iter().any(|item|item["label"]=="safeParse"),"{result}");
}

#[test]
fn throwing_parse_completes_and_infers_direct_output() {
    let source="import {s} from 'std:schema';const schema=s.string();function main(){schema.\nconsole.log('next');}";
    let items=request(source,"schema.","textDocument/completion");
    assert!(items.as_array().unwrap().iter().any(|item|item["label"]=="parse"),"{items}");
    let source="import {s} from 'std:schema';const schema=s.string();function main(){const value=schema.parse('hello');\nvalue.\nconsole.log('next');}";
    let items=request(source,"\nvalue.","textDocument/completion");
    assert!(items.as_array().unwrap().iter().any(|item|item["label"]=="toUpperCase"),"{items}");
}

#[test]
fn throwing_parse_catch_exposes_validation_issues() {
    let source="import {s} from 'std:schema';const schema=s.string();function main(){try{schema.parse('');}catch(error){console.log(error.issues);}}";
    use tsrust_lsp::{workspace::Workspace,project_diagnostics::source_id,project_intelligence::ProjectIndex};
    let uri="file:///schema-parse-catch.tsr";
    let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,source.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let offset=source.find("error.issues").unwrap()+"error.issues".len()-1;
    let hover=ProjectIndex::checked(&checked).request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
    assert!(hover.to_string().contains("SchemaIssue"),"{hover}");
}
