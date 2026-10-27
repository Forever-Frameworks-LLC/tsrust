# Gate 189 input carrier origins

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md and
docs/milestones/native-byte-views-implementation.md first. This checkpoint follows 21ed1af5.
Gate 189 and S07 remain open. No source view export is enabled.

native_view_parameters seeds dormant carried lifetime origins before callable bodies.
Native view parameters, record fields, nested array ranks and tagged union alternatives
retain precise source slots. Result origins keep their success guard. Moving an input
view can return its inner lifetime without borrowing its local parameter slot. Seeds
do not conflict with their own parameter. Ordinary programs take a constant empty
path via the native declaration flag initialized in Analyzer::new.

Formatted source tests verify consumed view returns, distinct record/array slots,
Result guards, independent alternative input origins, recursive view diagnostics and
unrelated recursive types. Native identities are injected into parsed source for these
private checks; no source factory or native returned-view acceptance is claimed.

Recursive carriers reaching a native view report native-view-shape until recursive
lifetime planning is implemented. Map/Set and callback carriers, captures, forwarding,
named backend lifetimes, collection transport and exception completion reconciliation
remain pending. Keep native_binding_types' borrowed-signature guard. The earlier
Carried unused-variant warning is resolved by production parameter seeding.

Checks passed: 38 focused compiler tests, 89 depth canaries and 16 native editor tests
(two diagnostic tests and fourteen intelligence tests). Whole-project formatting
changed zero files and fingerprints.

Exact red, setup failures, repairs and check logs are in records.sqlite under
runtime/results/native-view-inputs-2026-10-10-checks.json. No timing or whole-crate run.
Installed consumers remain f569190a; warm compiler rebuilt for project formatting.
Use the single /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0,
Node 24.11.1 first on PATH, and /opt/homebrew/bin/python3 for store.py. One agent/tree.

## Continuation prompt

Continue gate 189 from docs/handoffs/handoff-2026-10-10-native-view-inputs.md. Implement explicit
named Rust lifetimes from checked input/output origin slots, preserving distinct inputs
and safe returned slices. Add generated native O0/O3 witnesses. Then connect forwarding,
collection/control-flow provenance, recursive carriers and captures. Keep factories
guarded until analyzer and backend agree. Finish source/native/editor and matched
benchmark support before acceptance; full regressions/timing belong to S07 completion.
