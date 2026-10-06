//! Frontend-only editor baseline; no Rust emission, subprocesses or file I/O in timing.
use serde_json::json;
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let small =
        include_str!("../../tsrust-compiler/tests/fixtures/ownership/read_reuse.tsr").to_owned();
    let rejected =
        include_str!("../../tsrust-compiler/tests/fixtures/ownership/use_after_call.tsr")
            .to_owned();
    let large = (0..1000)
        .map(|n| format!("function read{n}(value: int): int {{ return value + 1; }}\n"))
        .collect::<String>()
        + "function main() {}\n";
    let mut rows = Vec::new();
    for (name, source) in [
        ("read-reuse", small),
        ("use-after-call", rejected),
        ("1000-functions", large),
    ] {
        for _ in 0..5 {
            let _ = black_box(tsrust_compiler::check(&source));
        }
        let mut times = Vec::new();
        for _ in 0..31 {
            let start = Instant::now();
            let result = black_box(tsrust_compiler::check(black_box(&source)));
            times.push(start.elapsed().as_secs_f64() * 1000.0);
            drop(result);
        }
        times.sort_by(f64::total_cmp);
        rows.push(json!({"workload":name,"sourceBytes":source.len(),"analysisMedianMs":times[15],"analysisSamplesMs":times}));
    }
    println!(
        "{}",
        json!({"profile":"release","warmups":5,"iterations":31,"excludes":["input generation","result destruction","Rust emission","rustc","editor transport"],"rows":rows})
    );
}
