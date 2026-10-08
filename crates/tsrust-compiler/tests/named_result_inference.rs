mod support;

#[test]
fn omitted_named_results_preserve_scalar_capture_and_owner_returns() {
    let directory=support::TempDirectory::new();
    let path=directory.source(include_str!("fixtures/named-result-inference/basic.tsr"));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn omitted_named_results_keep_void_and_report_inference_boundaries() {
    tsrust_compiler::check("function main(){function empty(){}empty();function recurse(n:int){if(n>0){recurse(n-1);}}recurse(2);}").unwrap();
    for (source,code) in [
        ("function main(){function loop(){return loop();}loop();}","result-inference"),
        ("function main(){function first(){return second();}function second(){return first();}first();}","result-inference"),
        ("function main(){function mixed(value:boolean){if(value){return 1;}return 'one';}mixed(true);}","type"),
        ("function main(){function missing(value:boolean){if(value){return 1;}}missing(true);}","type"),
        ("function main(){function explicit():void{return 1;}explicit();}","type"),
    ] {
        let error=tsrust_compiler::check(source).err().expect("inconsistent result or recursive boundary");
        assert_eq!(error.code,code,"{}: {}",error.code,error.message);
    }
}

#[test]
fn recursive_functions_infer_their_result_from_an_independent_return() {
    let source = include_str!("fixtures/named-result-inference/recursive.tsr");
    assert_eq!(tsrust_compiler::format(source).unwrap(), source);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    for (source, code) in [
        // The recursive return must agree with the independent one.
        ("function bad(n:int){if(n===0){return 1;}return \"x\"+bad(n-1);}function main(){bad(1);}", "type"),
        // A caller sees the inferred type.
        ("function count(n:int){if(n===0){return 0;}return 1+count(n-1);}function main(){const text:string=count(2);}", "type"),
        // No return is independent of the recursion or of a local.
        ("function spin(n:int){const next=n-1;return spin(next);}function main(){spin(1);}", "result-inference"),
    ] {
        let error = tsrust_compiler::check(source).err().unwrap_or_else(|| panic!("must be rejected: {source}"));
        assert_eq!(error.code, code, "{source}: {}", error.message);
    }
}
