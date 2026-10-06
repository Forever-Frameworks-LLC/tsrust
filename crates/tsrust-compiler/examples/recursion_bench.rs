//! Recursive access-inference workloads, separate from native execution.
use std::hint::black_box;
use std::time::Instant;
use tsrust_compiler::check;

fn measure(name: &str, parameters: usize, source: &str) {
    for _ in 0..5 {
        black_box(check(black_box(source)).unwrap_or_else(|e| panic!("{}", e.message)));
    }
    let mut samples = Vec::new();
    for _ in 0..31 {
        let start = Instant::now();
        let analysis = check(black_box(source)).unwrap_or_else(|e| panic!("{}", e.message));
        black_box(&analysis);
        samples.push(start.elapsed().as_secs_f64() * 1_000_000.0);
        // Result destruction is outside the measured interval.
    }
    samples.sort_by(f64::total_cmp);
    println!("{name},{parameters},{},31,{:.1}", source.len(), samples[15]);
}

fn main() {
    println!("workload,owned_parameters_per_function,source_bytes,iterations,median_analysis_us");
    measure(
        "tree",
        1,
        include_str!("../tests/fixtures/recursion/tree.tsr"),
    );
    // A consumed parameter rotates through every argument position. Effects
    // cannot remain local to the first position; all ownership contracts change.
    for width in [4, 16, 64] {
        let params = (0..width)
            .map(|i| format!("a{i}: A"))
            .collect::<Vec<_>>()
            .join(", ");
        let arguments = (0..width)
            .map(|i| format!("a{}", (i + 1) % width))
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!(
            "type A = {{ n: int; }}; function rotate({params}, n: int): A {{ if (n === 0) {{ return a0; }} return rotate({arguments}, n - 1); }} function main() {{}}"
        );
        measure("permuted", width, &source);
    }
}
