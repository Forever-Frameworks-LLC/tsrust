mod support;
use support::{execute, reject};

// The first three lines use characters below U+10000, where positions agree
// with Node; their expected output was produced by Node. The last two lines
// cover characters above U+FFFF, which are one position here.
#[test]
fn string_code_point_at_and_char_code_at_read_codes_by_character_position() {
    execute(
        r#"
function first(text: string): int { return text.codePointAt(0) ?? -1; }
function next(log: int[], position: int): int { log.push(position); return position; }
function main() {
    const text = "aé中€z";
    console.log(String(text.codePointAt(0)), String(text.codePointAt(1)), String(text.codePointAt(2)), String(text.codePointAt(4)), String(text.codePointAt(5)), String(text.codePointAt(-1)), String(text.codePointAt()));
    console.log(text.charCodeAt(0), text.charCodeAt(3), text.charCodeAt(), text.charCodeAt(5), text.charCodeAt(-1), "".charCodeAt(0), String("".codePointAt(0)));
    console.log(text.codePointAt(1) ?? -1, text.codePointAt(9) ?? -1, text.charCodeAt(1) + 1.0, Number.isNaN(text.charCodeAt(7)));
    const face = "a😀b".repeat(1);
    const code = face.codePointAt(1);
    console.log(String(code), String(face.codePointAt(2)), face.charCodeAt(1), face.charCodeAt(2), first(face), first(""), face);
    const log: int[] = [];
    const found = "xyz".codePointAt(next(log, 2));
    if (found !== undefined) { console.log(found + 1, log.join(","), String.fromCodePoint(found), ("héllo".toUpperCase().codePointAt(1) ?? 0) === 201); }
}
"#,
        "97 233 20013 122 undefined undefined 97\n97 8364 97 NaN NaN NaN undefined\n233 -1 234 true\n128512 98 55357 98 97 -1 a😀b\n123 2 z true\n",
        None,
    );
}

#[test]
fn string_code_at_methods_take_one_optional_int_position() {
    for (source, code) in [
        ("function main() { console.log(String(\"a\".codePointAt(0, 1))); }", "arity"),
        ("function main() { console.log(\"a\".charCodeAt(0, 1)); }", "arity"),
        ("function main() { console.log(String(\"a\".codePointAt(0.5))); }", "type"),
        ("function main() { console.log(\"a\".charCodeAt(\"0\")); }", "type"),
        ("function main() { const code: int = \"a\".codePointAt(0); }", "type"),
        ("function main() { const code: int = \"a\".charCodeAt(0); }", "type"),
        ("function main() { const text: string = \"a\".charCodeAt(0); }", "type"),
    ] {
        reject(source, code);
    }
}
