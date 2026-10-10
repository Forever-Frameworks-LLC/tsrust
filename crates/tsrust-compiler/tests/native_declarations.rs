mod support;

use tsrust_compiler::native_signatures::{NativeTypeId, Signature};
use tsrust_compiler::native_types::{declaration, lookup, Representation};

#[test]
fn native_declarations_preserve_identity_and_hide_owned_storage() {
    let bytes = lookup("std:bytes", "Bytes").unwrap();
    assert_eq!(bytes.id, NativeTypeId::Bytes);
    assert_eq!(bytes, declaration(NativeTypeId::Bytes));
    assert_eq!(bytes.representation, Representation::Opaque);
    assert!(bytes.fields.is_empty());
    assert!(bytes.independent_clone);
    assert!(!bytes.source_constructible());
    assert!(lookup("std:path", "Bytes").is_none());
    assert!(lookup("std:bytes", "bytes").is_none());
}

#[test]
fn native_error_fields_expose_only_public_contract() {
    let error = declaration(NativeTypeId::ByteError);
    assert_eq!(error.representation, Representation::Record);
    assert!(error.source_constructible());
    assert_eq!(error.fields.iter().map(|field| (field.name, field.signature, field.optional)).collect::<Vec<_>>(), vec![
        ("code", Signature::String, false),
        ("message", Signature::String, false),
        ("offset", Signature::Int, true),
    ]);
}

#[test]
fn native_exports_distinguish_types_from_functions_and_validate_modules() {
    use tsrust_compiler::native_exports::{lookup as export, Export};
    assert!(matches!(export("std:bytes", "Bytes").unwrap(), Export::Type(item) if item.id == NativeTypeId::Bytes));
    assert!(matches!(export("std:path", "dirname").unwrap(), Export::Function(item) if item.name == "dirname"));
    assert!(export("std:bytes", "missing").unwrap_err().contains("does not export"));
    assert!(export("std:missing", "Bytes").unwrap_err().contains("Unknown standard-library module"));
}

#[test]
fn strict_and_tolerant_graphs_keep_native_type_aliases() {
    use tsrust_compiler::project::{SourceId, SourceProvider, load_project, load_project_headers};
    struct Provider;
    impl SourceProvider for Provider {
        fn resolve(&self, _: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
            Ok(SourceId(path.into()))
        }
        fn read(&self, _: &SourceId) -> Result<String, String> {
            Ok("import { Bytes as Buffer, ByteError as Failure } from \"std:bytes\"; function main() {}".into())
        }
    }
    for graph in [load_project(&Provider, "main.tsr"), load_project_headers(&Provider, "main.tsr")] {
        let graph = graph.unwrap();
        let import = &graph.modules[&graph.entry].imports[0];
        assert_eq!(import.native, Some("std:bytes"));
        assert_eq!(import.bindings[0].exported, "Bytes");
        assert_eq!(import.bindings[0].local, "Buffer");
        assert_eq!(import.bindings[1].local, "Failure");
        assert!(import.target.is_none());
    }
}

#[test]
fn native_type_imports_do_not_select_callable_runtime_features() {
    let features = tsrust_compiler::native_bindings::selected_features([
        ("std:bytes", "Bytes"), ("std:bytes", "ByteError"), ("std:path", "dirname"),
    ]).unwrap();
    assert_eq!(features.into_iter().collect::<Vec<_>>(), vec!["path"]);
    assert!(tsrust_compiler::native_bindings::selected_features([("std:bytes", "missing")]).is_err());
}

#[test]
fn checked_native_arena_records_keep_identity_and_optional_field_types() {
    use tsrust_compiler::{native_type_resolution::intern, Span};
    let mut records = Vec::new();
    let span = Span::new(4, 9);
    let bytes = intern(&mut records, NativeTypeId::Bytes, span).unwrap();
    assert_eq!(intern(&mut records, NativeTypeId::Bytes, Span::new(20, 25)).unwrap(), bytes);
    let error = intern(&mut records, NativeTypeId::ByteError, span).unwrap();
    assert_ne!(bytes, error);
    assert_eq!(records.len(), 2);
    assert_eq!(records[bytes].native_type, Some(NativeTypeId::Bytes));
    assert!(records[bytes].fields.is_empty());
    assert!(matches!(&records[error].fields[2].annotation.kind, tsrust_compiler::type_syntax::ValueAnnotationKind::Union(parts) if parts.len() == 2));
    assert!(records[error].fields[2].optional);
}

