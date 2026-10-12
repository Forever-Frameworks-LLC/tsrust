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
    assert!(!bytes.collection_key);
    assert!(bytes.thread_transfer);
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

#[test]
fn opaque_byte_keys_and_value_equality_are_source_diagnostics() {
    for body in [
        "const keys=new Set<Bytes>();",
        "const keys=new Map<Bytes,int>();",
        "const keys=new Set<Row>();",
        "const keys=new Map<Row,int>();",
        "function compare(left:ReadOnly<Bytes>,right:ReadOnly<Bytes>):boolean{return left===right;}",
    ] {
        let source = if body.starts_with("function") {
            format!("import {{Bytes}} from 'std:bytes';{body}function main(){{}}")
        } else {
            format!("import {{Bytes}} from 'std:bytes';type Row={{bytes:Bytes;}};function main(){{{body}}}")
        };
        assert!(checked_source(&source).is_err(), "opaque capability accepted: {body}");
    }
}

#[test]
fn native_utf8_requires_exact_typed_arguments() {
    for (body,code) in [
        ("fromUtf8();","arity"),
        ("fromUtf8('a','b');","arity"),
        ("fromUtf8(65);","type"),
        ("toUtf8();","arity"),
        ("toUtf8('text');","type"),
    ] {
        let error=checked_source(&format!("import {{fromUtf8,toUtf8}} from 'std:bytes';function main(){{{body}}}")).err().expect(body);
        assert_eq!(error.code,code,"{error:?}");
    }
}

#[test]
fn recursive_borrowed_native_projection_keeps_distinct_identities() {
    use tsrust_compiler::{native_type_resolution::{intern, project, linked_name}, Span};
    use tsrust_compiler::type_syntax::ValueAnnotationKind as Kind;
    let span = Span::new(3, 7);
    let mut records = Vec::new();
    let signature = Signature::Result(&Signature::Array(&Signature::Named(NativeTypeId::ByteView)), &Signature::Named(NativeTypeId::ByteError));
    let projected = project(&mut records, &signature, span).unwrap();
    let Kind::Applied { arguments, .. } = projected.kind else { panic!("Result projection"); };
    let Kind::Array(element) = &arguments[0].kind else { panic!("view array projection"); };
    assert!(matches!(&element.kind, Kind::Named(name) if name.text == linked_name(NativeTypeId::ByteView)));
    let view = intern(&mut records, NativeTypeId::ByteView, span).unwrap();
    let mutable = intern(&mut records, NativeTypeId::ByteViewMut, span).unwrap();
    let owned = intern(&mut records, NativeTypeId::Bytes, span).unwrap();
    assert_ne!(view, mutable);
    assert_ne!(view, owned);
    assert_ne!(mutable, owned);
    assert_eq!(records.len(), 4);
    assert_eq!(intern(&mut records, NativeTypeId::ByteView, Span::default()).unwrap(), view);
    assert_eq!(records[view].native_type, Some(NativeTypeId::ByteView));
    assert_eq!(records[mutable].native_type, Some(NativeTypeId::ByteViewMut));
}

#[test]
fn schema_namespace_has_identity_without_selecting_runtime() {
    use tsrust_compiler::native_exports::{lookup, Export, Namespace};
    assert!(matches!(lookup("std:schema", "s").unwrap(), Export::Namespace(Namespace::Schema)));
    assert_eq!(lookup("std:schema", "s").unwrap().module(), "std:schema");
    assert!(lookup("std:schema", "missing").unwrap_err().contains("does not export"));
    assert!(tsrust_compiler::native_bindings::selected_features([("std:schema", "s")]).unwrap().is_empty());
}

#[test]
fn schema_namespace_import_alias_respects_parameter_shadowing() {
    use tsrust_compiler::project::{SourceId, SourceProvider, load_project, load_project_headers, check_graph};
    struct Provider;
    impl SourceProvider for Provider {
        fn resolve(&self, _: Option<&SourceId>, path: &str) -> Result<SourceId, String> { Ok(SourceId(path.into())) }
        fn read(&self, _: &SourceId) -> Result<String, String> {
            Ok(support::format_round_trip("import { s as schema } from 'std:schema'; function identity(schema: int): int { return schema; } function main() { console.log(identity(7)); }"))
        }
    }
    for graph in [load_project(&Provider, "main.tsr"), load_project_headers(&Provider, "main.tsr")] {
        let graph = graph.unwrap();
        let import = &graph.modules[&graph.entry].imports[0];
        assert_eq!(import.native, Some("std:schema"));
        assert_eq!(import.bindings[0].local, "schema");
        assert!(check_graph(graph).is_ok());
    }
}

