# Gate 189 returned native view sources

Read AGENTS.md, docs/development-workflow.md, docs/resources-bytes-plan.md and
 docs/native-byte-views-implementation.md first. This checkpoint follows 008b53d9.
Gate 189 and S07 remain open. Public native view signatures remain guarded.

The native_view_returns module traces returned origins through local aliases to
parameters and rejects local owner storage, consuming storage parameters and cyclic
aliases. Final parameter access inference revalidates the summary. CheckedFunction
retains native_returns for future backend lifetime planning. Loan normalization now
distinguishes Storage dependencies from Carried lifetime slots within input values.
Consuming a view value can preserve its inner lifetime; consuming owned storage cannot.

Focused checks: 31 private loan/return tests, 89 normal-stack depth canaries across ten
suites, and 16 native editor tests pass. No aborted target is reported. These private
checks inject loan origins; they do not prove source factory execution. Original logs
are stored byte-exact in records.sqlite under
runtime/results/native-view-returns-2026-10-10-checks.json. No timing or full regression
is claimed. Installed consumers remain f569190a. The warm debug compiler was rebuilt.

Next implement precise input carrier seeding and named Rust lifetime emission, keeping
independent input lifetimes distinct. Carried currently has no production constructor,
so non-test builds report its unused-variant warning. Do not hide this warning instead
of implementing seeding. Reconcile summaries with reaching exception completions;
returns superseded by finally are currently conservatively retained. Forwarding and
stored functions, captures, dynamic collection stores/removal, last use and the broader
control-flow/composition matrix remain open. Keep native_binding_types' public guard.

Use the single /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0,
Node 24.11.1 first on PATH, and /opt/homebrew/bin/python3 for store.py. One agent/tree.

## Continuation prompt

Continue gate 189 from docs/handoff-2026-10-10-native-view-returns.md. Implement input
carrier seeding and safe named source lifetimes, then forwarding/collection/control-flow
provenance. Preserve actual Rust slices without copies or hidden owners. Keep source
factories guarded until analyzer and backend agree. Finish source/native/editor checks
and matched benchmark support before gate acceptance. Full regressions and timing
remain the completed S07 family milestone.

Count correction: the original editor log has two diagnostic and fourteen intelligence
tests. The earlier metadata reported fourteen; the call-transport evidence retains the
correction without overwriting the original SQLite record.
