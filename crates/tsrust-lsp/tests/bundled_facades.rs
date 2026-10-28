use tsrust_compiler::project::{SourceId,SourceProvider,bundled_source};
use tsrust_lsp::{Snapshot,project_diagnostics::{file_uri,convert_all},project_intelligence::ProjectIndex,project_session::ProjectSession,workspace::Workspace};

#[test]
fn original_bundled_facades_have_standard_context_and_document_assistance(){
    for name in ["ui"]{
        let logical=SourceId(format!("std:{name}"));let source=bundled_source(&logical).unwrap();
        let path=std::fs::canonicalize(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../tsrust-compiler/src/stdlib/{name}/api.tsr"))).unwrap();
        let physical=SourceId(path.to_str().unwrap().into());let uri=file_uri(&physical).unwrap().to_string();
        assert_eq!(tsrust_compiler::project::FileSourceProvider.resolve(None,&physical.0).unwrap(),logical);
        let mut session=ProjectSession::default();session.open(&Snapshot{uri:uri.clone(),version:1,generation:1,text:source.into()});
        let input=session.workspace.snapshot();assert_eq!(input.resolve(None,&physical.0).unwrap(),logical);
        let checked=input.check_with_diagnostics(&physical).unwrap();
        let buckets=convert_all(&input,&checked.warnings()).unwrap();assert!(buckets.get(&physical).unwrap().is_empty());
        session.cache_index(&uri,&input,Some(ProjectIndex::checked(&checked)));assert!(session.covers(&uri));
        let needle=if name=="ui"{"export type Node"}else{"export function"};
        let offset=source.find(needle).unwrap()+needle.len()-1;let prefix=&source[..offset];
        let position=serde_json::json!({"line":prefix.bytes().filter(|b|*b==b'\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()});
        assert!(session.request("textDocument/hover",&serde_json::json!({"position":position}),&uri).is_some());
    }
}

#[test]
fn copied_facades_and_modified_original_buffers_cannot_import_private_runtime(){
    let source=bundled_source(&SourceId("std:ui".into())).unwrap();
    let original=std::fs::canonicalize(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tsrust-compiler/src/stdlib/ui/api.tsr")).unwrap();
    let copied=std::fs::canonicalize(std::env::temp_dir()).unwrap().join(format!("tsrust-facade-copy-{}",std::process::id())).join("src").join("stdlib").join("ui").join("api.tsr");
    for (path,text) in [(copied,source.to_owned()),(original,format!("{source}\n// modified buffer"))]{
        let id=SourceId(path.to_str().unwrap().into());let mut workspace=Workspace::default();workspace.open(id.clone(),1,text);
        let input=workspace.snapshot();assert_eq!(input.resolve(None,&id.0).unwrap(),id);
        let errors=input.check_with_diagnostics(&id).err().expect("private runtime import must be rejected");
        assert!(errors.iter().any(|error|error.code=="module-native"));
    }
}
