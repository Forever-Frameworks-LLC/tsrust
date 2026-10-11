use tsrust_compiler::{
    Span,
    project::{ProjectDiagnostic, SourceId, SourceLocation},
};
use tsrust_lsp::{
    project_diagnostics::{convert, file_uri, source_id},
    workspace::Workspace,
};
fn id(name: &str) -> SourceId {
    SourceId(format!("/tsrust diagnostics/{name}.tsr"))
}
#[test]
fn file_uri_roundtrip_handles_spaces_unicode_percent_and_path_normalization() {
    for name in ["hello world", "😀", "percent%#?", "café"] {
        let id = id(name);
        let uri = file_uri(&id).unwrap();
        assert_eq!(source_id(uri.as_str()).unwrap(), id);
    }
    assert_eq!(
        source_id("file:///tsrust%20diagnostics/sub/../main.tsr").unwrap(),
        id("main")
    );
    assert_eq!(
        source_id("file://localhost/tsrust%20diagnostics/main.tsr").unwrap(),
        id("main")
    );
    for uri in [
        "untitled:test",
        "file://remote/path",
        "file:///x%",
        "file:///x%GG",
        "file:///x%00",
        "file:///x?query",
        "file:///x#fragment",
        "file:///x%FF",
    ] {
        assert!(source_id(uri).is_err(), "{uri}");
    }
}
#[test]
fn errors_in_unsaved_dependency_map_to_its_own_utf16_range() {
    let mut workspace = Workspace::default();
    workspace.open(
        id("main"),
        1,
        "import { value } from './dep.tsr';function main(){value();}".into(),
    );
    let source = "export function value(){console.log('😀');console.log(missing);}";
    workspace.open(id("dep"), 4, source.into());
    let snapshot = workspace.snapshot();
    let error = snapshot.check(&id("main")).err().unwrap();
    let diagnostics = convert(&snapshot, Some(&error)).unwrap();
    assert!(diagnostics[&id("main")].is_empty());
    let diagnostic = &diagnostics[&id("dep")][0];
    let start = source.find("missing").unwrap();
    assert_eq!(
        diagnostic.range.start.character,
        source[..start].encode_utf16().count() as u32
    );
    assert_eq!(
        diagnostic.range.end.character - diagnostic.range.start.character,
        7
    );
    assert_eq!(snapshot.version(&id("dep")), Some(4));
}
#[test]
fn cross_file_related_ranges_use_each_exact_snapshot_and_invalid_spans_fail() {
    let mut workspace = Workspace::default();
    workspace.open(
        id("main"),
        1,
        "import { value } from './dep.tsr';function main(){value();}".into(),
    );
    workspace.open(id("dep"), 2, "export function value(){}".into());
    let snapshot = workspace.snapshot();
    snapshot.check(&id("main")).unwrap();
    let mut error = ProjectDiagnostic {
        code: "test",
        message: "test".into(),
        location: SourceLocation {
            source: id("main"),
            span: Span::new(0, 6),
        },
        related: vec![(
            SourceLocation {
                source: id("dep"),
                span: Span::new(0, 6),
            },
            "Dependency declaration".into(),
        )],
        fixes: Box::new([]),
    };
    let converted = convert(&snapshot, Some(&error)).unwrap();
    let related = &converted[&id("main")][0]
        .related_information
        .as_ref()
        .unwrap()[0];
    assert_eq!(related.location.uri, file_uri(&id("dep")).unwrap());
    error.related[0].0.span = Span::new(1000, 1001);
    assert!(convert(&snapshot, Some(&error)).is_err());
    assert!(
        convert(&snapshot, None)
            .unwrap()
            .values()
            .all(Vec::is_empty)
    );
}

#[test]
fn dependent_recheck_preserves_source_version_and_bounded_queue_order() {
    use std::time::{Duration, Instant};
    let now = Instant::now();
    let mut documents = tsrust_lsp::Documents::default();
    documents.open("first".into(), 4, "function main(){}".into(), now);
    documents.open("second".into(), 7, "function main(){}".into(), now);
    let first = documents.next(now).unwrap();
    assert_eq!(first.uri, "first");
    assert!(documents.reschedule("first", now, Duration::ZERO));
    for _ in 0..20 {
        assert!(documents.reschedule("first", now, Duration::ZERO));
    }
    assert_eq!(documents.next(now).unwrap().uri, "second");
    let refreshed = documents.next(now).unwrap();
    assert_eq!(refreshed.uri, "first");
    assert_eq!(refreshed.version, 4);
    assert_eq!(refreshed.text, first.text);
    assert_eq!(refreshed.generation, first.generation);
    assert!(documents.next(now).is_none());
    assert!(!documents.reschedule("closed", now, Duration::ZERO));
}

