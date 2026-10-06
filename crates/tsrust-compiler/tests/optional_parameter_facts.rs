use tsrust_compiler::{ParameterOmission, check};

#[test]
fn checked_defaults_separate_supplied_contract_from_initialized_body() {
    let source = "function choose<T>(fallback:T,value:T=fallback):T{return value;}function main(){console.log(choose(4));}";
    let checked = check(source).unwrap_or_else(|error| panic!("{error:?}"));
    let facts = checked.checked_type_facts();
    let declaration = source.find("value:T=").unwrap();
    let body = facts.parameter_types.iter().find(|fact| fact.span.start == declaration).expect("exact initialized parameter declaration");
    assert_eq!(&source[body.span.start..body.span.end], "value");
    assert_eq!(body.ty, "int");
    let call = facts.generic_calls.iter().find(|fact| source[fact.span.start..fact.span.end].starts_with("choose(")).expect("checked generic call");
    let value = &call.parameters[1];
    assert_eq!(value.omission, ParameterOmission::Defaulted);
    assert_eq!(value.body_ty.as_deref(), Some("int"));
    let input = value.ty.as_deref().expect("accepted argument type");
    assert!(input.contains("int") && input.contains("undefined"), "{input}");
}

#[test]
fn initializer_results_join_checked_expression_facts() {
    let source = "function label(value:string=' hi '.trim()):string{return value;}function main(){console.log(label());}";
    let checked = check(source).unwrap_or_else(|error| panic!("{error:?}"));
    let facts = checked.checked_type_facts();
    let value = facts.parameter_types.iter().find(|fact| &source[fact.span.start..fact.span.end] == "value").expect("parameter body fact");
    assert_eq!(value.ty, "string");
    // Callable names inside an initializer must retain their checked contract,
    // rather than a signature reconstructed from source text.
    let source = "function main(){const first=(value:int):int=>{return value+1;};const second=(action=first):int=>{return action(4);};console.log(second());}";
    let checked = check(source).unwrap_or_else(|error| panic!("{error:?}"));
    let facts = checked.checked_type_facts();
    let first = source.find("action=first").unwrap() + "action=".len();
    let result = facts.method_results.iter().find(|fact| fact.span.start == first).expect("initializer callable name fact");
    assert_eq!(result.ty, "Function<int, [value: int]>");
}

#[test]
fn default_body_metadata_preserves_distinct_generic_specializations() {
    let source = "function choose<T>(fallback:T,value:T=fallback):T{return value;}function main(){console.log(choose(4),choose(1.5));}";
    let checked = check(source).unwrap_or_else(|error| panic!("{error:?}"));
    let declaration = source.find("value:T=").unwrap();
    let mut types = checked.checked_type_facts().parameter_types.into_iter().filter(|fact| fact.span.start == declaration).map(|fact| fact.ty).collect::<Vec<_>>();
    types.sort(); types.dedup();
    assert_eq!(types, ["int", "number"], "shared generic source spans must not choose one specialization");
}
