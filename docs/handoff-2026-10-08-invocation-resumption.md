# Invocation resumption implementation checkpoint

## State

Cross-call resumption is implemented but the scheduling family is not accepted. The overall language goal remains active. Read AGENTS.md, docs/development-workflow.md and docs/plans/resumable-callable-summaries-plan.md before continuing. Preserve the fifteen owner-edited fixtures listed by Git status; never stage them.

Engine entries retain invocation frames and body drivers. Callee dependencies record parent requests and graph nodes. Summary changes requeue saved pre-node inputs rather than replaying completed statements. Capture-reader changes invalidate cached sessions, with an epoch preventing an invalidated in-flight session from being stored again.

Parameter entry records a suspended parameter index. Notifications resume that default from its predecessor state without repeating completed defaults. Completed-entry dependency growth still rebuilds the frame conservatively. Coordinate-specific invalidation of completed defaults remains open. One-shot dispatch helpers are retained as test-only references.

## Evidence and limits

The pending-default witness first fails, then completes with exactly `before`, `target`, `target`. Retained invocation results and call order match the original evaluator for absent/supplied parameters, defaults, rest, catch and finally. Idle drives perform no additional calls. The focused generic-call suite passed 98 controls before the additional equivalence control; that control passed separately.

All eight depth canaries, 23 affected compiler/native contracts and 57 editor project checks pass. These are focused checks, not fresh full-family regression acceptance.

The parsed-source adapter diagnostic completes in about 2.17 seconds: 341 requests, 660 evaluations and 1,013 invocation visits for workload. Its summary ends opaque. The full debug CLI source check succeeds within its 55-second bound, taking about 10.66 seconds. Neither observation is native runtime timing or a comparable release speedup. Source consumers have not been refreshed from this implementation checkpoint.

The initial resumption checkpoint leaves the original nested budget red: eight calls require 25 evaluations, above its ceiling of 18. Subsequent bounded wrapper completion makes the unchanged budget green at 8/32/64 calls, requiring 17/65/129 evaluations and 16/64/128 invocation visits. Eager dispatch permits only one wrapper plus an invocation-free leaf; dynamic, shadowed, recursive and deeper wrapper targets remain scheduled. Non-leaf prefix/finally controls independently pass. All 102 focused controls now pass with no budget excluded, along with eight depth canaries, 23 native contracts and 57 editor projects. Do not increase the stack or introduce synchronous recursion across arbitrary callees.

## Next work

Finish the scheduling-family acceptance audit, including dependency-valid ordered-entry invalidation, recursive monotone convergence, opacity propagation and finally completion. At this milestone, reconcile full compiler/editor regressions and compare representative emitted programs. Reverify affected groups after repairs and retain completed checks. Preserve existing runtime evidence where emitted source is unchanged. The earlier full adapter verification passes native and Node-after-tsc cases and matches historical emitted hashes; repeat affected verification after the bounded wrapper change before extending that claim.

Use /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0 and offline/locked Cargo. Node 24.11.1 goes first on PATH; use /opt/homebrew/bin/python3 for SQLite tools. Closed diagnostic evidence is imported into benchmarks/records.sqlite with exact readback. Whole-project formatting is required before source checkpoints. Publish the five consumers only after accepted source, by sibling .refresh-tmp copy and atomic rename.

The parallel std:path.relative worker may continue isolated development. Main integration remains required but needs exclusive tree ownership; do not merge through these active changes or overwrite owner fixtures or SQLite.

## Continuation prompt

Continue Tsrust from docs/handoff-2026-10-08-invocation-resumption.md. Read AGENTS.md, docs/development-workflow.md and docs/plans/resumable-callable-summaries-plan.md. Invocation resumption and bounded wrapper completion are implementation checkpoints, not family acceptance. The unchanged nested budget and all 102 focused controls pass. Preserve fifteen owner fixtures and use explicit Git paths. Complete the ordered-entry/capture/recursive acceptance audit and full compiler/editor milestone regressions. Preserve and resume completed evidence, compare emitted programs, retain applicable SQLite runtime measurements, then refresh five consumers and provide an accepted handoff.
