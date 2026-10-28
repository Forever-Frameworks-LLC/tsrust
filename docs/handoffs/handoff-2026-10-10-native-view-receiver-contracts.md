# Native view receiver contract groundwork

Continue from `9673ef98`. Read AGENTS.md, docs/development-workflow.md,
docs/milestones/native-byte-views-implementation.md and the shared-owner handoff first.
Gate 189 and S07 remain open; public borrowed signatures remain guarded.

Returned summaries distinguish Receiver sources for owned or mutable capture
storage. The value walker resolves a stable caller closure binding and adds a
capture-field owner footprint. It does not retain the moved-from outer variable.
Readonly loans allow reads and block mutation/transfer; exclusive loans block all
conflicting access. The private analyzer witness checks both, release/reacquisition,
local receiver return escape and consuming-invocation rejection.

NativeSignature carries an optional receiver mode. The native view planner names
the receiver lifetime and uses it for returned storage slots. Ordinary and
specialized method headers use the named shared/mutable receiver. Other function
signatures keep receiver=None. Mixed callback-record signature composition is not
accepted by this increment.

The native control extracts actual planned headers and supplies safe Vec-backed
bodies. It covers owned captures and mutable owner-reference captures, with shared
and exclusive result types at O0/O3. Pointer identity, shared concurrent reads where
the invocation allows them, exclusive write-through and later mutation all pass.
Independent negative compilations reject receiver escape, moves/drop, owner mutation
and overlapping mutable invocations. For mutable owner-reference captures, the
external owner stays live across the rejected receiver escape, proving that the
result is tied to the receiver rather than only to that external owner lifetime.

These are private provenance and signature controls. They do not compile complete
factory-generated source bodies. Source scope-exit/drop enforcement needs a witness
where an outer holder retains a view of an inner closure that is then destroyed.
Unused consuming closures also need validation before emission, beyond the existing
call-time rejection. Do not publish factories before those paths agree.

185 focused checks passed: 53 private view tests, nine registered signatures,
18 Result tests, 89 depth canaries across all ten suites and 16 native editor tests.
The final private batch includes the expanded mutable owner-reference controls;
only test code changed after the broader batch. No failed/aborted suites. Warm CLI
build was warning-free. Whole-project formatting changed zero files/fingerprints.
Original red, repair and check logs are retained in SQLite under
`runtime/results/native-view-receiver-contracts-2026-10-10.json`, after exact
readback. No timing/full regression. Installed consumers remain f569190a; refreshing
them, including the earlier public ReadOnly capture repair, remains pending.

## Continuation prompt

Continue gate 189 from this handoff. Finish source scope-exit/drop enforcement for
receiver-rooted loans and validate unused consuming closures before emission. Add
real source/factory-generated body witnesses rather than treating injected metadata
or planned-header controls as complete acceptance. Then complete mixed callback
records, escaped/rebound environments, specialized callback/storage families and
the remaining implementation-plan matrix. Preserve real slices and explicit
ownership. Keep factories guarded until the full matrix agrees; finish editor/docs,
matched benchmark support and executable consumer refresh. Full regressions and
timing belong to S07 completion.
