# Resumable callable summary acceptance

## State

The scheduling family is accepted. Source checkpoints are 366e0b0 for retained invocation sessions and ordered pending defaults, 974fa8e for bounded eager wrapper completion, and 294af82 for complete source acceptance. The overall language goal remains active; this family does not complete the language or standard-library roadmap.

Fifteen owner fixture edits remain uncommitted and untouched. Inspect Git status and use explicit staging paths. Read AGENTS.md, docs/development-workflow.md, docs/plans/resumable-callable-summaries-plan.md and docs/milestones/resumable-callable-summaries.md before continuing.

## Behavior and verification

Parent/request/node dependencies resume saved pre-node partitions without replaying completed statements. Capture changes invalidate cached sessions with an epoch protecting in-flight updates. Pending parameter defaults resume from their saved predecessor state. Changes to completed entry dependencies rebuild conservatively rather than reuse stale state. Normal, return, throwing, finally, guard and recursive convergence contracts remain checked.

Eager dispatch permits at most one fixed wrapper followed by an invocation-free leaf. Dynamic or shadowed targets, recursive calls, wrapper chains, default/rest signatures, argument calls and mutation remain scheduled. The original nested budget passes unchanged at 8/32/64 calls: 17/65/129 evaluations and 16/64/128 invocation visits. Independent non-leaf controls preserve completed prefixes and both finally completion paths.

Complete compiler regression: 2,342 passing checks, zero failures and three intentional ignores. These are the opt-in scheduling diagnostic, module frontend measurement and subprocess contention worker. Complete editor regression: 360 passing checks, zero failures or ignores. Focused flow controls: 102; all eight depth canaries and 23 affected native contracts pass.

Current release verification passes five native and Node-after-tsc cases for each of generic_value_adapters, nested_function_defaults and scoped_callable_records. Their emitted hashes match historical sources under the harness's trimmed-output convention. The first raw-versus-trimmed comparison was incorrect; both the rejected comparison and correction remain immutable evidence. No runtime timing is repeated. The earlier retained-session release source check completed within its 55-second bound in about 2.47 seconds; that single compiler observation is not a repeated throughput benchmark.

## Publication and evidence

All five consumers are current: target/debug/tsrust, target/editor/debug/tsrust, target/debug/tsrust-lsp, target/editor/debug/tsrust-lsp and target/release/tsrust. Atomic sibling-copy publication and byte hashes are recorded. All pass CLI/native/editor smoke checks, including the integrated std:path and std:os exports, generic callable state and the bounded wrapper path.

Canonical evidence is benchmarks/records.sqlite. Full regression receipts use tsrust-resumption-family-compiler* and tsrust-resumption-family-editor* identities. Current workload verification is runtime/results/2026-10-08T10-22-46-855Z-verify.json. Consumer refresh/smoke and cleanup records use tsrust-resumption-family-consumer-* and tsrust-resumption-family-cleanup.json. Closed loose copies are removed only after exact readback. Coverage metadata and generated inventory are updated without marking fresh runtime measurements.

Cleanup removes 339 closed files and 6,505,287,564 bytes. Root binaries, installed consumers and dependency libraries remain. No cache-warming rebuild follows cleanup. Reuse /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0, offline/locked Cargo, Node 24.11.1 first on PATH and /opt/homebrew/bin/python3 for storage tools.

## Next work and coordination

The parallel std:path.relative plan remains docs/plans/parallel-path-relative-plan.md. Branch acceptance must include integration into latest main, SQLite evidence and all five consumers. Obtain exclusive main ownership before mutation; do not merge through owner changes or replace main SQLite. Main has no uncommitted compiler source at this family handoff.

For subsequent compiler work, select a concrete open requirement from the current language/standard-library roadmap and write its bounded milestone plan before implementation. Keep full regressions at meaningful family milestones. Preserve the current scheduling controls, independent reference evaluator and exact analytics.

## Continuation prompt

Continue Tsrust from docs/handoffs/handoff-2026-10-08-resumable-summaries-accepted.md. Read AGENTS.md and docs/development-workflow.md first. Scheduling is accepted: compiler 2,342, editor 360, three matched workloads verified untimed, five consumers current. Preserve fifteen owner fixtures and explicit staging. Coordinate std:path.relative integration through exclusive ownership, or select the next concrete compiler requirement from the roadmap. Use TDD, isolated helpers, complete editor/docs/benchmark support, bounded checks and immutable SQLite evidence. Do not restart completed regressions or retime unchanged generated programs. Provide a physical handoff at the next completed checkpoint.
