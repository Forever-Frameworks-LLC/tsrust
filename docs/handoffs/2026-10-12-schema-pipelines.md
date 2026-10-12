# Schema pipeline checkpoint — 2026-10-12

This handoff and the pipeline implementation land in the same source checkpoint.
Use Git HEAD for that checkpoint; prior accepted union checkpoint was `c2c81e64`.
S01–S10 remain accepted. S11 and the full S21 goal remain active.

## Implemented and checked

- `.refine(predicate)` borrows validated output and preserves its type.
- `.transform(callback)` consumes validated output and infers the next type.
- `.coerce((input: Input) => value)` consumes raw input before source validation.
- Reusable immutable plans own readonly captures. Callback exceptions propagate.
- Exact plan identities retain input/output types through arrays, objects, unions,
  defaults, optional/nullable wrappers and intermediate variables.
- Ordered union rejection can restore raw input before a consuming callback.
  First transform/coerce commits; later failure never retries another branch.
- Restoration moves payloads without clones or shared owners. It preserves exact
  numbers, removed fields, unknown JSON data and absence before generated defaults.
- Pipeline defaults produce source input, then validate through the receiving
  chain. Supplied undefined/null do not invoke defaults. Outer optional wrappers
  preserve inner default failures. Discriminators select raw input tags.
- Compiler facts publish lowered schema identities to editor help. Catalog methods
  have executable examples; accepted input remains distinct from final output.

Focused evidence in SQLite `benchmarks/records.sqlite` is listed in the pipeline
acceptance section of `docs/plans/schema-plan.md`: 17 pipeline cases, 21 existing
enum/union cases, 68 runtime checks, 30 editor cases and 25 workload checks passed
across preserved runs. Failures and affected reruns are retained. Whole-project
formatting passed twice. Expanded native/Node-after-tsc parity passed at n=0,1,17,2000,
untimed. Final workload has three operations and checksum `seed + 7*n`; initial
untimed records preserve the earlier two-operation scenario. No S11 timing or
feature-family acceptance is claimed. Installed consumers remain accepted S10.

## Architecture

`stdlib/schema/pipelines.rs` keys exact schemas by internal record identity,
independently of output types. Analysis builds owned records containing child plans
and uniquely owned readonly callback storage. `emit/schema_pipeline_trials.rs`
dispatches native plans, stages, wrappers and composites through the trial protocol.
`stdlib/schema/trial.rs` distinguishes prepared, rejected, absent, committed and
aborted outcomes. Ordinary native checks borrow first; typed projections move only
after those checks. Prepared defaults are not rerun. JSON and owned projection
modules retain inverse metadata; array/object trial modules propagate commitment.
Boxed inverse metadata is uniquely owned and bounded by real lifetimes. Its
allocation costs remain to be measured; the matched workload includes rollback.

## Continue

1. Read AGENTS.md, development-workflow.md, standard-library-roadmap.md,
   standard-library-coverage.md and schema-plan.md. Reconcile against source and
   SQLite; historical backend prototypes do not establish public acceptance.
2. Continue S11 fused typed JSON decoding without an intermediate JsonValue tree
   or stringify/reparse. Then complete remaining composition/recursive/lazy,
   collection schemas, richer issue APIs and JSON Schema export. Dependency-gated
   async/regex/date validators remain tracked at their actual dependencies.
3. Complete S11 family regressions, matched timing review, required optimizations
   and installed consumer refresh before accepting the milestone. Then S12–S21.
4. Preserve settled semantics; ask only about unresolved material choices.

One agent. Write witnesses during implementation, then run checks after a coherent
section is complete. Rerun affected failures only. Full suites/timing belong at
family milestones. Reuse `CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`
with `CARGO_INCREMENTAL=0`, offline/locked Cargo. Current compiler is that target's
`debug/tsrust`. Before checkpoints run whole-project `node tools/format-project.mjs`
with TSRUST set to it. Node is under `/Users/seancotter/.nvm/versions/node/v24.11.1/bin`.
SQLite holds closed logs; verified copies are removed. Do not create archives.

Protected unrelated owner files remain untracked and unchanged:

- `UI_VISION.md`
- `docs/plans/axum-hono-http-ui-contract.md`
- `docs/plans/editable-list-app-example.md`
- `docs/plans/parallel-ui-foundations-plan.md`
- `docs/plans/ui-feasibility-review.md`
- `docs/plans/ui-route-contracts-first-task.md`

## Copyable continuation prompt

```text
Use $caveman. Continue the existing active Tsrust goal through S21 integrated
release acceptance in /Users/seancotter/Desktop/dev/learning/tsrust. Inspect Git
status and read AGENTS.md, docs/handoffs/2026-10-12-schema-pipelines.md,
docs/development-workflow.md, docs/standard-library-roadmap.md,
docs/standard-library-coverage.md and docs/plans/schema-plan.md.

S01–S10 are accepted. S11 pipeline focused acceptance is complete; family acceptance
and timing are not. Continue fused typed JSON decoding, the remaining S11 surface,
then S12–S21 in dependency order. Preserve approved contracts and unrelated owner
files. Use one agent, implementation-first sections, batched affected checks,
complete editor/examples/benchmark support, whole-project formatting before
checkpoints, the shared Cargo target with incremental disabled, and SQLite evidence.
Do not reopen settled semantics or mark the goal complete while required work remains.
```
