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

#[test]
fn async_generic_decoding_resolves_inferred_native_and_pipeline_factories() {
    let source = support::format_round_trip(include_str!("fixtures/named-result-inference/async-schema-factory.tsr"));
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 120);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Async inferred schema factories passed\n");
}

#[test]
fn inferred_factory_discovery_keeps_resource_move_errors() {
    let source = support::format_round_trip(r#"
import {trackedResource,closeResource,resourceId} from 'std:testing';
type Header={operation:string;};
function bad(){const resource=trackedResource();closeResource(resource);return resource;}
async function verify(body:string):Task<void>{const header=JSON.parse<Header>(body);const resource=bad();console.log(resourceId(resource));}
async function main():Task<void>{await verify('{"operation":"bad"}');}
"#);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("check").arg(directory.source(&source)), 30);
    assert!(!output.status.success());
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(errors.contains("error[transferred]") && errors.contains("Value transferred here."), "{errors}");
}
