mod support;

#[test]
fn named_function_expressions_recurse_capture_and_keep_a_private_name() {
    let source = include_str!("fixtures/named-function-expressions/values.tsr");
    assert_eq!(tsrust_compiler::format(source).unwrap(), source);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

// The hidden declaration of a named function expression belongs to the block
// that holds its statement. Destructured parameters open their own body.
#[test]
fn statement_headers_arguments_and_defaults_keep_declarations_in_scope() {
    let source = include_str!("fixtures/named-function-expressions/statement-headers.tsr");
    assert_eq!(tsrust_compiler::format(source).unwrap(), source);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn the_expression_name_is_not_visible_outside_and_obeys_function_rules() {
    for (source, code) in [
        // Not visible after the expression.
        ("function main(){const f=function inner():int{return 1;};console.log(inner());}", "name"),
        // Not visible in a sibling expression.
        ("function main(){const f=function inner():int{return 1;};const g=():int=>inner();}", "name"),
        // The name cannot be assigned inside the function.
        ("function main(){const f=function inner():int{inner=1;return 1;};}", "name"),
        // Recursion still needs its captures to have a value at the use.
        ("function main(){const f=function inner():int{return later;};console.log(f());const later=1;}", "unassigned"),
        // A parameter default has no block for the private declaration yet.
        ("function run(action:(n:int)=>int=function inc(n:int):int{return n+1;}):int{return action(1);}function main(){}", "unsupported"),
        // A function expression is a value, not a statement.
        ("function main(){function (n:int):int{return n;};}", "syntax"),
        // Ordinary type rules apply to the body.
        ("function main(){const f=function inner():int{return \"s\";};}", "type"),
    ] {
        let error = tsrust_compiler::check(source).err().unwrap_or_else(|| panic!("must be rejected: {source}"));
        assert_eq!(error.code, code, "{source}: {}", error.message);
    }
}
