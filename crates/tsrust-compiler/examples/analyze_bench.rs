//! An in-process analysis baseline, not a native-build or LSP benchmark.
use std::hint::black_box;
use std::time::Instant;
use tsrust_compiler::check;

fn main() {
    println!("functions,source_bytes,iterations,median_analysis_us");
    for functions in [10, 100, 1000] {
        let mut source = String::from("type User = { name: string; };\nfunction main() {}\n");
        for i in 0..functions {
            source.push_str(&format!(
                "function f{i}(user: User): string {{ return user.name.trim().toLowerCase(); }}\n"
            ));
        }
        for _ in 0..5 {
            black_box(check(black_box(&source)).unwrap_or_else(|e| panic!("{}", e.message)));
        }
        let mut samples = Vec::new();
        for _ in 0..31 {
            let start = Instant::now();
            let analysis = check(black_box(&source)).unwrap_or_else(|e| panic!("{}", e.message));
            black_box(&analysis);
            samples.push(start.elapsed().as_secs_f64() * 1_000_000.0);
            // Destruction is outside the measurement; parsing and analysis are inside.
        }
        samples.sort_by(f64::total_cmp);
        println!("{functions},{},31,{:.1}", source.len(), samples[15]);
    }
}
