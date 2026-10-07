mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let directory = support::TempDirectory::new();
    let path = directory.0.join("control.rs");
    std::fs::write(&path, analysis.emit_rust()).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("control-{level}"));
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
fn for_and_do_execute_tails_after_continue_but_not_break_or_return() {
    native(
        r#"
        type State = { n: int; };
        function condition(state: State, limit: int): boolean { console.log("condition"); return state.n < limit; }
        function update(state: State): int { console.log("update"); state.n = state.n + 1; return state.n; }
        function stop(state: State): int {
            for (; condition(state, 3); update(state)) { console.log("return"); return 7; }
            return 8;
        }
        function main() {
            const state: State = { n: 0 };
            for (; condition(state, 3); update(state)) {
                console.log(state.n);
                if (state.n === 0) { { continue; } }
                break;
            }
            console.log(state.n); console.log(stop(state)); console.log(state.n);
            state.n = 0;
            do { console.log("do"); state.n = state.n + 1; continue; } while (condition(state, 2));
            console.log(state.n);
            do { console.log("break"); break; } while (condition(state, 9));
            do { console.log("once"); } while (false);
            for (;;) { console.log("unconditional"); break; }
            for (let i: int = 0; i < 0; i = i + 1) { console.log("never"); }
            for (let of = 0; of < 1; of = of + 1) { console.log(of); }
        }
    "#,
        "condition\n0\nupdate\ncondition\n1\n1\ncondition\nreturn\n7\n1\ndo\ncondition\ndo\ncondition\n2\nbreak\nonce\nunconditional\n0\n",
    );
}

#[test]
fn nested_source_loops_replace_the_nearest_continue_and_break_context() {
    native(
        r#"
        function main() {
            const numbers = [0,1,2];
            for (let i = 0; i < 2; i = i + 1) {
                let n = 0;
                while (n < 2) { n = n + 1; if (n === 1) { continue; } break; }
                console.log(n);
                for (const item of numbers) { if (item === 0) { continue; } console.log(item); break; }
                do { let k = 0; for (; k < 1; k = k + 1) { continue; } console.log(k); break; } while (true);
                continue;
            }
            let n = 0;
            while (n < 2) {
                for (let i = 0; i < 2; i = i + 1) { if (i === 0) { continue; } break; }
                n = n + 1; continue;
            }
            console.log(n);
        }
    "#,
        "2\n1\n1\n2\n1\n1\n2\n",
    );
}

#[test]
fn loop_tails_restore_moved_storage_and_do_first_iteration_has_no_zero_path() {
    let analysis = native(
        r#"
        type Box = { text: string; };
        function restore(flag: boolean) {
            let text = "original";
            for (let n = 0; n < 2; text = "restored") {
                n = n + 1; const taken = text; console.log(taken);
                if (flag) { continue; }
            }
            console.log(text);
        }
        function fieldRestore() {
            const box: Box = { text: "field" };
            for (let n = 0; n < 2; box.text = "field-restored") {
                n = n + 1; const taken = box.text; console.log(taken); continue;
            }
            console.log(box.text);
        }
        function first() {
            let text = "old"; const old = text;
            do { text = "new"; } while (false);
            console.log(old); console.log(text);
        }
        function allReturn(text: string): string { do { return text; } while (false); }
        function unusedCondition(text: string): boolean { console.log("unreachable"); console.log(text); return true; }
        function returnOnly(text: string): string { do { return text; } while (unusedCondition(text)); }
        function breakReturn(text: string, stop: boolean): string {
            do { if (stop) { return text; } break; } while (false);
            return text;
        }
        function main() { restore(true); restore(false); fieldRestore(); first(); console.log(allReturn("return")); console.log(returnOnly("tail-skipped")); console.log(breakReturn("keep",false)); }
    "#,
        "original\nrestored\nrestored\noriginal\nrestored\nrestored\nfield\nfield-restored\nfield-restored\nold\nnew\nreturn\ntail-skipped\nkeep\n",
    );
    assert_eq!(
        analysis.contracts().get("allReturn.text"),
        Some(&Mode::Consume)
    );
    assert_eq!(
        analysis.contracts().get("breakReturn.text"),
        Some(&Mode::Consume)
    );
}

#[test]
fn lexical_blocks_release_capture_loans_before_loop_tails_and_preserve_factories() {
    let analysis = native(
        r#"
        type Box = { text: string; n: int; };
        function advance(box: Box, n: int): int { box.text = "next"; return n + 1; }
        function next(box: Box): boolean { box.text = "next"; box.n = box.n + 1; return box.n < 2; }
        function make(text: string): () => string { { const local = text; return (): string => { return local.trim(); }; } }
        function makeDo(text: string): () => string { const action = (): string => { return text.trim(); }; do { return action; } while (false); }
        function step(action: Mutable<Function<int>>, n: int): int { return n + action(); }
        function main() {
            const box: Box = { text: "start", n: 0 };
            for (let i = 0; i < 2; i = advance(box, i)) { const read = (): string => { return box.text.trim(); }; console.log(read()); continue; }
            box.text = "start";
            do { const read = (): string => { return box.text.trim(); }; console.log(read()); continue; } while (next(box));
            { const text = "block"; console.log(text); }
            const text = "outside"; console.log(text);
            const action = make(" owned ".toUpperCase()); console.log(action()); console.log(action());
            const other = makeDo(" do "); console.log(other()); console.log(other());
            const increment = (): int => { return 1; };
            for (let i = 0; i < 2; i = step(increment, i)) { console.log(i); continue; }
        }
    "#,
        "start\nnext\nstart\nnext\nblock\noutside\nOWNED\nOWNED\ndo\ndo\n0\n1\n",
    );
    assert_eq!(analysis.contracts().get("advance.box"), Some(&Mode::Mutate));
    assert_eq!(analysis.contracts().get("next.box"), Some(&Mode::Mutate));
    assert_eq!(analysis.contracts().get("make.text"), Some(&Mode::Consume));
}

