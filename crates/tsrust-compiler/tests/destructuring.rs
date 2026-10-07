mod support;
use support::{execute, reject};

// Expected output of each fixture was produced by tools/node-reference/run.mjs.

// Tuple, record, nested, renamed and skipped positions; for...of patterns.
#[test]
fn destructuring_patterns_bind_positions_and_fields() {
    execute(
        include_str!("fixtures/destructuring/patterns.tsr"),
        include_str!("fixtures/destructuring/patterns.stdout"),
        None,
    );
}

// A named source stays usable for positions that were copied or not taken;
// a ReadOnly parameter lends its positions.
#[test]
fn destructuring_follows_the_access_of_its_source() {
    execute(
        include_str!("fixtures/destructuring/sources.tsr"),
        include_str!("fixtures/destructuring/sources.stdout"),
        None,
    );
}

// A const taken from a loop element borrows it for reading.
#[test]
fn const_bindings_borrow_from_loop_elements() {
    execute(
        include_str!("fixtures/destructuring/borrowed-bindings.tsr"),
        include_str!("fixtures/destructuring/borrowed-bindings.stdout"),
        None,
    );
}

// A const taken from an indexed owned element, or a field of one, borrows it.
#[test]
fn const_bindings_borrow_from_indexed_elements() {
    execute(
        include_str!("fixtures/destructuring/indexed-borrows.tsr"),
        include_str!("fixtures/destructuring/indexed-borrows.stdout"),
        None,
    );
    for (body, code) in [
        ("const s = a[0]; take(s);", "borrowed-place"),
        ("const s = a[0]; a.push(\"x\");", "view-conflict"),
        ("const s = a[0]; const b = a;", "view-conflict"),
        ("take(a[0]);", "borrowed-element"),
    ] {
        reject(&format!("function take(s: string): string {{ return s; }} function main() {{ const a = [\"Pat\"]; {body} }}"), code);
    }
}

// Array positions of owned elements are readonly borrows; a short array
// stops the program at the missing position.
#[test]
fn array_patterns_borrow_owned_elements() {
    execute(
        include_str!("fixtures/destructuring/arrays.tsr"),
        include_str!("fixtures/destructuring/arrays.stdout"),
        None,
    );
    reject("function take(s: string): string { return s; } function main() { const words = [\"a\"]; const [first] = words; take(first); }", "borrowed-place");
}

// Function and arrow parameters accept patterns. Without a stated access the
// names borrow the argument; Owned<T> lets them move out.
#[test]
fn parameter_patterns_borrow_unless_owned() {
    execute(
        include_str!("fixtures/destructuring/parameters.tsr"),
        include_str!("fixtures/destructuring/parameters.stdout"),
        None,
    );
    for (source, code) in [
        ("type P = { label: string; }; function take({ label }: P): string { return label; } function main() {}", "borrowed-place"),
        ("type P = { label: string; }; function f({ missing }: P): int { return 1; } function main() {}", "type"),
        ("function f([...rest, a]: int[]): int { return a; } function main() {}", "syntax"),
    ] {
        reject(source, code);
    }
}

#[test]
fn destructuring_keeps_static_contracts() {
    for (source, code) in [
        ("function main() { const [a, ...rest = []] = [1, 2, 3]; }", "syntax"),
        ("function main() { for (let i = 0; i < 2; const [a, b] = [1, 2]) { } }", "syntax"),
        ("function pair(): [int, string] { return [1, \"a\"]; } function main() { const [a, b, c] = pair(); }", "bounds"),
        ("type P = { x: int; }; function main() { const p: P = { x: 1 }; const { y } = p; }", "type"),
        ("function pair(): [int, string] { return [1, \"a\"]; } function main() { const [a, b] = pair(); const n: int = b; }", "type"),
        ("function pair(): [int, string] { return [1, \"a\"]; } function main() { const [a, a] = pair(); }", "duplicate"),
        ("function keep(v: string): string { return v; } function main() { const words = [\"a\"]; for (const [i, w] of words.entries()) { keep(w); } }", "borrowed-place"),
        ("function main() { const [a, b] = 5; }", "type"),
    ] {
        let error = reject(source, code);
        assert!(error.span.end >= error.span.start, "{source}");
    }
}

#[test]
fn owned_array_parameter_patterns_transfer_elements_without_cloning() {
    execute("function head([first]: Owned<string[]>): string { return first; } function main() { console.log(head(['owner', 'extra'])); }", "owner\n", None);
}

#[test]
fn owned_array_parameter_patterns_keep_skips_defaults_and_rest_positions() {
    execute("function select([first, , third = 'fallback', ...rest]: Owned<string[]>): string { return first + ':' + third + ':' + rest.join(','); } function main() { console.log(select(['a','skip','c','d','e'])); console.log(select(['a'])); }", "a:c:d,e\na:fallback:\n", None);
}

#[test]
fn owned_array_patterns_keep_transfer_and_readonly_boundaries() {
    execute("function main() { const take = ([first, second]: Owned<string[]>): string => first + second; console.log(take(['a', 'b'])); }", "ab\n", None);
    reject("function head([first]: string[]): string { return first; } function main() {}", "borrowed-place");
    reject("function head([first]: Owned<string[]>): string { return first; } function main() { const values=['owner']; head(values); console.log(values.length); }", "transferred");
}

#[test]
fn owned_nested_array_parameter_patterns_transfer_nested_elements() {
    execute("function head([[first]]: Owned<string[][]>): string { return first; } function main() { console.log(head([['owner']])); }", "owner\n", None);
}
