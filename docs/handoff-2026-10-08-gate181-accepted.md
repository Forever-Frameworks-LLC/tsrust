# Gate 181 handoff

Truthiness is accepted. Read `AGENTS.md`, `docs/development-workflow.md`,
`docs/plans/compiler-completion-plan.md` and `docs/plans/scoped-callable-aggregate-plan.md`
before continuing. Gates 179 and 180 remain accepted; their prior handoff describes
the surrounding ownership decisions.

Non-boolean conditions and `!` borrow owned operands. Nullable fixed-path conditions
preserve narrowing. Non-boolean `||=` and `&&=` now use a borrowed predicate and
conditional store, followed by a read of the target in place. Binding an owned
result moves the target. Computed non-boolean targets remain unsupported.

Verification: 60 affected compiler tests, all eight depth canaries, complete editor
crate, four truthiness tests including assertion-based project execution, pinned
Node reference and benchmark registration guards. The matched workload passed all
five native/Node-after-tsc correctness cases. No timing was run. Acceptance summary,
depth log and exact benchmark verification bytes are in `benchmarks/records.sqlite`;
loose records were removed only after exact readback.

Fifteen fixture files had pre-existing changes when work resumed. Their authorship
was not verified. The owner authorized proper formatting and keeping the changes;
do not discard them or repeatedly seek spacing permission. Gate 181 does not alter
their semantics. Inspect Git status before taking the next checkpoint.

Next: B5 scoped aggregate composition, followed by B7 caller-borrowing record
callbacks and the remaining completion ledger. Start with focused failing witnesses
for arrays, Maps, Sets and nullable replacement of borrowing-callback records.
Preserve ownership rules; do not introduce implicit clones or shared capture cells.
Use bounded checks during implementation and full family regressions at meaningful
milestones. Keep benchmark support ready as each slice lands.

Use the warm target `/private/tmp/tsrust-gate100-editor-target` with
`CARGO_INCREMENTAL=0`, Node 24.11.1 first on PATH and `/opt/homebrew/bin/python3`
for SQLite storage. Refresh five executable consumers from the accepted commit.

Continuation prompt:

```text
/caveman Continue Tsrust from docs/handoff-2026-10-08-gate181-accepted.md.
Finish the remaining compiler completion ledger in coherent gates. Begin B5 scoped
aggregate composition, then B7. Follow AGENTS.md and development-workflow.md, use
TDD and explicit Git paths, preserve pre-existing changes, complete editor/docs and
matched benchmark support, and store verified evidence in SQLite. Keep the overall
goal active until every approved compiler family is complete and verified.
```
