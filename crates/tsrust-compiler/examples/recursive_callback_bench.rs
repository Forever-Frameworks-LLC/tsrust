//! Recursive callback analysis and concrete type-vector validation workloads.
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

fn permutations(width: usize, repeated: bool, owned: bool) -> String {
    let qualifier = if owned { "owned " } else { "" };
    let parameters = (0..width)
        .map(|i| format!("c{i}: {qualifier}() => int"))
        .collect::<Vec<_>>()
        .join(", ");
    let swap = (0..width)
        .map(|i| format!("c{}", if i < 2 { 1 - i } else { i }))
        .collect::<Vec<_>>()
        .join(", ");
    let rotate = (0..width)
        .map(|i| format!("c{}", (i + 1) % width))
        .collect::<Vec<_>>()
        .join(", ");
    let declarations = (0..width)
        .map(|i| {
            if repeated {
                format!("const c{i} = make({i});")
            } else {
                format!("const c{i} = (): int => {{ return {i}; }};")
            }
        })
        .collect::<String>();
    let arguments = (0..width)
        .map(|i| format!("c{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "function make(n: int): () => int {{ return (): int => {{ return n; }}; }} function choose({parameters}, n: int): int {{ if (n === 0) {{ return c0(); }} if (n === 1) {{ return choose({swap}, n - 1); }} return choose({rotate}, n - 1); }} function main() {{ {declarations} console.log(choose({arguments}, 2)); }}"
    )
}

fn main() {
    println!(
        "workload,callback_parameters_per_function,source_bytes,iterations,median_analysis_us"
    );
    measure(
        "callback_tree",
        1,
        include_str!("../tests/fixtures/recursive-callbacks/visit.tsr"),
    );
    for width in [4, 5] {
        measure(
            "distinct_permutations",
            width,
            &permutations(width, false, false),
        );
    }
    measure("repeated_permutations", 7, &permutations(7, true, false));
    measure(
        "owned_counter",
        1,
        include_str!("../tests/fixtures/recursive-owned-callbacks/countdown.tsr"),
    );
    for width in [4, 5] {
        measure(
            "owned_distinct_permutations",
            width,
            &permutations(width, false, true),
        );
    }
    measure(
        "owned_repeated_permutations",
        7,
        &permutations(7, true, true),
    );
    measure(
        "owned_fresh_arrow",
        1,
        "function call(c: owned () => int, n: int): int { if (n === 0) { return c(); } return call((): int => { return n; }, n - 1); } function main() { console.log(call((): int => { return 7; }, 3)); }",
    );
    measure(
        "recursive_counter_result",
        0,
        include_str!("../tests/fixtures/recursive-callable-results/counter.tsr"),
    );
    measure(
        "lazy_recursive_result",
        0,
        "function make(n: int): () => int { return (): int => { if (n === 0) { return 7; } const next = make(n - 1); return next(); }; } function main() { const action = make(3); console.log(action()); }",
    );
}