#[test]
fn schema_string_constructor_infers_output_and_rejects_wrong_schema_type() {
    use tsrust_compiler::project::{SourceId, SourceProvider, load_project, check_graph};
    struct Provider(&'static str);
    impl SourceProvider for Provider {
        fn resolve(&self, _: Option<&SourceId>, path: &str) -> Result<SourceId, String> { Ok(SourceId(path.into())) }
        fn read(&self, _: &SourceId) -> Result<String, String> { Ok(support::format_round_trip(self.0)) }
    }
    for (source, valid) in [
        ("import {s, Schema} from 'std:schema'; function make():Schema<string>{return s.string();} function main(){const value=make();}", true),
        ("import {s as schema, Schema} from 'std:schema'; function make():Schema<string>{return schema.string();} function main(){}", true),
        ("import {s, Schema} from 'std:schema'; function make():Schema<int>{return s.string();} function main(){}", false),
        ("import {s} from 'std:schema'; function main(){s.string(1);}", false),
    ] {
        let graph = load_project(&Provider(source), "main.tsr").unwrap();
        assert_eq!(check_graph(graph).is_ok(), valid, "{source}");
    }
}

#[test]
fn schema_string_constructor_executes_natively() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip("import {s, Schema} from 'std:schema'; function make():Schema<string>{return s.string();} function main(){const value=make(); console.log('schema created');}");
    let path = directory.source(&source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "schema created\n");
}

#[test]
fn schema_safe_parse_consumes_input_and_reuses_schema() {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip("import {s} from 'std:schema'; function main(){const schema=s.string();const input=JSON.parse('\"hello\"');match(schema.safeParse(input)){Ok(text)=>{console.log(text);},Err(error)=>{console.log(error.issues.length);}}match(schema.safeParse(JSON.parse('42'))){Ok(text)=>{console.log(text);},Err(error)=>{console.log(error.issues.length,error.truncated);for(const issue of error.issues){console.log(issue.code,issue.expected,issue.observed,issue.path.length);}}}}");
    let path = directory.source(&source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n1 false\ntype-mismatch string number 0\n");
    let invalid = "import {s} from 'std:schema';function main(){const schema=s.string();const input=JSON.parse('42');schema.safeParse(input);console.log(JSON.stringify(input));}";
    let error = checked_source(invalid).err().expect("input must be consumed even on failure");
    assert_eq!(error.code, "transferred", "{error:?}");
}

#[test]
fn unrelated_safe_parse_method_does_not_link_json_for_schema_constructor() {
    let source = "import {s} from 'std:schema';type Other={safeParse:(value:int)=>int;};function main(){const schema=s.string();const other:Other={safeParse:(value:int):int=>value};console.log(other.safeParse(3));}";
    let checked = checked_source(source).unwrap();
    let (rust, features) = checked.emit_rust_with_native_features();
    assert!(!features.contains(&"json"));
    assert!(!rust.contains("tsrust_native_json"), "unrelated method introduced a JSON runtime");
}

#[test]
fn schema_string_catalog_examples_execute_exact_sources() {
    let catalog: serde_json::Value = serde_json::from_str(include_str!("../../tsrust-lsp/src/builtins.json")).unwrap();
    for family in ["schemaBuilder", "Schema", "SchemaBoolean", "SchemaInt", "SchemaNumber", "SchemaNull", "SchemaArray"] {
        for entry in catalog[family].as_array().unwrap() {
            for example in entry["examples"].as_array().unwrap() {
                let directory = support::TempDirectory::new();
                let source = support::format_round_trip(example["source"].as_str().unwrap());
                let path = directory.source(&source);
                let output = support::run(support::cli().arg("run").arg(path), 60);
                support::success(&output);
                assert_eq!(String::from_utf8_lossy(&output.stdout), example["output"].as_str().unwrap());
            }
        }
    }
}

#[test]
fn schema_input_boundaries_reject_reuse_and_allow_explicit_clone() {
    for literal in ["42", "true", "null", "[]", "{}", "\\\"hello\\\""] {
        let source = format!("import {{s}} from 'std:schema';function main(){{const schema=s.string();const input=JSON.parse('{literal}');schema.safeParse(input);JSON.stringify(input);}}");
        assert_eq!(checked_source(&source).err().expect("reuse must reject").code, "transferred");
        let retained = source.replace("safeParse(input)", "safeParse(clone(input))");
        assert!(checked_source(&retained).is_ok(), "explicit clone: {retained}");
    }
    let invalid = "import {s} from 'std:schema';function main(){s.string().safeParse(42);}";
    assert_eq!(checked_source(invalid).err().expect("unsupported input must diagnose").code, "schema-input");
}

#[test]
fn schema_owned_string_validation_avoids_json_and_preserves_clone_choice() {
    let source="import {s} from 'std:schema';function main(){const schema=s.string();const text='hello'.repeat(2);match(schema.safeParse(clone(text))){Ok(value)=>{console.log(value);},Err(error)=>{throw 'unexpected';}}console.log(text);match(schema.safeParse(text)){Ok(value)=>{console.log(value);},Err(error)=>{throw 'unexpected';}}}";
    let checked=checked_source(source).unwrap();
    let (rust,features)=checked.emit_rust_with_native_features();
    assert!(!features.contains(&"json") && !rust.contains("tsrust_native_json"));
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"hellohello\nhellohello\nhellohello\n");
    let invalid="import {s} from 'std:schema';function main(){const schema=s.string();const text='hello'.repeat(2);schema.safeParse(text);console.log(text);}";
    assert_eq!(checked_source(invalid).err().expect("owned string must transfer").code,"transferred");
}

