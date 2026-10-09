# Handoff: B5 borrowed nullable record views

Date: 2026-10-09. Base checkpoint: `d6940c83`. This checkpoint accepts two forms;
B5 remains incomplete. Read AGENTS.md, docs/development-workflow.md,
docs/handoff-2026-10-09-b5-continuation.md and
 docs/b5-scoped-aggregate-acceptance-map.md before continuing.

## Accepted behavior

A `ReadOnly<Holder> | undefined` or null-based parameter borrows the caller's
nullable capturing record. Repeated calls preserve caller ownership. Concrete,
absent, owned-result and conditional environments work. Direct undefined/null
arguments receive empty concrete environments rather than declaration-only tags.

`const saved = boxes[0].saved` produces a readonly view of the nullable capturing
record field. The array retains its value. Mutation while that view remains live
reports `view-conflict`. The view can pass to a readonly nullable parameter,
including after a presence guard narrows it to its record payload. No clone occurs.

## Implementation

- `analysis.rs`: readonly record injection preserves borrowing; narrowed projections
  reinject the concrete record environment; absent arguments use concrete empty tags.
- `record_array_carriers.rs`: nominal nullable contracts inspect owning union forms
  when given views.
- `scoped_record_containers.rs`: accepts a viewed concrete borrowed argument.
- `nullable_record_fields.rs`: discovers saved field views during array carrier
  planning; removes the earlier element-field rejection.
- `array_storage_epochs.rs`: preserves borrowed-union types instead of replacing
  them with owning point types.
- `emit.rs`: nominal record matching for borrowed injection, explicit absent payload
  type parameters, and indexed tagged view construction.

The existing array-element rejection witness was converted into a positive native
witness. New tests cover caller reuse, saved field views, combined parameter/view
transport, null/undefined, owned results, conditional environments and mutation
protection. The guide example has its own formatted native fixture.

## Evidence

- Composition suite: 195 passed, zero failed. This run preceded the final absent
  literal repair; its affected null/undefined/owned/conditional test was rerun and
  passed afterward. Do not claim an unchanged whole-suite run on the final source.
- Eight focused compiler tests passed before the literal repair; the changed edge
  test passed afterward, including direct null and undefined arguments.
- Focused editor witness passes on the final source.
- All eight depth canaries pass on the final source.
- Registration, pinned snippets and mutation guards pass before and after formatting.
- Guide example prints `3 3` then `3`, on native execution and Node.
- Matched workload: 134 observations, five native/Node-after-tsc correctness cases,
  untimed. The new helper's independent formula is `3 * value + 2`.
- SQLite record: `runtime/results/2026-10-09T14-59-51-827Z-verify.json`.
  `verifiedOnly` is true; startup, compilation and execution arrays are empty.
- Final whole-project formatting changes zero files and zero fingerprints. Fifteen
  protected fixtures were preserved in memory and remain uncommitted.

The first workload attempt hit the existing 120-second optimized native compilation
timeout while the composition suite compiled native cases concurrently. The retry
ran without that competing suite and passed. Preserve both logs; the timeout is not
a correctness rejection and the timing protocol was not changed.

Closed diagnostic logs and the runtime result are imported into
`benchmarks/records.sqlite`, restored byte for byte, and their loose copies removed.
Log keys begin `tsrust-b5-borrowed-view` and `tsrust-b5-element-view`. No full compiler
or LSP regressions, timing or performance claims were made for this isolated step.

## Next work

Three B5 forms remain diagnosed:

1. Store a new callback record through a mutable record parameter. Extend existing
   parameter replacement families and nullable payload conversion bounds; preserve
   the rule that stored callbacks own their captures. Coordinate with ledger B7.
2. Independent environments for two parameters carrying the same nullable callback
   record type. Native generic callback names currently identify contracts rather
   than parameter leaves.
3. A union of a capturing record and another record. A question is pending with the
   owner about keeping this in B5 or deferring it to a tagged-union feature. Do not
   remove the row or implement a new narrowing policy without the answer.

Structural data-record keys remain outside B5, with Rust-style structural equality,
no number/callable fields, ownership transfer and explicit clone. B5 family timing,
full milestone regressions and ledger closure remain pending.

## Environment and continuation

One agent in this tree. Fifteen unrelated fixture edits remain unchanged and
uncommitted. Stage explicit paths. No snapshots or archives. Use the one target
`/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`, offline locked
Cargo, Node 24.11.1 first on PATH, and `/opt/homebrew/bin/python3` for store.py.
Follow AGENTS.md for bounded checks; old handoff suggestions do not override it.

Continue from this checkpoint and the acceptance map. Start the mutable-parameter
storage requirement with a focused red native witness and review the existing
record_parameter_carriers and record_parameter_replacement_emit mechanisms. Keep
scope and ownership decisions unchanged. Finish matched workload support and
editor/docs evidence with the implementation. Do not count diagnosed forms as
supported. Provide a handoff at the next accepted checkpoint.
