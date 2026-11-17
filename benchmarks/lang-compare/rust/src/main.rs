//! The same CPU-bound phases as ../tsrust, written as ordinary Rust. Each phase
//! prints its time and a checksum; both versions must print the same checksums.
mod data;
mod numeric;
mod text;

use std::time::Instant;

fn report(name: &str, started: Instant, checksum: String) {
    println!("{name} {:.3} {checksum}", started.elapsed().as_secs_f64() * 1000.0);
}

fn main() {
    // SCALE multiplies each phase's element or step count (SCALE=0.1 for quick
    // checks); PHASE runs one phase alone, for profiling.
    let scale: f64 = std::env::var("SCALE").ok().and_then(|value| value.parse().ok()).unwrap_or(1.0);
    let only = std::env::var("PHASE").unwrap_or_default();
    let size = |value: usize| ((value as f64 * scale).floor() as usize).max(1);
    let wanted = |name: &str| only.is_empty() || only == name;
    let started = Instant::now();
    if wanted("sieve") {report("sieve", started, numeric::sieve(size(20_000_000)).to_string());}
    let started = Instant::now();
    if wanted("nbody") {report("nbody", started, format!("{:.9}", numeric::nbody(size(1_000_000))));}
    let started = Instant::now();
    if wanted("matrix") {report("matrix", started, format!("{:.6}", numeric::matrix(300)));}
    let started = Instant::now();
    if wanted("fib") {report("fib", started, numeric::fib(32).to_string());}
    let started = Instant::now();
    let text = if wanted("text") || wanted("words") {text::make_text(size(3_000_000))} else {String::new()};
    if wanted("text") {report("text", started, text.len().to_string());}
    let started = Instant::now();
    if wanted("words") {report("words", started, text::word_count(&text).to_string());}
    let started = Instant::now();
    if wanted("format") {report("format", started, text::formatting(size(1_000_000)).to_string());}
    let started = Instant::now();
    if wanted("orders") {report("orders", started, data::orders(size(1_000_000)).to_string());}
    let started = Instant::now();
    if wanted("trees") {report("trees", started, data::trees(18).to_string());}
    let started = Instant::now();
    if wanted("sets") {report("sets", started, data::sets(size(3_000_000)).to_string());}
}
