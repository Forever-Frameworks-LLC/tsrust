# Native view source storage matrix

Read AGENTS.md, docs/development-workflow.md and
plans/native-view-callable-storage-plan.md. This checkpoint follows a48f4f1e.
Gate 189/S07 and the overall plan remain open. Public view/viewMut are still guarded.

## Implemented source paths

Moved callback fields, already stored callback moves, direct record replacement,
replacement after moving the previous callback, single-element callback arrays
removed through pop, present nullable callback fields and nested concrete captures
now pass normal analysis and complete native emission. Readonly and exclusive view
cases preserve owner pointer identity, and exclusive writes reach the owner.
Independent native compilation rejects selected-owner escape. A second invocation
of a consumed callback is rejected with closure-consumed and a related transfer span.

Conditional fields and distinct returned-record callback alternatives now join in
one scoped owning family. Conversions move enum payloads; they do not copy bytes or
hide owners. Returned-record alternatives execute both branches for both access modes.

native_view_storage_contracts.rs retains concrete scoped identities through structural
array and nullable destinations, and interns finite joined families.
native_view_storage_epochs.rs marks direct local replacements for fresh lifetime
inference. It excludes parameters, physical capture slots, borrowed aliases and
self-dependent RHS expressions. It does not implement branch/loop replacement joins.
The existing storage-epoch emitter evaluates the RHS first, drops available old
record leaves and redeclares the record leaves with newly inferred lifetimes.
Transferred old fields are skipped using checked statement-entry availability.

native_view_loans transports pop/shift element loans and follows record-carrier
wrappers. Native type rendering handles scoped callable arrays. Family declaration
discovery now traverses concrete union roots, omitting unused symbolic families at
generic record fields. source_walk adds an iterative single-expression entry point;
existing body and function entry points retain their traversal behavior.

## Remaining work

Finish the source matrix before publication: mixed alternative arrays, absent nullable
values, captures of already stored scoped families, field replacement, and replacement
through blocks/branches/loops. Existing nested-capture witnesses capture concrete
closures; do not claim they prove stored-family capture ABI support.

Then finish reusable invocation, arguments borrowing/owning data, omission/default/rest
contracts, results without returned capture loans and throwing callback/error transport.
The current ABI guard diagnoses only its implemented checks; do not assume untested
counted calling conventions are already covered. Keep ordinary lifetime-free storage
and physical capture guards intact. No implicit clone or hidden shared owner.

Publish factories only after shared byte observer/mutator contracts, IntelliSense
signatures/help/examples, executable source fixtures and matched subarray workload
support are ready. Full regressions, measurements and consumer refresh belong to the
completed S07 family. Current installed consumers remain f569190a; only the warm CLI
is rebuilt for formatting at checkpoints.

## Evidence and continuation

SQLite record: runtime/results/native-view-storage-matrix-2026-10-10.json.
Preserve the original matrix failures and repaired affected runs. One final negative
control initially expected transferred, but the established diagnostic is
closure-consumed; the diagnostic assertion was corrected without weakening the
rejection or related-span requirement. All native temporary compilations self-clean.

Continuation prompt:

Continue from this checkpoint and finish the remaining native byte-view source
storage matrix and invocation ABI as a coherent feature. Read the documents above;
use the worktree as authoritative. Do not redo accepted private ABI proofs. Add focused
red source witnesses for the remaining forms, implement shared typed provenance and
lifetime transport, then public factories/observers, editor documentation and matched
benchmark support. Keep exact owner and invocation rules, bounded checks and SQLite
evidence. Gate 189/S07 must remain open until the complete documented matrix is proved.

Focused acceptance covers 107 native-view tests across the retained 106-pass run and
the corrected negative-control rerun; no production source changed between them.
The callable unit batch passes 114 tests. The affected integration batch passes
152 tests: 45 callable field/return cases and all 107 normal-stack canaries. Native
editor checks pass 16 tests. All successful processes exit zero; no suite aborts.
The CLI builds without warnings, and project formatting changes zero files or hashes.
No full regression or timing was run. Workspace target is about 1.2 GB; shared target
about 2.5 GB, with incremental compilation disabled and native scratch outputs cleaned.
