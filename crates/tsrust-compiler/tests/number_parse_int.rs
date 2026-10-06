mod support;
use support::{execute, reject};

#[test]
fn parse_int_reads_integer_prefixes_in_any_supported_radix() {
    execute(
        r#"
function supply(log: int[]): string { log.push(1); return "77"; }
function main() {
    console.log(Number.parseInt("42"), Number.parseInt("  -17px"), Number.parseInt("+7"), Number.parseInt("3.9"), Number.parseInt("1e3"));
    console.log(Number.parseInt("0x1F"), Number.parseInt("0X1f"), Number.parseInt("-0x10"), Number.parseInt("0x"), Number.parseInt("0xg"));
    console.log(Number.parseInt("ff", 16), Number.parseInt("0xff", 16), Number.parseInt("101", 2), Number.parseInt("z", 36), Number.parseInt("Zz", 36));
    console.log(Number.parseInt("12", 1), Number.parseInt("12", 37), Number.parseInt("12", -2), Number.parseInt("12", 0), Number.parseInt("0x12", 0));
    console.log(Number.parseInt(""), Number.parseInt("abc"), Number.parseInt("8", 8), Number.parseInt("78", 8), Number.parseInt("-"), Number.parseInt("0x12", 10));
    console.log(Number.parseInt("-0"), Number.parseInt("9007199254740993"), Number.parseInt("123456789012345678901234567890"));
    console.log(Number.parseInt(" \t 12"), Number.parseInt("１２"), Number.parseInt("12abc", 16));
    const log: int[] = [];
    const radix = 8;
    const text = "17";
    const value: number = Number.parseInt(supply(log), radix);
    console.log(value, log.length, Number.parseInt(text, radix) + 0.5, text);
    console.log(Number.isNaN(Number.parseInt("x")), Number.parseInt("5").toFixed(1));
}
"#,
        "42 -17 7 3 1\n31 31 -16 NaN NaN\n255 255 5 35 1295\nNaN NaN NaN 12 18\nNaN NaN NaN 7 NaN 0\n-0 9007199254740992 1.2345678901234568e+29\n12 NaN 76476\n63 1 15.5 17\ntrue 5.0\n",
        None,
    );
}

#[test]
fn parse_int_requires_a_string_and_an_optional_int_radix() {
    for (source, code) in [
        ("function main() { console.log(Number.parseInt()); }", "arity"),
        ("function main() { console.log(Number.parseInt(\"1\", 10, 2)); }", "arity"),
        ("function main() { console.log(Number.parseInt(12)); }", "type"),
        ("function main() { console.log(Number.parseInt(\"12\", 10.5)); }", "type"),
        ("function main() { console.log(Number.parseInt(\"12\", \"10\")); }", "type"),
        ("function main() { const value: int = Number.parseInt(\"12\"); }", "type"),
    ] {
        reject(source, code);
    }
}
