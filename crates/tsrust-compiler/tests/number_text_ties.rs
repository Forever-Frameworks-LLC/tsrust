mod support;
use support::execute;

// Some values sit exactly halfway between two shortest digit strings. Number
// text takes the string whose last digit is even when both read back as the
// value. Expected output was produced by Node for the same 339 values through
// console output, String(), templates, toString(), join() and array display.
#[test]
fn number_text_takes_the_even_digit_on_exact_ties() {
    execute(
        include_str!("fixtures/number-text-ties/values.tsr"),
        include_str!("fixtures/number-text-ties/values.stdout"),
        None,
    );
}

// Expected output was produced by Node.
#[test]
fn template_number_text_matches_string_conversion() {
    execute(
        r#"
function main() {
    const zero = 0.0;
    const tiny = 5.960464477539062e-7;
    const values = [1.0 / zero, -1.0 / zero, zero / zero, -zero, 1e21, 1.5e300, tiny, 123456789012345680000.0, 0.1, -2.5];
    for (const value of values) { console.log(`${value}`, String(value), value.toString(), [value].join()); }
    const big = 2.5; const flag = true;
    console.log(`${big} ${flag} ${7}`);
}
"#,
        "Infinity Infinity Infinity Infinity\n-Infinity -Infinity -Infinity -Infinity\nNaN NaN NaN NaN\n0 0 0 0\n1e+21 1e+21 1e+21 1e+21\n1.5e+300 1.5e+300 1.5e+300 1.5e+300\n5.960464477539062e-7 5.960464477539062e-7 5.960464477539062e-7 5.960464477539062e-7\n123456789012345680000 123456789012345680000 123456789012345680000 123456789012345680000\n0.1 0.1 0.1 0.1\n-2.5 -2.5 -2.5 -2.5\n2.5 true 7\n",
        None,
    );
}
