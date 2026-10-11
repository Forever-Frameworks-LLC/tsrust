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
