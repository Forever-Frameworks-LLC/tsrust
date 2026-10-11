# Handoff: separate callable body provenance WIP

Read AGENTS.md, docs/development-workflow.md, docs/plans/captured-call-effects-plan.md
and docs/handoff-2026-10-07-effect-alias-identities.md first. This checkpoint adds
isolated extraction of actual callable bodies. It does not execute summaries or
accept Gate173. Accepted executable consumers remain the 399a1c4 image.

Three body-origin tests pass. Assertions preserve the actual arrow span; unknown
values cannot invent bodies from a callback signature. Selected alternatives keep
distinct body identities. Next carry these origins through Binding and expression
Value state independently of Contract, including initializer assignment, branch
joins, type assertions and supplied callback arguments. Resolve aliases by the
existing BindingId index. Then execute ordered normal/exceptional summaries after
arguments using saved callee snapshots and finite recursive worklists.

A retained red exposed conditional closure lookahead scanning across the arm colon.
Exact annotation parsing repairs that syntax ambiguity. A record callback native
control and editor result hover pass. Seventy-two bounded compiler tests pass,
including all eight unfiltered depth canaries. Whole-editor tests pass 328 tests.
The unannotated_conditional_arrows_infer_common_callable_carrier test remains red:
ordinary conditional arrow locals, including annotated locals, need a shared
callable storage carrier. The original eight invoked-capture effect reds remain
open too. Neither defect is skipped without being identified in evidence.

Node independently verifies the selected values 7 and 13. This parser repair adds
no workload observations or measurements; existing conditional callback benchmark
support remains unchanged. Whole-project formatting changes zero sources/hashes.
Closed red, corrected and final logs are imported, read back and removed from loose
storage. Preserve all fifteen owner fixture edits and use explicit Git paths.
Reuse the warm target with CARGO_INCREMENTAL=0 and offline locked Cargo. Refresh
consumers only after verifying the intended checkpoint. Keep the full plan active.
