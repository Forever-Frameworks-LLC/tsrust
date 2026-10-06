# Gate 107: Object.is

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the object-functions feature-family milestone.

## Behavior

`Object.is(left, right)` returns a `boolean` that says whether two values are the same value.

- Numbers compare by value with two exceptions to `===`: `NaN` is the same as `NaN`, and `0` is not the same as `-0`. An `int` and a `number` compare as numbers, so `Object.is(1, 1.0)` is `true`.
- Strings compare by content. Booleans compare by value. `null` matches `null` and `undefined` matches `undefined`.
- Arrays and records are the same only when both arguments name the same stored value. Two separately built arrays with equal contents are different.
- Values of different kinds, such as an int and a string, are never the same.
- Exactly two arguments are required (`arity`). Callable values are rejected (`unsupported`).

Both arguments evaluate once, left to right, and are only read. Nothing is moved, copied or allocated.

Every case in the compiler test was checked against Node 24.11.1.

## Implementation

A new call target, `CallTarget::ObjectIs`, is checked by `Analyzer::object_is` and lowered inline in `emit.rs`: value comparison for ints, booleans and strings, bit comparison with a `NaN` case for numbers, and address comparison for arrays and records. `Object` is a new editor namespace.

## Evidence

- `crates/tsrust-compiler/tests/object_is.rs`: two native tests written before the implementation.
- `crates/tsrust-lsp/tests/intelligence.rs`: `object_is_has_namespace_hover_completion_help_and_boolean_result`; the catalog example for `Object.is` executes natively.
- Focused compiler suites passed: `object_is`, `array_search`, `numbers`, `value_types`, `fixed_storage`, `generic_default_builtins`, `primitive_identity`. All 157 editor tests passed.
- Benchmark support: `object_is_values` with an independent reasoned oracle, source contract and registration test. Untimed native and compiled-Node verification passed; all 296 harness tests passed. No timing was run.