#[test]
fn opaque_native_capability_errors_keep_the_argument_range() {
    let source = tsrust_compiler::format("import {Bytes} from 'std:bytes';function inspect(value:ReadOnly<Bytes>):void{console.log(value);}function main(){}").unwrap();
    let mut workspace = Workspace::default();
    workspace.open(id("main"), 1, source.clone());
    let snapshot = workspace.snapshot();
    let error = snapshot.check(&id("main")).err().unwrap();
    assert_eq!(error.code, "native-capability");
    let start = source.rfind("value);").unwrap();
    assert_eq!(error.location.span, Span::new(start, start + 5));
    let diagnostics = convert(&snapshot, Some(&error)).unwrap();
    let diagnostic = serde_json::to_value(&diagnostics[&id("main")][0]).unwrap();
    assert_eq!(diagnostic["code"], "native-capability");
    assert!(diagnostic["message"].as_str().unwrap().contains("Bytes"));
}

#[test]
fn native_byte_write_type_errors_keep_the_value_range() {
    let source = tsrust_compiler::format("import {Bytes,set} from 'std:bytes';function change(bytes:Bytes){set(bytes,0,'x');}function main(){}").unwrap();
    let mut workspace = Workspace::default();
    workspace.open(id("main"), 1, source.clone());
    let snapshot = workspace.snapshot();
    let error = snapshot.check(&id("main")).err().unwrap();
    assert_eq!(error.code, "type");
    let start = source.rfind("\"x\"").or_else(|| source.rfind("'x'")).unwrap();
    assert_eq!(error.location.span, Span::new(start, start + 3));
    let diagnostics = convert(&snapshot, Some(&error)).unwrap();
    let diagnostic = serde_json::to_value(&diagnostics[&id("main")][0]).unwrap();
    assert_eq!(diagnostic["code"], "type");
    assert!(diagnostic["message"].as_str().unwrap().contains("Expected int"));
}

#[test]
fn byte_view_conflicts_report_the_owner_and_the_view_location() {
    let source = tsrust_compiler::format("import {alloc,view,set,length} from 'std:bytes';function main(){match(alloc(2,1)){Ok(bytes)=>{match(view(bytes,0,1)){Ok(part)=>{match(set(bytes,0,9)){Ok(_)=>{},Err(error)=>{}}console.log(length(part));},Err(error)=>{}}},Err(error)=>{}}}").unwrap();
    let mut workspace = Workspace::default();
    workspace.open(id("main"), 1, source.clone());
    let snapshot = workspace.snapshot();
    let error = snapshot.check(&id("main")).err().unwrap();
    assert_eq!(error.code, "native-view-conflict");
    let diagnostics = convert(&snapshot, Some(&error)).unwrap();
    let diagnostic = serde_json::to_value(&diagnostics[&id("main")][0]).unwrap();
    assert_eq!(diagnostic["code"], "native-view-conflict");
    assert!(diagnostic["message"].as_str().unwrap().contains("bytes"));
    assert!(diagnostic["relatedInformation"].as_array().is_some_and(|related| !related.is_empty()), "{diagnostic}");
}

#[test]
fn recursive_byte_views_keep_short_owner_escape_diagnostics() {
    for (fixture,append) in [
        (include_str!("../../tsrust-compiler/tests/fixtures/native-byte-views/recursive-arrays.tsr"),"root.children.push(leaf(read(second)));"),
        (include_str!("../../tsrust-compiler/tests/fixtures/native-byte-views/recursive-links.tsr"),"root.next=leaf(read(second));"),
    ] {
        let mut workspace = Workspace::default();
        workspace.open(id("main"), 1, fixture.into());
        assert!(workspace.snapshot().check(&id("main")).is_ok());
        let helpers = fixture.split("function main()").next().unwrap();
        let source = tsrust_compiler::format(&format!("{helpers}function main(){{const first=bytes(3);let escaped:Chunk;{{const second=bytes(7);const root=leaf(read(first));{append}escaped=root;}}console.log(length(escaped.data));}}" )).unwrap();
        workspace.open(id("main"), 2, source);
        let snapshot = workspace.snapshot();
        let error = snapshot.check(&id("main")).err().unwrap();
        assert_eq!(error.code, "native-view-escape");
        let diagnostics = convert(&snapshot, Some(&error)).unwrap();
        let diagnostic = serde_json::to_value(&diagnostics[&id("main")][0]).unwrap();
        assert_eq!(diagnostic["code"], "native-view-escape");
        assert!(diagnostic["relatedInformation"].as_array().is_some_and(|related| !related.is_empty()));
    }
}

