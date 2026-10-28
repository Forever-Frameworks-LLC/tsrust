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
        fn read(&self, id: &SourceId) -> Result<String, String> {
            if id.0 != "main.tsr" { return Err(format!("Missing source {}",id.0)); }
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
    let Kind::Applied { name, arguments } = annotation.into_kind() else { panic!("Result annotation"); };
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
        fn read(&self, id: &SourceId) -> Result<String, String> {
            if id.0 != "main.tsr" { return Err(format!("Missing source {}",id.0)); } Ok(self.0.into()) }
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
    let Kind::Applied { arguments, .. } = projected.into_kind() else { panic!("Result projection"); };
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
        fn read(&self, id: &SourceId) -> Result<String, String> {
            if id.0 != "main.tsr" { return Err(format!("Missing source {}",id.0)); }
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
        fn read(&self, id: &SourceId) -> Result<String, String> {
            if id.0 != "main.tsr" { return Err(format!("Missing source {}",id.0)); } Ok(support::format_round_trip(self.0)) }
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
    let source = support::format_round_trip("import {s} from 'std:schema'; function main(){const schema=s.string();const input=JSON.parse('\"hello\"');{const outcome0=schema.safeParse(input);if(outcome0.success){const text=outcome0.data;console.log(text);}else{const error=outcome0.error;console.log(error.issues.length);}}{const outcome1=schema.safeParse(JSON.parse('42'));if(outcome1.success){const text=outcome1.data;console.log(text);}else{const error=outcome1.error;console.log(error.issues.length,error.truncated);for(const issue of error.issues){console.log(issue.code,issue.expected,issue.observed,issue.path.length);}}}}");
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
    let mut seen=std::collections::BTreeMap::new();
    for family in ["schemaBuilder", "Schema", "SchemaBoolean", "SchemaInt", "SchemaNumber", "SchemaNull", "SchemaArray", "SchemaObject", "SchemaRetained", "SchemaPresence"] {
        for entry in catalog[family].as_array().unwrap() {
            for example in entry["examples"].as_array().unwrap() {
                let text=example["source"].as_str().unwrap();let expected=example["output"].as_str().unwrap();
                if let Some(previous)=seen.insert(text,expected){assert_eq!(previous,expected,"shared example output contract");continue;}
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
    let source="import {s} from 'std:schema';function main(){const schema=s.string();const text='hello'.repeat(2);{const outcome2=schema.safeParse(clone(text));if(outcome2.success){const value=outcome2.data;console.log(value);}else{const error=outcome2.error;throw 'unexpected';}}console.log(text);{const outcome3=schema.safeParse(text);if(outcome3.success){const value=outcome3.data;console.log(value);}else{const error=outcome3.error;throw 'unexpected';}}}";
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
    let source="import {s,Schema} from 'std:schema';function make():Schema<boolean>{return s.boolean();}function main(){const schema=make();const input=false;{const outcome4=schema.safeParse(input);if(outcome4.success){const value=outcome4.data;console.log(value,input);}else{const error=outcome4.error;throw 'unexpected';}}{const outcome5=schema.safeParse(JSON.parse('true'));if(outcome5.success){const value=outcome5.data;console.log(value);}else{const error=outcome5.error;throw 'unexpected';}}{const outcome6=schema.safeParse(JSON.parse('0'));if(outcome6.success){const value=outcome6.data;throw 'coerced';}else{const error=outcome6.error;for(const issue of error.issues){console.log(issue.code,issue.expected,issue.observed);}}}}";
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
 {const outcome7=schema.safeParse(JSON.parse('9223372036854775807'));if(outcome7.success){const value=outcome7.data;strictEqual(value,9223372036854775807);}else{const error=outcome7.error;throw 'exact integer rejected';}}
 const input=42;
 {const outcome8=schema.safeParse(input);if(outcome8.success){const value=outcome8.data;strictEqual(value,input);}else{const error=outcome8.error;throw 'integer rejected';}}
 for(const text of ['1.5','18446744073709551615','true']){
 {const outcome9=schema.safeParse(JSON.parse(text));if(outcome9.success){const value=outcome9.data;throw 'invalid accepted';}else{const error=outcome9.error;for(const issue of error.issues){console.log(issue.code,issue.expected,issue.observed);}}}
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
 {const outcome10=schema.safeParse(-0.0);if(outcome10.success){const value=outcome10.data;strictEqual(1.0/value,Number.NEGATIVE_INFINITY);}else{const error=outcome10.error;throw 'zero rejected';}}
 {const outcome11=schema.safeParse(JSON.parse('1.5'));if(outcome11.success){const value=outcome11.data;strictEqual(value,1.5);}else{const error=outcome11.error;throw 'fraction rejected';}}
 {const outcome12=schema.safeParse(JSON.parse('9007199254740993'));if(outcome12.success){const value=outcome12.data;throw 'precision lost';}else{const error=outcome12.error;for(const issue of error.issues){strictEqual(issue.code,'number-precision');}}}
 for(const input of [Number.NaN,Number.POSITIVE_INFINITY,Number.NEGATIVE_INFINITY]){{const outcome13=schema.safeParse(input);if(outcome13.success){const value=outcome13.data;throw 'nonfinite accepted';}else{const error=outcome13.error;for(const issue of error.issues){strictEqual(issue.code,'number-nonfinite');strictEqual(issue.expected,'number');strictEqual(issue.observed,'number');}}}}
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
 {const outcome14=schema.safeParse(input);if(outcome14.success){const value=outcome14.data;strictEqual(value,Number(input));}else{const error=outcome14.error;throw 'exact integer rejected';}}
 strictEqual(input===input,true);
 }
 for(const input of [9007199254740993,-9007199254740993,9223372036854775807]){
 {const outcome15=schema.safeParse(input);if(outcome15.success){const value=outcome15.data;throw 'rounded';}else{const error=outcome15.error;for(const issue of error.issues){strictEqual(issue.code,'number-precision');strictEqual(issue.expected,'number');strictEqual(issue.observed,'integer');strictEqual(issue.path.length,0);strictEqual(issue.pathTruncated,false);}strictEqual(error.truncated,false);}}
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
 {const outcome16=schema.safeParse(null);if(outcome16.success){const value=outcome16.data;strictEqual(value,null);}else{const error=outcome16.error;throw 'null rejected';}}
 {const outcome17=schema.safeParse(JSON.parse('null'));if(outcome17.success){const value=outcome17.data;strictEqual(value,null);}else{const error=outcome17.error;throw 'JSON null rejected';}}
 for(const text of ['false','0','""','[]','{}']){{const outcome18=schema.safeParse(JSON.parse(text));if(outcome18.success){const value=outcome18.data;throw 'wrong kind accepted';}else{const error=outcome18.error;strictEqual(error.issues.length,1);for(const issue of error.issues){strictEqual(issue.code,'type-mismatch');strictEqual(issue.expected,'null');}}}}
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
    let calls=["a","😀a","abcd"].map(|input|format!("{{const result=schema.safeParse('{input}');if(result.success){{console.log(result.data);}}else{{for(const issue of result.error.issues){{console.log(issue.code);}}}}}}" )).join("");
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
 {const outcome19=schema.safeParse('😀a');if(outcome19.success){const value=outcome19.data;strictEqual(value,'😀a');}else{const error=outcome19.error;throw 'valid rejected';}}
 {const outcome20=schema.safeParse(JSON.parse('"abcd"'));if(outcome20.success){const value=outcome20.data;throw 'long accepted';}else{const error=outcome20.error;for(const issue of error.issues){strictEqual(issue.code,'too-large');}}}
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
 {const outcome21=schema.safeParse(9223372036854775807);if(outcome21.success){const value=outcome21.data;strictEqual(value,9223372036854775807);}else{const error=outcome21.error;throw 'endpoint rejected';}}
 {const outcome22=schema.safeParse(JSON.parse('9223372036854775805'));if(outcome22.success){const value=outcome22.data;throw 'small accepted';}else{const error=outcome22.error;for(const issue of error.issues){strictEqual(issue.code,'too-small');}}}
 const low=s.int().min(-9223372036854775808).max(-9223372036854775807);
 {const outcome23=low.safeParse(-9223372036854775808);if(outcome23.success){const value=outcome23.data;strictEqual(value,-9223372036854775808);}else{const error=outcome23.error;throw 'low endpoint rejected';}}
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
  {const outcome24=schema.safeParse(input);if(outcome24.success){const value=outcome24.data;console.log(value);}else{const error=outcome24.error;for(const issue of error.issues){console.log(issue.code);}}}
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
 {const outcome25=schema.safeParse(9007199254740993);if(outcome25.success){const value=outcome25.data;strictEqual(value,9007199254740993);}else{const error=outcome25.error;throw 'exact rejected';}}
 {const outcome26=schema.safeParse(JSON.parse('9007199254740994'));if(outcome26.success){const value=outcome26.data;throw 'endpoint accepted';}else{const error=outcome26.error;for(const issue of error.issues){strictEqual(issue.code,'too-large');}}}
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
 for(const input of [1.0,1.5,2.0]){{const outcome27=schema.safeParse(input);if(outcome27.success){const value=outcome27.data;console.log(value);}else{const error=outcome27.error;for(const issue of error.issues){console.log(issue.code);}}}}
 {const outcome28=schema.safeParse(1);if(outcome28.success){const value=outcome28.data;throw 'integer bound bypass';}else{const error=outcome28.error;for(const issue of error.issues){console.log(issue.code);console.log(issue.observed);}}}
 {const outcome29=schema.safeParse(9007199254740993);if(outcome29.success){const value=outcome29.data;throw 'precision bypass';}else{const error=outcome29.error;for(const issue of error.issues){console.log(issue.code);}}}
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
 {const outcome30=schema.safeParse(1.5);if(outcome30.success){const value=outcome30.data;strictEqual(value,1.5);}else{const error=outcome30.error;throw 'fraction rejected';}}
 {const outcome31=schema.safeParse(JSON.parse('2.0'));if(outcome31.success){const value=outcome31.data;throw 'endpoint accepted';}else{const error=outcome31.error;for(const issue of error.issues){strictEqual(issue.code,'too-large');}}}
 {const outcome32=schema.safeParse(1);if(outcome32.success){const value=outcome32.data;throw 'integer endpoint accepted';}else{const error=outcome32.error;for(const issue of error.issues){strictEqual(issue.code,'too-small');}}}
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
    let source="import {s} from 'std:schema';function main(){const schema=s.array(s.int().min(2));const input=[2,3];{const outcome33=schema.safeParse(input);if(outcome33.success){const value=outcome33.data;console.log(value.length);}else{const error=outcome33.error;throw 'unexpected';}}{const outcome34=schema.safeParse([1,0]);if(outcome34.success){const value=outcome34.data;throw 'unexpected';}else{const error=outcome34.error;console.log(error.issues.length);for(const issue of error.issues){console.log(issue.code);}}}}";
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
      {const outcome35=s.array(s.int()).safeParse([5]);if(outcome35.success){const value=outcome35.data;console.log(value.length);}else{const error=outcome35.error;throw 'unexpected';}}
      const rows=s.array(s.array(s.string().min(2)));
      {const outcome36=rows.safeParse([['hello','😀a']]);if(outcome36.success){const value=outcome36.data;console.log(value.length);}else{const error=outcome36.error;throw 'unexpected';}}
      {const outcome37=rows.safeParse([['x']]);if(outcome37.success){const value=outcome37.data;throw 'unexpected';}else{const error=outcome37.error;for(const issue of error.issues){console.log(issue.path.length,issue.code);}}}
      const flags=s.array(s.boolean());{const outcome38=flags.safeParse([true,false]);if(outcome38.success){const value=outcome38.data;console.log(value.length);}else{const error=outcome38.error;throw 'unexpected';}}
      const numbers=s.array(s.number().gt(0.0));{const outcome39=numbers.safeParse([1.5,2.5]);if(outcome39.success){const value=outcome39.data;console.log(value.length);}else{const error=outcome39.error;throw 'unexpected';}}
      const nulls=s.array(s.null());{const outcome40=nulls.safeParse([null,null]);if(outcome40.success){const value=outcome40.data;console.log(value.length);}else{const error=outcome40.error;throw 'unexpected';}}
      const positive=s.array(s.int().min(1));let invalid:int[]=[];let i=0;while(i<65){invalid.push(0);i++;}
      {const outcome41=positive.safeParse(invalid);if(outcome41.success){const value=outcome41.data;throw 'unexpected';}else{const error=outcome41.error;console.log(error.issues.length,error.truncated);}}
      const empty:int[]=[];{const outcome42=positive.safeParse(empty);if(outcome42.success){const value=outcome42.data;console.log(value.length);}else{const error=outcome42.error;throw 'unexpected';}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"1\n1\n2 too-small\n2\n2\n2\n64 true\n0\n");
}

#[test]
fn schema_json_arrays_preserve_exact_values_and_indexed_failures() {
    let source=r#"import {s} from 'std:schema';function main(){
      const flags=s.array(s.boolean());{const outcome43=flags.safeParse(JSON.parse('[true,false]'));if(outcome43.success){const values=outcome43.data;console.log(values.length);}else{const error=outcome43.error;throw 'boolean failure';}}
      const nulls=s.array(s.null());{const outcome44=nulls.safeParse(JSON.parse('[null]'));if(outcome44.success){const values=outcome44.data;console.log(values.length);}else{const error=outcome44.error;throw 'null failure';}}
      const texts=s.array(s.string());{const outcome45=texts.safeParse(JSON.parse('["hello","😀"]'));if(outcome45.success){const values=outcome45.data;for(const value of values){console.log(value);}}else{const error=outcome45.error;throw 'string failure';}}
      const fractions=s.array(s.number().min(1.0));{const outcome46=fractions.safeParse(JSON.parse('[1.5]'));if(outcome46.success){const values=outcome46.data;for(const value of values){console.log(value);}}else{const error=outcome46.error;throw 'number failure';}}
      const integers=s.array(s.int());
      {const outcome47=integers.safeParse(JSON.parse('[9223372036854775807,-9223372036854775808]'));if(outcome47.success){const values=outcome47.data;for(const value of values){console.log(value);}}else{const error=outcome47.error;throw 'integer range';}}
      {const outcome48=integers.safeParse(JSON.parse('[1.5,18446744073709551615,true]'));if(outcome48.success){const values=outcome48.data;throw 'invalid accepted';}else{const error=outcome48.error;for(const issue of error.issues){console.log(issue.code,issue.observed,issue.path.length);}}}
      const rows=s.array(s.array(s.string().min(2)));
      {const outcome49=rows.safeParse(JSON.parse('[["yes"]]'));if(outcome49.success){const values=outcome49.data;console.log(values.length);}else{const error=outcome49.error;throw 'nested success';}}
      {const outcome50=rows.safeParse(JSON.parse('[["hello"],["x",true],null]'));if(outcome50.success){const values=outcome50.data;throw 'invalid accepted';}else{const error=outcome50.error;for(const issue of error.issues){console.log(issue.code,issue.expected,issue.observed,issue.path.length);}}}
      const numbers=s.array(s.number());
      {const outcome51=numbers.safeParse(JSON.parse('[9007199254740993]'));if(outcome51.success){const values=outcome51.data;throw 'precision lost';}else{const error=outcome51.error;for(const issue of error.issues){console.log(issue.code);}}}
      {const outcome52=integers.safeParse(JSON.parse('{}'));if(outcome52.success){const values=outcome52.data;throw 'object accepted';}else{const error=outcome52.error;for(const issue of error.issues){console.log(issue.expected,issue.observed,issue.path.length);}}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"2\n1\nhello\n😀\n1.5\n9223372036854775807\n-9223372036854775808\ninteger-fractional number 1\ninteger-range number 1\ntype-mismatch boolean 1\n1\ntoo-small string string 2\ntype-mismatch string boolean 2\ntype-mismatch array null 1\nnumber-precision\narray object 0\n");
    let invalid="import {s} from 'std:schema';function main(){const schema=s.array(s.int());const input=JSON.parse('[1]');schema.safeParse(input);console.log(typeof input);}";
    assert_eq!(checked_source(invalid).err().unwrap().code,"transferred");

}

#[test]
fn schema_number_arrays_check_integer_precision_before_bounds() {
    let source=r#"import {s} from 'std:schema';function main(){
      const schema=s.array(s.number());
      {const outcome53=schema.safeParse([9007199254740993]);if(outcome53.success){const values=outcome53.data;throw 'literal rounded';}else{const error=outcome53.error;for(const issue of error.issues){console.log(issue.code);}}}
      const exact:int[]=[42,-9223372036854775808];
      {const outcome54=schema.safeParse(exact);if(outcome54.success){const values=outcome54.data;console.log(values.length);}else{const error=outcome54.error;throw 'exact rejected';}}
      const invalid:int[]=[9007199254740993,9223372036854775807];
      {const outcome55=schema.safeParse(invalid);if(outcome55.success){const values=outcome55.data;throw 'rounded';}else{const error=outcome55.error;for(const issue of error.issues){console.log(issue.code,issue.observed,issue.path.length);}}}
      const nested=s.array(s.array(s.number().max(10.0)));const input:int[][]=[[42,9007199254740993]];
      {const outcome56=nested.safeParse(input);if(outcome56.success){const values=outcome56.data;throw 'invalid accepted';}else{const error=outcome56.error;for(const issue of error.issues){console.log(issue.code,issue.observed,issue.path.length);}}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"number-precision\n2\nnumber-precision integer 1\nnumber-precision integer 1\ntoo-large integer 2\nnumber-precision integer 2\n");
    let invalid="import {s} from 'std:schema';function main(){const schema=s.array(s.number());const values:int[]=[1];schema.safeParse(values);console.log(values.length);}";
    assert_eq!(checked_source(invalid).err().unwrap().code,"transferred");
}

#[test]
fn schema_object_infers_owned_fields_and_rejects_unknown_json() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
    function main(){
      const name=s.string().min(2);
      const schema=s.object({name:name,age:s.int().min(1),tags:s.array(s.string())});
      {const outcome57=schema.safeParse(JSON.parse('{"name":"Ada","age":37,"tags":["native"]}'));if(outcome57.success){const value=outcome57.data;strictEqual(value.name,'Ada');strictEqual(value.age,37);strictEqual(value.tags.length,1);}else{const error=outcome57.error;throw 'valid rejected';}}
      {const outcome58=schema.safeParse({name:'Al',age:2,tags:['owned']});if(outcome58.success){const value=outcome58.data;strictEqual(value.name,'Al');}else{const error=outcome58.error;throw 'owned rejected';}}
      {const outcome59=schema.safeParse(JSON.parse('{"name":"A","extra":true}'));if(outcome59.success){const value=outcome59.data;throw 'invalid accepted';}else{const error=outcome59.error;strictEqual(error.issues.length,4);strictEqual(error.truncated,false);let missing=0;let unknown=0;for(const issue of error.issues){strictEqual(issue.path.length,1);if(issue.code==='missing-field'){missing++;}if(issue.code==='unknown-field'){unknown++;}}strictEqual(missing,2);strictEqual(unknown,1);}}
      const nested=s.array(s.object({child:s.object({flag:s.boolean()})}));
      {const outcome60=nested.safeParse(JSON.parse('[{"child":{"flag":true}}]'));if(outcome60.success){const value=outcome60.data;strictEqual(value.length,1);}else{const error=outcome60.error;throw 'nested rejected';}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    let invalid="import {s} from 'std:schema';function main(){const child=s.string();s.object({name:child});child.safeParse('x');}";
    assert_eq!(checked_source(invalid).err().unwrap().code,"transferred");
}

#[test]
fn schema_objects_support_named_empty_and_nested_error_paths() {
    let source=r#"import {s,Schema} from 'std:schema';import {strictEqual} from 'std:testing';
    type Person={name:string;age:int;};
    function main(){
      const schema:Schema<Person>=s.object({age:s.int(),name:s.string()});
      const input:Person={name:'Ada',age:37};
      {const outcome61=schema.safeParse(input);if(outcome61.success){const value=outcome61.data;strictEqual(value.name,'Ada');strictEqual(value.age,37);}else{const error=outcome61.error;throw 'named failed';}}
      const empty=s.object({});{const outcome62=empty.safeParse(JSON.parse('{}'));if(outcome62.success){const value=outcome62.data;}else{const error=outcome62.error;throw 'empty failed';}}
      const nested=s.object({rows:s.array(s.object({label:s.string().min(2)}))});
      {const outcome63=nested.safeParse(JSON.parse('{"rows":[{"label":"x"}]}'));if(outcome63.success){const value=outcome63.data;throw 'invalid accepted';}else{const error=outcome63.error;
        strictEqual(error.issues.length,1);for(const issue of error.issues){strictEqual(issue.path.length,3);for(const part of issue.path){if(part.field!==undefined){console.log(part.field);}else{console.log(part.index);}}}
      }}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout),"rows\n0\nlabel\n");
    for source in ["import {s} from 'std:schema';function main(){s.object({name:1});}","import {s} from 'std:schema';function main(){s.object([s.int()]);}"] {
        assert_eq!(checked_source(source).err().unwrap().code,"schema-type");
    }
    let source="import {s} from 'std:schema';function main(){const schema=s.object({name:s.string()});const input=JSON.parse('{}');schema.safeParse(input);console.log(typeof input);}";
    assert_eq!(checked_source(source).err().unwrap().code,"transferred");
}

#[test]
fn schema_object_strip_is_explicit_consuming_and_reusable() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
    function main(){
      const original=s.object({name:s.string()});const schema=original.strip();
      {const outcome64=schema.safeParse(JSON.parse('{"name":"Ada","extra":true}'));if(outcome64.success){const value=outcome64.data;strictEqual(value.name,'Ada');}else{const error=outcome64.error;throw 'strip rejected extras';}}
      {const outcome65=schema.safeParse(JSON.parse('{"name":"Al","other":null}'));if(outcome65.success){const value=outcome65.data;strictEqual(value.name,'Al');}else{const error=outcome65.error;throw 'reuse failed';}}
      {const outcome66=s.object({name:s.string()}).safeParse(JSON.parse('{"name":"Ada","extra":true}'));if(outcome66.success){const value=outcome66.data;throw 'default changed';}else{const error=outcome66.error;strictEqual(error.issues.length,1);}}
      {const outcome67=s.object({}).strip().safeParse(JSON.parse('{"any":1}'));if(outcome67.success){const value=outcome67.data;}else{const error=outcome67.error;throw 'empty strip failed';}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    let source="import {s} from 'std:schema';function main(){const original=s.object({name:s.string()});const schema=original.strip();original.safeParse(JSON.parse('{}'));}";
    assert_eq!(checked_source(source).err().unwrap().code,"transferred");
}

#[test]
fn schema_object_retains_extras_without_field_collisions() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
    function main(){
      const schema=s.object({name:s.string()}).retain();
      {const outcome68=schema.safeParse(JSON.parse('{"name":"Ada","value":true,"extra":null,"get":7}'));if(outcome68.success){const output=outcome68.data;
        strictEqual(output.value.name,'Ada');strictEqual(output.extra.size,3);
        strictEqual(output.extra.has('name'),false);strictEqual(output.extra.has('get'),true);
        const extra=output.extra.get('get');if(extra!==undefined){strictEqual(JSON.stringify(extra),'7');}else{throw 'lost extra';}
      }else{const error=outcome68.error;throw 'retention failed';}}
      {const outcome69=schema.safeParse({name:'Owned'});if(outcome69.success){const output=outcome69.data;strictEqual(output.value.name,'Owned');strictEqual(output.extra.size,0);}else{const error=outcome69.error;throw 'owned retain failed';}}
      const nested=s.array(s.object({name:s.string()}).retain());
      {const outcome70=nested.safeParse(JSON.parse('[{"name":"Nested","other":true}]'));if(outcome70.success){const outputs=outcome70.data;{const outcome0=nested.safeParse(outputs);if(outcome0.success){const again=outcome0.data;for(const output of again){strictEqual(output.value.name,'Nested');strictEqual(output.extra.size,1);}}else{const error=outcome0.error;throw 'owned nested retain failed';}}}else{const error=outcome70.error;throw 'nested retain failed';}}
      {const outcome71=schema.safeParse(JSON.parse('{"name":"Al"}'));if(outcome71.success){const output=outcome71.data;strictEqual(output.extra.size,0);}else{const error=outcome71.error;throw 'reuse failed';}}
      {const outcome72=schema.safeParse(JSON.parse('{"other":true}'));if(outcome72.success){const output=outcome72.data;throw 'missing accepted';}else{const error=outcome72.error;strictEqual(error.issues.length,1);}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn schema_optional_and_nullable_fields_keep_missing_and_null_distinct() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
    function main(){
      const schema=s.object({nickname:s.string().min(2).optional(),note:s.string().nullable()});
      {const outcome73=schema.safeParse(JSON.parse('{"note":null}'));if(outcome73.success){const value=outcome73.data;strictEqual(value.nickname===undefined,true);strictEqual(value.note===null,true);}else{const error=outcome73.error;throw 'presence failed';}}
      {const outcome74=schema.safeParse(JSON.parse('{"nickname":"Ada","note":"ready"}'));if(outcome74.success){const value=outcome74.data;if(value.nickname!==undefined){strictEqual(value.nickname,'Ada');}if(value.note!==null){strictEqual(value.note,'ready');}}else{const error=outcome74.error;throw 'values failed';}}
      {const outcome75=schema.safeParse(JSON.parse('{"nickname":null}'));if(outcome75.success){const value=outcome75.data;throw 'invalid accepted';}else{const error=outcome75.error;strictEqual(error.issues.length,2);}}
      {const outcome76=schema.safeParse({note:null});if(outcome76.success){const value=outcome76.data;strictEqual(value.nickname===undefined,true);}else{const error=outcome76.error;throw 'owned optional failed';}}
      const numeric=s.number().optional();{const outcome77=numeric.safeParse(9007199254740993);if(outcome77.success){const value=outcome77.data;throw 'precision lost';}else{const error=outcome77.error;for(const issue of error.issues){strictEqual(issue.code,'number-precision');}}}
      {const outcome78=numeric.safeParse(42);if(outcome78.success){const value=outcome78.data;if(value!==undefined){strictEqual(value,42.0);}}else{const error=outcome78.error;throw 'exact conversion failed';}}
      const nulls=s.null().optional();{const outcome79=nulls.safeParse(JSON.parse('null'));if(outcome79.success){const value=outcome79.data;strictEqual(value===null,true);}else{const error=outcome79.error;throw 'null presence failed';}}
      const root=s.int().optional();{const outcome80=root.safeParse(undefined);if(outcome80.success){const value=outcome80.data;strictEqual(value===undefined,true);}else{const error=outcome80.error;throw 'root missing failed';}}
      {const outcome81=root.safeParse(7);if(outcome81.success){const value=outcome81.data;if(value!==undefined){strictEqual(value,7);}}else{const error=outcome81.error;throw 'root value failed';}}
      const nullable=s.array(s.string().nullable());{const outcome82=nullable.safeParse(JSON.parse('["ok",null]'));if(outcome82.success){const values=outcome82.data;strictEqual(values.length,2);}else{const error=outcome82.error;throw 'nullable array failed';}}
      const optional=s.array(s.string().optional());{const outcome83=optional.safeParse(['ok',undefined]);if(outcome83.success){const values=outcome83.data;strictEqual(values.length,2);}else{const error=outcome83.error;throw 'optional array failed';}}
      const both=s.object({value:s.int().optional().nullable()});
      {const outcome84=both.safeParse(JSON.parse('{}'));if(outcome84.success){const value=outcome84.data;strictEqual(value.value===undefined,true);}else{const error=outcome84.error;throw 'combined missing failed';}}
      {const outcome85=both.safeParse(JSON.parse('{"value":null}'));if(outcome85.success){const value=outcome85.data;strictEqual(value.value===null,true);}else{const error=outcome85.error;throw 'combined null failed';}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    let invalid="import {s} from 'std:schema';function main(){const child=s.string();const schema=child.optional();child.safeParse('reuse');}";
    assert_eq!(checked_source(invalid).err().unwrap().code,"transferred");
    let invalid="import {s} from 'std:schema';function main(){const schema=s.string().optional();const input='owned';schema.safeParse(input);console.log(input);}";
    assert_eq!(checked_source(invalid).err().unwrap().code,"transferred");
}

#[test]
fn schema_defaults_validate_fresh_factory_results_only_for_missing_fields() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
    function fresh(): string[] { return ['fresh']; }
    function main(){
      const schema=s.object({items:s.array(s.string()).default(fresh)});
      {const outcome86=schema.safeParse(JSON.parse('{}'));if(outcome86.success){const value=outcome86.data;value.items.push('changed');strictEqual(value.items.length,2);}else{const error=outcome86.error;throw 'missing default failed';}}
      {const outcome87=schema.safeParse(JSON.parse('{}'));if(outcome87.success){const value=outcome87.data;strictEqual(value.items.length,1);}else{const error=outcome87.error;throw 'fresh default failed';}}
      {const outcome88=schema.safeParse(JSON.parse('{"items":[]}'));if(outcome88.success){const value=outcome88.data;strictEqual(value.items.length,0);}else{const error=outcome88.error;throw 'supplied value failed';}}
      {const outcome89=schema.safeParse(JSON.parse('{"items":null}'));if(outcome89.success){const value=outcome89.data;throw 'null accepted';}else{const error=outcome89.error;strictEqual(error.issues.length,1);}}
      const captured=[1,2,3];
      const reusable=s.object({count:s.int().default(()=>captured.length)});
      {const outcome90=reusable.safeParse(JSON.parse('{}'));if(outcome90.success){const value=outcome90.data;strictEqual(value.count,3);}else{const error=outcome90.error;throw 'captured default failed';}}
      {const outcome91=reusable.safeParse(JSON.parse('{}'));if(outcome91.success){const value=outcome91.data;strictEqual(value.count,3);}else{const error=outcome91.error;throw 'reused capture failed';}}
      const required=s.object({count:s.int()});
      {const outcome92=required.safeParse(JSON.parse('{}'));if(outcome92.success){const value=outcome92.data;throw 'default leaked between plans';}else{const error=outcome92.error;strictEqual(error.issues.length,1);}}
      const presence=s.object({flag:s.boolean().default(()=>true),nothing:s.null().default(()=>null),note:s.string().nullable().default(()=>null)});
      {const outcome93=presence.safeParse(JSON.parse('{}'));if(outcome93.success){const value=outcome93.data;strictEqual(value.flag,true);strictEqual(value.nothing===null,true);strictEqual(value.note===null,true);}else{const error=outcome93.error;throw 'presence defaults failed';}}
      const invalid=s.object({count:s.int().min(1).default(()=>0)});
      {const outcome94=invalid.safeParse(JSON.parse('{}'));if(outcome94.success){const value=outcome94.data;throw 'invalid factory accepted';}else{const error=outcome94.error;strictEqual(error.issues.length,1);}}
    }"#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
    for (source,code) in [
      ("import {s} from 'std:schema';function main(){const values=[1];const plan=s.int().default(()=>{values.push(2);return values.length;});}","callback-access"),
      ("import {s} from 'std:schema';function main(){const text='owned';const plan=s.string().default(()=>text);}","callback-once"),
      ("import {s} from 'std:schema';function main(){const original=s.int();const plan=original.default(()=>1);original.safeParse(1);}","transferred"),
    ] {assert_eq!(checked_source(source).err().unwrap().code,code);}

}

#[test]
fn schema_default_factory_exceptions_propagate_without_becoming_validation_issues() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
    type FactoryError={code:string;};
    function failing():int{throw 'factory failed';}
    function later():string{throw 'later factory ran';}
    function structured():int{const error:FactoryError={code:'factory'};throw error;}
    function main(){
      const schema=s.object({nested:s.object({value:s.int().default(failing)}),later:s.string().default(later)});
      const input=JSON.parse('{"nested":{}}');
      const typed=s.object({value:s.int().default(structured)});const typedInput=JSON.parse('{}');
      try{schema.safeParse(input);throw 'factory did not throw';}catch(error){strictEqual(error,'factory failed');}
      try{typed.safeParse(typedInput);throw 'structured factory did not throw';}catch(error){if(typeof error==='string'){throw error;}strictEqual(error.code,'factory');}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn schema_default_owned_record_omission_builds_required_output() {
    let source="import {s} from 'std:schema';import {strictEqual} from 'std:testing';function main(){const schema=s.object({value:s.int().default(()=>7)});{const outcome95=schema.safeParse({});if(outcome95.success){const value=outcome95.data;strictEqual(value.value,7);}else{const error=outcome95.error;throw 'owned omission failed';}}}";
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn schema_default_late_factory_error_widening_rechecks_catch_calls() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
    type FactoryError={code:string;};
    function first():int{throw 'first';}
    function second():int{const error:FactoryError={code:'second'};throw error;}
    function main(){const earlier=s.object({value:s.int().default(first)});const input=JSON.parse('{}');try{earlier.safeParse(input);throw 'missing failure';}catch(error){strictEqual(error,'first');}const later=s.object({value:s.int().default(second)});}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn schema_default_forwarded_plan_refreshes_factory_exception_effects() {
    let source=r#"import {s,Schema} from 'std:schema';import {JsonValue} from 'std:json';import {strictEqual} from 'std:testing';
    type Payload={value:int;};type ForwardError={code:string;};
    function forward(schema:ReadOnly<Schema<Payload>>,input:JsonValue):int{{const outcome96=schema.safeParse(input);if(outcome96.success){const value=outcome96.data;return value.value;}else{const error=outcome96.error;throw 'invalid';}}}
    function failing():int{const error:ForwardError={code:'factory failed'};throw error;}
    function main(){const schema:Schema<Payload>=s.object({value:s.int().default(failing)});const input=JSON.parse('{}');try{forward(schema,input);throw 'missing failure';}catch(error){if(typeof error==='string'){throw error;}strictEqual(error.code,'factory failed');}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn schema_defaults_preserve_factories_through_presence_and_constraints() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
    function main(){
      const schema=s.object({count:s.int().default(()=>7).min(5),note:s.string().default(()=>'ready').optional(),other:s.string().default(()=>'ok').nullable()});
      {const outcome97=schema.safeParse(JSON.parse('{}'));if(outcome97.success){const value=outcome97.data;strictEqual(value.count,7);if(value.note===undefined){throw 'optional erased default';}strictEqual(value.note,'ready');if(value.other===null){throw 'nullable erased default';}strictEqual(value.other,'ok');}else{const error=outcome97.error;throw 'composed defaults failed';}}
      {const outcome98=schema.safeParse({});if(outcome98.success){const value=outcome98.data;strictEqual(value.count,7);if(value.note===undefined){throw 'owned optional erased default';}strictEqual(value.note,'ready');if(value.other===null){throw 'owned nullable erased default';}strictEqual(value.other,'ok');}else{const error=outcome98.error;throw 'owned composed defaults failed';}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn schema_defaults_validate_distinct_owned_input_shapes_for_same_output() {
    let source=r#"import {s,Schema} from 'std:schema';import {strictEqual} from 'std:testing';
    type Item={value:int;};
    function main(){const left:Schema<Item>=s.object({value:s.int().default(()=>7)});const right:Schema<Item>=s.object({value:s.int().default(()=>9)});const schema=s.object({left:left,right:right});
      {const outcome99=schema.safeParse({left:{},right:{value:11}});if(outcome99.success){const value=outcome99.data;strictEqual(value.left.value,7);strictEqual(value.right.value,11);}else{const error=outcome99.error;throw 'owned shapes failed';}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn schema_default_chains_reject_incompatible_input_and_unnarrowed_output() {
    for source in [
      "import {s} from 'std:schema';function main(){const schema=s.int().default(()=>7).min(5);schema.safeParse('seven');}",
      "import {s} from 'std:schema';function main(){const schema=s.string().default(()=>'ready').optional();{const outcome100=schema.safeParse(undefined);if(outcome100.success){const value=outcome100.data;const size=value.length;}else{const error=outcome100.error;}}}",
      "import {s} from 'std:schema';function main(){const schema=s.int().default(()=>'wrong').min(5);}",
    ] {assert!(checked_source(source).is_err(),"accepted incompatible chain: {source}");}
}

#[test]
fn schema_defaults_survive_presence_widening_without_copying_factories() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
    function fail():int|undefined{throw 'widened factory error';}
    function main(){
      const schema=s.object({left:s.string().optional().default(()=>'left').nullable(),right:s.string().nullable().default(()=>'right').optional()});
      {const outcome101=schema.safeParse(JSON.parse('{}'));if(outcome101.success){const value=outcome101.data;if(value.left===null||value.left===undefined){throw 'left default lost';}if(value.right===null||value.right===undefined){throw 'right default lost';}strictEqual(value.left,'left');strictEqual(value.right,'right');}else{const error=outcome101.error;throw 'widening failed';}}
      const throwing=s.object({value:s.int().optional().default(fail).nullable()});const input=JSON.parse('{}');try{throwing.safeParse(input);throw 'missing exception';}catch(error){strictEqual(error,'widened factory error');}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn schema_default_accepts_owned_runtime_factory_with_readonly_captures() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
    function make():Function<int>{const values=[1,2,3];return ()=>values.length;}
    function main(){const factory=make();const schema=s.object({count:s.int().default(factory)});{const outcome102=schema.safeParse(JSON.parse('{}'));if(outcome102.success){const value=outcome102.data;strictEqual(value.count,3);}else{const error=outcome102.error;throw 'runtime factory failed';}}{const outcome103=schema.safeParse(JSON.parse('{}'));if(outcome103.success){const value=outcome103.data;strictEqual(value.count,3);}else{const error=outcome103.error;throw 'runtime reuse failed';}}}
    "#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}

#[test]
fn schema_default_object_factory_survives_retention_and_stripping() {
    let source=r#"import {s} from 'std:schema';import {strictEqual} from 'std:testing';
    function main(){const schema=s.object({retained:s.object({count:s.int()}).default(()=>({count:7})).retain(),stripped:s.object({count:s.int()}).default(()=>({count:9})).strip()});
      {const outcome104=schema.safeParse(JSON.parse('{}'));if(outcome104.success){const value=outcome104.data;strictEqual(value.retained.value.count,7);strictEqual(value.retained.extra.size,0);strictEqual(value.stripped.count,9);}else{const error=outcome104.error;throw 'policy default failed';}}
    }"#;
    let directory=support::TempDirectory::new();let path=directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path),60));
}
