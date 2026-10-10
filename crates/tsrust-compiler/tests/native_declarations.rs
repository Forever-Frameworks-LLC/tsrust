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
