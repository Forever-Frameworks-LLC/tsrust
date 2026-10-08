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

#[test]
fn nested_declarations_infer_nonrecursive_result_when_omitted() {
    native(include_str!("fixtures/nested-function-declarations/inferred-result.tsr"));
}

#[test]
fn deep_nested_declarations_check_with_the_original_worker_stack() {
    let source=format!("function main(){{{}return 1;{}{}}}",
        "function next():int{".repeat(100), "}return 1;".repeat(99), "}");
    tsrust_compiler::check(&source).unwrap();
}

#[test]
fn nested_declarations_hoist_inside_catch_and_finally_with_live_captures() {
    native(&support::format_round_trip(r#"import {strictEqual} from "std:testing";
function main(){let total=0;try{const value=enter(2);function enter(value:int):int{total+=value;return total;}strictEqual(value,2);throw 3;}catch(error){const value=recover(4);function recover(value:int):int{total+=value;return total;}strictEqual(value,6);strictEqual(error===3,true);}finally{finish();function finish():void{total+=8;}}strictEqual(total,14);}"#));
}

#[test]
fn unreachable_catch_declarations_still_check_their_bodies() {
    for (body,code) in [("return \"wrong\";","type"),("return missing;","name")] {
        let source=support::format_round_trip(&format!("function main(){{try{{const value=1;}}catch(error){{function broken():int{{{body}}}broken();}}}}"));
        support::reject(&source,code);
    }
}
