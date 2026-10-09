# B7 borrowed callback input checkpoint

B7/gate 183 remains in progress. This continues the array-input checkpoint
`946e307`; B5 remains accepted at `00214a11`. Read AGENTS.md,
`docs/development-workflow.md`, `docs/caller-borrowed-record-stores-plan.md` and the
B5 acceptance handoff first. Discriminated unions remain a dedicated required D2
feature. There is no pending owner decision for this slice.

## Current implementation

The native input lifetime grouping now handles the same stored origin receiving
multiple inputs at one destination. References are reborrowed for a common
caller-selected lifetime. Different destination slots remain independent.
Nested record destinations and multiple array captures have executable witnesses.

Stable readonly data views resolve to their actual owners. Stable callback aliases
also preserve their original input roots. Named relay summaries map alias roots
before mapping source parameter positions. Source scope validation uses those roots
instead of treating a local reference binding as its captured owner.

Readonly and mutable callback inputs can be captured by a stored wrapper. Captured
callback native types have independent generic slots, including inputs with the same
signature. Borrowed family conversions carry these generic environment types as
well as lifetimes. Generic substitutions match complete identifiers. Locally bound
callback environments infer their native capture types from their initializer.
`callback_capture_emit.rs` isolates that support from the main emitter.

An owned escaping closure still cannot consume a borrowed callback input. Mutable
callback inputs remain exclusively borrowed while a holder retains their wrapper.
Calling the original callback after holder disposal works and preserves its state.

## Evidence and scope

Closed evidence is stored byte-exactly in benchmarks/records.sqlite. Log keys use
the `tsrust-b7-` prefix. Preserve failed witnesses and supplement them with repairs.

- `callback-final-batch.log`: seventeen B7 tests and nine existing record replacement
  tests. This is an affected batch, not a complete compiler run.
- `local-callback-environment.log`: the affected higher-order ownership/native test,
  including the newly supported local borrowing capture, passes at both optimization
  settings used by that test.
- `boundary-native.log`: the old borrowed-array-input rejection is now an executable
  assertion; callee-local and direct borrowed callback store controls remain.
- `once-escape-control.log`: the existing Once borrowed-environment escape rejection
  remains enforced.
- `callback-depth-canaries.log`: all eight depth canaries pass after source dispatch
  changes. The later emitter fallback preserves existing non-function contexts.
- `callback-editor.log`: diagnostics, hover, signature help and transfer-conflict
  control pass for an aliased borrowed callback input.
- `expanded-registration-guards.log`: workload sources, hashes, denominator, mutation
  controls, independent oracle, ABI and runner/profile/report hooks pass.
- `callback-checkpoint-format.log`: two project formatting passes make no changes;
  fifteen unrelated fixture edits remain untouched and uncommitted.

The matched caller_borrowed_record_stores workload now retains eleven scalar
observations per iteration. It covers array aliases and relays, repeated stores,
nested multiple captures, readonly callback aliases, distinct callback input types,
mutable array captures and mutable callback state/release. Its TypeScript counterpart
and arithmetic oracle are independent of native execution. Verification is untimed;
no B7 timing or complete family acceptance is claimed. The original eight-observation
verify record remains preserved separately from the expanded workload record. Both
source bytes and the original oracle/source contract are preserved under their exact
recorded hashes in SQLite, without loose snapshots. The expanded record is
`runtime/results/2026-10-09T17-21-26-368Z-verify.json`: five correctness cases on each
of native and Node-after-tsc, with no execution, startup or compilation samples.

## Next work

0. Fix `caller_borrowed_callback_replacement.rs`, an intentional red B7 witness.
   Calling the same installer twice on one holder with different callback environment
   types fails in generated Rust: the first captured input fixes the native payload
   type of the shared stored origin, so the second input cannot enter that carrier.
   The source callable contract matches; do not restrict the source program or hide
   the backend error with an unsupported diagnostic. Preserve separate source origin
   and native environment identities. `callback-replacement-red.log` preserves E0308.
   The seventeen-test positive B7 file passes; this additional test does not. The
   complete compiler crate was not run at this checkpoint.
1. Implement direct borrowed callback assignment (`holder.run = action`) without
   requiring the user to write a forwarding arrow. Do not remove the carrier guard
   without a native representation and source ownership proof.
2. Cover nullable payload stores, mutable data aliases and nested borrowed callback
   forwarding. Keep callback contracts and native environment identities independent.
3. Establish finite summary behavior for recursive/nested relays. The current summary
   repeatedly appends destination paths; do not accept an unbounded fixed point.
4. Integrate failure before/after stores, branch/loop stores, exact replacement and
   clearing, early exits and disposal with record_call_capture_release. Its existing
   conservative guards are intentional until those paths have explicit evidence.
5. Extend the matched workload as remaining forms land. Finish the whole family
   before complete regressions, timing and performance review. Keep measured false
   and implementationAccepted false until their separate acceptance requirements hold.

Continue using the single shared Cargo target, CARGO_INCREMENTAL=0, Node 24.11.1
first on PATH and Python 3.11+ for SQLite. Format the whole project before source
checkpoints. Use explicit Git paths; preserve the fifteen unrelated fixtures.

## Consumer state

Private debug/compiler/editor and release builds include this slice. The five
installed consumers remain at the last accepted B5 source. Refresh them atomically
when the B7 acceptance requirements are met; this checkpoint is not family acceptance.
Disposable standalone B7 emit/build probes were removed after preserving their logs.
