# Gate 122: Object.values and Object.entries

Status: focused compiler/native, editor, documentation and benchmark acceptance complete. Full regressions and timing remain deferred.

## Behavior

`Object.values(value)` returns the field values of a record as a new array. `Object.entries(value)` returns `[name, value]` pairs. Both use the order of `Object.keys`: the field order of that value's constructor literal.

- When every field has one type, the element is that type: `Object.values(size)` is `int[]`. When field types differ, the element is the union of the field types, such as `(int | string | boolean)[]`. A field that is itself a union contributes its alternatives. Narrow an element with `typeof` or a null check.
- `Object.entries` returns `[string, V][]`, where `V` is the same element type. Read a pair with `pair[0]` and `pair[1]`.
- A tuple gives its positions in order; entry names are `"0"`, `"1"` and so on. `Object.values(array)` gives a copy of the array.
- The argument is evaluated once and only read. Each value is copied, so the result is independent of the record. JavaScript shares nested arrays and objects with the source.
- Exactly one argument is required (`arity`). Strings, numbers, booleans, null, undefined, nullable values, an empty record and a record with a function field are `unsupported`. `Object.entries(array)` is `unsupported` with advice to use `array.entries()`.

Both compiler fixtures match Node v24.11.1 output, produced by `tools/node-reference/run.mjs`.

## Implementation

`Analyzer::object_values` in `object_values.rs` checks the argument, interns the union of the field types and, for entries, materializes the pair tuple as `array.entries()` does. The call targets are `CallTarget::ObjectValues` and `CallTarget::ObjectEntries(pair)`. The emitter walks the value's constructor-order index array, copies each field and places it into its alternative through `element_alternative`. A compact local record first takes its ordinary layout. Record types passed to either function now carry the constructor-order index array, as for `Object.keys`.

`tools/node-reference/run.mjs` is new. It runs a Tsrust fixture in Node through the TypeScript transpiler, with `clone()` as a deep copy and the ownership wrapper types as identities.

## Repairs

- A callback over an array of pairs failed with `Tuple annotation has not been materialized`, for example `values.entries().map((entry) => entry[0])`. The contextual parameter now names the tuple record directly. This defect dated from gate 118.

## Record display order

At this gate console display used declaration order. Gate 123 changed display to constructor order. The fixtures still write nested record literals in declaration order, which is valid under both rules.

## Editor

The `Object` catalog family documents `values` and `entries` with four natively verified examples. Hover on a result shows the checked type, such as `(int | string)[]` or `[string, int | string][]`. Completion after `Object.` lists `is`, `keys`, `values` and `entries`.

## Focused evidence

- `crates/tsrust-compiler/tests/object_values_entries.rs`: four tests; two Node-matched fixtures in `tests/fixtures/object-values-entries/`, nine static rejections and the pair-callback regression.
- Compiler suites passed: unit tests, `object_values_entries`, `object_keys`, `array_entries`, `tuple_types`, `data_unions`, `union_array_elements`, `nested_records`, `fixed_storage`, `array_transform`, `generic_default_builtins`, `contextual_arrows`, `array_predicates`, `array_for_each`.
- All 182 editor tests passed, including `object_values_and_entries_have_docs_checked_results_completion_and_help` and the native catalog examples.
- Benchmark support: `object_values_entries` with an independent written-order oracle, source contract and registration test. Untimed native and Node-after-tsc verification passed for it, `object_keys_order` and `union_array_values`. All 326 harness checks passed in one run.

The byte-exact baseline is `archive/object-values-entries-start`. No full compiler regression, timing or allocation profiling was run. Each call copies every field value; that cost is unmeasured. SQLite measurement records are unchanged.
