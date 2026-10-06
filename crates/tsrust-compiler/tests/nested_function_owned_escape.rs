mod support;

#[test]
fn escaping_named_functions_transfer_independent_captured_owners() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/nested-function-owned-escape/take-owner.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn consuming_named_captures_reject_reuse_and_conflicting_argument_borrows() {
    for (source, code) in [
        ("type User={name:string;};function main(){const user:User={name:\"Pat\"};function take():User{return user;}const saved=take();console.log(user.name);}", "transferred"),
        ("type User={name:string;};function make(user:User):()=>User{function take():User{return user;}return take;}function main(){const take=make({name:\"Pat\"});const first=take();const second=take();}", "closure-consumed"),
    ] {
        let error = tsrust_compiler::check(source).err().expect("transferred capture or callable");
        assert_eq!(error.code, code, "{}", error.message);
    }
    let source = "type User={name:string;};function main(){const user:User={name:\"Pat\"};function take(other:ReadOnly<User>):User{console.log(other.name);return user;}const saved=take(user);}";
    let error = tsrust_compiler::check(source).err().expect("capture transfer overlaps argument borrow");
    assert_eq!(error.code, "conflicting-access", "{}", error.message);
}