#[test]
fn schema_validation_cannot_consume_a_borrowed_string() {
    let source="import {s} from 'std:schema';function inspect(input:ReadOnly<string>){const schema=s.string();schema.safeParse(input);}function main(){const input='hello'.repeat(2);inspect(input);console.log(input);}";
    let error=checked_source(source).err().expect("borrowed parameter must not be consumed");
    assert!(error.message.contains("consume") || error.message.contains("transfer") || error.message.contains("ReadOnly"), "{error:?}");
}

#[test]
fn schema_boolean_infers_and_validates_without_truthiness() {
    let source="import {s,Schema} from 'std:schema';function make():Schema<boolean>{return s.boolean();}function main(){const schema=make();const input=false;match(schema.safeParse(input)){Ok(value)=>{console.log(value,input);},Err(error)=>{throw 'unexpected';}}match(schema.safeParse(JSON.parse('true'))){Ok(value)=>{console.log(value);},Err(error)=>{throw 'unexpected';}}match(schema.safeParse(JSON.parse('0'))){Ok(value)=>{throw 'coerced';},Err(error)=>{for(const issue of error.issues){console.log(issue.code,issue.expected,issue.observed);}}}}";
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"false false\ntrue\ntype-mismatch boolean number\n");
    assert!(checked_source("import {s,Schema} from 'std:schema';function make():Schema<string>{return s.boolean();}function main(){}").is_err());
}

#[test]
fn schema_integer_preserves_exact_values_and_checked_errors() {
    let source = r#"import {s,Schema} from 'std:schema';
import {strictEqual} from 'std:testing';
function make():Schema<int>{return s.int();}
function main(){
 const schema=make();
 match(schema.safeParse(JSON.parse('9223372036854775807'))){Ok(value)=>{strictEqual(value,9223372036854775807);},Err(error)=>{throw 'exact integer rejected';}}
 const input=42;
 match(schema.safeParse(input)){Ok(value)=>{strictEqual(value,input);},Err(error)=>{throw 'integer rejected';}}
 for(const text of ['1.5','18446744073709551615','true']){
 match(schema.safeParse(JSON.parse(text))){Ok(value)=>{throw 'invalid accepted';},Err(error)=>{for(const issue of error.issues){console.log(issue.code,issue.expected,issue.observed);}}}
 }
}"#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"integer-fractional integer number\ninteger-range integer number\ntype-mismatch integer boolean\n");
    let direct=checked_source("import {s} from 'std:schema';function main(){s.int().safeParse(42);}").unwrap();
    assert!(!direct.emit_rust_with_native_features().0.contains("tsrust_native_json"));
    let invalid="import {s} from 'std:schema';function main(){const input=JSON.parse('1');s.int().safeParse(input);JSON.stringify(input);}";
    assert_eq!(checked_source(invalid).err().expect("input transfers").code,"transferred");
}

