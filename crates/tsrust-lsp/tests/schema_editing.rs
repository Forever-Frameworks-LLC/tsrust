use serde_json::{json,Value};
use tsrust_lsp::intelligence::SourceIndex;

fn fixture_uri(name: &str) -> String {
    let path = std::env::current_dir().unwrap().join(name);
    tsrust_lsp::project_diagnostics::file_uri(&tsrust_compiler::project::SourceId(path.to_str().unwrap().into())).unwrap().to_string()
}

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
    assert!(text.contains("parsed.success")&&text.contains("parsed.data")&&text.contains("parsed.error")&&text.contains("issues"),"{text}");
}
#[test]
fn safe_parse_branches_complete_during_incomplete_editing() {
    for (condition,member,expected) in [("parsed.success","data","toUpperCase"),("!parsed.success","error","issues")] {
        let source=format!("import {{s}} from 'std:schema';const schema=s.string();function main(){{const parsed=schema.safeParse('hello');if({condition}){{\nparsed.{member}.\nconsole.log('next');\n}}}}");
        let result=request(&source,&format!("\nparsed.{member}."),"textDocument/completion");
        assert!(result.as_array().unwrap().iter().any(|item|item["label"]==expected),"{condition}: {result}");
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
    let source="import {s} from 'std:schema';const greeting=s.string({message:'Expected a greeting'}).min(1);function main(){\ngreeting.\nconst result=greeting.safeParse('Hello');if(result.success){console.log(result.data);}else{console.log(result.error.issues);}}";
    let result=request(source,"\ngreeting.","textDocument/completion");
    assert!(result.as_array().unwrap().iter().any(|item|item["label"]=="safeParse"),"{result}");
    use tsrust_lsp::{workspace::Workspace,project_diagnostics::source_id,project_intelligence::ProjectIndex};
    let uri=fixture_uri("schema-editing.tsr");let uri=uri.as_str();
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
    let uri=fixture_uri("schema-parse-catch.tsr");let uri=uri.as_str();
    let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,source.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let offset=source.find("error.issues").unwrap()+"error.issues".len()-1;
    let hover=ProjectIndex::checked(&checked).request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
    assert!(hover.to_string().contains("SchemaIssue"),"{hover}");
}

#[test]
fn constraint_message_chains_keep_member_completion_and_parameter_help() {
    let source="import {s} from 'std:schema';const schema=s.string().min(1,{message:'Required'}).max(8,{message:'Too long'});function main(){\nschema.\nconsole.log('next');}";
    let items=request(source,"\nschema.","textDocument/completion");
    for expected in ["min","max","parse","safeParse"] {
        assert!(items.as_array().unwrap().iter().any(|item|item["label"]==expected),"{expected}: {items}");
    }
    let source="import {s} from 'std:schema';function main(){const schema=s.string().min(1, );}";
    let help=request(source,"min(1, ","textDocument/signatureHelp");
    assert!(help.to_string().contains("message: string"),"{help}");
}

#[test]
fn constraint_options_offer_message_inside_second_argument() {
    let source="import {s} from 'std:schema';function main(){const schema=s.string().min(1, { mes });}";
    let items=request(source,"{ mes","textDocument/completion");
    assert!(items.as_array().unwrap().iter().any(|item|item["label"]=="message"),"{items}");
}

#[test]
fn recursive_builder_completes_with_aliases_and_explains_self() {
    for namespace in ["s", "rules"] {
        let source=format!("import {{s as {namespace}}} from 'std:schema';function main(){{{namespace}.\nconsole.log('next');}}");
        let items=request(&source,&format!("{namespace}."),"textDocument/completion");
        assert!(items.as_array().unwrap().iter().any(|item|item["label"]=="recursive"),"{items}");
        let source=format!("import {{s as {namespace}}} from 'std:schema';const tree={namespace}.recursive(self=>{namespace}.object({{value:{namespace}.int(),children:{namespace}.array(self)}}));function main(){{}}");
        let hover=request(&source,".recursive","textDocument/hover").to_string();
        assert!(hover.contains("once") && hover.contains("self") && hover.contains("owned"),"{hover}");
    }
}

#[test]
fn recursive_checked_outputs_keep_nested_members_and_bounded_hover() {
    use tsrust_lsp::{workspace::Workspace,project_diagnostics::source_id,project_intelligence::ProjectIndex};
    let source="import {s} from 'std:schema';const tree=s.recursive(self=>s.object({value:s.int(),children:s.array(self)}));function main(){{const outcome0=tree.safeParseJson('{\"value\":1,\"children\":[]}');if(outcome0.success){const value=outcome0.data;console.log(value.children[0].value);}else{const error=outcome0.error;console.log('invalid');}}}";
    let uri=fixture_uri("recursive-schema-editing.tsr");let uri=uri.as_str();
    let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,source.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    for needle in ["console.log(value.","console.log(value.children[0]."] {
        let offset=source.find(needle).unwrap()+needle.len();
        let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
        for field in ["value","children"] {
            assert!(items.as_array().unwrap().iter().any(|item|item["label"]==field),"{needle}: {items}");
        }
    }
    let offset=source.find("const tree").unwrap()+"const tr".len();
    let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap().to_string();
    assert!(hover.contains("children") && hover.contains("value"),"{hover}");
    assert!(!hover.contains("__tsrust_private_") && hover.len()<12000,"{hover}");
}

#[test]
fn recursive_named_and_array_outputs_keep_member_completion() {
    use tsrust_lsp::{workspace::Workspace,project_diagnostics::source_id,project_intelligence::ProjectIndex};
    for (definition, body, needle, expected) in [
        ("type Tree={value:int;children:Tree[];};const schema=s.recursive<Tree>(self=>s.object({value:s.int(),children:s.array(self)}));",
         "{const outcome0=schema.safeParseJson('{\"value\":1,\"children\":[]}');if(outcome0.success){const value=outcome0.data;console.log(value.children[0].value);}else{const error=outcome0.error;}}", "value.children[0].", "value"),
        ("const schema=s.recursive(self=>s.array(self));",
         "{const outcome0=schema.safeParseJson('[[]]');if(outcome0.success){const value=outcome0.data;console.log(value[0].length);}else{const error=outcome0.error;}}", "value[0].", "length"),
    ] {
        let source=format!("import {{s}} from 'std:schema';{definition}function main(){{{body}}}");
        let uri=fixture_uri("recursive-output-editing.tsr");let uri=uri.as_str();
        let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,source.clone());
        let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
        let index=ProjectIndex::checked(&checked);
        let offset=source.find(needle).unwrap()+needle.len();
        let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
        assert!(items.as_array().unwrap().iter().any(|item|item["label"]==expected),"{definition}: {items}");
        let offset=source.find("const schema").unwrap()+"const sch".len();
        let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap().to_string();
        assert!(!hover.contains("__tsrust_private_")&&hover.len()<12000,"{hover}");
    }
}

