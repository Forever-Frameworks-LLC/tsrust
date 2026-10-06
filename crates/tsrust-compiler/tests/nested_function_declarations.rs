mod support;

fn native(source: &str) {
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn nested_declarations_hoist_recurse_shadow_and_pass_as_values() {
    native(include_str!(
        "fixtures/nested-function-declarations/hoisting.tsr"
    ));
}

#[test]
fn nested_declarations_borrow_captures_and_transfer_escaping_environments() {
    native(include_str!(
        "fixtures/nested-function-declarations/captures.tsr"
    ));
}

#[test]
fn nested_declarations_keep_generic_default_rest_and_deep_scope_signatures() {
    native(include_str!(
        "fixtures/nested-function-declarations/signatures.tsr"
    ));
}

#[test]
fn nested_declaration_scope_conflicts_and_local_main_are_checked() {
    for source in [
        "function main(){function f():int{return 1;} function f():int{return 2;}}",
        "function main(){function f():int{return 1;} const f=2;}",
        "function root(f:int){function f():int{return 1;}} function main(){}",
    ] {
        let error = tsrust_compiler::check(source)
            .err()
            .expect("duplicate declaration");
        assert_eq!(error.code, "duplicate");
    }
    let error =
        tsrust_compiler::check("function main(){{function f():int{return 1;}} console.log(f());}")
            .err()
            .unwrap();
    assert_eq!(error.code, "name");
    assert!(tsrust_compiler::check("function root():int{function main():int{return 7;} return main();}function main(){console.log(root());}").is_ok());
}

#[test]
fn nested_declaration_names_do_not_rewrite_shadowing_closure_parameters() {
    native(
        r#"import { strictEqual } from "std:testing";
function main(){
    function value():int{return 40;}
    const values=[1,2].map((value:int):int=>value+1);
    strictEqual(values[0],2); strictEqual(values[1],3);
    { const value=5; strictEqual(value,5); }
    strictEqual(value(),40);
}"#,
    );
}
