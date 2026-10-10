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
