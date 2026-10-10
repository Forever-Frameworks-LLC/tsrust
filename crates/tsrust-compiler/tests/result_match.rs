mod support;

#[test]
fn result_construction_and_exhaustive_match_execute() {
    let directory = support::TempDirectory::new();
    let path = directory.source(r#"
import { strictEqual } from "std:testing";

function divide(a: int, b: int): Result<int, string> {
    if (b === 0) { return Err("zero"); }
    return Ok(a / b);
}

function main() {
    match (divide(10, 2)) {
        Ok(value) => { strictEqual(value, 5); },
        Err(error) => { strictEqual(error, "unexpected"); },
    }
    match (divide(10, 0)) {
        Ok(value) => { strictEqual(value, -1); },
        Err(error) => { strictEqual(error, "zero"); },
    }
}
"#);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}
