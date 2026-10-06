use tsrust_compiler::check;

#[test]
fn default_parameter_declarations_and_initializer_calls_have_checked_facts() {
    let source = "function seed():int{return 4;}function value(input:int=seed()):int{return input;}function main(){console.log(value());}";
    let checked = check(source).unwrap_or_else(|error| panic!("{error:?}"));
    let facts = checked.checked_type_facts();
    let declaration = source.find("input:int=").unwrap();
    let parameter = facts.parameter_types.iter().find(|fact| fact.span.start == declaration).expect("exact checked parameter declaration");
    assert_eq!(&source[parameter.span.start..parameter.span.end], "input");
    assert_eq!(parameter.ty, "int");
    let initializer = source.find("=seed()").unwrap() + 1;
    let result = facts.method_results.iter().find(|fact| fact.span.start == initializer).expect("checked initializer call result");
    assert_eq!(&source[result.span.start..result.span.end], "seed()");
    assert_eq!(result.ty, "int");
}
