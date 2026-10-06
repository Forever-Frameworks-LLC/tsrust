mod support;
#[test]
fn scalar_unary_plus_borrows_and_converts_once() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/scalar-unary-plus/values.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}
#[test]
fn scalar_unary_plus_retains_static_numeric_contracts() {
    for source in [
        "function bad(value:string|int[]):number{return +value;}function main(){}",
        "function bad(value:(()=>int)|null):number{return +value;}function main(){}",
        "function main(){ const x = +[1]; }",
        "type R={x:int;}; function main(){const r:R={x:1}; const x=+r;}",
        "function main(){const f=():int=>1; const x=+f;}",
        "function main(){const x:int=+\"2\";}",
        "function main(){const x=-\"2\";}",
    ] { support::reject(source,"type"); }
}
