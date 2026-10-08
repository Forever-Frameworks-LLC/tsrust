use tsrust_compiler::{
    Mode,
    native_bindings::{NativeType, lookup, selected_features},
};
#[test]
fn binding_contract_keeps_arity_types_ownership_and_feature_selection_consistent() {
    let binding = lookup("std:path", "basename").unwrap();
    assert_eq!(binding.parameters[0].ty, NativeType::String);
    assert!(
        binding
            .parameters
            .iter()
            .all(|parameter| parameter.access == Mode::Read)
    );
    assert_eq!(binding.result, NativeType::String);
    assert!(!binding.accepts_arity(0));
    assert!(binding.accepts_arity(1));
    assert!(binding.accepts_arity(2));
    assert!(!binding.accepts_arity(3));
    assert!(selected_features([]).unwrap().is_empty());
    let features = selected_features([("std:path", "basename"), ("std:path", "basename")]).unwrap();
    assert_eq!(features.into_iter().collect::<Vec<_>>(), vec!["path"]);
    assert!(
        lookup("std:http", "basename")
            .unwrap_err()
            .contains("Unknown standard-library module")
    );
    assert!(
        lookup("std:path", "missing")
            .unwrap_err()
            .contains("does not export")
    );
    assert!(selected_features([("std:path", "missing")]).is_err());
}

#[test]
fn lexical_path_fixed_arity_and_boolean_result() {
    for name in ["dirname","extname","isAbsolute","normalize"] {
        let binding=lookup("std:path",name).unwrap();
        assert_eq!(binding.parameters.len(),1);assert_eq!(binding.parameters[0].access,Mode::Read);
        assert_eq!(binding.parameters[0].ty,NativeType::String);
        assert_eq!(binding.result,if name=="isAbsolute" {NativeType::Boolean} else {NativeType::String});
        assert!(!binding.accepts_arity(0));assert!(binding.accepts_arity(1));assert!(!binding.accepts_arity(2));
        assert_eq!(binding.features,&["path"]);
    }
}

#[test]
fn os_identity_registry_has_exact_zero_argument_string_exports() {
    for name in ["platform", "arch"] {
        let binding=tsrust_compiler::native_bindings::lookup("std:os",name).unwrap();
        assert!(binding.accepts_arity(0));
        assert!(!binding.accepts_arity(1));
        assert!(binding.parameters.is_empty());
        assert_eq!(binding.result,tsrust_compiler::native_bindings::NativeType::String);
    }
    assert!(tsrust_compiler::native_bindings::lookup("std:os","hostname").is_err());
}
