# Gate 131: classic-for pattern initializers

Status: focused compiler/native, editor, guide, current consumer and matched benchmark acceptance complete. Full compiler/LSP regressions and measurements remain deferred.

## Behavior

The initializer of a classic `for` loop accepts a destructuring declaration: `for (let { start: i, end } = bounds(); i < end; i++)`. The source expression is evaluated once, before the first condition check. The names are in scope for the condition, the update clause and the body, and end with the loop. Defaults, rest and nested patterns work as in other declarations. `continue` runs the update clause and `break` leaves the loop.

Access follows the source, as in every pattern. A named source keeps positions that were copied or not taken. A rest name is an independent copy. A name that borrows an array element keeps the array loaned until the loop ends.

The names share one binding for the whole loop; they are not created again per iteration. A name cannot repeat a name of the enclosing scope, which is the existing rule for classic-for declarations. A pattern declaration in the update clause is rejected.

Behavior review found three limits that are not specific to patterns and were left for later gates: one declarator per declaration (`let a = 1, b = 2` is a syntax error everywhere), no comma expression in the update clause (`i++, j++`), and no destructuring assignment to existing names.

See [the syntax guide](../guides/language/destructuring.md).

## Implementation

`for_statement` in `syntax.rs` appends the pending position declarations after the initializer, inside the loop's enclosing lexical block and before the condition. No other pass changed.

`tools/node-reference/run.mjs` now resolves `std:testing` imports to `node:assert/strict`, so assertion fixtures run in Node unchanged.

## Evidence

- Snapshots: `archive/for-pattern-initializers-start` (sources), `archive/for-pattern-initializers-benchmark-start` (harness, guide and tracking files), `archive/for-pattern-initializers-consumers-start` (gate 130 consumers).
- Earlier evidence under `target/editor/for-pattern-initializers-*` is retained: the initial red run, the first focused run, the editor check and the independent Node assertions.
- `for-pattern-initializers-final-compiler.log`: `for_pattern_initializers` 2, `destructuring` 7, `loops` 9, `control_flow` 11, `higher_order_function_values` 6, `type_syntax` 8 and `formatting` 14 pass on the final sources.
- `for-pattern-initializers-final-editor.log`: six editor checks for destructured names pass, including `for_initializer_patterns_have_types_and_loop_scoped_definitions`.
- `for-pattern-initializers-guide-verification.json`: the guide example and two further loops from the guide text pass natively and in Node with empty output.
- The formatter writes a pattern initializer on the loop line: `for (let { start: i, end } = make(); i < end; i++) {`.
- Debug compiler/LSP, editor consumers and the release compiler were rebuilt and replaced by atomic rename before any execution check; hashes are in `for-pattern-initializers-final-debug.log`.
- `for-pattern-initializers-harness.log`: all 345 runtime harness checks pass in one run.

## Matched support

`for_pattern_initializers` retains four cells per iteration: a record-pattern loop over call-result bounds with one skipped value, a defaulted array-pattern loop, a rest-pattern loop whose update clause extends the copied tail, and the unchanged source length. Its independent oracle uses closed-form arithmetic and text joins without loops or destructuring; the registration test also compares it with the platform's loops on 2048 cases. Source hashes, complete-retention guards, the whole-scenario denominator, registration, runtime and allocation-profile hooks, oracle fingerprints and the report category are in place.

Both registration checks pass in `for-pattern-initializers-registration.log`. Three current-release native and Node-after-tsc workloads (`for_pattern_initializers`, `nested_rest_patterns`, `destructuring_patterns`) pass untimed in `benchmarks/runtime/results/2026-10-05T11-29-19-849Z-verify.json` and `for-pattern-initializers-verify.log`.

No timing, allocation profiling or complete compiler/LSP regression was run. SQLite measurements are unchanged. Next gate is 132.
