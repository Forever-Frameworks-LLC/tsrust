# Gate 113: String.raw

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing wait for the string-methods feature-family milestone.

## Behavior

`String.raw(template, ...substitutions)` joins a record's `raw: string[]` segments with text substitutions between consecutive segments.

- Each supported substitution is converted with `String()`. Scalar values, arrays and records are accepted; a spread substitution must be a `string[]`.
- Missing substitutions insert empty text. Extra substitutions do not appear in the result, but every argument expression still evaluates once, left to right.
- An empty raw array returns an empty string; one segment returns that segment. An empty array needs a string element type supplied by an annotation or the template's named record type.
- Backslashes already in the segments stay unchanged. Source string literals still decode their escapes normally.
- Template and substitution sources stay usable. The result is a new owned string; string-array spread copies its source.
- No template is an `arity` error. An invalid raw field or non-string spread is a `type` error. Missing fields, a spread template and function-source conversions are rejected. Mutating a borrowed template through a later argument is a `conflicting-access` error.

Tagged templates, string-valued raw fields and arbitrary array-like objects are unsupported. Text is valid Unicode; lone surrogate halves are unavailable. Tsrust record literals assigned to variables need named type annotations; an inline template literal is supported in the call.

## Implementation

The call lowers to a borrowed raw segment array and a checked array of string substitutions. Scalar substitutions reuse `String()` conversion; spread arrays are copied with `slice()` before construction. `CallTarget::StringRaw` assembles the result in native code. The checker keeps temporary loan state in a separate helper rather than adding it to the shared recursive call frame. Existing string-array ABI and observers handle the benchmark results.

The editor infers `string`, completes `String.raw`, and supplies hover and two-parameter signature help from `StringConstructor.raw`. Both catalog examples execute natively.

## Evidence

- `crates/tsrust-compiler/tests/string_raw.rs`: the first three tests failed before implementation. Five tests now pass, including native copy/order/output checks, inline templates, scalar conversions, ignored argument evaluation, spread types and source loans.
- `target/editor/string-raw-node-check.log`: all three executable fixtures match Node v24.11.1; the exact compiler-test source fingerprint and fixture sources are retained in `string-raw-node-fixtures.json`.
- `target/editor/string-raw-compiler.log`: `array_from` (3) and `string_from_codes` (3) passed before a malformed nested-function rejection fixture stopped the run. The fixture was corrected to a top-level function. `string-raw-compiler-supplement.log` records `string_raw` (5) and `value_types` (8) passing; earlier passes are preserved.
- `target/editor/string-raw-editor.log`: the focused editor protocol test and both catalog examples pass. `array-constructor-docs-red.log` and `array-constructor-docs-green.log` retain the stale Array hover regression and repair; `array-from-resume-check.log` records all three Array.from compiler tests passing.
- Workspace compiler, editor compiler, editor LSP and release compiler were rebuilt/refreshed. Debug and release build logs are retained.
- Benchmark pair: `string_raw_segments`, with an independent text-assembly oracle, fingerprint/AST source contract, registration tests, driver and allocation-profile guards, and manifest row. `string-raw-workload-verify.log` and `benchmarks/runtime/results/2026-10-04T19-06-29-880Z-verify.json` record five untimed correctness cases on native and Node after tsc. No timing was run.
- `target/editor/string-raw-harness.log`: 307 harness tests pass; one test used a stale editor compiler. After refreshing that binary, `string-raw-harness-supplement.log` records the affected slice-observer file passing. All 308 harness tests have passing evidence across those runs.

## Repairs made in passing

The Array namespace hover still claimed Array.from was unavailable. A focused failing hover assertion now guards the corrected description. The coverage generator still printed the retired five-gate regression schedule; generated guidance now follows completed feature-family milestones.

## Pending measurements

String-methods full regressions, timing and allocation comparisons remain pending. String.raw currently builds the substitution array and converts supplied values before assembly, including unused values; no performance measurement or optimization claim is made. Existing SQLite measurement records are unchanged. Exact prior file and binary snapshots are retained in `archive/string-raw-start` and `archive/array-constructor-docs-start`.
