use tsrust_lsp::{project_diagnostics::{convert, source_id}, workspace::Workspace};

#[test]
fn declaration_only_buffers_report_body_errors_and_clear_after_repair() {
    let entry=source_id("file:///optional-main/library.tsr").unwrap();
    let mut workspace=Workspace::default();
    workspace.open(entry.clone(),1,"export function value():int{return \"bad\";}".into());
    let snapshot=workspace.snapshot();
    let error=snapshot.check(&entry).err().unwrap();
    assert_eq!(error.code,"type");
    assert_eq!(convert(&snapshot,Some(&error)).unwrap()[&entry].len(),1);
    for (version,source) in [(2,"export function value():int{return 7;}"),(3,"")] {
        workspace.open(entry.clone(),version,source.into());
        let snapshot=workspace.snapshot();
        assert!(snapshot.check(&entry).is_ok());
        assert!(convert(&snapshot,None).unwrap()[&entry].is_empty());
    }
}

#[test]
fn standalone_syntax_diagnostics_keep_multiple_utf16_ranges() {
    let source="function main(){console.log(\"😀\");const first=;const second=;}";
    let errors=tsrust_lsp::diagnostics(source,"file:///optional-main/errors.tsr").unwrap();
    assert_eq!(errors.len(),2);
    for (error,(offset,_)) in errors.iter().zip(source.match_indices("=;")) {
        assert_eq!(error.range.start.character,source[..offset+1].encode_utf16().count() as u32);
    }
}

#[test]
fn module_syntax_errors_use_unsaved_dependency_sources_and_clear_on_repair() {
    let entry=source_id("file:///optional-main/entry.tsr").unwrap();
    let dependency=source_id("file:///optional-main/library.tsr").unwrap();
    let mut workspace=Workspace::default();
    workspace.open(entry.clone(),1,"import {value} from './library.tsr';function main(){value();}".into());
    let source="export function value(){console.log(\"😀\");const first=;const second=;}";
    workspace.open(dependency.clone(),1,source.into());
    let snapshot=workspace.snapshot();
    let errors=snapshot.check_with_diagnostics(&entry).err().unwrap();
    assert_eq!(errors.len(),2,"{errors:?}");
    assert!(snapshot.check(&entry).is_err());
    let buckets=tsrust_lsp::project_diagnostics::convert_all(&snapshot,&errors).unwrap();
    assert!(buckets[&entry].is_empty());
    assert_eq!(buckets[&dependency].len(),2);
    for (error,(offset,_)) in buckets[&dependency].iter().zip(source.match_indices("=;")) {
        assert_eq!(error.range.start.character,source[..offset+1].encode_utf16().count() as u32);
    }
    workspace.open(dependency.clone(),2,"export function value(){}".into());
    assert!(!workspace.current(&snapshot));
    let snapshot=workspace.snapshot();
    assert!(snapshot.check_with_diagnostics(&entry).is_ok());
    assert!(tsrust_lsp::project_diagnostics::convert_all(&snapshot,&[]).unwrap()[&dependency].is_empty());
}

#[test]
fn module_semantic_diagnostics_publish_independent_errors_without_name_cascades() {
    let entry=source_id("file:///semantic-errors/entry.tsr").unwrap();
    let dependency=source_id("file:///semantic-errors/library.tsr").unwrap();
    let mut workspace=Workspace::default();
    workspace.open(entry.clone(),1,"import {bad} from './library.tsr';function main(){bad();}".into());
    workspace.open(dependency.clone(),1,"export function bad(){const first:int=\"bad\";const dependent=first;console.log(dependent);const second:boolean=42;}".into());
    let snapshot=workspace.snapshot();
    let errors=snapshot.check_with_diagnostics(&entry).err().unwrap();
    assert_eq!(errors.len(),2,"{errors:?}");assert!(errors.iter().all(|error|error.code=="type"));
    let buckets=tsrust_lsp::project_diagnostics::convert_all(&snapshot,&errors).unwrap();
    assert_eq!(buckets[&dependency].len(),2);assert!(buckets[&entry].is_empty());
}