#[test]
fn schema_number_checks_precision_finiteness_and_negative_zero() {
    let source=r#"import {s,Schema} from 'std:schema';import {strictEqual} from 'std:testing';
function make():Schema<number>{return s.number();}
function main(){const schema=make();
 match(schema.safeParse(-0.0)){Ok(value)=>{strictEqual(1.0/value,Number.NEGATIVE_INFINITY);},Err(error)=>{throw 'zero rejected';}}
 match(schema.safeParse(JSON.parse('1.5'))){Ok(value)=>{strictEqual(value,1.5);},Err(error)=>{throw 'fraction rejected';}}
 match(schema.safeParse(JSON.parse('9007199254740993'))){Ok(value)=>{throw 'precision lost';},Err(error)=>{for(const issue of error.issues){strictEqual(issue.code,'number-precision');}}}
 for(const input of [Number.NaN,Number.POSITIVE_INFINITY,Number.NEGATIVE_INFINITY]){match(schema.safeParse(input)){Ok(value)=>{throw 'nonfinite accepted';},Err(error)=>{for(const issue of error.issues){strictEqual(issue.code,'number-nonfinite');strictEqual(issue.expected,'number');strictEqual(issue.observed,'number');}}}}
}"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    let direct=checked_source("import {s} from 'std:schema';function main(){s.number().safeParse(1.5);}").unwrap();
    assert!(!direct.emit_rust_with_native_features().0.contains("tsrust_native_json"));
}

#[test]
fn schema_number_accepts_only_exact_native_integer_conversions() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
function main(){const schema=s.number();
 for(const input of [0,42,9007199254740992,9007199254740994,-9223372036854775808]){
 match(schema.safeParse(input)){Ok(value)=>{strictEqual(value,Number(input));},Err(error)=>{throw 'exact integer rejected';}}
 strictEqual(input===input,true);
 }
 for(const input of [9007199254740993,-9007199254740993,9223372036854775807]){
 match(schema.safeParse(input)){Ok(value)=>{throw 'rounded';},Err(error)=>{for(const issue of error.issues){strictEqual(issue.code,'number-precision');strictEqual(issue.expected,'number');strictEqual(issue.observed,'integer');strictEqual(issue.path.length,0);strictEqual(issue.pathTruncated,false);}strictEqual(error.truncated,false);}}
 }
}"#;
    let checked=checked_source(source).unwrap();
    assert!(!checked.emit_rust_with_native_features().0.contains("tsrust_native_json"));
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn schema_null_distinguishes_null_from_missing_and_other_values() {
    let source=r#"import {s,Schema} from 'std:schema';import {strictEqual} from 'std:testing';
function make():Schema<null>{return s.null();}
function main(){const schema=make();
 match(schema.safeParse(null)){Ok(value)=>{strictEqual(value,null);},Err(error)=>{throw 'null rejected';}}
 match(schema.safeParse(JSON.parse('null'))){Ok(value)=>{strictEqual(value,null);},Err(error)=>{throw 'JSON null rejected';}}
 for(const text of ['false','0','""','[]','{}']){match(schema.safeParse(JSON.parse(text))){Ok(value)=>{throw 'wrong kind accepted';},Err(error)=>{strictEqual(error.issues.length,1);for(const issue of error.issues){strictEqual(issue.code,'type-mismatch');strictEqual(issue.expected,'null');}}}}
}"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    let direct=checked_source("import {s} from 'std:schema';function main(){s.null().safeParse(null);}").unwrap();
    assert!(!direct.emit_rust_with_native_features().0.contains("tsrust_native_json"));
    assert_eq!(checked_source("import {s} from 'std:schema';function main(){s.null().safeParse(undefined);}").err().unwrap().code,"schema-input");
}

#[test]
fn schema_generated_string_plan_enforces_stored_bounds() {
    // Backend injection only: public consuming builders are not implemented yet.
    let calls=["a","😀a","abcd"].map(|input|format!("match(schema.safeParse('{input}')){{Ok(value)=>{{console.log(value);}},Err(error)=>{{for(const issue of error.issues){{console.log(issue.code);}}}}}}" )).join("");
    let source=format!("import {{s}} from 'std:schema';function main(){{const schema=s.string();{calls}}}");
    let checked=checked_source(&source).unwrap();let (rust,features)=checked.emit_rust_with_native_features();
    assert!(features.is_empty());
    let marker="bounds: tsrust_schema_plan::StringBounds::default()";
    assert_eq!(rust.matches(marker).count(),1);
    let rust=rust.replace(marker,"bounds: tsrust_schema_plan::StringBounds::default().min(2).unwrap().max(3).unwrap()");
    let directory=support::TempDirectory::new();let file=directory.0.join("schema.rs");let binary=directory.0.join("schema");
    std::fs::write(&file,rust).unwrap();
    support::success(&support::run(support::rustc().arg("--edition=2024").arg(file).arg("-o").arg(&binary),60));
    let output=support::run(&mut std::process::Command::new(binary),10);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"too-small\n😀a\ntoo-large\n");
}

