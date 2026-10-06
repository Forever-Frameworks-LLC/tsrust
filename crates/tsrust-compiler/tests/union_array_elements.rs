mod support;
use support::execute;

// Expected output was produced by Node for the same program, with type
// annotations removed and clone() supplied as a deep copy.
#[test]
fn union_array_elements_keep_tags_through_storage_iteration_and_clone() {
    execute(
        include_str!("fixtures/union-array-elements/basics.tsr"),
        include_str!("fixtures/union-array-elements/basics.stdout"),
        None,
    );
}

// Expected output was produced by Node for the same program. Spread sources
// are cloned first because spreading transfers owned elements.
#[test]
fn union_array_elements_work_with_array_methods_search_and_text() {
    execute(
        include_str!("fixtures/union-array-elements/methods.tsr"),
        include_str!("fixtures/union-array-elements/methods.stdout"),
        None,
    );
}

// Strict equality on union values, in callbacks, loops and parameters.
// Expected output was produced by Node.
#[test]
fn union_values_compare_with_strict_equality() {
    execute(
        include_str!("fixtures/union-array-elements/equality.tsr"),
        include_str!("fixtures/union-array-elements/equality.stdout"),
        None,
    );
    for source in [
        "type Item = { label: string; }; function main() { const a: Item | int = 1; const b: Item | int = 2; console.log(a === b); }",
        "function main() { const a: int[] | string = \"x\"; const b: int[] | string = \"x\"; console.log(a === b); }",
    ] {
        support::reject(source, "unsupported");
    }
}

// An array literal without a declared type takes the union of its value
// types. Expected output was produced by Node.
#[test]
fn untyped_mixed_array_literals_take_a_union_element_type() {
    execute(
        include_str!("fixtures/union-array-elements/mixed-literals.tsr"),
        include_str!("fixtures/union-array-elements/mixed-literals.stdout"),
        None,
    );
    // A declared element type still rejects other values.
    support::reject("function main() { const v: int[] = [1, \"a\"]; }", "type");
    support::reject("function main() { const v = [1, 2]; v.push(\"a\"); }", "type");
}

// Equality with one primitive value narrows a union place on the equal path.
// Expected output was produced by Node.
#[test]
fn union_equality_narrows_the_equal_path() {
    execute(
        include_str!("fixtures/union-array-elements/equality-narrowing.tsr"),
        include_str!("fixtures/union-array-elements/equality-narrowing.stdout"),
        None,
    );
    // The unequal path keeps every alternative.
    support::reject("function f(value: ReadOnly<int | string>): int { if (value === \"a\") { return 0; } return value + 1; } function main() {}", "type");
}

#[test]
fn union_array_elements_keep_static_contracts() {
    for (source, code) in [
        ("function main() { const v: (int | string)[] = [1, true]; }", "type"),
        ("function main() { const v: (int | string)[] = [1]; v.push(1.5); }", "type"),
        ("function main() { const v: (int | boolean)[] = [1]; const n: int = v[0]; }", "type"),
        ("function main() { const v: (int | string)[] = [1]; console.log(v.includes(true)); }", "type"),
        ("type Item = { label: string; }; function main() { const v: (Item | int)[] = [1]; console.log(v.includes(1)); }", "unsupported"),
        ("type A = { x: int; }; type B = { x: int; }; function main() { const v: (A | B)[] = [{ x: 1 }]; }", "type"),
        ("function main() { const v: (int | void)[] = []; }", "type"),
    ] {
        support::reject(source, code);
    }
}
