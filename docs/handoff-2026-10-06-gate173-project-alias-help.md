# Handoff: project generic alias IntelliSense

Read AGENTS.md, docs/development-workflow.md and both function plans. Previous
local editor checkpoints: dc7c62e and cf7d447. Gate173 remains open.

ProjectCheckedTypeFacts.generic_values maps checked local binding locations to
original declaration locations across modules. Project editor application copies
generic source signatures, parameter names and docs from the authoritative source
index. Conflicting declarations at one binding location remain unknown. Imported
aliases and captured factory results now retain their generic contract; unsaved
dependency edits update names and documentation. Concrete methods supply call
help with imported nominal aliases and default omission.

ProjectType projection now rejects deferred empty prototypes, matching single-file
projection. It no longer presents a generic value as Function<void>. Generic call
labels include checked Owned, ReadOnly and Mutable access; rest and callable slots
retain their existing representation. Five earlier assertion strings omitted
access and were updated to require the correct owning/borrowing contracts. Original
failed runs are retained rather than overwritten. No runtime semantics changed.

Two project controls cover imported capture-free aliases and captured factory
results, including documentation, nominal aliases and unsaved parameter names.
A local control checks exact Owned<string> and ReadOnly<int[]> parameter labels.
All 17 generic-value compiler tests pass. Complete editor coverage passes across
preserved/resumed groups: 210 tests before project, repaired project 47 tests,
remaining suites 37 tests, total 294. Documentation tests also pass (zero cases).
Only affected and unrun groups were resumed after expectation repairs.

Five generic_function_results native/Node-after-tsc cases pass untimed. Canonical
verify record: benchmarks/runtime/results/2026-10-06T23-30-35-956Z-verify.json.
All 117 compiler and 262 oracle input hashes match final source. verifiedOnly is
true and execution is empty. No timing or family acceptance is claimed.

Current diagnostic/build/check logs use the tsrust-project-alias- prefix. Import
and exact-byte readback are recorded in benchmarks/records.sqlite before removing
loose logs and verify copies. All processes are terminal. Five consumers are
atomically refreshed and hash-verified. Obsolete executable scratch is pruned,
retaining libraries and the five consumers. Four owner fixtures remain
untouched and uncommitted. Hourly checks remain paused.

Next audit mixed generic source signatures with fixed imported/private nominal
types, conditional/optional value contexts, navigation and ambiguous shared spans.
Then implement real polymorphic callback/fixed-contract/record/array adapters.
Keep the empty prototype compatibility guard until actual adaptation exists;
do not clone captures or introduce shared capture cells. Continue the full gate173
context audit and eventual named-declaration family milestone.

Continuation prompt: Continue Tsrust from this handoff. Finish mixed nominal and
conditional/optional generic alias editor contracts, then implement real callable
storage/callback adaptation and the wider gate173 audit. Preserve owner fixtures,
explicit clone policy, bounded checking, SQLite evidence and the full objective.
Do not mark gate173 complete based on this editor checkpoint.
