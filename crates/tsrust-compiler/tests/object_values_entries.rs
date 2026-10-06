mod support;
use support::{execute, reject};

// Expected output was produced by tools/node-reference/run.mjs for the same
// program. Nested record literals use declaration order, because console
// display of a record follows its declaration order.
#[test]
fn object_values_and_entries_copy_fields_in_each_value_key_order() {
    execute(
        include_str!("fixtures/object-values-entries/records.tsr"),
        include_str!("fixtures/object-values-entries/records.stdout"),
        None,
    );
}

#[test]
fn object_values_and_entries_keep_static_contracts() {
    for (source, code) in [
        ("type P = { x: int; }; function main() { const p: P = { x: 1 }; Object.values(); }", "arity"),
        ("type P = { x: int; }; function main() { const p: P = { x: 1 }; Object.entries(p, p); }", "arity"),
        ("function main() { Object.values(\"text\"); }", "unsupported"),
        ("function main() { Object.values(7); }", "unsupported"),
        ("function main() { Object.entries([1, 2]); }", "unsupported"),
        ("type P = { x: int; }; function main() { const p: P | null = null; Object.values(p); }", "unsupported"),
        ("type P = { x: int; run: Function<int>; }; function main() { const p: P = { x: 1, run: (): int => { return 1; } }; Object.values(p); }", "unsupported"),
        ("type P = { x: int; s: string; }; function main() { const p: P = { x: 1, s: \"a\" }; const v: int[] = Object.values(p); }", "type"),
        ("type P = { x: int; s: string; }; function main() { const p: P = { x: 1, s: \"a\" }; const n: int = Object.values(p)[0]; }", "type"),
    ] {
        reject(source, code);
    }
}

// Generic records, nested field receivers, mixed field types with widening
// union fields, closures and arrays of records. Output produced by Node.
#[test]
fn object_values_and_entries_cover_generic_nested_and_mixed_records() {
    execute(
        include_str!("fixtures/object-values-entries/shapes.tsr"),
        include_str!("fixtures/object-values-entries/shapes.stdout"),
        None,
    );
}

// A callback parameter takes its tuple type from an array of pairs.
#[test]
fn pair_arrays_give_callbacks_a_tuple_parameter() {
    execute(
        "function main() { const v = [\"a\", \"b\"]; console.log(v.entries().map((entry) => entry[0] + 1).join(\",\"), v.entries().filter((entry) => entry[1] === \"b\").length); }\n",
        "1,2 1\n",
        None,
    );
}
