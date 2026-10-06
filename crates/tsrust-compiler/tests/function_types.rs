mod support;
use tsrust_compiler::check;

fn emitted(source: &str) -> String {
    check(source)
        .unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)))
        .emit_rust()
}

fn native(source: &str, expected: &str) {
    assert!(!emitted(source).contains("unsafe"));
    support::execute(source, expected, None);
}

fn rejected(source: &str, code: &str, message: &str) {
    let error = check(source).err().expect("source must be rejected");
    assert_eq!(error.code, code, "{source}\n{}", error.message);
    assert!(error.message.contains(message), "{}", error.message);
}

/// Source byte offsets appear in native failure messages; the two spellings
/// have different lengths, so compare everything else.
fn without_offsets(rust: &str) -> String {
    let mut output = String::new();
    let mut rest = rust;
    while let Some(at) = rest.find("bytes ") {
        output.push_str(&rest[..at + 6]);
        rest = rest[at + 6..].trim_start_matches(|c: char| c.is_ascii_digit() || c == '.');
    }
    output.push_str(rest);
    output
}

const BODY: &str = r#"
function length(text: string): int { return text.length; }
function main() {
    let count = 0;
    console.log(twice((): int => { count += 1; return count; }), count);
    console.log(peek(length, "four"));
    console.log(finish((value: int): string => { return String(value); }));
    const kept = keep((value: int): int => { return value + 1; });
    console.log(kept(1));
    const row: Row = { n: 1 };
    console.log(bump((target: Row): void => { target.n = target.n + 1; }, row));
    console.log(take((given: Row): Row => { return given; }, { n: 9 }));
    console.log(total((first: int, limit?: int, ...rest: int[]): int => { return first + rest.length; }));
    console.log(maybe(null), maybe((value: int): int => { return value * 2; }));
    const holder: Holder = { action: (value: int): int => { return value - 1; } };
    console.log(holder.action(5));
}
"#;

const ARROW_TYPES: &str = r#"
type Row = { n: int; };
type Holder = { action: (value: int) => int; };
function twice(action: Mutable<Function<int>>): int { return action() + action(); }
function peek(action: ReadOnly<Function<int, [text: ReadOnly<string>]>>, text: string): int { return action(text); }
function finish(action: Once<Function<string, [value: int]>>): string { return action(7); }
function keep(action: (value: int) => int): (value: int) => int { return action; }
function bump(change: Mutable<Function<void, [row: Mutable<Row>]>>, row: Row): int { change(row); return row.n; }
function take(sink: ReadOnly<Function<Row, [row: Owned<Row>]>>, row: Row): int { return sink(row).n; }
function total(action: ReadOnly<Function<int, [first: int, limit?: int, ...rest: int[]]>>): int { return action(1) + action(1, 2, 3, 4); }
function maybe(action: (ReadOnly<Function<int, [value: int]>>) | null): int { if (action !== null) { return action(3); } return 0; }
"#;

const NAMED: &str = r#"
type Row = { n: int; };
type Holder = { action: Function<int, [value: int]>; };
function twice(action: Mutable<Function<int>>): int { return action() + action(); }
function peek(action: ReadOnly<Function<int, [text: ReadOnly<string>]>>, text: string): int { return action(text); }
function finish(action: Once<Function<string, [value: int]>>): string { return action(7); }
function keep(action: Function<int, [value: int]>): Function<int, [value: int]> { return action; }
function bump(change: Mutable<Function<void, [row: Mutable<Row>]>>, row: Row): int { change(row); return row.n; }
function take(sink: ReadOnly<Function<Row, [row: Owned<Row>]>>, row: Row): int { return sink(row).n; }
function total(action: ReadOnly<Function<int, [first: int, limit?: int, ...rest: int[]]>>): int { return action(1) + action(1, 2, 3, 4); }
function maybe(action: ReadOnly<Function<int, [value: int]>> | null): int { if (action !== null) { return action(3); } return 0; }
"#;

#[test]
fn function_types_match_arrow_contracts_and_native_output() {
    let named = format!("{NAMED}{BODY}");
    let keywords = format!("{ARROW_TYPES}{BODY}");
    native(&named, "3 2\n4\n7\n2\n2\n9\n4\n0 6\n4\n");
    assert_eq!(without_offsets(&emitted(&named)), without_offsets(&emitted(&keywords)));
}

#[test]
fn function_types_accept_unlabelled_nested_and_stored_forms() {
    native(
        r#"
type Step = { run: Function<int, [int]>; };
function apply(action: Mutable<Function<int, [int, int]>>): int { return action(2, 3); }
function build(seed: int): Function<int, [int]> { return (value: int): int => { return value + seed; }; }
function nested(make: Function<Function<int>, [int]>): int { const inner = make(4); return inner(); }
function each(actions: Function<int>[]): int { let total = 0; for (const action of actions) { total += 1; } return total; }
function main() {
    console.log(apply((left: int, right: int): int => { return left * right; }));
    const add = build(10);
    console.log(add(1));
    console.log(nested((value: int): Function<int> => { return (): int => { return value; }; }));
    const step: Step = { run: (value: int): int => { return value * 2; } };
    console.log(step.run(4));
    const local: ReadOnly<Function<int, [int]>> = (value: int): int => { return value; };
    console.log(local(5), each([]));
}
"#,
        "6\n11\n4\n8\n5 0\n",
    );
}

