fn benchmark_checksum(value: i64) -> i64 { value }
// Timing hooks only. The generated workload body is unchanged.
fn main() {
    let arguments: std::vec::Vec<std::string::String> = std::env::args().collect();
    let iterations: i64 = arguments[1].parse().unwrap();
    let seed: i64 = arguments[2].parse().unwrap();
    let warmups: usize = arguments[3].parse().unwrap();
    let samples: usize = arguments[4].parse().unwrap();
    let mut expected = None;
    let mut validate = |value: i64| {
        assert!(value.unsigned_abs() <= 9_007_199_254_740_991);
        if let Some(previous) = expected {
            assert_eq!(previous, value);
        }
        expected = Some(value);
    };
    for _ in 0..warmups {
        validate(benchmark_checksum(std::hint::black_box(function0(
            std::hint::black_box(iterations),
            std::hint::black_box(seed),
        ))));
    }
    let mut durations = Vec::new();
    for _ in 0..samples {
        let start = std::time::Instant::now();
        let value = std::hint::black_box(function0(
            std::hint::black_box(iterations),
            std::hint::black_box(seed),
        ));
        durations.push(start.elapsed().as_secs_f64() * 1000.0);
        validate(benchmark_checksum(value));
    }
    println!(
        "{{\"checksum\":{},\"durations\":{:?}}}",
        expected.unwrap(),
        durations
    );
}
