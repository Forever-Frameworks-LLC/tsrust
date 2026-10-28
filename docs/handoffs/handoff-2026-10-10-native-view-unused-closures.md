# Unused closure returned-view validation

Continue from `a3378e89`. Read AGENTS.md, docs/development-workflow.md and
docs/milestones/native-byte-views-implementation.md. Gate 189 and S07 remain open.

Final analysis validates native returned storage contracts across checked closures
and specialized methods, even without a call. Call summary mapping reuses the same
storage validator. Consuming an environment cannot return a borrow of its owned or
mutable capture storage. Shared captures permit readonly views; exclusive receiver
views require mutable invocation. Carried views are distinct and remain permitted.

The focused red witness parsed and checked a real unused consuming closure, then
injected its native storage return summary before final analysis. Previously final
analysis accepted it. It now diagnoses native-view-call. Positive controls preserve
shared invocation and consuming carried-view contracts. This is not complete public
factory-body acceptance; the borrowed factory interface is still private/guarded.

193 focused checks passed: 61 native-view private tests, 107 source checks (Result
and all ten normal-stack depth canaries), nine registered signatures and 16 native
editor checks. No failed or aborted targets. CLI build was warning-free; project
formatting changed zero files and fingerprints. Original red and green logs are
retained with exact readback in SQLite:
`runtime/results/native-view-unused-closures-2026-10-10.json`.
No full regressions or timing. Installed consumers remain f569190a.

## Continuation prompt

Continue gate 189 by implementing and verifying complete factory-generated source
bodies. Finish escaped/rebound environments, mixed callback records and specialized
callback/storage families against the implementation-plan matrix. Include targeted
completion joins and borrowed payload controls as required. Keep real slices,
explicit ownership and public borrowed signatures guarded until the matrix agrees.
Use focused TDD, bounded compiler/native/editor checks, project formatting, SQLite
exact evidence and explicit-path commits. Refresh consumers before checking their
executables. Full regressions and timing belong to S07 completion.