#[test]
fn recursive_native_projection_keeps_ids_and_rolls_back_failures() {
    use tsrust_compiler::{native_type_resolution::{intern, project, linked_name}, Span};
    use tsrust_compiler::native_signatures::Intrinsic;
    use tsrust_compiler::type_syntax::ValueAnnotationKind as Kind;
    let span = Span::new(8, 12);
    let mut records = Vec::new();
    let signature = Signature::Result(&Signature::Array(&Signature::Named(NativeTypeId::Bytes)), &Signature::Named(NativeTypeId::ByteError));
    let annotation = project(&mut records, &signature, span).unwrap();
    let Kind::Applied { name, arguments } = annotation.kind else { panic!("Result annotation"); };
    assert_eq!(name.text, "Result");
    let Kind::Array(element) = &arguments[0].kind else { panic!("array payload"); };
    assert!(matches!(&element.kind, Kind::Named(name) if name.text == linked_name(NativeTypeId::Bytes)));
    assert_eq!(records.len(), 2);
    let mut retained = Vec::new();
    intern(&mut retained, NativeTypeId::ByteError, span).unwrap();
    let invalid = Signature::Result(&Signature::Named(NativeTypeId::Bytes), &Signature::Intrinsic(Intrinsic::ObservedValue));
    assert_eq!(project(&mut retained, &invalid, span).unwrap_err().code, "native-type");
    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0].native_type, Some(NativeTypeId::ByteError));
    let mut deep: &'static Signature = &Signature::Int;
    for _ in 0..129 { deep = Box::leak(Box::new(Signature::Array(deep))); }
    assert_eq!(project(&mut retained, deep, span).unwrap_err().code, "native-type-limit");
    assert_eq!(retained.len(), 1);
}

fn checked_source(source: &str) -> Result<tsrust_compiler::project::ProjectAnalysis, tsrust_compiler::project::ProjectDiagnostic> {
    use tsrust_compiler::project::{SourceId, SourceProvider, load_project, check_graph};
    struct Provider<'a>(&'a str);
    impl SourceProvider for Provider<'_> {
        fn resolve(&self, _: Option<&SourceId>, path: &str) -> Result<SourceId, String> { Ok(SourceId(path.into())) }
        fn read(&self, _: &SourceId) -> Result<String, String> { Ok(self.0.into()) }
    }
    let formatted = support::format_round_trip(source);
    check_graph(load_project(&Provider(&formatted), "main.tsr")?)
}

#[test]
fn imported_native_types_check_aliases_and_reject_opaque_construction() {
    let source = "import {Bytes as Buffer, ByteError as Failure} from 'std:bytes'; type Alias = Failure; function borrow(value: ReadOnly<Buffer>): void {} function main(){const error: Alias = {code:'range',message:'bad'}; console.log(error.code,error.offset);}";
    let checked = checked_source(source).unwrap();
    assert!(checked.emit_rust().contains("Vec<u8>"));
    let error = checked_source("import {Bytes} from 'std:bytes'; function main(){const value: Bytes = {};}").err().unwrap();
    assert_eq!(error.code, "native-opaque");
    assert!(error.message.contains("cannot be constructed"));
}

#[test]
fn native_public_error_aliases_execute_with_optional_fields() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(include_str!("fixtures/native-type-imports/errors.tsr"));
    let path = directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn opaque_native_values_reject_record_observation_and_spread() {
    for (parameter, body) in [
        ("ReadOnly<Bytes>", "console.log(value);"),
        ("ReadOnly<Bytes[]>", "console.log(value);"),
        ("ReadOnly<Bytes | null>", "console.log(value);"),
        ("ReadOnly<Map<string, Bytes>>", "console.log(value);"),
        ("ReadOnly<Row>", "console.log(value);"),
        ("ReadOnly<Bytes>", "String(value);"),
        ("ReadOnly<Bytes>", "const text = `${value}`;"),
        ("ReadOnly<Bytes>", "Object.keys(value);"),
        ("ReadOnly<Bytes>", "Object.values(value);"),
        ("ReadOnly<Bytes>", "Object.entries(value);"),
        ("ReadOnly<Bytes>", "Object.is(value,value);"),
        ("ReadOnly<Bytes>", "Object.hasOwn(value, 'bytes');"),
        ("ReadOnly<Bytes>", "const copy = {...value};"),
    ] {
        let source = format!("import {{Bytes}} from 'std:bytes';type Row={{bytes:Bytes;}};function inspect(value:{parameter}):void{{{body}}}function main(){{}}");
        let error = checked_source(&source).err().unwrap_or_else(|| panic!("accepted {body}"));
        assert_eq!(error.code, "native-capability", "{body}: {}", error.message);
        assert!(error.message.contains("Bytes"));
    }
}

#[test]
fn record_spread_cannot_forge_an_opaque_native_owner() {
    let error = checked_source("import {Bytes} from 'std:bytes';type Empty={};function main(){const empty:Empty={};const bytes:Bytes={...empty};}").err().expect("opaque spread target");
    assert_eq!(error.code, "native-opaque");
    let error = checked_source("import {Bytes} from 'std:bytes';type Empty={};function take(bytes:Bytes):void{}function main(){const empty:Empty={};const copy={...empty};take(copy);}").err().expect("inferred record cannot become Bytes");
    assert_eq!(error.code, "type");
}

