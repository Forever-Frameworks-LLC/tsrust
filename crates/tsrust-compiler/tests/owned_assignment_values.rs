mod support;

#[test]
fn owned_assignment_values_read_the_target_in_place() {
    let source = include_str!("fixtures/assignment-values/owned.tsr");
    assert_eq!(tsrust_compiler::format(source).unwrap(), source);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn owned_assignment_values_follow_ownership_and_target_rules() {
    for (source, code) in [
        // Binding the value moves the target, as naming the target would.
        ("function main(){let a=\"x\";const b=(a=\"moved\");console.log(a);}", "transferred"),
        // So a chain of owned assignments leaves only the outer target usable.
        ("function main(){let a=\"x\";let b=\"y\";a=b=\"same\";console.log(a,b);}", "transferred"),
        // An element or computed place would be evaluated twice.
        ("function main(){const a=[\"x\"];console.log((a[0]=\"long\").length);}", "unsupported"),
        ("function main(){const text=\"a\";console.log((text=\"b\").length);}", "unsupported"),
        ("function main(){let text=\"a\";console.log((text=1).length);}", "type"),
    ] {
        let error = tsrust_compiler::check(source).err().unwrap_or_else(|| panic!("must be rejected: {source}"));
        assert_eq!(error.code, code, "{source}: {}", error.message);
    }
    // Scalars keep copying: the chain works.
    tsrust_compiler::check("function main(){let a=0;let b=0;a=b=4;console.log(a,b);}").unwrap();
}
