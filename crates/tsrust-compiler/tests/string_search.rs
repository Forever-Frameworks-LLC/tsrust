mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, assertions: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis
        .emit_rust()
        .replace("fn main()", "fn source_main()")
        + &format!("\nfn main() {{ source_main(); {assertions} }}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("search.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("search-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(&path)
                .arg("-o")
                .arg(&executable),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(&executable), 10);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn string_search_matches_standard_utf8_content_and_clamped_byte_positions() {
    let source = r#"
        function starts(text: string, needle: string, position: int): boolean { return text.startsWith(needle, position); }
        function ends(text: string, needle: string, position: int): boolean { return text.endsWith(needle, position); }
        function includes(text: string, needle: string, position: int): boolean { return text.includes(needle, position); }
        function index(text: string, needle: string, position: int): int { return text.indexOf(needle, position); }
        function main() {
            const text = "Aé😀\u0000Aé";
            console.log(text.startsWith("A")); console.log(text.endsWith("é"));
            console.log(text.includes("😀")); console.log(text.indexOf("😀"));
            console.log(text.indexOf("é", -1)); console.log(text.endsWith("", -1));
            console.log(text.startsWith("", 999)); console.log(text.indexOf("", 999));
            console.log(text); console.log("".indexOf(""));
            const decomposed = "e\u0301"; const composed = "é";
            console.log(decomposed.startsWith(composed)); console.log(decomposed.endsWith(composed));
            console.log(decomposed.includes(composed)); console.log(decomposed.indexOf(composed));
            console.log(composed.startsWith(decomposed)); console.log(composed.endsWith(decomposed));
            console.log(composed.includes(decomposed)); console.log(composed.indexOf(decomposed));
            console.log(decomposed.startsWith("\u0301", 1)); console.log(decomposed.endsWith("e", 1));
            console.log(decomposed.includes("\u0301", 1)); console.log(decomposed.indexOf("\u0301", 1));
        }
    "#;
    let analysis = native(
        source,
        "true\ntrue\ntrue\n3\n1\ntrue\ntrue\n11\nAé😀\0Aé\n0\nfalse\nfalse\nfalse\n-1\nfalse\nfalse\nfalse\n-1\ntrue\ntrue\ntrue\n1\n",
        r#"
        for text in ["", "aAa\0Z", "Aé😀\0Aé", "e\u{0301}", "é"] {
            for owned in [false, true] {
                let receiver = if owned { String::Owned(text.to_owned()) } else { String::Borrowed(text) };
                let mut positions = vec![i64::MIN, -3, -1, i64::MAX];
                positions.extend(text.char_indices().map(|(byte,_)| byte as i64)); positions.extend([text.len() as i64, text.len() as i64 + 1]);
                for needle in ["", "A", "é", "😀", "\0", "Aé", "é😀", "\0A", "e\u{0301}", "\u{0301}", "missing"] {
                    let needle = String::Owned(needle.to_owned());
                    for &position in &positions {
                        let normalized = if position < 0 { 0 } else { (position as u128).min(text.len() as u128) as usize };
                        let suffix = &text[normalized..]; let prefix = &text[..normalized];
                        assert_eq!(function0(&receiver,&needle,position), suffix.starts_with(needle.as_ref()));
                        assert_eq!(function1(&receiver,&needle,position), prefix.ends_with(needle.as_ref()));
                        assert_eq!(function2(&receiver,&needle,position), suffix.contains(needle.as_ref()));
                        assert_eq!(function3(&receiver,&needle,position), suffix.find(needle.as_ref()).map_or(-1,|offset| (normalized+offset) as i64));
                        assert_eq!(receiver.as_ref(),text);
                    }
                }
            }
        }
        let text = String::Borrowed("😀"); let empty = String::Borrowed("");
        let hook = std::panic::take_hook(); std::panic::set_hook(Box::new(|_| {}));
        for operation in [0,1,2,3] {
            assert!(std::panic::catch_unwind(|| match operation {
                0 => {function0(&text,&empty,1);}, 1 => {function1(&text,&empty,1);},
                2 => {function2(&text,&empty,1);}, _ => {function3(&text,&empty,1);}
            }).is_err());
        }
        std::panic::set_hook(hook);
    "#,
    );
    for function in ["starts", "ends", "includes", "index"] {
        for parameter in ["text", "needle"] {
            assert_eq!(
                analysis.contracts().get(&format!("{function}.{parameter}")),
                Some(&Mode::Read)
            );
        }
    }
}

#[test]
fn search_receiver_needle_position_evaluate_once_and_temporaries_remain_independent() {
    native(
        r#"
        type Box = {text: string;};
        function receiver(box: Box): string { console.log("receiver"); return clone(box.text); }
        function needle(): string { console.log("needle"); return "a"; }
        function position(box: Box): int { console.log("position"); box.text = "changed"; return 0; }
        function main() {
            const box: Box = {text:"abc"};
            console.log(receiver(box).startsWith(needle(),position(box))); console.log(box.text);
            box.text = "abc"; console.log(receiver(box).endsWith(needle(),position(box)));
            box.text = "abc"; console.log(receiver(box).includes(needle(),position(box)));
            box.text = "abc"; console.log(receiver(box).indexOf(needle(),position(box)));
        }
    "#,
        "receiver\nneedle\nposition\ntrue\nchanged\nreceiver\nneedle\nposition\nfalse\nreceiver\nneedle\nposition\ntrue\nreceiver\nneedle\nposition\n0\n",
        "",
    );
}

#[test]
fn search_reads_reusable_captures_records_array_elements_and_array_overloads() {
    let analysis = native(
        r#"
        type Box = {text: string;};
        function read(box: Box, needle: string): boolean { return box.text.includes(needle); }
        function main() {
            const text = "é😀é"; const needle = "😀";
            const same = (): boolean => { return text.startsWith("é") && "é😀é".endsWith(needle,6) && text.includes(needle) && text.indexOf(needle) === 2; };
            console.log(same()); console.log(same());
            const box: Box = {text:text.toUpperCase()}; const words = ["é", "😀"];
            console.log(read(box,words[1])); console.log(words[0].endsWith("é"));
            console.log(words.includes(needle)); console.log(words.indexOf(needle));
            const numbers = [1,2,1]; console.log(numbers.includes(1,-1)); console.log(numbers.indexOf(1,1));
            console.log(text); console.log(needle); console.log(words[0]);
        }
    "#,
        "true\ntrue\ntrue\ntrue\ntrue\n1\ntrue\n2\né😀é\n😀\né\n",
        "",
    );
    assert_eq!(analysis.contracts().get("read.box"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("read.needle"), Some(&Mode::Read));
    assert_eq!(analysis.closures()[0].invocation, Mode::Read);
}

#[test]
fn string_search_checks_types_arity_and_receiver_needle_position_loans() {
    for method in ["startsWith", "endsWith", "includes", "indexOf"] {
        for invocation in [
            format!("\"x\".{method}()"),
            format!("\"x\".{method}(\"x\",0,1)"),
        ] {
            assert_eq!(
                check(&format!("function main() {{ {invocation}; }}"))
                    .err()
                    .unwrap()
                    .code,
                "arity"
            );
        }
        for invocation in [
            format!("\"x\".{method}(1)"),
            format!("\"x\".{method}(\"x\",true)"),
            format!("(1).{method}(\"x\")"),
        ] {
            assert_eq!(
                check(&format!("function main() {{ {invocation}; }}"))
                    .err()
                    .unwrap()
                    .code,
                "type"
            );
        }
        let declarations = "type Box = {text:string;}; function change(box: Box): string {box.text=\"changed\";return \"changed\";} function position(box: Box): int {box.text=\"changed\";return 0;} function take(text: string): string {return text;}";
        for expression in [
            format!("box.text.{method}(change(box))"),
            format!("box.text.{method}(\"x\",position(box))"),
            format!("\"receiver\".{method}(box.text,position(box))"),
            format!("text.{method}(take(text))"),
        ] {
            let source = format!(
                "{declarations} function main() {{ const box: Box={{text:\"x\"}};const text=\"x\";console.log({expression}); }}"
            );
            let error = check(&source).err().unwrap();
            assert_eq!(error.code, "conflicting-access");
            assert!(!error.related.is_empty());
        }
        let moved = format!(
            "function main() {{ const text=\"x\";const moved=text;console.log(text.{method}(\"x\")); }}"
        );
        assert_eq!(check(&moved).err().unwrap().code, "transferred");
    }
}

#[test]
fn short_and_long_haystack_searches_agree_at_the_switch_over() {
    // Haystacks of 63-66 bytes straddle the short-search limit; expected byte
    // offsets were computed independently from the UTF-8 encodings.
    support::execute(&support::format_round_trip("function main(){const hays:string[]=[\"éééééééééééééééééééééééééééééééx\",\"éééééééééééééééééééééééééééééééé\",\"ééééééééééééééééééééééééééééééééx\",\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaé\",\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaé😀\"];const needles:string[]=[\"x\",\"é\",\"éx\",\"😀\",\"a\",\"\",\"zz\"];for(const h of hays){for(const n of needles){console.log(h.indexOf(n),h.lastIndexOf(n),h.includes(n),h.indexOf(n,2));}}}"), "62 62 true 62\n0 60 true 2\n60 60 true 60\n-1 -1 false -1\n-1 -1 false -1\n0 63 true 2\n-1 -1 false -1\n-1 -1 false -1\n0 62 true 2\n-1 -1 false -1\n-1 -1 false -1\n-1 -1 false -1\n0 64 true 2\n-1 -1 false -1\n64 64 true 64\n0 62 true 2\n62 62 true 62\n-1 -1 false -1\n-1 -1 false -1\n0 65 true 2\n-1 -1 false -1\n-1 -1 false -1\n62 62 true 62\n-1 -1 false -1\n-1 -1 false -1\n0 61 true 2\n0 64 true 2\n-1 -1 false -1\n-1 -1 false -1\n63 63 true 63\n-1 -1 false -1\n65 65 true 65\n0 62 true 2\n0 69 true 2\n-1 -1 false -1\n", None);
}
