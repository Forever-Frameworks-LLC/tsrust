# Gate 102: Array.from

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the array-methods feature-family milestone.

## Behavior

- `Array.from(source)` returns a new array. An array source is copied in order. A string source yields one element per Unicode character.
- `Array.from(source, map)` calls `map` once per value, in order, with the value and its zero-based index, and stores the results.
- The source expression is evaluated once. The source is borrowed and remains usable.
- The result is an independent array: owned elements are copied, so changing the result never changes the source. Node copies element references instead.

Rejected with diagnostics: zero arguments or more than two (`arity`), a mapping function with more than two parameters (`arity`), a source that is not an array or string (`type`), and spread arguments (`unsupported`). Other iterables, array-like objects and `thisArg` are not supported.

## Implementation

`Analyzer::array_from` in `analysis.rs` rewrites the call before ordinary call checking: an array source becomes the checked `slice()` copy or `map(...)`, and a string source becomes `split("")`, optionally followed by `map(...)`. A nested `Array.from` source is rewritten first so its type is known. No new native runtime code or call target is added, and the shared recursive expression frame only gains one boolean test.

The editor infers the result type for hover, offers `from` after `Array.`, and shows documented parameter help. The catalog entry is `ArrayConstructor.from` in `builtins.json`.

## Evidence

- `crates/tsrust-compiler/tests/array_from.rs`: three native tests written before the implementation (copy, mapping, string characters, single evaluation, rejections).
- `crates/tsrust-lsp/tests/intelligence.rs`: `array_from_has_hover_completion_parameter_help_and_checked_results`; catalog examples execute natively in `documented_builtin_examples_match_native_output`.
- `target/editor/array-from-compiler.log`: ten focused compiler suites, 70 tests passed.
- `target/editor/array-from-editor.log` plus the repaired `Array.of` example count: all editor tests pass.
- Benchmark support: four matched pairs (`array_from_copy_int`, `array_from_map_int`, `array_from_string_characters`, `array_from_map_string`) with an independent oracle, source contract and registration test. `target/editor/array-from-workload-verify.log` records untimed native and compiled-Node correctness; `target/editor/array-from-harness.log` records the harness tests. No timing was run.

## Repairs made in passing

- The editor test for `Array.of` expected six catalog examples; the catalog has had seven since array-literal spread was documented. The expectation now matches.
- The report category test did not know the `array_from_` prefix.