#[test]
fn schema_string_builders_consume_and_throw_structured_errors() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
function main(){const schema=s.string().min(2).max(3);
 match(schema.safeParse('😀a')){Ok(value)=>{strictEqual(value,'😀a');},Err(error)=>{throw 'valid rejected';}}
 match(schema.safeParse(JSON.parse('"abcd"'))){Ok(value)=>{throw 'long accepted';},Err(error)=>{for(const issue of error.issues){strictEqual(issue.code,'too-large');}}}
 let caught=false;try{s.string().min(-1);}catch(error){caught=true;strictEqual(error.code,'negative-length');strictEqual(error.operation,'min');strictEqual(error.limit,-1);}strictEqual(caught,true);
 caught=false;try{s.string().max(1).min(2);}catch(error){caught=true;strictEqual(error.code,'contradictory-bounds');strictEqual(error.operation,'min');strictEqual(error.limit,2);}strictEqual(caught,true);
}"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    let invalid="import {s} from 'std:schema';function main(){const original=s.string();const bounded=original.min(2);original.safeParse('abc');}";
    assert_eq!(checked_source(invalid).err().unwrap().code,"transferred");
}

#[test]
fn schema_integer_bounds_preserve_endpoints_and_reject_contradictions() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
function main(){const schema=s.int().min(9223372036854775806).max(9223372036854775807);
 match(schema.safeParse(9223372036854775807)){Ok(value)=>{strictEqual(value,9223372036854775807);},Err(error)=>{throw 'endpoint rejected';}}
 match(schema.safeParse(JSON.parse('9223372036854775805'))){Ok(value)=>{throw 'small accepted';},Err(error)=>{for(const issue of error.issues){strictEqual(issue.code,'too-small');}}}
 const low=s.int().min(-9223372036854775808).max(-9223372036854775807);
 match(low.safeParse(-9223372036854775808)){Ok(value)=>{strictEqual(value,-9223372036854775808);},Err(error)=>{throw 'low endpoint rejected';}}
 let caught=false;try{s.int().max(-2).min(-1);}catch(error){caught=true;strictEqual(error.code,'contradictory-bounds');strictEqual(error.limit,-1);}strictEqual(caught,true);
}"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    assert_eq!(checked_source("import {s} from 'std:schema';function main(){const original=s.int();original.min(0);original.safeParse(1);}").err().unwrap().code,"transferred");
}

#[test]
fn schema_generated_exclusive_integer_plan_preserves_exact_limits() {
    // Inject the backend plan until the public exclusive builders are integrated.
    let source=r#"import {s} from 'std:schema';
function main(){const schema=s.int();
 for(const input of [9007199254740992,9007199254740993,9007199254740994]){
  match(schema.safeParse(input)){Ok(value)=>{console.log(value);},Err(error)=>{for(const issue of error.issues){console.log(issue.code);}}}
 }
}"#;
    let source=support::format_round_trip(source);
    let checked=checked_source(&source).unwrap();let (rust,features)=checked.emit_rust_with_native_features();
    assert!(features.is_empty());
    let marker="bounds: tsrust_schema_plan::IntBounds::default()";
    assert_eq!(rust.matches(marker).count(),1);
    let rust=rust.replace(marker,"bounds: tsrust_schema_plan::IntBounds::default().gt(9007199254740992).unwrap().lt(9007199254740994).unwrap()");
    let directory=support::TempDirectory::new();let file=directory.0.join("schema.rs");let binary=directory.0.join("schema");
    std::fs::write(&file,rust).unwrap();
    support::success(&support::run(support::rustc().arg("--edition=2024").arg(file).arg("-o").arg(&binary),60));
    let output=support::run(&mut std::process::Command::new(binary),10);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"too-small\n9007199254740993\ntoo-large\n");
}

