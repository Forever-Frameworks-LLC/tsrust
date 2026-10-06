# Gate 136: function expressions

Status: focused compiler/native, editor, guide, current consumer and matched benchmark acceptance complete. Full compiler/LSP regressions and measurements remain deferred.

## Behavior

`function (...) { ... }` in a value position is an anonymous function value. It has the parameter forms, result inference, captures and ownership rules of an arrow function with a block body: annotations, defaults, rest, destructuring patterns, and callback parameters whose types come from the receiving method.

A function expression cannot have a name (`unsupported`, with advice to bind it or declare it at the top level). It needs a block body (`syntax`). A statement that starts with `function` inside a function body is a `syntax` error that explains where declarations belong.

See the new section of [the arrow function guide](../arrow-functions.md).

## Implementation

`Parser::function_primary` in `syntax.rs` builds the same closure node as an arrow literal. `statement` rejects a leading `function`. No analysis or emission change was needed.

The editor declares the parameters of every function expression in its body scope and treats `const name = function (...) { ... }` as a callable binding with signature help, as it does for arrows.

The formatter writes `function (value: int): int {` with a space after the keyword.

## Repair: closed output pipe

The command line panicked with a broken-pipe message when its standard output closed early, for example `tsrust check file | head -1`. On Unix it now restores the default handling of the pipe signal at start, so the command ends quietly like other tools. `acceptance::closed_standard_output_does_not_panic` reproduces the case; `target/editor/broken-pipe-red.log` keeps the failing run.

## Found, not changed

Calling the result of a call or a parenthesized function directly is unsupported for every function value: `make(10)(1)` and `(function (x: int): int { return x; })(3)`. Bind the function to a name first. This limit predates this gate and also applies to arrows.

## Process notes

- The failing test for function expressions was written and run in Node before the implementation, but its first compiler run happened after the parser change; `function-expressions-first.log` is that run, with one failure from an unsupported inline record type in the test source. There is no separate red log for this part.
- One build ran without the scratch target directory and rebuilt `target/debug/tsrust` in place while no process used it. All five consumers were then replaced by atomic rename before execution checks.

## Evidence

- Snapshots: `archive/function-expressions-start`, `archive/function-expressions-benchmark-start`, `archive/function-expressions-consumers-start` (gate 135 editor and release consumers).
- `function-expressions-final-compiler.log`: fifteen suites pass on the final sources: `function_expressions` (an executable `std:testing` fixture and six static rejections), `closures`, `contextual_arrows`, `function_types`, `callbacks`, `higher_order_function_values`, `type_syntax`, `formatting`, `array_transform`, `array_sort`, `array_for_each`, `optional_parameters`, `rest_parameters`, `destructuring` and `acceptance` (25).
- The executable fixture passes in Node through `tools/node-reference/run.mjs`. `function-expressions-guide-verification.log`: the guide example passes natively and in Node.
- `function-expressions-final-editor.log`: all 215 editor tests pass, including `function_expressions_declare_parameters_and_callable_bindings`.
- The workload source was checked with the refreshed release compiler before execution checks; hashes of the refreshed consumers are in `function-expressions-final-debug.log`.
- `function-expressions-harness.log`: all 355 runtime harness checks pass in one run.

## Matched support

`function_expressions` retains five cells per iteration: a capturing function expression passed to a higher-order function, a map callback with an inferred parameter, a sort comparator, a forEach callback that changes a captured total, and a function expression with a default and a rest parameter. The independent oracle uses direct arithmetic and a code-unit comparison; the registration test compares it with the platform on 2048 cases for four inputs, and the source contract counts five anonymous function expressions in the TypeScript syntax tree and rejects arrows. Hashes, guards, registration, runtime and allocation-profile hooks, oracle fingerprints and the report category are in place.

`function_expressions` and `switch_statements` pass untimed on native and Node-after-tsc in `benchmarks/runtime/results/2026-10-05T12-18-44-070Z-verify.json`.

No timing, allocation profiling or complete compiler/LSP regression was run. SQLite measurements are unchanged. Next gate is 137.
