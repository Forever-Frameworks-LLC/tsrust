mod support;
use support::{execute, reject};

// Values differ per run, so the program prints only properties that hold for
// every sequence of uniform draws in [0, 1). A false line would need a
// coincidence far below one in 10^30.
#[test]
fn math_random_returns_varied_numbers_from_zero_up_to_one() {
    execute(
        r#"
function draw(): number { return Math.random(); }
function main() {
    let inRange = true;
    let sum = 0.0;
    let low = 0;
    let repeats = 0;
    let previous = -1.0;
    const buckets: int[] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let i = 0;
    while (i < 20000) {
        const value = Math.random();
        if (value < 0.0 || value >= 1.0) { inRange = false; }
        if (value === previous) { repeats += 1; }
        if (value < 0.5) { low += 1; }
        let bucket = 0;
        while (bucket < 9 && value >= Number(bucket + 1) / 10.0) { bucket += 1; }
        buckets[bucket] += 1;
        previous = value;
        sum += value;
        i += 1;
    }
    const mean = sum / 20000.0;
    console.log(inRange, repeats, mean > 0.48 && mean < 0.52, low > 9400 && low < 10600);
    console.log(buckets.every((count) => count > 1700 && count < 2300));
    console.log(Math.random() === Math.random(), draw() !== draw(), Math.floor(Math.random() * 6.0) < 6.0);
    const first = [Math.random(), Math.random(), Math.random()];
    console.log(first[0] !== first[1] && first[1] !== first[2], first.length);
}
"#,
        "true 0 true true\ntrue\nfalse true true\ntrue 3\n",
        None,
    );
}

#[test]
fn math_random_takes_no_arguments_and_returns_a_number() {
    for (source, code) in [
        ("function main() { Math.random(1.0); }", "arity"),
        ("function main() { Math.random(...[1.0]); }", "arity"),
        ("function main() { const value: int = Math.random(); }", "type"),
        ("function main() { const value: string = Math.random(); }", "type"),
    ] {
        reject(source, code);
    }
}
