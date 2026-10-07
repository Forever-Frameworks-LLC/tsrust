# Polymorphic argument transport checkpoint

## Implemented prerequisite

Generic factories can retain polymorphic named functions without erasing their
source contract. Named and arrow selectors preserve independent one-argument and
two-argument functions. A reusable retained callable supports integer and string
specializations. The internal checked annotation carries source identity through
substitution and definitive closure compaction; it is never source syntax.

Ownership remains explicit. Borrowed captures cannot escape. Transferred callables
cannot be reused through the original binding. Consuming captures permit one call.
Four original captured owners dispose exactly once after success, callback failure
and abandoned selectors, verified at optimization levels 0 and 3.

## Evidence

The compiler integration batch passes 191 checks, including all eight depth
canaries. Generic unit checks pass 78 tests. The complete editor crate passes
330 tests. A focused definitive-compaction test also passes. Signature help
preserves parameter counts and concrete types; hover reveals no internal identity.
The nullable contextual-inference regression found during verification is repaired
without changing its original witness.

The existing generic value adapter benchmark family now observes 127 values.
Native and Node-after-tsc verification passes all five representative cases against
the independent oracle. Registration mutation guards pass. No timing was performed.
The workload remains marked unmeasured and the larger gate remains unaccepted.
Whole-project formatting changes zero files and zero fingerprints.

Exact logs, including earlier failing witnesses and corrections, and the runtime
verify record are imported into benchmarks/records.sqlite, read back and removed
from loose storage. Record paths use runtime/diagnostics/polymorphic-argument-transport
and runtime/results/2026-10-07T09-15-06-199Z-verify.json.

## Remaining work

Gate 173 and the captured-effects plan remain open. Follow
docs/polymorphic-callable-transport-plan.md for point-sensitive mutable environment
state, checked capture modes, strong updates for one owned instance, weak updates
for merged instances, consumption and recursive convergence. Do not remove the
read-only environment guards before the native/source witnesses prove these cases.
Keep stable environment allocation identity separate from state at a program point.
Do not introduce shared JavaScript capture cells or implicit cloning.

Two native common-carrier witnesses remain red:
conditional_expressions::unannotated_conditional_arrows_infer_common_callable_carrier
and generic_call_capture_effects::literal_guard_native_callable_replacement_has_common_carrier.
The latter is explicitly excluded from this prerequisite batch, not repaired.
Retain these witnesses and the broader environment plan.

## Workspace and continuation

One agent only. Preserve all 15 unrelated owner-edited fixtures unstaged. Use
explicit Git paths. Reuse /private/tmp/tsrust-gate100-editor-target and set
CARGO_INCREMENTAL=0. Use the pinned Node v24.11.1 executable. Run bounded checks
and format the entire project before source checkpoints. Store closed evidence
in SQLite and verify readback before removing loose records.

Workspace target/debug and target/editor/debug compiler and language server
consumers are refreshed atomically. Workspace target/release/tsrust is refreshed
for the matched verification. A running VS Code server still needs the Tsrust
restart command. The installed ~/.cargo/bin/tsrust was not refreshed here.

Continue prompt:

> Continue Tsrust from docs/handoff-2026-10-07-polymorphic-argument-transport.md.
> Read AGENTS.md, docs/development-workflow.md,
> docs/polymorphic-callable-transport-plan.md and
> docs/handoff-2026-10-07-invocation-worklist.md first. The polymorphic factory
> transport prerequisite passes native ownership/disposal and editor checks;
> gate 173 remains open. Preserve 15 owner fixture edits. Next implement
> point-sensitive mutable captured environments with checked capture modes,
> independent factory instances and recursive convergence, using retained red
> witnesses. Do not lift read-only guards or claim common native callable carriers
> supported without their focused native evidence. Keep benchmarks ready and
> unmeasured until timing, use bounded checks, whole-project formatting and SQLite.
