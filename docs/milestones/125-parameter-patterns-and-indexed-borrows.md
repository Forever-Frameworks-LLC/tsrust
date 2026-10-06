# Gate 125: parameter patterns, indexed borrows and equality narrowing

Status: focused compiler/native, editor and documentation acceptance complete. Full regressions and timing remain deferred. No new benchmark pair; `destructuring_patterns` and two related pairs were verified untimed after the change.

## Parameter patterns

Function and arrow parameters accept array and record patterns: `function describe({ x, label }: Point)`, `pairs.map(([name, amount]) => …)`, `rows.filter(({ id }) => id > 7)`. The names are declared first in the body.

A destructured parameter with no stated access is read, so its names borrow the argument and the caller keeps the value. `Owned<T>` lets the names move out. Without this default an unannotated record pattern would have consumed every caller's argument as soon as one owned field was named.

## Indexed borrows

A `const` taken from an element of a named array, or from a field of such an element, is a readonly borrow for its scope: `const first = words[0];`, `const name = boxes[0].name;`. Before this gate these were `borrowed-element` errors that required `clone()`. Transferring the borrowed name is a `borrowed-place` error. Changing or transferring the array while the name is in scope is a `view-conflict` error whose message names the borrowing const and suggests an inner block or `clone()`.

With this rule array patterns work on arrays of owned elements: `const [first, second] = words;` borrows both positions. A position past the end stops the program at run time, as indexing does. The earlier plan to move elements out with a length check was not needed for reading and was not built.

## Equality narrowing

Where `value === x` is true for a stable union variable and one int, number, boolean or string `x`, the variable is narrowed to the alternative of `x` (int and number together for a numeric `x`). `!==` narrows the path where the comparison is false. The other path keeps every alternative.

```ts
function label(value: ReadOnly<int | string | boolean>): string {
    if (value === "yes") { return value.toUpperCase(); }
    if (value !== 7) { return String(value); }
    return String(value + 1);
}
```

## Repairs

- Type annotations nested beyond the limit overflowed the stack before the recursive guard was reached, for example 200 nested `(p: …) => int` or `() => …` levels. A linear scan now rejects them with the existing `unsupported` message before parsing. Archived sources from gate 120 show the same overflow, so it predates this session's work.
- `acceptance::missing_backend_reports_setup_without_fallback_or_output` failed because the backend is now also found under the Cargo home, so an empty `PATH` no longer hid it. The test hides the Cargo home too and passes. Gates 123 and 124 listed this failure as not investigated.
- Two `type_syntax` tests expected an older diagnostic for `Box<int>`; both the current compiler and the archived gate 119 binary report `Unknown generic record type`.
- Four tests expected `borrowed-element` for a const taken from an indexed element or loop row; they now cover the borrow and its transfer rejection.

## Implementation

Parameter patterns reuse the declaration lowering from gate 124: the parameter takes a hidden name and the position declarations are placed first in the body. Pattern parsing lives in non-inlined helpers so the recursive parser frames did not grow.

`Analyzer::indexed_element_hint` marks a const initializer that ends in an index of named array storage, under any number of field reads, as a borrowed place. The existing view-root tracking keeps the array loaned for the binding's scope.

The editor declares the names of a parameter pattern and takes their types from the checked positions.

## Focused evidence

- `crates/tsrust-compiler/tests/destructuring.rs`: seven tests; six Node-matched fixtures in `tests/fixtures/destructuring/` and seventeen static rejections.
- 33 compiler suites passed in one run: the unit tests, `destructuring`, `closures`, `contextual_arrows`, `function_types`, `optional_parameters`, `rest_parameters`, `callbacks`, `array_transform`, `array_for_each`, `array_reduce`, `array_predicates`, `callable_fields`, `callable_locals`, `higher_order_function_values`, `formatting`, `generic_default_builtins`, `call_spread`, `nullable_callables`, `owned_closure_arguments`, `nested_closures`, `field_moves`, `owned_arrays`, `nested_arrays`, `type_syntax`, `arrays`, `for_of`, `loops`, `nullish_coalescing`, `optional_chaining`, `union_array_elements`, `object_values_entries`, `data_unions`.
- Equality narrowing: `union_equality_narrows_the_equal_path` with a Node-matched fixture; `union_array_elements`, `data_unions`, `nullable_unions`, `nullable_narrowing`, `nullable_callables`, `nullish_coalescing`, `optional_chaining`, `logical_values`, `operators`, `acceptance`, `conditional_expressions` and `control_flow` passed.
- All 184 editor tests passed, including `destructured_parameters_and_indexed_borrows_have_hover_types`.
- All 328 harness checks passed after rebuilding; `destructuring_patterns`, `object_values_entries` and `union_array_values` verified untimed.

Baseline: `archive/indexed-borrow-start`.
