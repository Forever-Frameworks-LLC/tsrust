# Gate 103: Array keys and values

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the array-methods feature-family milestone.

## Behavior

- `array.keys()` returns an `int[]` holding every index in order, `0` through `length - 1`.
- `array.values()` returns the elements in order as a new array of the same element type.
- Both take no arguments (`arity` otherwise), evaluate the receiver once, borrow it and leave it unchanged.
- Each result is an ordinary independent array computed when called. It works in `for...of`, `Array.from`, array spread and every array method. It does not follow later changes to the receiver, and there is no iterator object with `next()`. Node returns live iterators instead.

`entries()` is not included: it needs index/value pairs, and heterogeneous tuples are not available.

## Implementation

`keys()` is a new call target, `CallTarget::ArrayKeys`, checked in `analysis.rs` and lowered in `emit.rs` to a counted loop that fills the native int array. `values()` is rewritten before call checking onto the checked `slice()` copy (`Analyzer::array_values`). A callable record field named `keys` or `values` is not affected because the rewrite and the new arm apply only to array receivers.

The editor reports `int[]` for `keys()` and the receiver type for `values()`; both have catalog entries under `Array` in `builtins.json`.

## Evidence

- `crates/tsrust-compiler/tests/array_keys_values.rs`: two native tests written before the implementation.
- `crates/tsrust-lsp/tests/intelligence.rs`: `array_keys_and_values_have_hover_completion_help_and_result_types`; catalog examples execute natively.
- `target/editor/array-keys-values-compiler.log`: eleven focused compiler suites and the unit tests, 189 passed.
- `target/editor/array-keys-values-editor.log`: all 153 editor tests passed.
- Benchmark support: `array_keys_int` and `array_values_int` with an independent oracle, source contract and registration test. The Node counterparts collect the iterator with spread so both sides retain the same values. `target/editor/array-keys-values-workload-verify.log` records untimed native and compiled-Node correctness; `target/editor/array-keys-values-harness.log` records all 288 harness tests passing. No timing was run.

## Repairs made in passing

- `number[].includes(NaN)` returned `false`. It now uses the same comparison as Node: `NaN` is found and signed zeros are equal. `indexOf` and `lastIndexOf` keep strict equality and still return `-1` for `NaN`. Regression test: `array_search::number_includes_treats_nan_as_present_while_index_of_does_not`.
