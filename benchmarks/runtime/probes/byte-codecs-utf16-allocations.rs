// Compile with rustc --cfg test --edition=2024 -O and the pinned base64 extern.
// The test configuration includes the exact backend codec source, with no ABI copies.
#[allow(dead_code, unused_imports)]
#[path = "../../../crates/tsrust-compiler/src/stdlib/bytes/runtime/mod.rs"]
mod backend;
include!("counting-allocator.rs");

fn observe(corpus: &str, input_bytes: usize, engine: &str, expected: &str, decode: impl FnOnce() -> String) {
    let baseline = LIVE.load(Relaxed);
    ALLOCS.store(0, Relaxed); REALLOCS.store(0, Relaxed); REQUESTED.store(0, Relaxed); PEAK.store(baseline, Relaxed);
    let output = std::hint::black_box(decode());
    let allocations = ALLOCS.load(Relaxed); let reallocations = REALLOCS.load(Relaxed);
    let requested = REQUESTED.load(Relaxed); let retained = LIVE.load(Relaxed) - baseline; let peak = PEAK.load(Relaxed) - baseline;
    assert_eq!(output, expected);
    let output_bytes = output.len(); drop(output);
    let after_drop = LIVE.load(Relaxed) as i64 - baseline as i64;
    assert_eq!(after_drop, 0);
    println!("{{\"corpus\":\"{corpus}\",\"input_bytes\":{input_bytes},\"output_bytes\":{output_bytes},\"engine\":\"{engine}\",\"allocations\":{allocations},\"reallocations\":{reallocations},\"requested\":{requested},\"retained\":{retained},\"peak\":{peak},\"after_drop\":{after_drop}}}");
}
fn main() {
    for size in [32usize, 4096, 1048576] {
        for (corpus, text) in [("mixed", "é😀Ab".repeat(size / 8)), ("cjk", "汉字".repeat(size / 4))] {
            // Both inputs and the independent known-text oracle exist before counters reset.
            let units: Vec<u16> = text.encode_utf16().collect();
            let bytes: Vec<u8> = units.iter().flat_map(|unit| unit.to_le_bytes()).collect();
            observe(corpus, bytes.len(), "std", &text, || String::from_utf16(std::hint::black_box(&units)).unwrap());
            observe(corpus, bytes.len(), "backend", &text, || backend::to_utf16(std::hint::black_box(&bytes), "little").unwrap_or_else(|error| panic!("{}", error.code)));
        }
    }
}
