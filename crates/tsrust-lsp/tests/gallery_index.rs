//! Real retained gallery graph: checked facts must finish before editor requests settle.
use serde_json::json;
use std::{sync::mpsc,time::{Duration,Instant}};
use tsrust_lsp::{project_diagnostics::file_uri,project_intelligence::ProjectIndex,workspace::Workspace,LineIndex};

#[test]
fn gallery_header_index_preserves_route_help_within_a_bounded_request() {gallery_index(false);}
#[test]
fn gallery_checked_index_preserves_route_help_within_a_bounded_request() {gallery_index(true);}

fn gallery_index(checked:bool) {
    let (ready,setup)=mpsc::channel();
    let (finished,result)=mpsc::channel();
    let worker=std::thread::spawn(move|| {
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        let path=root.join("examples/gpui-starter/src/main.tsrx").canonicalize().unwrap();
        let source=std::fs::read_to_string(&path).unwrap();
        let id=tsrust_compiler::project::SourceId(path.to_string_lossy().into_owned());
        let uri=file_uri(&id).unwrap();
        let mut workspace=Workspace::default();workspace.open(id.clone(),1,source.clone());
        let started=Instant::now();
        let input=workspace.snapshot();
        let index=if checked {
            let analysis=input.check(&id).unwrap();
            eprintln!("gallery graph setup finished in {:?}",started.elapsed());ready.send(()).unwrap();
            ProjectIndex::checked(&analysis)
        }else{
            let graph=tsrust_compiler::project::load_project_headers(&input,&id.0).unwrap();
            eprintln!("gallery header setup finished in {:?}",started.elapsed());ready.send(()).unwrap();
            ProjectIndex::new(&graph)
        };
        let offset=source.rfind("<Route path=").unwrap()+2;
        let position=LineIndex::new(&source).range(&source,tsrust_compiler::Span::new(offset,offset)).unwrap().start;
        let hover=index.request("textDocument/hover",&json!({"position":position}),uri.as_str()).unwrap().unwrap();
        let definition=index.request("textDocument/definition",&json!({"position":position}),uri.as_str()).unwrap().unwrap();
        eprintln!("gallery index and requests including setup finished in {:?}",started.elapsed());
        finished.send((hover,definition)).unwrap();
    });
    setup.recv_timeout(Duration::from_secs(180)).expect("gallery compiler setup must finish within three minutes");
    let (hover,definition)=result.recv_timeout(Duration::from_secs(30)).expect("gallery checked index exceeded 30 seconds after compiler setup");
    worker.join().unwrap();
    assert!(hover.to_string().contains("Route"),"{hover}");
    assert_eq!(definition["uri"],"tsrust-stdlib:///std:ui","{definition}");
}
