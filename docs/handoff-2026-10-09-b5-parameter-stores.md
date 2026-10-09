# Handoff: B5 nullable record stores through mutable parameters

Read AGENTS.md, docs/development-workflow.md,
docs/handoff-2026-10-09-b5-continuation.md,
docs/b5-scoped-aggregate-acceptance-map.md and this file. The preceding accepted
checkpoint is `9a94c53b`, which accepts borrowed nullable record views. B5 is still
incomplete. The fifteen unrelated owner fixture changes remain uncommitted.

## Behavior and implementation

A mutable record parameter can install or replace a nullable callback record whose
captures are owned, and can clear it. Scalar captures copy their creation-time
values. Named nullable values, null-based fields, nested record paths and relay
calls work. Callbacks borrowing the storing function's local or parameter storage
remain rejected with `callback-escape`. Existing caller capture loans stay
protected after replacement; this does not implement ledger B7's borrowing escapes.

`record_parameter_carriers.rs` propagates nullable replacement obligations and
widens callers with the existing nullable join. Caller roots are marked as control
stored so precise initialization cannot erase the widened environment.
`nullable_parameter_store_emit.rs` emits conversion bounds for callback leaves
inside nullable payloads. `record_parameter_replacement_emit.rs` evaluates the
replacement once and converts its payload through `record_carrier_emit.rs`.

A named nullable value needs its concrete environment preserved during injection;
`array_result_storage.rs` handles compatible nominal nullable unions. Readonly
injection in `analysis.rs` must retain a borrowed union. Forgetting that last step
moved caller values and broke two previously accepted borrowed-view tests. Those
failures and their repairs are retained in SQLite evidence. Do not remove this
borrowing distinction when changing native generic names next.

## Verification

The first composition run produced 194 passes and three failures out of 197 tests.
Two were the readonly injection regression described above. One could not spawn
its subprocess. After repair, all 24 nullable composition tests pass, and the
subprocess test passes independently. Completed unrelated evidence was retained;
there was no repeated full compiler/LSP run. The focused editor witness and all
eight depth canaries pass. The guide example prints `4` then `7` on native and Node.

The matched workload adds `nullableParameterStores`, with independent formula
`2 * value + 4`, hand values 6 and 8, pinned source behavior and mutation controls.
It has 135 observations per scenario. Verification is untimed. Family timing and
complete compiler/LSP milestone regressions remain pending; do not mark B5 or the
workload measured.

The first workload verification compiled native code, then TypeScript rejected an
optional call on a field known to be undefined after clearing. Both counterparts
now observe clearing with a direct undefined comparison; the independent formula
is unchanged. This failed run is retained as evidence. Final formatting changes
zero files and zero fingerprints. Obsolete executable variants were pruned from
the shared target: 31 files, 776,430,648 bytes; current variants and all consumers
were retained. Final workload verification and SQLite identifiers follow below.

## Remaining B5 work

1. Independent concrete environments for two parameters carrying the same nullable
   callback record declaration. Native generics currently use `C{contract id}`;
   they need identity per parameter leaf, consistently in every native type emitted
   inside the callee. Preserve callback error contracts and conversion bounds.
2. A union such as `Holder | Other` where Holder contains a callback. The owner's
   scope question remains unanswered: defer to a tagged-union feature, or include
   explicit discriminant narrowing in B5. Do not invent a narrowing rule or remove
   the acceptance row without the answer.

Structural data-record Set/Map keys remain separate, with the decided Rust-style
structural equality and rejection of number and callable fields. B14 switch
fallthrough is also undecided. Do not reopen accepted ownership rules.

## Environment and checkpoint discipline

Use one agent, explicit Git paths, no archives or snapshots. Preserve the fifteen
unrelated fixture changes. Format the whole project with protected bytes held in
memory; restore only formatter changes to those files. Use the single target
`/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`, offline locked
Cargo, Node 24.11.1 first on PATH, and `/opt/homebrew/bin/python3` for store.py.
Do not run optimized workload compilation alongside native-heavy test suites.
Import closed logs and results into SQLite, restore and compare exact bytes, then
remove loose copies. Refresh all five consumers atomically after the source commit.

## Closed checkpoint evidence

Final formatted registration and mutation guards pass. Five correctness cases pass
on native and Node-after-tsc with 135 observations. The immutable SQLite key is
`runtime/results/2026-10-09T15-20-51-970Z-verify.json`. `verifiedOnly` is true;
startup, compilation and execution timing arrays are empty. Twenty-five closed
logs with prefix `tsrust-b5-parameter-store` and the runtime result were imported,
restored byte for byte and removed from loose storage. Final source formatting
changes zero files and zero fingerprints. No full compiler/LSP or timing run was
performed for this isolated requirement.

### Continuation prompt

Continue Tsrust from this handoff and the B5 acceptance map. Finish independent
callback environments for two same-type nullable record parameters, with focused
red native/editor witnesses, matched workload support, formatting and SQLite
evidence. Preserve accepted owned parameter stores and readonly views. The mixed
record union scope question is still pending; do not invent a narrowing policy.
Finish B5 milestone regressions and timing only after its scope is complete.
