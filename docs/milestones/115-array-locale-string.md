# Gate 115: Array.toLocaleString

Status: focused implementation, editor support, documentation and benchmark support accepted. Timing and full regressions remain deferred.

## Behavior

`values.toLocaleString()` borrows a homogeneous array and returns independent text. It accepts no arguments. Elements are separated by commas, including commas around elements whose text is empty.

- Integers use exact signed i64 comma grouping. Numbers use the fixed decimal formatting from gate 114: at most three fractional places, decimal ties away from zero, preserved negative zero, infinity symbols and full decimal notation.
- Strings retain their text. Booleans become `true` or `false`; records become `[object Object]`.
- Null and undefined contribute empty text. An empty array returns an empty string.
- Nested arrays apply these rules recursively, without brackets or quoting. Separators and commas inside elements are not escaped.
- The receiver evaluates once and every nested element remains usable. Changing an array after formatting cannot change the returned string.
- Locale/options arguments and function-source elements are rejected with `unsupported` diagnostics. Transferred receivers are rejected. Custom element formatting, sparse arrays and other language rules are unavailable.

## Implementation

`CallTarget::ArrayToLocaleString` reads the receiver once and uses its native iterator, including the literal-string representation. The existing join-text trait gains a separate locale-text operation. Strings, booleans, null, undefined and records retain their ordinary display behavior. Numbers override the locale operation; array representations recurse into it. Ordinary join behavior remains unchanged.

The editor infers string results for array receivers. `Array.toLocaleString` supplies consistent hover, completion and zero-argument signature help. Both catalog examples execute natively.

## Evidence

- Four focused compiler tests cover native scalar and nested output, all supported leaf types, empty fields, independent text, receiver evaluation, ownership and diagnostics. The initial implementation tests fail before implementation; a separate callable-array rejection test also fails before its diagnostic is added.
- `target/editor/array-locale-string-compiler.log`: `array_join` (4), `array_locale_string` (4) and `number_locale_string` (3) pass.
- `target/editor/array-locale-string-editor.log`: two focused tests pass, including both executable catalog examples.
- `target/editor/array-locale-string-node-check.log`: both executable compiler fixtures match Node v24.11.1; their exact sources and expected output remain in `array-locale-string-node-fixtures.json`.
- Debug compiler/LSP and release compiler rebuilt; workspace and editor consumer binaries refreshed. Build logs are retained.
- Matched workload `array_locale_string_values` retains eight text results per iteration, covering every supported leaf kind and nested numeric arrays. Independent results use written rounding cells and explicit decimal grouping, without calling array locale formatting. Fingerprint/AST guards, registration tests, ABI/driver support and allocation-profile hooks are present.

- `target/editor/array-locale-string-workload-verify.log` and `benchmarks/runtime/results/2026-10-04T19-28-52-963Z-verify.json`: five untimed cases pass on native and Node after tsc. The Node workload uses the host default locale; the independent oracle verifies the documented fixed numeric display.
- `target/editor/array-locale-string-harness.log`: 310 initial harness passes; two failures are retained. The observer failure came from transferring input text inside the workload loop. Both sources now use `String(input)` explicitly, with current hashes. `array-locale-string-harness-supplement.log` records the affected observer and registration tests passing (6 tests). The category test needed the new array-locale prefix; `array-locale-string-report-supplement.log` records all 16 report tests passing. All 312 harness tests have passing evidence across the original run and affected supplements.
- Integer workload negation uses `0-whole` on both sides to avoid introducing a Node negative zero that native integers cannot represent. This is distinct from the number-array negative-zero case, which is covered explicitly.

## Pending measurements

No timing, allocation profile or full compiler/LSP regression is performed in this gate. Numeric element formatting can allocate intermediate text before appending to the result; no optimization claim is made. SQLite measurement records remain unchanged. Exact prior files and binaries are retained in `archive/array-locale-string-start`.
