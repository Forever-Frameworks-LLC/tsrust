# Scoped borrowed conditional carriers checkpoint

## Current behavior and implementation

The retained borrowed conditional witness now executes with `1 2` and `15 40`.
Type::BorrowedCallable preserves scoped lifetime semantics separately from owned
Type::StoredCallable. The registry identifies borrowed families by signature and
concrete origin pair, with constant-time borrowed lookup. Compaction remaps both
origin lists and identity keys transactionally. Borrowed family admission never
certifies an owned environment.

borrowed_callable_contracts.rs checks actual closure metadata.
borrowed_conditional_carriers.rs registers compatible alternatives once; conditional
flow joins their capture loans. borrowed_callable_emit.rs emits concrete native
lifetime parameters and forwards call/count methods to the selected environment.
borrowed_callback_result_adapters.rs converts joined failure payloads at callback
boundaries. error_effects.rs recognizes borrowed family dependencies, including
callback parameter origins and executable callers. Projections expose the common
contract for hover, signature help and result chains.

Return, throw and owning callback transfers reject invalid scoped escapes.
Readonly/mutable parameters can borrow matching families. Direct conditional
calls construct a family only for distinct alternatives; the same-origin path
retains its original receiver borrowing behavior. The conditional regression found
and repaired the initial same-origin consumption/mutable-receiver change. A second
native witness found and repaired missing callback error facts and mismatched
callback versus storage Result types.

## Verification and durable evidence

The final bounded compiler batch passes 116 tests, with 18 conditional tests,
35 callable field controls and all eight depth canaries. The additional mixed
read/mutate and owning-alternative witness passes, as do expanded scope/throw/owning
transfer rejections. Twenty-two focused unit controls and the entire editor crate
(332 tests) pass. Original record owners dispose exactly once at optimization 0/3.
The formatted guide prints `12 14` then `0 14`.

The matched generic adapter workload retains 131 observations. All five native
and Node-after-tsc cases pass the independent oracle. Token guards, hash registration
and mutation rejection pass. Existing oracle/run/profile/report hooks remain
registered. Timing was not run. The larger workload's implementationAccepted and
measured flags remain false because Gate173 and its feature family are incomplete.

Whole-project formatting changes one workload and its fingerprint with tokens
preserved. The final checkpoint pass changes zero files or fingerprints. Exact
red, intermediate failures and green logs are stored under
`runtime/diagnostics/borrowed-conditional-source/` in benchmarks/records.sqlite.
The matched verification key is
`runtime/results/2026-10-07T10-40-13-922Z-verify.json`. Restore and compare every
record byte for byte before removing loose copies. The source checkpoint includes
this handoff; resolve its commit with git log for this file.

## Remaining gates

Gate173 remains open. Read docs/plans/conditional-callable-carriers-plan.md and
docs/plans/polymorphic-callable-transport-plan.md. Consuming borrowed alternatives need
an explicit once contract and lifetime-bearing representation. Nullable/aggregate
and higher-order builtin transport of scoped sums require focused source/native
and editor coverage. Do not claim them from direct invocation or callback-parameter
checks. Polymorphic selection/replacement must preserve deferred type parameters
and witness-specific arity. The retained
literal_guard_native_callable_replacement_has_common_carrier red still needs its
native-compatible carrier.

The point-sensitive mutable factory sequence unit witness remains red. FlowState
transports capture maps (4db64c6), and native-checked capture-mode facts are indexed
(17cc5f7), but the early/late validation boundary and per-instance environment
updates are still needed. Storage ownership is not proof of unique allocation
across loops or recursion. Keep read-only/conflict guards until restoration and
native execution are proved. No shared JavaScript-style capture cells or implicit
clones are authorized.

## Workspace and continuation prompt

Preserve the 15 owner-edited fixtures unstaged. One agent works in this tree.
Use explicit Git paths; never git add -A crates or git commit -a. Reuse
/private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0 and pinned Node.
Format the whole project before source checkpoints. Preserve immutable SQLite
records, prune disposable matched artifacts, and atomically refresh consumers.
Already running editor servers must restart to load refreshed executables.

> Continue Tsrust from docs/handoffs/handoff-2026-10-07-borrowed-conditional-carriers.md.
> Read AGENTS.md, docs/development-workflow.md,
> docs/plans/conditional-callable-carriers-plan.md and
> docs/plans/polymorphic-callable-transport-plan.md first. Scoped monomorphic borrowed
> conditional carriers now work with native disposal, default/cardinality,
> callback failure and editor checks. Gate173 remains open. Preserve 15 owner
> fixture edits. Continue consuming borrowed sums and scoped transport coverage,
> then polymorphic carriers/replacement and the explicit early/late capture-fact
> bridge for point-sensitive environment restoration. Keep ownership rules and
> retained reds. Use bounded TDD/native/editor checks, ready matched workloads,
> whole-project formatting, SQLite readback and commit checkpoints. No timing
> until the meaningful feature-family milestone.
