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
