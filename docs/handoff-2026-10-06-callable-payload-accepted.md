# Generic callable payload checkpoint

Read AGENTS.md, docs/development-workflow.md, docs/plans/generic-value-adapters-plan.md
and docs/plans/generic-function-values-plan.md first. This checkpoint accepts focused
generic callable payload support after red checkpoint cc4b487. Full Gate173 and
the thread objective remain open.

## Accepted behavior

Generic T can be a callable, including borrowed readonly/mutable and once payloads.
Complete checked annotations survive inference, source substitution and callable
results. Source keys preserve ownership, access, omission and rest metadata, with
bare/owned and optional/defaulted normalization. Checked borrowed closures retain
their borrowed invocation kind. Callable source applications bypass data-only seed
materialization and acquire checked cache identities when contracts resolve.
Factory state remains shared across concrete payload signatures without cloning.

The original callable-payload red witness and int/string signatures pass. Borrowed
payload owners remain usable; repeated once invocation and access mismatches remain
errors. O0/O3 disposal observes four original records and exact identity mask 30 on
success and failure. Editor hover, signature help and source navigation retain
concrete returned callable contracts. Mutable reference coverage now has retained
Tsrust and strict TypeScript fixtures rather than only an inline witness.

## Evidence

All 45 adapters and 72 total bounded generic/default/value controls pass. All 63
depth canaries and the whole editor crate's 308 tests pass. One key normalization
and two checked reconstruction unit controls pass. The mutable fixture and strict
TypeScript/Node reference both produce 1 1, then 2 2, then 3.

Matched native/Node-after-tsc verification passes all five cases with 96 observations
per scenario. Independent oracle and source guards observe int/string payloads and
retained factory state through a third boolean specialization. Exact record
benchmarks/runtime/results/2026-10-07T01-48-46-795Z-verify.json is imported and verified
byte-for-byte in benchmarks/records.sqlite; its loose copy is removed. Diagnostic
records under /private/tmp/tsrust-callable-payload-*.log include original failures,
corrected checks and final acceptance. SQLite quick_check passes. No timing or full
compiler regression was run. The workload's broad implementationAccepted flag stays
false until the wider family audit closes.

Five executable consumers are refreshed atomically from the warm debug build,
with SHA256 verification. Keep CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target
and CARGO_INCREMENTAL=0. Pruned obsolete test executables and generated matched
outputs after importing evidence; retained libraries and five current consumers.

## Remaining work

Continue the generic adapter context audit before full Gate173 acceptance. Explicit
source-only generic callable applications, parameter-pattern contexts, alias help,
nested defaults/rest payloads and borrowed result escapes need requirement-by-
requirement controls. Check explicit versus inferred data access normalization
without weakening ownership contracts. Do not infer broad acceptance from passing
scalar callback payloads. Keep unresolved deferred origins unknown and preserve
capture/cache identity and bounded specialization limits.

Four owner fixtures remain uncommitted and untouched: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr and map-group-by/documented.tsr. Never
stage them. Use explicit git paths. No processes remain live at checkpoint.

## Continuation prompt

Continue Tsrust from docs/handoff-2026-10-06-callable-payload-accepted.md. Read AGENTS.md,
workflow and both generic value plans. Generic callable payload support is accepted;
full Gate173 stays open. Preserve four owner fixtures. Audit the remaining concrete
contexts with red witnesses, original ownership/cache identities, std:testing,
strict Node references, matched support, focused checks, editor and SQLite evidence.
Do not run timing or full regressions before a meaningful completed family milestone.