#[test]
fn capturing_callbacks_keep_argument_view_owner_diagnostics() {
    let fixture=include_str!("../../tsrust-compiler/tests/fixtures/native-byte-views/callbacks.tsr");
    let mut workspace=Workspace::default();
    workspace.open(id("main"),1,fixture.into());
    assert!(workspace.snapshot().check(&id("main")).is_ok());
    let helpers=fixture.split("function main()").next().unwrap();
    let source=tsrust_compiler::format(&format!("{helpers}function leak(first:ByteView):ByteView{{match(fromUtf8(\"short\")){{Ok(owner)=>{{return captured(first,slice(owner,0,1));}},Err(error)=>{{throw error.message;}}}}}}function main(){{}}" )).unwrap();
    workspace.open(id("main"),2,source);
    let snapshot=workspace.snapshot();
    let error=snapshot.check(&id("main")).err().unwrap();
    assert_eq!(error.code,"native-view-escape");
    let diagnostics=convert(&snapshot,Some(&error)).unwrap();
    let diagnostic=serde_json::to_value(&diagnostics[&id("main")][0]).unwrap();
    assert!(diagnostic["relatedInformation"].as_array().is_some_and(|related|!related.is_empty()));
}

#[test]
fn callbacks_borrow_owner_arguments_without_losing_provenance() {
    let fixture=include_str!("../../tsrust-compiler/tests/fixtures/native-byte-views/callback-owners.tsr");
    let mut workspace=Workspace::default();workspace.open(id("main"),1,fixture.into());
    assert!(workspace.snapshot().check(&id("main")).is_ok());
    let helpers=fixture.split("function main()").next().unwrap();
    let source=tsrust_compiler::format(&format!("{helpers}function leak(captured:ByteView):ByteView{{const owner=bytes(\"short\");return saveRead(captured).pick(owner);}}function main(){{}}" )).unwrap();
    workspace.open(id("main"),2,source);let snapshot=workspace.snapshot();
    let error=snapshot.check(&id("main")).err().unwrap();assert_eq!(error.code,"native-view-escape");
    let diagnostics=convert(&snapshot,Some(&error)).unwrap();
    assert!(!diagnostics[&id("main")].is_empty());
}

#[test]
fn recursive_view_callback_storage_keeps_short_owner_diagnostics() {
    let source = include_str!("../../tsrust-compiler/tests/fixtures/native-byte-views/recursive-callbacks.tsr");
    let source = source.replace("function main()", "function other(first:ByteView,second:ByteView):Holder{return {data:first,read:()=>second};}\nfunction main()")
        .replace("holder.next = build(read(second), read(first));", "holder.next = other(read(second), read(first));");
    let mut workspace = Workspace::default();
    workspace.open(id("main"), 1, source.clone());
    assert!(workspace.snapshot().check(&id("main")).is_ok());
    let helpers = source.split("function main()").next().unwrap();
    let invalid = format!("{helpers}function main(){{const first=bytes(3);let escaped:Holder;{{const second=bytes(7);escaped=build(read(first),read(second));}}const result=escaped.read();console.log(length(result));}}");
    workspace.open(id("main"), 2, invalid);
    let snapshot = workspace.snapshot();
    let error = snapshot.check(&id("main")).err().unwrap();
    assert_eq!(error.code, "native-view-escape");
    let diagnostics = convert(&snapshot, Some(&error)).unwrap();
    let diagnostic = serde_json::to_value(&diagnostics[&id("main")][0]).unwrap();
    assert_eq!(diagnostic["code"], "native-view-escape");
}

