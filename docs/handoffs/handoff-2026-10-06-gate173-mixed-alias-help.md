# Handoff: fixed nominal generic alias signatures

Read AGENTS.md, docs/development-workflow.md and both function plans. Previous
checkpoint: ab15ba0. Gate173 and the broad objective remain open.

The red project witness showed imported User as Person still displayed as User
in an unresolved generic alias header, where a local User could mean another
record. generic_value_metadata.rs now collects original annotation use/declaration
pairs from deferred templates. Its iterative walk includes array, group, union,
tuple, applied and callable syntax and excludes generic type parameters. Generic
record template identity is available before materialization. Metadata is collected
before moving analyzer state into CheckedProgram, outside recursive dispatch.

ProjectCheckedTypeFacts.generic_nominals preserves both source locations. Editor
header edits use those exact annotation ranges and project nominal spelling.
Parameter labels, string defaults and generic placeholders remain unchanged.
Source parameter edits proceed in declaration order. No runtime semantics change.
No parser/analyzer dispatch changed, so depth canaries were not rerun for this chunk.

Three project controls verify imported fixed types despite local name collisions,
applied generic records with unchanged labels/defaults, and private result types
with original dependency field navigation. Whole editor coverage passes 296 tests;
the subsequently added private-result control and final parameter-order repair
pass in the three-test focused group. All 17 generic-value compiler tests pass.

Five generic_function_results native/Node-after-tsc cases pass untimed. Canonical
record: benchmarks/runtime/results/2026-10-06T23-37-40-929Z-verify.json. All 118 compiler and 262 oracle
input fingerprints match final source. verifiedOnly is true and execution is
empty; no family acceptance or performance measurement is claimed.

Eleven logs use the tsrust-mixed-alias- prefix. Original red and failed build
evidence is preserved alongside repaired runs in benchmarks/records.sqlite.
Exact-byte readback is verified and loose logs and the verify copy are removed. All processes are terminal. Five consumers
are atomically refreshed and hash-verified. Obsolete executable scratch is pruned,
retaining libraries and five consumers. Four owner fixtures stay untouched and
uncommitted. Hourly checks remain paused.

Next audit alias-expanded annotations, conditional/optional generic values,
navigation and ambiguous shared spans. Then implement real polymorphic callback,
fixed-contract and aggregate storage adapters; keep prototype compatibility guards
until actual adapters exist. Preserve one captured environment, existing ownership
and explicit clone policy. Continue the full gate173 context audit and eventual
named-declaration feature-family regression/performance milestone.

Continuation prompt: Continue Tsrust from this handoff. Verify remaining generic
alias editor contexts, then implement real callback/storage adapters and the wider
gate173 audit. Read both plans. Preserve owner fixtures, bounded checks and SQLite
evidence. Do not treat this metadata checkpoint as full generic-value acceptance.
