# Owned conditional callable carriers checkpoint

## Current behavior

The unannotated_conditional_arrows_infer_common_callable_carrier red now passes.
Concrete owned monomorphic alternatives use a common native storage family while
retaining distinct closure origins. Parameter/result incompatibility remains an
error. A consuming alternative makes the common contract consuming; reusable
alternatives adapt once without reconstructing captures. Source expressions and
ownership effects are checked once.

checked_callable_storage.rs isolates checked conversion and registration.
conditional_callable_carriers.rs isolates contract inference. projections.rs
exports checked conditional result facts; editor grouping logic retains inner
source delimiters without swallowing caller delimiters. Guide examples and
IntelliSense reflect the inferred callable signature and result chains.

## Verification

Eleven conditional native tests pass. Four selected original record owners dispose
exactly once at optimization levels 0 and 3, covering success, failure and
abandonment. The compiler batch passes 115 tests with all eight depth canaries
and callable field controls. Two stored-record native unit controls pass. The
whole editor crate passes 331 tests; three grouped forms pass the new focused
editor witness.

The matched generic adapter family now retains 129 values. Five native and
Node-after-tsc verification cases pass against the independent oracle. Source
guards and registration reject mutated captures. Measurement remains pending.
The guide prints 12 through the current CLI. Whole-project formatting changes
zero files and fingerprints. Exact verification records belong in SQLite, with
readback before loose copies are removed.

## Remaining work

Read docs/conditional-callable-carriers-plan.md. Borrowed environments need an
actual lifetime-aware carrier; do not force local captures to own their source.
Polymorphic functions retain their own type parameters and witness-specific
invocation contracts; do not infer a fabricated fixed signature.

The literal_guard_native_callable_replacement_has_common_carrier red remains
excluded from the 115-test compiler batch. The mutable factory sequence unit
witness remains red. Native-compatible replacement and source environment
restoration must both be proved. Do not remove read-only/conflict guards early.

FlowState already transports capture maps through branches, defaults, cleanup,
exceptions, frames and requests (4db64c6). The capture-mode facts index (17cc5f7)
reads native checker storage/access and retains missing/conflicting alternatives
as unknown. It is not yet wired: source cardinality validation runs before native
capture checking. Add the explicit staged boundary, then install per-instance
state, read captures into frames and update each completion path. Storage ownership
does not prove unique allocation across loops or recursion.

Gate173 and the full captured-effects/environment plans remain open. No new
timing is claimed.

## Workspace

Preserve the 15 owner-edited fixtures unstaged. One agent works in this tree. Use
explicit Git paths, the shared /private/tmp/tsrust-gate100-editor-target and
CARGO_INCREMENTAL=0. Use pinned Node v24.11.1. Format the whole project before
source checkpoints. Keep closed evidence in benchmarks/records.sqlite.

Continue prompt:

> Continue Tsrust from docs/handoff-2026-10-07-owned-conditional-carriers.md. Read
> AGENTS.md, docs/development-workflow.md,
> docs/conditional-callable-carriers-plan.md and
> docs/polymorphic-callable-transport-plan.md first. Owned monomorphic conditional
> carriers now pass native disposal and editor checks. Gate173 remains open.
> Preserve 15 owner fixture edits. Continue borrowed lifetime carriers and
> polymorphic replacement, then connect checked capture facts to flow-local
> environment updates through an explicit early/late validation boundary. Keep
> retained reds, ownership rules and source/native parity. Use bounded checks,
> prepared matched workloads, whole-project formatting and SQLite readback.

## Durable evidence and consumers

All 19 diagnostic logs, including the original failures, are stored under
`runtime/diagnostics/owned-conditional-carriers/` in benchmarks/records.sqlite.
The five-case verification is stored as
`runtime/results/2026-10-07T09-56-20-995Z-verify.json`. Every record was restored
and compared byte for byte before its loose copy was removed. SQLite quick_check
passes. The four workspace debug/editor consumers and installed CLI have been
refreshed atomically. Already running editor servers require a restart.

## Next gate red

Owned prerequisite accepted at 30c92f1. The borrowed carrier witness now fails
at distinct readonly capture origins; its mutable local selection cases remain
part of the intended behavior. Read the final section of
docs/conditional-callable-carriers-plan.md before implementation. The red is
intentional and does not revoke owned prerequisite acceptance. Its exact failure
and the zero-change whole-project formatting check are stored in SQLite under
`runtime/diagnostics/borrowed-conditional-carriers/`.
