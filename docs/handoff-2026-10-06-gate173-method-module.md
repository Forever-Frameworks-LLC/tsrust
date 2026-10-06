# Handoff: isolated invocation replay

Read AGENTS.md, docs/development-workflow.md,
docs/generic-function-values-plan.md and
docs/handoff-2026-10-06-gate173-generic-data-boundary.md first. This checkpoint
extracts the existing method materialization functions into closure_methods.rs.
The functions are byte-identical to the prior implementation except for visibility.
No new generic value behavior is accepted. Five red native witnesses remain.

Six implemented generic-value tests, all 63 depth canaries and the whole editor
crate (282 tests) pass. Exact logs and the extraction check are verified in SQLite
under target/editor/closure-method-{module-check,editor-check,canaries,source-check}.log;
loose copies were removed. No timing or benchmark semantics changed.

Next implement explicit deferred generic source metadata and argument contracts
using the original template solver. The isolated replay module owns cache bounds,
substitution, body checking and capture-layout verification. Preserve one environment
across methods; retain original parameter/default/rest contracts and ownership.
Do not use void as a generic wildcard. Gate173 and the family milestone remain open.

The four owner fixture edits remain untouched and uncommitted. Use explicit Git
paths, the single warm target and CARGO_INCREMENTAL=0. All check processes are
terminal. Executable consumers retain prior equivalent behavior; rebuilding is
required when behavior changes. Hourly checks remain paused.
