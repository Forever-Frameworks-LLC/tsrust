# Handoff: B5 scoped aggregate composition accepted

Date: 2026-10-09. Source repair checkpoint: `ace0a4bf`. The family publication
commit contains this handoff. Read AGENTS.md and docs/development-workflow.md first,
then this handoff, docs/plans/compiler-completion-plan.md,
docs/milestones/182-scoped-aggregate-composition.md and the B5 acceptance map.

## State and decisions

B5 is accepted for the agreed storage and transport scope. The owner explicitly
assigned mixed-record unions to a dedicated discriminated-union feature in D2.
The requirement remains open; docs/plans/discriminated-unions-plan.md records its gates
and validation-library motivation. Do not reopen the choice or claim support.

No implicit clone. Scalar captures copy on creation; local owner captures borrow.
Nullable control-flow and array-element stores retain loans to scope end; direct
sequences retain their precise release rule. Mutable record parameter replacement
retains caller capture loans. Data-record structural keys reject number/callable
fields; that separate feature is not started. B14 switch fallthrough remains open.

Fifteen unrelated fixture edits remain uncommitted. Preserve their semantics and
index state. Use explicit Git paths. One agent only; no archives or snapshots.

## Correctness and repair

The whole compiler run completed 2,585 passes, one failing historical boundary and
three intentional ignores. The stronger boundary exposed a real readonly default
parameter gap. Default argument analysis discarded concrete environments; native
injection then moved callback fields. Nullable injection now preserves borrowed
record values, and nullable readonly native parameters use the nominal record bridge.
The isolated emitter module is nullable_record_parameter_emit.rs. Nested data/string
fields use the same associated-field accessors. No runtime clone occurs.

All 208 affected compiler tests pass after repair (seven boundaries and 201 composition
tests). The whole editor run had 427 passes and two stale rejection assertions for
previously accepted B5 forms. Those now assert supported behavior and retain conflict
and escape controls; all 279 intelligence tests pass. Together, retained whole runs
and repaired suites prove 2,586 compiler passes, three intentional ignores, and 429
editor passes. This is not a claim of a second complete rerun. All eight depth canaries
pass. Native and pinned Node default-parameter references pass.

## Measurement and durable evidence

Both scoped_record_composition and fallible_record_callbacks pass five native and
Node-after-tsc correctness cases. Timing uses the original three-process, eight-warmup,
31-sample protocol: 372 execution samples total. Composition retains 136 observations.
SQLite keys: runtime/results/2026-10-09T16-27-56-788Z.json and
tsrust-b5-family-acceptance.json. Original bytes are restored and compared exactly.
Normalized analytics contain all 372 samples. Allocation profiles, startup and
compilation latency are not measured.

Composition batch medians: native 153.913583 ms, Node 155.239750 ms (near parity).
Fallible callbacks: native 0.072625 ms, Node 2.184375 ms (30.08 ratio). These are first
measured baselines for their source pairs. There is no comparable historical timing
regression to infer, and no demonstrated performance defect requiring optimization.
Existing scoped-callable measurements remain intact. Closed log keys use prefix
tsrust-b5-family; failing runs are preserved too. Loose originals are removed only
after exact restoration.

## Next: gate 183, caller-borrowing parameter stores (B7)

A mutable record parameter may currently receive callbacks owning all captures, but
not a callback that keeps a borrowed input from its caller. The requirement remains
incomplete. Start with a red source/native witness that installs a callback borrowing
a readonly array parameter into caller-owned record storage, then invokes it after
return. Protect the caller array against mutation/move until the held callback's loan
ends. Callee locals must still fail callback-escape.

Extend parameter replacement summaries with capture provenance mapped to actual
caller arguments, including relays and nested paths. The current blanket lifetime
rejection lives in record_parameter_carriers.rs. Native closure capture must borrow
the underlying caller owner, not a stack-local reference binding. Bind replacement
lifetimes in record_parameter_replacement_emit.rs instead of erasing them, cloning,
using shared cells or emitting lifetime casts.

Cover readonly and mutable capture access, callback arguments lent for the call,
replacement/clearing, aliases, failure before and after stores, early exits, nested
relays and disposal. Keep precise and scope-end release regimes separate. Add editor
checks, guide and matched workload support as each form lands. Ask only for a real
ownership policy decision; do not treat a diagnostic as implementation.

Continue through the remaining ledger after B7. B8 efficiency, project editor, type
foundations, resources, threads, async and D2-D5 remain incomplete. Do not claim
compiler completion from this callable-family milestone.

## Environment, consumers and cleanup

Node 24.11.1 first on PATH; /opt/homebrew/bin/python3 for storage. Use one Cargo
target, /private/tmp/tsrust-gate100-editor-target, CARGO_INCREMENTAL=0, offline locked
Cargo. Serialize Cargo, native-heavy verification, timing and SQLite publication.
Format the entire project before source checkpoints, preserving unrelated fixture
bytes in memory; no file snapshots. Atomic refresh of all five consumers and native
smokes follows the family commit, with exact evidence under the same log prefix.

Cleanup removes 31 obsolete executable variants (778,113,064 bytes) and retains the
newest variant per target stem and current consumers. Receipt: tsrust-b5-family-cleanup.json.
Do not rebuild merely to warm removed variants.

## Continuation prompt

Continue Tsrust from this handoff. B5/gate 182 is accepted; start gate 183/B7 caller
borrowing stores through mutable record parameters. Preserve the existing ownership
model and owner fixture edits. Use focused red/native/editor witnesses, isolated
modules, matched workloads, whole-project formatting and exact SQLite evidence.
Mixed-record unions remain a dedicated D2 feature. Continue the full ledger; bounded
checks per chunk and family regressions/timing at meaningful milestones.