#[test]
fn schema_exclusive_integer_builders_consume_and_throw() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
function main(){const schema=s.int().gt(9007199254740992).lt(9007199254740994);
 match(schema.safeParse(9007199254740993)){Ok(value)=>{strictEqual(value,9007199254740993);},Err(error)=>{throw 'exact rejected';}}
 match(schema.safeParse(JSON.parse('9007199254740994'))){Ok(value)=>{throw 'endpoint accepted';},Err(error)=>{for(const issue of error.issues){strictEqual(issue.code,'too-large');}}}
 let caught=false;try{s.int().gt(9223372036854775807);}catch(error){caught=true;strictEqual(error.code,'contradictory-bounds');strictEqual(error.operation,'gt');strictEqual(error.limit,9223372036854775807);}strictEqual(caught,true);
 caught=false;try{s.int().lt(-9223372036854775808);}catch(error){caught=true;strictEqual(error.operation,'lt');strictEqual(error.limit,-9223372036854775808);}strictEqual(caught,true);
 caught=false;try{s.int().gt(2).lt(3);}catch(error){caught=true;strictEqual(error.operation,'lt');strictEqual(error.limit,3);}strictEqual(caught,true);
}"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    for method in ["gt","lt"] {
        let source=format!("import {{s}} from 'std:schema';function main(){{const original=s.int();original.{method}(0);original.safeParse(1);}}");
        assert_eq!(checked_source(&source).err().unwrap().code,"transferred");
    }
    assert_eq!(checked_source("import {s} from 'std:schema';function main(){s.string().gt(0);}").err().unwrap().code,"schema-member");
}

#[test]
fn schema_generated_number_bounds_check_direct_and_integer_input() {
    let source=r#"import {s} from 'std:schema';function main(){const schema=s.number();
 for(const input of [1.0,1.5,2.0]){match(schema.safeParse(input)){Ok(value)=>{console.log(value);},Err(error)=>{for(const issue of error.issues){console.log(issue.code);}}}}
 match(schema.safeParse(1)){Ok(value)=>{throw 'integer bound bypass';},Err(error)=>{for(const issue of error.issues){console.log(issue.code);console.log(issue.observed);}}}
 match(schema.safeParse(9007199254740993)){Ok(value)=>{throw 'precision bypass';},Err(error)=>{for(const issue of error.issues){console.log(issue.code);}}}
}"#;
    let checked=checked_source(&support::format_round_trip(source)).unwrap();
    let (rust,features)=checked.emit_rust_with_native_features();assert!(features.is_empty());
    let marker="bounds: tsrust_schema_plan::NumberBounds::default()";
    assert_eq!(rust.matches(marker).count(),1);
    let rust=rust.replace(marker,"bounds: tsrust_schema_plan::NumberBounds::default().gt(1.0).unwrap().lt(2.0).unwrap()");
    let directory=support::TempDirectory::new();let file=directory.0.join("schema.rs");let binary=directory.0.join("schema");
    std::fs::write(&file,rust).unwrap();
    support::success(&support::run(support::rustc().arg("--edition=2024").arg(file).arg("-o").arg(&binary),60));
    let output=support::run(&mut std::process::Command::new(binary),10);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"too-small\n1.5\ntoo-large\ntoo-small\ninteger\nnumber-precision\n");
}

#[test]
fn schema_construction_error_preserves_integer_and_float_payloads() {
    let source=r#"import {SchemaConstructionError} from 'std:schema';
function report(error:SchemaConstructionError){console.log(error.limit);}
function main(){
 const integer:SchemaConstructionError={code:'contradictory-bounds',operation:'gt',limit:9223372036854775807};report(integer);
 const floating:SchemaConstructionError={code:'contradictory-bounds',operation:'lt',limit:1.5};report(floating);
}"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"9223372036854775807\n1.5\n");
}

#[test]
fn schema_floating_builders_validate_and_throw_typed_limits() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
function main(){const schema=s.number().min(0.0).max(3.0).gt(1.0).lt(2.0);
 match(schema.safeParse(1.5)){Ok(value)=>{strictEqual(value,1.5);},Err(error)=>{throw 'fraction rejected';}}
 match(schema.safeParse(JSON.parse('2.0'))){Ok(value)=>{throw 'endpoint accepted';},Err(error)=>{for(const issue of error.issues){strictEqual(issue.code,'too-large');}}}
 match(schema.safeParse(1)){Ok(value)=>{throw 'integer endpoint accepted';},Err(error)=>{for(const issue of error.issues){strictEqual(issue.code,'too-small');}}}
 let caught=false;try{s.number().min(2.5).max(1.5);}catch(error){caught=true;strictEqual(error.code,'contradictory-bounds');strictEqual(error.operation,'max');strictEqual(error.limit,1.5);}strictEqual(caught,true);
 caught=false;try{s.number().gt(1.0/0.0);}catch(error){caught=true;strictEqual(error.code,'nonfinite-limit');strictEqual(error.operation,'gt');}strictEqual(caught,true);
}"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    for method in ["min","max","gt","lt"] {
        let source=format!("import {{s}} from 'std:schema';function main(){{const original=s.number();original.{method}(0.0);original.safeParse(1.0);}}");
        assert_eq!(checked_source(&source).err().unwrap().code,"transferred");
    }
}

