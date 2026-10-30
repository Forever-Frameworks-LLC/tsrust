mod support;

// A chain of one operator family is length, not nesting: `a + b + c + ...`
// has no fixed limit. The checker and emitter walk it without recursion and
// the emitted Rust is flat, so only operands nested inside count.

fn run(source: &str, expected: &str) -> String {
    let rust = tsrust_compiler::check(source)
        .unwrap_or_else(|error| panic!("{}", error.render("chains.tsr", source)))
        .emit_rust();
    let directory = support::TempDirectory::new();
    let path = directory.0.join("chains.rs");
    let executable = directory.0.join("chains");
    std::fs::write(&path, &rust).unwrap();
    support::success(&support::run(
        support::rustc().args(["--edition=2024", "-C", "overflow-checks=on"]).arg(&path).arg("-o").arg(&executable),
        300,
    ));
    let output = support::run(&mut std::process::Command::new(&executable), 30);
    support::success(&output);
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    // The rustc warnings for redundant grouping must not be emitted per level.
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
    rust
}

fn chain(operand: &str, operator: &str, count: usize) -> String {
    vec![operand; count].join(operator)
}

#[test]
fn long_operator_chains_compile_flat_and_keep_their_results() {
    let count = 2000;
    let source = format!(
        "function main() {{
  const k = 2;
  const x = 0.5;
  const yes = true;
  const no = false;
  const text = {};
  const sum = {};
  const mixed = {};
  const half = {};
  const all = {};
  const any = {};
  console.log(text.length, sum, mixed, half, all, any);
}}",
        chain("\"ab\"", " + ", count),
        chain("k", " + ", count),
        chain("k * 3 - 1", " + ", count),
        chain("x", " + ", count),
        chain("yes", " && ", count),
        chain("no", " || ", count) + " || yes",
    );
    let rust = run(&source, "4000 4000 10000 1000 true true\n");
    assert!(rust.contains("let mut chain: i64"), "int chains reuse one running value");
    assert!(rust.contains("let mut chain: bool"), "boolean chains reuse one running value");
    assert!(!rust.contains("((((("), "chains do not nest parentheses per level");
}

#[test]
fn chains_short_circuit_and_check_overflow_at_their_own_operator() {
    let source = r#"function seen(log: string[], value: boolean): boolean { log.push(String(value)); return value; }
function main() {
  const log: string[] = [];
  const all = seen(log, true) && seen(log, false) && seen(log, true) && seen(log, true);
  const any = seen(log, false) || seen(log, true) || seen(log, false);
  const big = 9223372036854775000;
  console.log(all, any, log.join(","));
  try {
    const sum = big + 100 + 700 + 10;
    console.log(sum);
  } catch (error) {
    console.log("overflow");
  }
}"#;
    run(source, "false true true,false,false,true\noverflow\n");
}

#[test]
fn chains_that_are_not_emitted_flat_still_compile() {
    let operands = chain("a", " || ", 300);
    let source = format!(
        "function pick(value: int | undefined): int | undefined {{ return value; }}
function main() {{ const a = pick(3); const result = {operands}; console.log(result); }}"
    );
    support::accept_deep(&source);
}

#[test]
fn checking_a_chain_is_linear_in_its_length() {
    // Quadratic type hints took minutes here; linear work takes well under a second.
    for operator in [" + ", " && "] {
        let operand = if operator == " + " { "k" } else { "yes" };
        let source = format!("function main() {{ const k = 1; const yes = true; const value = {}; }}", chain(operand, operator, 50_000));
        let started = std::time::Instant::now();
        tsrust_compiler::check(&source).unwrap_or_else(|error| panic!("{}", error.message)).emit_rust();
        assert!(started.elapsed() < std::time::Duration::from_secs(60), "{operator} chain took {:?}", started.elapsed());
    }
}