#[test]
fn recursive_record_parameter_callbacks_keep_shared_owner_diagnostics() {
    let source = include_str!("../../tsrust-compiler/tests/fixtures/native-byte-views/recursive-parameters.tsr");
    let mut workspace = Workspace::default();
    workspace.open(id("main"), 1, source.into());
    assert!(workspace.snapshot().check(&id("main")).is_ok());
    let helpers = source.split("function main()").next().unwrap();
    for (version, arguments) in [(2, "read(short),other"), (3, "other,read(short)")] {
        let source = format!("{helpers}function leak(other:ByteView){{const short=bytes(3);return take(build({arguments}));}}function main(){{const owner=bytes(7);const callback=leak(read(owner));const result=callback();console.log(length(result));}}");
        workspace.open(id("main"), version, source);
        let snapshot = workspace.snapshot();
        let error = snapshot.check(&id("main")).err().unwrap();
        assert_eq!(error.code, "native-view-escape");
        let diagnostics = convert(&snapshot, Some(&error)).unwrap();
        let diagnostic = serde_json::to_value(&diagnostics[&id("main")][0]).unwrap();
        assert_eq!(diagnostic["code"], "native-view-escape");
    }
}

#[test]
fn recursive_callback_only_records_keep_captured_owner_diagnostics() {
    let source=include_str!("../../tsrust-compiler/tests/fixtures/native-byte-views/recursive-captured-views.tsr");
    let mut workspace=Workspace::default();
    workspace.open(id("main"),1,source.into());
    assert!(workspace.snapshot().check(&id("main")).is_ok());
    let helpers=source.split("function main()").next().unwrap();
    let invalid=format!("{helpers}function leak(first:ByteView):Holder{{const short=bytes(7);return build(first,read(short));}}function main(){{const owner=bytes(3);const holder=leak(read(owner));const result=holder.read();console.log(length(result));}}");
    workspace.open(id("main"),2,invalid);
    let snapshot=workspace.snapshot();let error=snapshot.check(&id("main")).err().unwrap();
    assert_eq!(error.code,"native-view-escape");
    let diagnostics=convert(&snapshot,Some(&error)).unwrap();
    let diagnostic=serde_json::to_value(&diagnostics[&id("main")][0]).unwrap();
    assert_eq!(diagnostic["code"],"native-view-escape");
}

#[test]
fn nullable_recursive_callbacks_keep_captured_owner_diagnostics() {
    let source=include_str!("../../tsrust-compiler/tests/fixtures/native-byte-views/nullable-recursive-callbacks.tsr");
    let mut workspace=Workspace::default();
    workspace.open(id("main"),1,source.into());
    assert!(workspace.snapshot().check(&id("main")).is_ok());
    let helpers=source.split("function main()").next().unwrap();
    let invalid=format!("{helpers}function leak(first:ByteView):Holder{{const short=bytes(7);return other(first,read(short));}}function main(){{const owner=bytes(3);const holder=leak(read(owner));const result=holder.read!();console.log(length(result));}}");
    workspace.open(id("main"),2,invalid);
    let snapshot=workspace.snapshot();let error=snapshot.check(&id("main")).err().unwrap();
    assert_eq!(error.code,"native-view-escape");
    let diagnostics=convert(&snapshot,Some(&error)).unwrap();
    let diagnostic=serde_json::to_value(&diagnostics[&id("main")][0]).unwrap();
    assert_eq!(diagnostic["code"],"native-view-escape");
}

#[test]
fn recursive_parameter_mutation_keeps_caller_owner_diagnostics() {
    let source=include_str!("../../tsrust-compiler/tests/fixtures/native-byte-views/recursive-mutation.tsr");
    let mut workspace=Workspace::default();workspace.open(id("main"),1,source.into());
    assert!(workspace.snapshot().check(&id("main")).is_ok());
    let invalid=format!("import {{set}} from 'std:bytes';{}",source.replace("const child", "set(third,0,19);const child"));
    workspace.open(id("main"),2,invalid);
    let snapshot=workspace.snapshot();let error=snapshot.check(&id("main")).err().unwrap();assert_eq!(error.code,"native-view-conflict");
    let diagnostics=convert(&snapshot,Some(&error)).unwrap();let diagnostic=serde_json::to_value(&diagnostics[&id("main")][0]).unwrap();assert_eq!(diagnostic["code"],"native-view-conflict");
}