#[test]
fn schema_array_construction_moves_child_plan_and_keeps_type() {
    let source="import {s,Schema} from 'std:schema';function make():Schema<string[][]>{const child=s.string().min(2);const array=s.array(child);return s.array(array);}function main(){const schema=make();}";
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    let invalid="import {s} from 'std:schema';function main(){const child=s.string();s.array(child);child.safeParse('x');}";
    assert_eq!(checked_source(invalid).err().unwrap().code,"transferred");
}

#[test]
fn schema_generated_issue_bridge_preserves_bounded_errors() {
    let checked=checked_source("import {s} from 'std:schema';function main(){s.string().safeParse('ok');}").unwrap();
    let (rust,features)=checked.emit_rust_with_native_features();assert!(features.is_empty());
    assert!(rust.contains("fn tsrust_schema_error("));
    let mut rust=rust.replace("fn main()", "fn original_main()");
    rust.push_str(r#"
fn main(){
 use tsrust_schema_issues::{Issues,Segment,Code,Kind};
 let mut errors=Issues::new(1,1,8);
 errors.push(&[Segment::Index(3),Segment::Index(4)],Code::Type,Kind::String,Kind::Boolean);
 errors.push(&[],Code::Missing,Kind::Integer,Kind::Missing);
 let result=tsrust_schema_error(errors);
 assert!(result.field1);assert_eq!(result.field0.len(),1);
 let issue=&result.field0[0];
 assert_eq!(issue.field0.as_ref(),"type-mismatch");
 assert_eq!(issue.field1.len(),1);assert!(issue.field2);
 assert_eq!(issue.field3.as_ref(),"string");assert_eq!(issue.field4.as_ref(),"boolean");
 let mut errors=Issues::new(1,4,3);
 errors.push(&[Segment::Field("é😀secret"),Segment::Index(2)],Code::Missing,Kind::Integer,Kind::Missing);
 let result=tsrust_schema_error(errors);
 assert!(!result.field1);assert!(result.field0[0].field2);
 assert_eq!(result.field0[0].field1.len(),1);
 assert_eq!(result.field0[0].field0.as_ref(),"missing-field");
 assert_eq!(result.field0[0].field4.as_ref(),"missing");
 let mut errors=Issues::new(0,0,0);
 errors.push(&[],Code::Type,Kind::Array,Kind::Object);
 let result=tsrust_schema_error(errors);
 assert!(result.field1);assert!(result.field0.is_empty());
}
"#);
    let directory=support::TempDirectory::new();let file=directory.0.join("schema.rs");let binary=directory.0.join("schema");
    std::fs::write(&file,rust).unwrap();
    support::success(&support::run(support::rustc().arg("--edition=2024").arg(file).arg("-o").arg(&binary),60));
    support::success(&support::run(&mut std::process::Command::new(binary),10));
}

#[test]
fn schema_owned_array_validation_reuses_plan_and_consumes_input() {
    let source="import {s} from 'std:schema';function main(){const schema=s.array(s.int().min(2));const input=[2,3];match(schema.safeParse(input)){Ok(value)=>{console.log(value.length);},Err(error)=>{throw 'unexpected';}}match(schema.safeParse([1,0])){Ok(value)=>{throw 'unexpected';},Err(error)=>{console.log(error.issues.length);for(const issue of error.issues){console.log(issue.code);}}}}";
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"2\n2\ntoo-small\ntoo-small\n");
    let invalid="import {s} from 'std:schema';function main(){const schema=s.array(s.int());const input=[1,2];schema.safeParse(input);console.log(input.length);}";
    assert_eq!(checked_source(invalid).err().unwrap().code,"transferred");
}

