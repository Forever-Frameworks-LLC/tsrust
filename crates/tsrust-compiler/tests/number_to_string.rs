mod support;
use support::{execute, reject};

// Expected output was produced by Node for the same values and radices.
#[test]
fn number_to_string_matches_reference_output_in_every_tested_radix() {
    execute(
        include_str!("fixtures/number-to-string/radix.tsr"),
        include_str!("fixtures/number-to-string/radix.stdout"),
        None,
    );
}

// 400 pseudo-random large, fractional, tiny and negative values in nine radices.
#[test]
fn number_to_string_matches_reference_output_for_generated_values() {
    execute(
        include_str!("fixtures/number-to-string/rand.tsr"),
        include_str!("fixtures/number-to-string/rand.stdout"),
        None,
    );
}

#[test]
fn number_to_string_checks_its_radix() {
    for (source, code) in [
        ("function main() { const v = 1.5; console.log(v.toString(2, 3)); }", "arity"),
        ("function main() { const v = 1.5; console.log(v.toString(2.5)); }", "type"),
        ("function main() { const v = 1.5; console.log(v.toString(\"2\")); }", "type"),
        ("function main() { const v = 1.5; const n: number = v.toString(); }", "type"),
    ] {
        reject(source, code);
    }
    for radix in [1, 37, 0, -2] {
        let source = format!("function main() {{ const v = 1.5; const radix = {radix}; console.log(v.toString(radix)); }}");
        let analysis = tsrust_compiler::check(&source).unwrap();
        let directory = support::TempDirectory::new();
        let path = directory.0.join("radix.rs");
        std::fs::write(&path, analysis.emit_rust()).unwrap();
        let binary = directory.0.join("radix");
        support::success(&support::run(support::rustc().arg("--edition=2024").arg(&path).arg("-o").arg(&binary), 60));
        let output = support::run(&mut std::process::Command::new(&binary), 10);
        assert!(!output.status.success(), "radix {radix}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("number.toString radix must be between 2 and 36"), "radix {radix}");
    }
}

#[test]
fn radix_storage_covers_binary_exponents_and_exact_integer_boundaries_at_o0_o3() {
    let source = "function main(){const value:number=1.5;console.log(value.toString(2));}";
    let analysis = tsrust_compiler::check(source).unwrap();
    let emitted = analysis.emit_rust();
    let witness = r#"
fn main() {
    for exponent in -1074..=1023 {
        let value = if exponent < -1022 {
            f64::from_bits(1u64 << (exponent + 1074))
        } else {
            f64::from_bits(((exponent + 1023) as u64) << 52)
        };
        let expected = if exponent >= 0 {
            "1".to_owned() + &"0".repeat(exponent as usize)
        } else {
            "0.".to_owned() + &"0".repeat((-exponent - 1) as usize) + "1"
        };
        assert_eq!(tsr_number_to_string(value, 2, "range"), expected);
        assert_eq!(tsr_number_to_string(-value, 2, "range"), "-".to_owned() + &expected);
    }
    let maximum = "1".repeat(53) + &"0".repeat(971);
    assert_eq!(tsr_number_to_string(f64::MAX, 2, "range"), maximum);
    assert_eq!(tsr_number_to_string(-f64::MAX, 2, "range"), "-".to_owned() + &maximum);
    for radix in 2..=36 {
        for (value, expected) in [(f64::NAN,"NaN"),(f64::INFINITY,"Infinity"),(f64::NEG_INFINITY,"-Infinity"),(0.0,"0"),(-0.0,"0")] {
            assert_eq!(tsr_number_to_string(value,radix,"range"),expected);
        }
        for value in [0u64,1,2,35,36,97,255,256,65535,1u64<<32,(1u64<<53)-1,1u64<<53] {
            for negative in [false,true] {
                let text=tsr_number_to_string(if negative {-(value as f64)} else {value as f64},radix,"range");
                if radix==10 {continue;}
                assert_eq!(text.starts_with('-'),negative && value!=0);
                let parsed=text.trim_start_matches('-').chars().fold(0u64,|whole,digit|whole.checked_mul(radix as u64).unwrap().checked_add(digit.to_digit(radix as u32).unwrap() as u64).unwrap());
                assert_eq!(parsed,value,"{value} radix {radix}: {text}");
            }
        }
        if radix==10 {continue;}
        for value in [f64::from_bits(1),f64::from_bits((1u64<<52)-1),f64::MIN_POSITIVE,f64::MAX,9007199254740994.0] {
            let positive=tsr_number_to_string(value,radix,"range");
            assert!(positive.len()<=f64::MAX_EXP as usize+(f64::MANTISSA_DIGITS as i32-f64::MIN_EXP) as usize+1);
            assert!(positive.chars().all(|digit|digit=='.'||digit.to_digit(radix as u32).is_some()));
            assert_eq!(tsr_number_to_string(-value,radix,"range"),"-".to_owned()+positive.as_ref());
        }
    }
}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.0.join("radix-storage.rs");
    std::fs::write(&path, emitted.replace("fn main()", "fn source_main()") + witness).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("radix-storage-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024", "-C", &format!("opt-level={level}"), "-C", "overflow-checks=on", "-F", "unsafe_code"]).arg(&path).arg("-o").arg(&binary), 60));
        support::success(&support::run(&mut std::process::Command::new(binary), 30));
    }
}