#[test]
fn new_loops_keep_scope_type_transfer_and_control_diagnostics() {
    for (source, code) in [
        (
            "function main() { { const local = 1; } console.log(local); }",
            "name",
        ),
        (
            "function main() { for (let i = 0; i < 1; i = i + 1) {} console.log(i); }",
            "name",
        ),
        ("function main() { for (; 1;) {} }", "type"),
        ("function main() { do {} while (1); }", "type"),
        ("function main() { do { return; } while (1); }", "type"),
        (
            "function main() { do { return; } while (missing); }",
            "name",
        ),
        ("function main() { do { return; } while (false) }", "syntax"),
        ("function main() { for (;true;return) {} }", "syntax"),
        ("function main() { do {} while (local); }", "name"),
        (
            "function main() { do { const local = true; } while (local); }",
            "name",
        ),
        ("function main() { { continue; } }", "loop-control"),
        (
            "function main() { for (let i=0; i<1; i=i+1) { const action=(): void => { continue; }; } }",
            "loop-control",
        ),
        (
            // A value named alone is only read; passing it away each iteration moves it.
            "function take(s:string):string{return s;} function main() { const text=\"x\"; for (;;take(text)) { continue; } }",
            "loop-transfer",
        ),
        (
            "function main() { const text=\"x\"; do { text; continue; } while (false); }",
            "loop-transfer",
        ),
        (
            "function main() { let text=\"x\";const taken=text;for (;false;) {text=\"new\";}console.log(text); }",
            "transferred",
        ),
        (
            "function main() { const text=\"x\";do {text;break;}while(false);console.log(text); }",
            "transferred",
        ),
        (
            "function main() { let text=\"x\";for(;true;text=\"restored\") {text;break;}console.log(text); }",
            "transferred",
        ),
        (
            "function main() { for (;;) { return; } console.log(1); }",
            "unsupported",
        ),
        ("function main() { for (;false;let i=1) {} }", "syntax"),
        ("function main() { do {} while (false) }", "syntax"),
    ] {
        let error = check(source)
            .err()
            .unwrap_or_else(|| panic!("Expected {code}: {source}"));
        assert_eq!(error.code, code, "{source}");
    }
}

#[test]
fn else_if_chains_preserve_mixed_returns_captures_and_loop_control() {
    let analysis = native(
        r#"
        function select(text: string, n: int): string {
            if (n === 0) { return text.trim(); }
            else if (n === 1) { return text.toUpperCase(); }
            else if (n === 2) { return text.toLowerCase(); }
            else { return clone(text); }
        }
        function move(text: string, n: int): string {
            if (n === 0) { return text; }
            else if (n === 1) { return text; }
            else { return text; }
        }
        function main() {
            const text = " Mixed ";
            console.log(select(text,0)); console.log(select(text,1)); console.log(select(text,2)); console.log(select(text,3));
            const read = (n: int): string => {
                if (n === 0) { return text.trim(); }
                else if (n === 1) { return text.toUpperCase(); }
                else { return text.toLowerCase(); }
            };
            console.log(read(0)); console.log(read(1)); console.log(read(2)); console.log(text);
            console.log(move("zero",0)); console.log(move("one",1)); console.log(move("else",2));
            for (let i = 0; i < 4; i = i + 1) {
                if (i === 0) { continue; }
                else if (i === 1) { console.log("one"); }
                else if (i === 2) { break; }
                else { console.log("never"); }
            }
        }
    "#,
        "Mixed\n MIXED \n mixed \n Mixed \nMixed\n MIXED \n mixed \n Mixed \nzero\none\nelse\none\n",
    );
    assert_eq!(analysis.contracts().get("select.text"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("move.text"), Some(&Mode::Consume));
    assert_eq!(analysis.closures()[0].invocation, Mode::Read);
}

#[test]
fn else_if_chains_retain_branch_ownership_and_bounded_nesting() {
    let moved = "function f(text: string, n: int) { if (n === 0) { text; } else if (n === 1) {} else { return; } console.log(text); } function main() {}";
    assert_eq!(check(moved).err().unwrap().code, "transferred");
    let short = format!(
        "function main() {{ if (false) {{}} {} else {{ console.log(1); }} }}",
        "else if (false) {} ".repeat(60)
    );
    check(&short)
        .expect("bounded conditional chain")
        .emit_rust();
    let deep = format!(
        "function main() {{ if (false) {{}} {} else {{}} }}",
        "else if (false) {} ".repeat(200)
    );
    assert_eq!(check(&deep).err().unwrap().code, "unsupported");
}
