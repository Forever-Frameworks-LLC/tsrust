//! Frontend partial-field checking; excludes emission and native compilation.
use std::hint::black_box;
use std::time::Instant;
use tsrust_compiler::check;

fn measure(workload: &str, functions: usize, source: &str) {
    for _ in 0..5 {
        black_box(check(black_box(source)).unwrap_or_else(|e| panic!("{}", e.message)));
    }
    let mut samples = Vec::new();
    for _ in 0..31 {
        let start = Instant::now();
        let analysis = check(black_box(source)).unwrap_or_else(|e| panic!("{}", e.message));
        black_box(&analysis);
        samples.push(start.elapsed().as_secs_f64() * 1_000_000.0);
        // Result destruction and source generation remain outside the timer.
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "{workload},{functions},{},31,{:.1}",
        source.len(),
        samples[15]
    );
}

fn main() {
    println!("workload,functions,source_bytes,iterations,median_analysis_us");
    measure(
        "profile",
        1,
        include_str!("../tests/fixtures/field-moves/profile.tsr"),
    );
    for functions in [10, 100, 1000] {
        let mut source = String::from(
            "type User = { name: string; backup: string; active: boolean; };\nfunction main() {}\n",
        );
        for i in 0..functions {
            source.push_str(&format!(
                "function f{i}(user: User, flag: boolean): string {{ const name = user.name; if (flag) {{ user.name = name; }} else {{ user.name = \"New\"; }} const saved = user; return saved.backup; }}\n"
            ));
        }
        measure("branch_restore", functions, &source);
    }
}
