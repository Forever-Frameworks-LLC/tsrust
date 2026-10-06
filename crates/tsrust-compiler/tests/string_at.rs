mod support;
use support::{execute, reject};

#[test]
fn string_at_selects_characters_from_either_end() {
    execute(
        r#"
function pick(text: string, index: int): string { return text.at(index) ?? "none"; }
function supply(log: int[]): string { log.push(1); return "xyz"; }
function main() {
    const text = "aé😀b";
    console.log(String(text.at(0)), String(text.at(1)), String(text.at(2)), String(text.at(3)));
    console.log(String(text.at(-1)), String(text.at(-4)), String(text.at(4)), String(text.at(-5)));
    console.log(pick(text, 2), pick(text, 9), pick("", 0), pick("", -1));
    const first = text.at(0);
    if (first !== undefined) { console.log(first.length, first + "!"); }
    const log: int[] = [];
    console.log(String(supply(log).at(-1)), log.length, text);
    console.log(String("literal".at(1)), String(text.toUpperCase().at(0)));
}
"#,
        "a é 😀 b\nb a undefined undefined\n😀 none none none\n1 a!\nz 1 aé😀b\ni A\n",
        None,
    );
}

#[test]
fn string_at_requires_one_int_index() {
    for (source, code) in [
        ("function main() { console.log(String(\"ab\".at())); }", "arity"),
        ("function main() { console.log(String(\"ab\".at(0, 1))); }", "arity"),
        ("function main() { console.log(String(\"ab\".at(1.5))); }", "type"),
        ("function main() { console.log(String(\"ab\".at(\"0\"))); }", "type"),
        ("function main() { const letter: string = \"ab\".at(0); }", "type"),
    ] {
        reject(source, code);
    }
}
