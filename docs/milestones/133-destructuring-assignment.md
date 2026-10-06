# Gate 133: destructuring assignment and shorthand record properties

Status: focused compiler/native, editor, guide, current consumer and matched benchmark acceptance complete. Full compiler/LSP regressions and measurements remain deferred.

## Behavior

A pattern on the left of `=` assigns to existing places: `[a, b] = [b, a];`, `[count, word] = pair();`, `({ x, label: text } = point);`, `[values[0], point.x] = [5, 6];`. Targets are whatever a plain assignment accepts. Patterns nest, and an array pattern may end with a rest target, which assigns an explicit copy of the remaining elements.

The right side is evaluated once before any target changes. A written array on the right is not built: each value is held and then assigned in order, so a swap of owned values moves them and clones nothing. A written array with fewer values than targets is a `bounds` error. Any other right side is held once, or read in place when it is a name, and each target takes its position or field under the ordinary access rules.

A destructuring assignment is a statement and may be an update expression of a classic `for` loop. It has no value. Default values and record rest targets are rejected with `unsupported`; a rest target that is not last is a `syntax` error.

A record literal accepts shorthand properties: `{ x, y, label }` is `{ x: x, y: y, label: label }`. The record pattern form of assignment uses the same shorthand.

See the two new sections of [the syntax guide](../destructuring.md).

## Implementation

`Parser::destructuring_assignment` in `syntax.rs` lowers the statement to ordinary declarations of hidden held values and ordinary assignments, through the pending-statement queue. `assign_pattern_positions` and `assign_through_pattern` walk nested targets. `for_update` accepts the lowered statements of a pattern assignment and still rejects a pattern declaration. `record_literal` reads a lone name as a shorthand property. All helpers are non-inlined and outside the recursive statement frame. No analysis or emission pass changed, and the editor needed no change.

The formatter keeps an assigned record pattern on one line: `({ x, y: z } = point);`.

## Evidence

- Snapshots: `archive/destructuring-assignment-start` (sources, guide), `archive/destructuring-assignment-benchmark-start` (harness and tracking files), `archive/destructuring-assignment-consumers-start` (gate 132 consumers).
- `target/editor/destructuring-assignment-red.log` keeps the initial failing run.
- `destructuring-assignment-final-compiler.log`: fifteen suites pass on the final sources: `destructuring_assignment` (an executable `std:testing` fixture and eleven static rejections), `declaration_lists`, `for_pattern_initializers`, `formatting`, `destructuring`, `destructuring_defaults`, `destructuring_rest`, `loops`, `control_flow`, `control_flow_native`, `higher_order_function_values`, `type_syntax`, `nested_records`, `object_keys` and `field_moves`.
- The executable fixture passes in Node through `tools/node-reference/run.mjs`.
- `destructuring-assignment-guide-verification.log`: both guide examples pass natively and in Node. The stated errors for an owned array element source and for use as a value were checked by hand.
- `destructuring-assignment-final-editor.log`: all 212 editor tests pass, including `destructuring_assignment_targets_and_shorthand_properties_resolve_to_their_names` (hover, definition and references).
- Consumers were rebuilt and replaced by atomic rename before execution checks; hashes are in `destructuring-assignment-final-debug.log`.
- `destructuring-assignment-harness.log`: all 349 runtime harness checks pass in one run.

## Matched support

`destructuring_assignment` retains seven cells per iteration: a swap of two owned strings, a Fibonacci step written as a pattern assignment in a loop update clause, a tuple result assigned to two names, a shorthand record literal, and a record pattern assignment. The independent oracle uses a Fibonacci table and closed forms; the registration test also compares it with the platform on 2048 cases. Hashes, guards, the whole-scenario denominator, registration, runtime and allocation-profile hooks, oracle fingerprints and the report category are in place.

Both registration checks pass in `destructuring-assignment-registration.log`. `destructuring_assignment`, `declaration_lists` and `destructuring_patterns` pass untimed on native and Node-after-tsc in `benchmarks/runtime/results/2026-10-05T11-51-56-653Z-verify.json`.

No timing, allocation profiling or complete compiler/LSP regression was run. SQLite measurements are unchanged. Next gate is 134.
