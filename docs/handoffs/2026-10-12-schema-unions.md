# Tsrust: schema union checkpoint

Implementation checkpoint: `92df22f7` on `main`. Active goal remains the full
roadmap through S21 integrated release acceptance. S01–S10 are accepted; S11 is
in progress. Installed CLI/editor consumers remain at the accepted S10 checkpoint.

## Read first

- `AGENTS.md`
- `docs/development-workflow.md`
- `docs/standard-library-roadmap.md`
- `docs/standard-library-coverage.md`
- `docs/plans/schema-plan.md`

Use one agent. The latest user override is section-batched validation: implement
compiler/native/editor/examples/workload support together, then run affected
checks. Write witnesses during implementation, but do not repeatedly execute
partial work. After repairs, rerun affected checks only. Full regressions and
timing belong at meaningful feature-family milestones.

## Completed section

Public `s.union` and `s.discriminatedUnion` preserve ordered runtime plans separately
from normalized output types. Preparation borrows input, retains generated defaults
once and checkpoints bounded issues; commitment moves selected storage. Exceptions
stop selection. Presence adapters preserve full branch plans. Object schemas with
identical ordered output shapes share type identity, not runtime plan ownership.
Explicit outer stripping preserves branch-local policy unless overridden.

Common record-union fields expose borrowed values, including nested records/arrays
and mixed scalar/nominal enum alternatives. Literal/enum equality narrows record
alternatives. Replacing the owner invalidates those facts. Exact JSON discriminator
selection preserves integers above 2^53. Mixed runtime int/number comparisons retain
conservative facts where rounding prevents exact exclusion. Move a whole narrowed
record to a local before consuming its owned fields; existing ownership rules stay.

Keep approved future transform semantics: once preceding checks pass and a branch
consumes input for a transform, later failure must not retry another branch or clone
input for rollback. Runtime witnesses exist; public transform APIs are still pending.

## Evidence

- 11 public compiler/native union cases, including executable catalog examples.
- 28 schema editor checks.
- 23 workload contract checks.
- 56 schema runtime unit checks, including preparation, ownership and issue budgets.
- Earlier affected runs: 10 enum and 9 literal native cases passed.
- Ordered/discriminated workloads agree with Node-after-tsc at n=0,1,17,2000.
- Whole-project formatting preserved tokens, checked idempotence and refreshed two
  workload fingerprints; formatted native workloads were executed again.

All results are untimed. Closed logs, including failures and affected repairs, are
byte-verified in `benchmarks/records.sqlite` under `tsrust-unions-section-*.log`.
No S11 family acceptance or measurements are claimed.

## Next

Continue the remaining S11 requirements in the plan: refinements/transforms,
preprocessing and cross-field checks, recursive/composed plans, collection schemas,
JSON Schema export, fused typed JSON decoding and family acceptance. Respect the
documented dependency gates for async and format-specific validators. Existing
internal prototypes do not constitute completed public APIs. Then continue S12–S21
in dependency order. Ask only about unresolved material semantics.

Reuse `CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target` and
`CARGO_INCREMENTAL=0`; Cargo checks use `--offline --locked`. Verified compiler:
`/private/tmp/tsrust-gate100-editor-target/debug/tsrust`. Format the whole project
with `TSRUST` pointing there and `node tools/format-project.mjs` before checkpoints.
Node is available at `/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node`.

Preserve unrelated untracked owner documents: `UI_VISION.md` and plans
`axum-hono-http-ui-contract.md`, `editable-list-app-example.md`,
`parallel-ui-foundations-plan.md`, `ui-feasibility-review.md`,
`ui-route-contracts-first-task.md`. No source work is pending outside the checkpoint.

## Copyable continuation prompt

```text
Use $caveman. Continue Tsrust in /Users/seancotter/Desktop/dev/learning/tsrust toward the existing roadmap through S21 integrated release acceptance. Keep the full active goal; do not mark it complete while required work remains.

Inspect Git status, then read AGENTS.md, docs/handoffs/2026-10-12-schema-unions.md, docs/development-workflow.md, docs/standard-library-roadmap.md, docs/standard-library-coverage.md and docs/plans/schema-plan.md. Implementation checkpoint: 92df22f7.

S01–S10 are accepted. Public ordered/discriminated unions now have focused native/editor/example checks and untimed matched workloads. S11 remains incomplete. Preserve approved contracts and owner files. Reconcile source and SQLite evidence; do not treat internal prototypes as public completion.

Continue remaining S11 implementation and acceptance, then S12–S21 in dependency order. One agent; implement whole coherent sections before running checks. Complete IntelliSense, examples and benchmark support as features land. Reuse the documented Cargo target with incremental compilation disabled. Format the whole project before checkpoints. Preserve evidence in SQLite and distinguish implementation acceptance from measurements. Ask only about unresolved material semantics.
```
