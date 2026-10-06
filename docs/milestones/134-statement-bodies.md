# Gate 134: statement bodies without braces

Status: focused compiler/native, editor, guide, current consumer and matched benchmark acceptance complete. Full compiler/LSP regressions and measurements remain deferred.

## Behavior

The body of `if`, `else`, `while`, `do`, classic `for` and `for...of` can be one statement without braces: `if (value < 0) return "negative";`, `for (let i = 0; i < 4; i++) total += i;`. The statement forms a block of its own. Bodies nest, and an `else` belongs to the nearest `if` without one.

A `const` or `let` declaration and an empty statement are rejected as bodies with `syntax` errors that say what to write. A body without braces counts toward the nesting limit of 128. Loop-header names stay scoped to the loop. A destructuring assignment can be a body.

See [the syntax guide](../statement-bodies.md).

## Implementation

`Parser::body` in `syntax.rs` returns a block, or parses one statement and drains the pending-statement queue into a one-statement block. The six body positions call it. It is a non-inlined helper and increments the block depth. No other pass changed, and the editor source needed no change.

## Repairs

- Three tests expected a `syntax` error for a body without braces; they now check a declaration body instead.
- `acceptance::cli_release_rejects_unsupported_commands_and_duplicate_options` asserted an older help line. The help text changed when the `test` command was added; the assertion now matches the current line.

## Evidence

- Snapshots: `archive/braceless-bodies-start` (sources and changed tests), `archive/braceless-bodies-benchmark-start`, `archive/braceless-bodies-consumers-start` (gate 133 consumers).
- `target/editor/braceless-bodies-red.log` keeps the initial failing run.
- `braceless-bodies-final-compiler.log`: twelve suites pass on the final sources: `braceless_bodies` (an executable `std:testing` fixture, seven static rejections and a 200-level nesting rejection), `loops`, `control_flow`, `control_flow_native`, `for_of`, `higher_order_function_values`, `type_syntax`, `formatting`, `destructuring_assignment`, `declaration_lists`, `for_pattern_initializers` and `acceptance` (24).
- The executable fixture passes in Node through `tools/node-reference/run.mjs`. `braceless-bodies-guide-verification.log`: the guide example passes natively and in Node.
- `braceless-bodies-final-editor.log`: all 213 editor tests pass, including `single_statement_bodies_keep_names_types_and_loop_scopes`.
- Consumers were rebuilt and replaced by atomic rename before execution checks; hashes are in `braceless-bodies-final-debug.log`.
- Runtime harness: the first run (`braceless-bodies-harness.log`) records 350 passes and 1 failure. The failure and the first untimed verification both came from an ownership error in the new workload source (an array literal took the `input` parameter inside the loop). After the workload was corrected, the affected group of 22 checks passes in `braceless-bodies-harness-repaired.log`, and both registration checks pass in `braceless-bodies-registration.log`. The other 329 original passes are retained; the full harness was not restarted.

## Matched support

`braceless_bodies` retains five cells per iteration: a braceless if/else-if/else chain with returns, a counted loop, a `for...of`, nested loops with a dangling-else chain, and a `while` followed by `do...while`. The independent oracle is closed-form; the registration test also compares it with the platform's braced loops on 2048 cases. Hashes, guards, the whole-scenario denominator, registration, runtime and allocation-profile hooks, oracle fingerprints and the report category are in place.

`braceless_bodies`, `classic_for_control` and `else_if_control` pass untimed on native and Node-after-tsc in `benchmarks/runtime/results/2026-10-05T12-02-38-193Z-verify.json`.

No timing, allocation profiling or complete compiler/LSP regression was run. SQLite measurements are unchanged. Next gate is 135.
