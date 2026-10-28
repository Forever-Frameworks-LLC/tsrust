# Gate 135: switch statements

Status: focused compiler/native, editor, guide, current consumer and matched benchmark acceptance complete. Full compiler/LSP regressions and measurements remain deferred.

## Behavior

`switch (value) { case a: ... default: ... }` evaluates the value once, compares it with each case by strict equality in source order, and runs the first matching clause. Later case expressions are not evaluated. Adjacent labels share a clause. `default` may appear anywhere and runs only after every case has failed. A matching case narrows a union variable inside its clause.

Each clause ends with `break`, `return` or `continue`, or is the last clause. `continue` continues the enclosing loop. Each clause is its own block.

Two JavaScript forms are rejected with `unsupported` and a message that says what to write: a clause with statements that runs into the next clause, and a `break` inside a nested statement of a clause. A second `default`, a statement before the first label and a missing colon are `syntax` errors. A switch supports at most 120 clauses.

See [the syntax guide](../../website/src/content/docs/language/switch.md).

## Implementation

`Parser::switch_statement` in `syntax.rs` lowers a switch to an if chain over `===`, joined with `||` for grouped labels, with `default` as the final else. A named discriminant is compared in place, which keeps flow narrowing; any other discriminant is held once in a hidden binding. `end_switch_clause` removes the ending `break`, and `switch_break` finds a nested one. All are non-inlined helpers. No analysis, emission or editor source change was needed.

## Formatter

- Each `case` or `default` label ends its line, and the clause statements sit one level deeper. Nested switches indent cumulatively.
- Repair: the colon of a conditional expression had no space before it (`x ? 1: 2`). It now reads `x ? 1 : 2`. Optional parameters and record fields keep their tight colon.

## Observed, not changed

The command line panics with a broken-pipe message when its standard output is closed early, for example `tsrust check file | head -1`. The check result itself is unaffected.

## Evidence

- Snapshots: `archive/switch-statements-start`, `archive/switch-statements-benchmark-start`, `archive/switch-statements-consumers-start` (gate 134 consumers).
- `target/editor/switch-statements-red.log` keeps the initial failing run.
- `switch-statements-final-compiler.log`: twelve suites pass on the final sources: `switch_statements` (an executable `std:testing` fixture and eight static rejections), `braceless_bodies`, `loops`, `control_flow`, `control_flow_native`, `higher_order_function_values`, `type_syntax`, `formatting`, `conditional_expressions`, `nullable_narrowing`, `union_array_elements` and `acceptance`.
- The executable fixture passes in Node through `tools/node-reference/run.mjs`. It covers evaluation order of case expressions, grouped labels, a block clause, `continue` in a loop, a default without `break`, a leading default, a string discriminant that stays usable, union narrowing, and empty and single-case switches.
- `switch-statements-guide-verification.log`: the guide example passes natively and in Node.
- `switch-statements-final-editor.log`: all 214 editor tests pass, including `switch_clauses_keep_names_types_and_narrowed_members`.
- The workload source was checked with the refreshed release compiler before execution checks. Consumers were replaced by atomic rename; hashes are in `switch-statements-final-debug.log`.
- `switch-statements-harness.log`: all 353 runtime harness checks pass in one run.

## Matched support

`switch_statements` retains four cells per iteration: an int switch with grouped cases and returns, a switch inside a loop with `continue`, `break` and a default without `break`, a string switch with a leading default, and a union switch with a narrowed call. The independent oracle uses tables and conditions; the registration test compares it with the platform's switch statements on 2048 cases for three inputs, and its source contract counts four switch statements and fourteen clauses in the TypeScript syntax tree. Hashes, guards, the whole-scenario denominator, registration, runtime and allocation-profile hooks, oracle fingerprints and the report category are in place.

`switch_statements`, `braceless_bodies` and `else_if_control` pass untimed on native and Node-after-tsc in `benchmarks/runtime/results/2026-10-05T12-09-23-165Z-verify.json`.

No timing, allocation profiling or complete compiler/LSP regression was run. SQLite measurements are unchanged. Next gate is 136.
