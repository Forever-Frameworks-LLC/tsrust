// Appended to the unchanged emitted workload after number-observation.rs.
fn main() {
    let arguments: Vec<std::string::String> = std::env::args().collect();
    let iterations: i64 = arguments[1].parse().unwrap();
    let seed: i64 = arguments[2].parse().unwrap();
    let warmups: usize = arguments[3].parse().unwrap();
    let samples: usize = arguments[4].parse().unwrap();
    let mut expected = None;
    let mut validate = |value| {
        let observed = (number_checksum(&value), number_exact_bits(&value));
        if let Some(previous) = &expected { assert_eq!(previous, &observed); }
        expected = Some(observed);
    };
    for _ in 0..warmups {
        validate(std::hint::black_box(function0(
            std::hint::black_box(iterations),
            std::hint::black_box(seed),
        )));
    }
    let mut durations = Vec::new();
    for _ in 0..samples {
        let start = std::time::Instant::now();
        let value = std::hint::black_box(function0(
            std::hint::black_box(iterations),
            std::hint::black_box(seed),
        ));
        durations.push(start.elapsed().as_secs_f64() * 1000.0);
        validate(value);
    }
    let (checksum, exact_bits) = expected.unwrap();
    println!("{{\"checksum\":{},\"exactBits\":{:?},\"durations\":{:?}}}", checksum, exact_bits, durations);
}