#[test]
fn exported_native_aliases_execute_and_preserve_opaque_identity() {
    let directory = support::TempDirectory::new();
    let library = support::format_round_trip("import {Bytes as NativeBytes,ByteError as NativeError} from 'std:bytes';export type Buffer=NativeBytes;export type Failure=NativeError;export type Failures=Failure[];export function make():Failure{return {code:'range',message:'bad'};}");
    std::fs::write(directory.0.join("types.tsr"), library).unwrap();
    let main = support::format_round_trip("import {Buffer,Failure,Failures,make} from './types.tsr';import {Bytes} from 'std:bytes';import {strictEqual} from 'std:testing';function relay(value:Buffer):Bytes{return value;}function report(value:ReadOnly<Failure>):string{return value.code+':'+value.message;}function main(){const errors:Failures=[make()];strictEqual(report(errors[0]),'range:bad');strictEqual(errors[0].offset,undefined);}");
    let path = directory.source(&main);
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
    for (source, code) in [
        ("import {Buffer} from './types.tsr';function main(){const forged:Buffer={};}", "native-opaque"),
        ("import {NativeError} from './types.tsr';function main(){}", "module-private"),
    ] {
        let path = directory.source(&support::format_round_trip(source));
        let output = support::run(support::cli().arg("check").arg(path), 60);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("error[{code}]")), "{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn native_byte_arguments_enforce_nominal_ownership_types_and_arity() {
    let prefix = "import {Bytes,fromArray,length} from 'std:bytes';";
    for (body, code) in [
        ("function main(){fromArray();}", "arity"),
        ("function main(){fromArray([1], [2]);}", "arity"),
        ("function main(){fromArray(['a']);}", "type"),
        ("function main(){length({});}", "native-opaque"),
        ("function main(){match(fromArray([1])){Ok(bytes)=>{const moved=bytes;length(bytes);},Err(error)=>{}}}", "transferred"),
    ] {
        let error = checked_source(&format!("{prefix}{body}")).err().expect(body);
        assert_eq!(error.code, code, "{body}: {error:?}");
    }
}

#[test]
fn native_byte_writes_require_exclusive_access_and_checked_scalar_arguments() {
    let prefix = "import {Bytes,at,set} from 'std:bytes';";
    for (body, code) in [
        ("function change(bytes:ReadOnly<Bytes>){set(bytes,0,1);}", "parameter-access"),
        ("function change(bytes:Bytes){set(bytes,0,'x');}", "type"),
        ("function change(bytes:Bytes){at(bytes);}", "arity"),
        ("function change(bytes:Bytes){const old=bytes;set(bytes,0,1);}", "transferred"),
    ] {
        let error = checked_source(&format!("{prefix}{body}function main(){{}}")).err().expect(body);
        assert_eq!(error.code, code, "{body}: {error:?}");
    }
}

#[test]
fn native_byte_growth_enforces_optional_arity_and_exclusive_access() {
    let prefix = "import {Bytes,alloc,resize} from 'std:bytes';";
    for (body, code) in [
        ("function main(){alloc();}", "arity"),
        ("function main(){alloc(1,0,0);}", "arity"),
        ("function main(){alloc('1');}", "type"),
        ("function main(){alloc(1,'0');}", "type"),
        ("function change(bytes:ReadOnly<Bytes>){resize(bytes,0);}", "parameter-access"),
        ("function change(bytes:Bytes){resize(bytes);}", "arity"),
        ("function change(bytes:Bytes){resize(bytes,0,0,0);}", "arity"),
        ("function change(bytes:Bytes){const moved=bytes;resize(bytes,0);}", "transferred"),
    ] {
        let entry = if body.contains("function main()") { "" } else { "function main(){}" };
        let error = checked_source(&format!("{prefix}{body}{entry}")).err().expect(body);
        assert_eq!(error.code, code, "{body}: {error:?}");
    }
}

#[test]
fn native_byte_append_and_copy_enforce_access_and_scalar_contracts() {
    let prefix = "import {Bytes,append,copy} from 'std:bytes';";
    for (body, code) in [
        ("function change(bytes:ReadOnly<Bytes>){append(bytes,[1]);}", "parameter-access"),
        ("function change(bytes:Bytes){append(bytes,['a']);}", "type"),
        ("function change(bytes:Bytes){append(bytes);}", "arity"),
        ("function change(bytes:Bytes){copy(bytes,0);}", "arity"),
        ("function change(bytes:Bytes){copy(bytes,0,'1');}", "type"),
        ("function change(bytes:Bytes){const moved=bytes;copy(bytes,0,0);}", "transferred"),
    ] {
        let error = checked_source(&format!("{prefix}{body}function main(){{}}")).err().expect(body);
        assert_eq!(error.code, code, "{body}: {error:?}");
    }
}
