# Handoff: ownership-taking named captures

Continue Tsrust from this checkpoint. Read AGENTS.md, docs/development-workflow.md,
docs/nested-function-declarations-plan.md, docs/handoff-2026-10-06-gate171-accepted.md
and docs/milestones/172-nested-function-owned-escape.md first. Gate171 remains accepted
at 1c6dd58; gate172 is in progress, not accepted. Inspect Git for this checkpoint hash.

The first gate172 chunk removes blanket consuming-capture rejection in
Analyzer::check_named_capture_arguments and named_capture_lowering::prepare. Existing
Self::access checks transfers and active argument loans. Existing native hidden
parameter emission handles owned parameters. The named-value arrow wrapper uses
owned/once callable transport. Independent factories execute natively; direct owner
reuse, once-callable reuse and overlapping argument borrows are rejected. The unused
consuming-declaration test now accepts a well-typed uncalled body without execution.

Next complete all chunks in the physical gate172 milestone: aggregate retained
storage, disposal and callback failures, recursive retained lifetime, dead-inner
capture propagation, editor examples and matched workload appended at END of the
manifest. Do not claim gate acceptance until these have direct evidence. Gate173
omitted-result inference and optional/default/generic function values remain open.

Keep four owner fixtures untouched and unstaged: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr, map-group-by/documented.tsr under
crates/tsrust-compiler/tests/fixtures. Explicit Git paths only. One agent.
Use the single warm target /private/tmp/tsrust-gate100-editor-target,
CARGO_INCREMENTAL=0, offline locked Cargo and pinned Node v24.11.1. Read current
type syntax: borrowing parameters now use ReadOnly<T>, not the retired read keyword.
Use std:testing, native assertions, Node reference and eight canaries after dispatch
changes. Whole editor before acceptance. No routine full regression, timing, archives
or binary backups. SQLite is canonical; exact read-back before loose removal.
Library caches remain but pruned tests will relink. Hourly checks remain paused.

Checkpoint evidence: capture suite17 and owned-escape tests2 pass, including native
independent factories and precise transfer/once/conflict diagnostics. Node reference
passes the factory fixture. Canaries63 and editor276 pass. Existing matched capture
cases5 pass untimed; new owned-escape workload remains owed before gate acceptance.
Canonical result 2026-10-06T19-52-48-838Z-verify.json has verified source fingerprints
and no timing samples. 11 exact original records are imported/read back in SQLite;
failed assertion and retired-keyword test drafts are retained. Five consumers are
refreshed atomically. 59 scratch executables and runtime output are pruned.