#[test]
fn schema_owned_array_validation_handles_nested_and_scalar_storage() {
    let source=r#"import {s} from 'std:schema';function main(){
      match(s.array(s.int()).safeParse([5])){Ok(value)=>{console.log(value.length);},Err(error)=>{throw 'unexpected';}}
      const rows=s.array(s.array(s.string().min(2)));
      match(rows.safeParse([['hello','😀a']])){Ok(value)=>{console.log(value.length);},Err(error)=>{throw 'unexpected';}}
      match(rows.safeParse([['x']])){Ok(value)=>{throw 'unexpected';},Err(error)=>{for(const issue of error.issues){console.log(issue.path.length,issue.code);}}}
      const flags=s.array(s.boolean());match(flags.safeParse([true,false])){Ok(value)=>{console.log(value.length);},Err(error)=>{throw 'unexpected';}}
      const numbers=s.array(s.number().gt(0.0));match(numbers.safeParse([1.5,2.5])){Ok(value)=>{console.log(value.length);},Err(error)=>{throw 'unexpected';}}
      const nulls=s.array(s.null());match(nulls.safeParse([null,null])){Ok(value)=>{console.log(value.length);},Err(error)=>{throw 'unexpected';}}
      const positive=s.array(s.int().min(1));let invalid:int[]=[];let i=0;while(i<65){invalid.push(0);i++;}
      match(positive.safeParse(invalid)){Ok(value)=>{throw 'unexpected';},Err(error)=>{console.log(error.issues.length,error.truncated);}}
      const empty:int[]=[];match(positive.safeParse(empty)){Ok(value)=>{console.log(value.length);},Err(error)=>{throw 'unexpected';}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"1\n1\n2 too-small\n2\n2\n2\n64 true\n0\n");
}

#[test]
fn schema_json_arrays_preserve_exact_values_and_indexed_failures() {
    let source=r#"import {s} from 'std:schema';function main(){
      const flags=s.array(s.boolean());match(flags.safeParse(JSON.parse('[true,false]'))){Ok(values)=>{console.log(values.length);},Err(error)=>{throw 'boolean failure';}}
      const nulls=s.array(s.null());match(nulls.safeParse(JSON.parse('[null]'))){Ok(values)=>{console.log(values.length);},Err(error)=>{throw 'null failure';}}
      const texts=s.array(s.string());match(texts.safeParse(JSON.parse('["hello","😀"]'))){Ok(values)=>{for(const value of values){console.log(value);}},Err(error)=>{throw 'string failure';}}
      const fractions=s.array(s.number().min(1.0));match(fractions.safeParse(JSON.parse('[1.5]'))){Ok(values)=>{for(const value of values){console.log(value);}},Err(error)=>{throw 'number failure';}}
      const integers=s.array(s.int());
      match(integers.safeParse(JSON.parse('[9223372036854775807,-9223372036854775808]'))){Ok(values)=>{for(const value of values){console.log(value);}},Err(error)=>{throw 'integer range';}}
      match(integers.safeParse(JSON.parse('[1.5,18446744073709551615,true]'))){Ok(values)=>{throw 'invalid accepted';},Err(error)=>{for(const issue of error.issues){console.log(issue.code,issue.observed,issue.path.length);}}}
      const rows=s.array(s.array(s.string().min(2)));
      match(rows.safeParse(JSON.parse('[["yes"]]'))){Ok(values)=>{console.log(values.length);},Err(error)=>{throw 'nested success';}}
      match(rows.safeParse(JSON.parse('[["hello"],["x",true],null]'))){Ok(values)=>{throw 'invalid accepted';},Err(error)=>{for(const issue of error.issues){console.log(issue.code,issue.expected,issue.observed,issue.path.length);}}}
      const numbers=s.array(s.number());
      match(numbers.safeParse(JSON.parse('[9007199254740993]'))){Ok(values)=>{throw 'precision lost';},Err(error)=>{for(const issue of error.issues){console.log(issue.code);}}}
      match(integers.safeParse(JSON.parse('{}'))){Ok(values)=>{throw 'object accepted';},Err(error)=>{for(const issue of error.issues){console.log(issue.expected,issue.observed,issue.path.length);}}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"2\n1\nhello\n😀\n1.5\n9223372036854775807\n-9223372036854775808\ninteger-fractional number 1\ninteger-range number 1\ntype-mismatch boolean 1\n1\ntoo-small string string 2\ntype-mismatch string boolean 2\ntype-mismatch array null 1\nnumber-precision\narray object 0\n");
    let invalid="import {s} from 'std:schema';function main(){const schema=s.array(s.int());const input=JSON.parse('[1]');schema.safeParse(input);console.log(typeof input);}";
    assert_eq!(checked_source(invalid).err().unwrap().code,"transferred");

}