#[test]
fn recursive_schema_completion_recovers_before_following_statement() {
    use tsrust_lsp::{workspace::Workspace,project_diagnostics::source_id,project_intelligence::ProjectIndex};
    for suffix in ["\nconsole.log('next');}","}"] {
        let source=format!("import {{s}} from 'std:schema';const tree=s.recursive(self=>s.object({{value:s.int(),children:s.array(self)}}));function main(){{tree.{suffix}");
        let uri=fixture_uri("recursive-unfinished.tsr");let uri=uri.as_str();
        let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,source.clone());
        let graph=tsrust_compiler::project::load_project_headers(&workspace.snapshot(),&source_id(uri).unwrap().0).unwrap();
        let offset=source.find("tree.").unwrap()+"tree.".len();
        let items=ProjectIndex::new(&graph).request("textDocument/completion",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
        for method in ["parse","safeParse","safeParseJson"] {
            assert!(items.as_array().unwrap().iter().any(|item|item["label"]==method),"{method}: {items}");
        }
    }
}

#[test]
fn recursive_union_root_keeps_common_fields_and_finite_hover() {
    use tsrust_lsp::{workspace::Workspace,project_diagnostics::source_id,project_intelligence::ProjectIndex};
    let source="import {s} from 'std:schema';const schema=s.recursive(self=>s.union([s.object({kind:s.literal('leaf'),value:s.int()}),s.object({kind:s.literal('branch'),children:s.array(self)})]));function main(){{const outcome0=schema.safeParseJson('{\"kind\":\"leaf\",\"value\":1}');if(outcome0.success){const value=outcome0.data;console.log(value.kind);}else{const error=outcome0.error;}}}";
    let uri=fixture_uri("recursive-union-editing.tsr");let uri=uri.as_str();
    let mut workspace=Workspace::default();workspace.open(source_id(uri).unwrap(),1,source.into());
    let checked=workspace.snapshot().check(&source_id(uri).unwrap()).unwrap();
    let index=ProjectIndex::checked(&checked);
    let offset=source.find("console.log(value.").unwrap()+"console.log(value.".len();
    let items=index.request("textDocument/completion",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap();
    assert!(items.as_array().unwrap().iter().any(|item|item["label"]=="kind"),"{items}");
    assert!(!items.as_array().unwrap().iter().any(|item|item["label"]=="children"||item["label"]=="value"),"{items}");
    let offset=source.find("const schema").unwrap()+"const sch".len();
    let hover=index.request("textDocument/hover",&json!({"position":{"line":0,"character":offset}}),uri).unwrap().unwrap().to_string();
    assert!(hover.contains("children")&&hover.contains("kind"),"{hover}");
    assert!(!hover.contains("__tsrust_private_")&&hover.len()<12000,"{hover}");
}

#[test]
fn async_generic_decode_keeps_inferred_factory_results_in_editor_facts() {
    use tsrust_lsp::{workspace::Workspace,project_diagnostics::source_id,project_intelligence::ProjectIndex};
    let source="import {s} from 'std:schema';type Header={operation:string;};function rule(){return s.object({operation:s.string(),title:s.string()});}async function verify(body:string):Task<void>{const header=JSON.parse<Header>(body);const parsed=rule().safeParseJson(body);if(parsed.success){console.log(parsed.data.title);}}async function main():Task<void>{await verify('{\"operation\":\"test\",\"title\":\"ready\"}');}";
    let source=tsrust_compiler::format(source).unwrap();
    assert_eq!(tsrust_compiler::format(&source).unwrap(),source);
    let uri=fixture_uri("async-schema-factory.tsr");let uri=uri.as_str();
    let id=source_id(uri).unwrap();
    let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.clone());
    let checked=workspace.snapshot().check(&id).unwrap();
    let index=ProjectIndex::checked(&checked);
    let offset=source.find("parsed.data.title").unwrap()+"parsed.data.".len();
    let prefix=&source[..offset];
    let items=index.request("textDocument/completion",&json!({"position":{"line":prefix.matches('\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()}}),uri).unwrap().unwrap();
    for field in ["operation","title"] { assert!(items.as_array().unwrap().iter().any(|item|item["label"]==field),"{items}"); }
    let offset=source.find("rule().safeParseJson").unwrap()+2;
    let prefix=&source[..offset];
    let hover=index.request("textDocument/hover",&json!({"position":{"line":prefix.matches('\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()}}),uri).unwrap().unwrap().to_string();
    assert!(hover.contains("Schema")&&!hover.contains(": void"),"{hover}");
}