#[test]
fn function_types_reject_malformed_and_misplaced_wrappers() {
    for (source, code, message) in [
        ("function use(f: Function): int { return 1; } function main() {}", "type", "Function needs a result type"),
        ("function use(f: Mutable): int { return 1; } function main() {}", "type", "Mutable wraps one type"),
        ("function use(f: Function<int, string>): int { return 1; } function main() {}", "syntax", "written as a list"),
        ("function use(f: Function<int, [int], [int]>): int { return 1; } function main() {}", "syntax", "one optional argument list"),
        ("function use(f: Mutable<Function<int>, int>): int { return 1; } function main() {}", "syntax", "exactly one type"),
        ("function use(f: ReadOnly<Mutable<Function<int>>>): int { return 1; } function main() {}", "callback-access", "one ownership wrapper"),
        ("function use(f: Once<int>): int { return 1; } function main() {}", "callback-access", "Once applies to a Function"),
        ("function main() { const values: Mutable<int[]> = [1]; }", "callback-access", "cannot carry data access"),
        ("type Box = { values: ReadOnly<int[]>; }; function main() {}", "callback-access", "cannot carry data access"),
        ("function make(): Owned<string> { return \"a\"; } function main() {}", "callback-access", "cannot carry data access"),
        ("function use(f: Function<int, [ReadOnly<Mutable<string>>]>): int { return 1; } function main() {}", "callback-access", "cannot be nested"),
        ("function use(f: Function<int, [string]>): int { return 1; } function main() {}", "callback-access", "ReadOnly<T>, Mutable<T> or Owned<T>"),
        ("function use(f: Function<int, [ReadOnly<int>]>): int { return 1; } function main() {}", "callback-access", "data-owning arguments"),
        ("function use(f: Function<int, [a: int, b?: int, c: int]>): int { return 1; } function main() {}", "parameter-order", "required parameter"),
        ("function use(f: Function<int, [...a: int[], b: int]>): int { return 1; } function main() {}", "rest-parameter", "must be last"),
        ("type Function = { n: int; }; function main() {}", "duplicate", "reserved"),
        ("type Owned = { n: int; }; function main() {}", "duplicate", "reserved"),
    ] {
        rejected(source, code, message);
    }
    rejected(
        "function twice(f: ReadOnly<Function<int>>): int { return f(); } function main() { let n = 0; console.log(twice((): int => { n += 1; return n; })); }",
        "callback-access",
        "ReadOnly<Function<...>> callback cannot mutate",
    );
    rejected(
        "function once(f: Once<Function<int>>): int { return f() + f(); } function main() {}",
        "callback-consumed",
        "",
    );
    rejected(
        "function keep(f: Mutable<Function<int>>): Function<int> { return f; } function main() {}",
        "callback-escape",
        "",
    );
}

#[test]
fn access_wrappers_apply_to_arrow_parameters_and_nullable_function_arguments() {
    native(
        r#"
function one(): int { return 1; }
function run(action: ReadOnly<Function<int, [inner: ReadOnly<Function<int>> | null, values: Mutable<int[]>]>>, values: int[], present: boolean): int { return action(present ? one : null, values); }
function main() {
    const values = [1, 2];
    const grow = (inner: ReadOnly<Function<int>> | null, list: Mutable<int[]>): int => { list.push(3); return inner === null ? list.length : inner(); };
    console.log(run(grow, values, false), run(grow, values, true), values.join(","));
    const sizes = ["a", "bcd"].map((text: ReadOnly<string>): int => { return text.length; });
    console.log(sizes.join(","));
    console.log([3, 1, 2].some((value: int, index: int, source: ReadOnly<int[]>): boolean => { return value > source.length - 1; }));
}
"#,
        "3 1 1,2,3,3\n1,3\ntrue\n",
    );
    // A wrapped Function can join an absent alternative inside an argument
    // list; both spellings produce the same checked program.
    let named = "function run(action: ReadOnly<Function<int, [inner: ReadOnly<Function<int>> | null]>>): int { return 1; } function main() {}";
    let keywords = "function run(action: ReadOnly<Function<int, [inner: (ReadOnly<Function<int>>) | null]>>): int { return 1; } function main() {}";
    assert_eq!(without_offsets(&emitted(named)), without_offsets(&emitted(keywords)));
    for (source, message) in [
        ("function main() { const f = (n: Mutable<int>): void => {}; }", "scalars copy"),
        ("function main() { const f = (x: void): void => {}; }", "cannot have type void"),
        ("function main() { const f = (n: Once<string>): void => {}; }", "Once applies to a Function"),
    ] {
        let error = check(source).err().expect("source must be rejected");
        assert!(error.message.contains(message), "{source}: {}", error.message);
    }
}

