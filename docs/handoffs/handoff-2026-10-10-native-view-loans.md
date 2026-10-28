# Gate 189 retained native source loans

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md and
docs/milestones/native-byte-views-implementation.md first. This checkpoint follows 119dbf6c.
Gate 189 and S07 remain open. Public native view factories remain unavailable.

## Implementation

native_view_loans.rs isolates loan storage and checks. Binding stores optional boxed
loan vectors; Context indexes holders by stable binding identity. Ordinary functions
without views take empty-index and empty-provenance fast paths. Native returned-borrow
call checking validates source alternatives and activates initializer provenance
collection. Initializers and whole-binding replacement retain supported aggregate and
wrapper origins. Ownership access and replacement now enforce retained native loans.

Readonly loans block mutation/transfer; exclusive loans also block reads. Moving a
holder releases its old enforcement while retaining provenance for the moved value.
Lexical scope identity prevents dead or shadowed holders from keeping loans active.
Nested reborrows retain owner and parent identities. Conditional sources must have
stable storage in every arm; a named arm cannot hide a temporary arm.

## Limits and next work

This is private analyzer integration, tested by injecting loans into real analyzer
contexts. It is not end-to-end Tsrust view acceptance. No registry factory was added,
and native_binding_types still rejects borrowed signatures. Keep that guard.

Implement branch-state joins, Result success/error projections and release, precise
aggregate field provenance, return-source inference and backend lifetime signatures.
Then cover forwarding functions, stored factory values, reborrows, captures, scope
and exception/loop exits, and last-use release. Do not copy storage or introduce hidden
shared owners. Recursive wrappers and callback/view ABI composition also remain open.
Publish factories, byte contracts, complete IntelliSense and matched view workloads
only when these paths are supported together. The full plan remains authoritative.

## Evidence and environment

Seven loan checks pass. They cover readonly/exclusive conflicts, actual analyzer
replacement checks, moving, scope release, reborrows, aggregate roots, temporary
rejection and a conditional-temporary repair. Ten normal-stack canary suites pass
89 tests with no failed or aborted targets. Fourteen native project editor checks
pass. The canary batch preceded the narrow conditional-source helper repair; the
seven final loan checks cover that repair. No full regression or timing ran.

SQLite record runtime/results/native-view-loans-2026-10-10-checks.json preserves
original failures, repairs and final outputs. Readback must match before loose copies
are removed. Installed consumers remain f569190a; the shared debug compiler is rebuilt
for project formatting. Reuse /private/tmp/tsrust-gate100-editor-target with
CARGO_INCREMENTAL=0, Node 24.11.1 first on PATH and /opt/homebrew/bin/python3 for
store.py. No unrelated owner edits remain.

## Continuation prompt

Continue gate 189 from docs/handoffs/handoff-2026-10-10-native-view-loans.md. Read AGENTS.md
and docs/development-workflow.md first. Finish native view branch/Result provenance,
return lifetime relationships and source execution. Keep source exports guarded until
the analyzer and emitter agree. Retain real zero-copy slice storage and explicit
ownership. Add source-level TDD, full editor help and matched benchmark support.
Gate 189 remains open; S07 family measurement remains deferred until implementation.
