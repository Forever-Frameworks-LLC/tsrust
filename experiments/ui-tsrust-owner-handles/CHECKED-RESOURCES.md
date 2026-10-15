# Checked resource continuation

This continuation supersedes the external-bridge blocker for the narrow
`std:testing.TrackedResource` route. The external projected-record bridge still
rejects `Drop`, deliberately. The historical L/H/R/I matrices retain their original
source identities and observations; the current L-row mapping below records the
new route. Compiler acceptance passes 7 resource, 56 testing-library and 4 callback-error tests.
Final editor acceptance and consumer refresh remain pending.

## Representation and behavior

Each registration owns its callback slot and optional resource state. New
registrations are prepended. Since this candidate does not reparent components,
descending creation order destroys every descendant before its ancestors on owner
scope exit. Display ordering and event delivery never reorder registrations.
Dispatch extracts only the callback, restores it to the same registration after
return or failure, then processes deferred retirement. Mounting during a callback
can shift indices, so restoration searches by the nonreused component identity.

Logical retirement immediately ends source state access. Physical release waits
until the active callback finishes. Replacement releases displaced state exactly
once. Rejected state installation and enqueue return their resource in `Err`.
Accepted queue entries own their payload through callback completion, stale discard,
or owner destruction. Queue payload order is independent of component-tree order.
No source counter or owner-retaining audit stands in for native destruction.

## Current matrix mapping

| Historical rows | Current checked witness / disposition |
| --- | --- |
| L01–L03 | Compiler source checkpoint `3c23f7b8`; `std:testing` native allocation/Drop implementation, non-retaining integer audit, explicit thread affinity. Current UI representation is this continuation. |
| L04–L05 | Compiler `tracked_resources` tests and `examples/resource-lifetimes`: storage, moves, replacement, removal, capture, consuming close, automatic destruction and transfer rejection. |
| L06 | Process-lifetime integer audit survives resource destruction without retaining it. There is no owned audit handle or reset; audit-handle teardown subcases do not apply. |
| L07 | Compiler negative controls reject premature/missing/duplicate/wrong-ID release and reversed order. Positive examples and UI witnesses use the same public audit. |
| L08–L09 | `groupedRelease`, `component`, existing named-handler regressions and factory capture witnesses: state and captures remain owned through delivery and release once. |
| L10–L12 | `retained`, `activeRelease`, `treeRelease`: self/root/ancestor retirement, nested mounting, throwing active callback and deferred destruction. |
| L13–L14 | `independentRelease` and `treeRelease`: repeated/overlapping retirement, active sibling access, depth-three state/capture dependencies, reordered siblings. No incidental sibling order is required. |
| L15 | `replacementRelease`: active same-key replacement, fresh retained state/capture, stale queued event and display reorder. |
| L16–L17 | `queuedRelease`, `activeRelease`, `independentRelease`: accepted payload remains live through callback; success, throw, stale discard, foreign/rejected enqueue and owner-drop destruction. Rejection returns the original identity. Queue-clear remains absent. |
| L18–L19 | `treeRelease`, `groupedRelease`, `independentRelease`, `queuedRelease`: automatic drop after delivery, escaped handles, surviving allocator and independently live owner. |
| L20 | `activeRelease`: 1,000 cycles with nested state/captures and payloads return to the independent native live-resource baseline. Audit integers remain until process exit. |
| L21 | Existing feasibility/identity suites: 12 tests passed initially; one emitted-record-number assertion required updating after the additional native types. The affected test then passed. Original failure retained. |
| L22 | Whole-project formatter, token preservation/idempotence and formatted native harness execution. |
| L23 | Intentional external Drop rejection and accepted plain-data bridge remain separate from this compiler-owned opaque resource. |
| L24–L26 | Current mapping and compiler/UI evidence are in progress. Historical matrices and SQLite evidence remain attributed; no worker readiness or measurement claim. |

The candidate CLI pin is built from `3c23f7b8`, with no compiler source changes in
this continuation. Tests and UI source change independently. The harness preserves
the old matrix pin separately from the executable candidate pin. Disposable native
bridge dependencies are rebuilt by the ordinary compiler project runner; historical
library paths are not required to survive restart.

## Executed evidence

Current records use `tsrust-resource-ui-*`, `tsrust-resource-compiler-final.log`
and `tsrust-resource-demo-final.*` in the canonical analytics SQLite store once
closed and verified. The full native demo exits 0 and separately reports the
historical external Drop rejection. Native calibration and the plain-data bridge
control pass through the supported project runner.

Initial UI checks used CLI SHA-256
`3b63dd086028f69aee1efbc1039b0478dd3cc41c84a50fec5e4935f951bf49e1`.
The compiler integration-test build unified development dependency features and
rebuilt the same compiler source as
`3155aef0f9ffa39eee8535848aa4d4fa43a6cc2d988449f9764a6a0554ef6dce`.
The pin guard rejected the first demo attempt after that rebuild. Provenance was
checked, the candidate pin updated, and the final demo passed. This was not a
language or ownership failure; the original guard failure is retained.

## Limits and benchmark applicability

This is synchronous UI lifetime acceptance, not an async/worker UI scheduler.
Concurrent runtime allocation, owner executor authority, late async completion,
cooperative cancellation and affinity-aware cleanup remain explicit dependencies.
Local async resource ownership is covered by the compiler example; started tasks,
workers and channels reject this deliberately thread-affine resource.

The observer measures synchronous native allocation destruction, not asynchronous
backend cleanup. Its process audit grows with the bounded test workload. There is
no meaningful matched Node-after-tsc destructor timing workload: JavaScript garbage
collection does not provide the deterministic lifetime contract under test. These
are correctness witnesses, not performance measurements. SQL/task lifecycle code is
unchanged; accepted S18 three-driver evidence is reused.