#[test]
fn wrappers_compose_with_arrow_type_spelling() {
    let wrapped = "type Row = { n: int; }; function use(action: ReadOnly<(row: ReadOnly<Row>, count: int) => int>, change: Mutable<(row: Mutable<Row>) => void>, last: Once<() => int>): int { return 1; } function main() {}";
    let named = "type Row = { n: int; }; function use(action: ReadOnly<Function<int, [row: ReadOnly<Row>, count: int]>>, change: Mutable<Function<void, [row: Mutable<Row>]>>, last: Once<Function<int>>): int { return 1; } function main() {}";
    let keywords = "type Row = { n: int; }; function use(action: ReadOnly<Function<int, [row: ReadOnly<Row>, count: int]>>, change: Mutable<Function<void, [row: Mutable<Row>]>>, last: Once<Function<int>>): int { return 1; } function main() {}";
    let expected = without_offsets(&emitted(keywords));
    assert_eq!(without_offsets(&emitted(wrapped)), expected);
    assert_eq!(without_offsets(&emitted(named)), expected);
    assert_eq!(
        check(wrapped).unwrap().contracts(),
        check(keywords).unwrap().contracts()
    );
}

#[test]
fn stated_parameter_access_is_a_contract_for_declarations_and_arrows() {
    native(
        r#"
type Row = { n: int; };
function grow(values: Mutable<int[]>): int { values.push(9); return values.length; }
function size(values: Mutable<int[]>): int { return values.length; }
function measure(text: ReadOnly<string>): int { return text.length; }
function keep(text: Owned<string>): int { return text.length; }
function bump(row: Mutable<Row>, by: int = 1): int { row.n = row.n + by; return row.n; }
function walk(values: Mutable<int[]>, n: int): int { if (n === 0) { return values.length; } values.push(n); return walk(values, n - 1); }
function first<T>(values: ReadOnly<T[]>): int { return values.length; }
function take(action: ReadOnly<Function<int, [Owned<string>]>>): int { return action("ab".repeat(1)); }
function lend(action: ReadOnly<Function<int, [Mutable<int[]>]>>, values: int[]): int { return action(values); }
function main() {
    const values = [1];
    console.log(grow(values), size(values), values.join(","), measure("abc"), keep("x".repeat(2)));
    const row: Row = { n: 1 };
    console.log(bump(row), bump(row, 5), row.n);
    const path: int[] = [];
    console.log(walk(path, 3), path.join(","), first([1, 2]));
    console.log(take((text: Owned<string>): int => { return text.length; }), lend(size, values), lend((list: Mutable<int[]>): int => { return list.length; }, values));
}
"#,
        "2 2 1,9 3 2\n2 7 7\n3 3,2,1 2\n2 2 2\n",
    );
    for (source, code, message) in [
        ("function grow(values: ReadOnly<int[]>): int { values.push(1); return values.length; } function main() {}", "parameter-access", "declare it Mutable"),
        ("function keep(text: ReadOnly<string>): string { return text; } function main() {}", "parameter-access", "declare it Owned"),
        ("function keep(text: Mutable<string>): string { return text; } function main() {}", "parameter-access", "declare it Owned"),
        ("function count(n: Mutable<int>): int { return n; } function main() {}", "callback-access", "scalars copy"),
        ("function keep(text: Owned<string>): int { return text.length; } function main() { const word = \"a\".repeat(1); console.log(keep(word)); console.log(word); }", "transferred", ""),
        ("function use(f: ReadOnly<Function<int, [ReadOnly<string>]>>): int { return f(\"a\"); } function main() { console.log(use((text: Owned<string>): int => { return text.length; })); }", "callback-access", "Expected read argument access, found consume"),
        ("function use(f: ReadOnly<Function<int, [Owned<string>]>>): int { return f(\"a\"); } function main() { console.log(use((text: string): int => { return text.length; })); }", "callback-access", "Expected consume argument access, found read"),
        ("function main() { const f = (values: ReadOnly<int[]>): int => { values.push(1); return 1; }; }", "parameter-access", "declare it Mutable"),
    ] {
        rejected(source, code, message);
    }
}

#[test]
fn retired_access_keywords_name_the_replacement_types() {
    for (old, replacement) in [("read", "ReadOnly<T>"), ("mutate", "Mutable<T>"), ("consume", "Owned<T>"), ("owned", "Owned<T>"), ("once", "Once<Function<R>>")] {
        let source = format!("function use(action: {old} () => int) {{}} function main() {{}}");
        rejected(&source, "retired-access-keyword", replacement);
        let source = format!("function use(value: {old} string) {{}} function main() {{}}");
        rejected(&source, "retired-access-keyword", replacement);
        let source = format!("function main() {{ const f = (value: {old} string): int => {{ return value.length; }}; }}");
        rejected(&source, "retired-access-keyword", replacement);
    }
}
