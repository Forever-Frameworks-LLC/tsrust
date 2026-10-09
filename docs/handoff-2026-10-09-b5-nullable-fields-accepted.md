# Handoff: B5 nullable fields checkpoint accepted

Date: 2026-10-09. Supersedes `docs/handoff-2026-10-09-b5-nullable-fields-review.md`.
Remaining B5 scope: `docs/b5-scoped-aggregate-acceptance-map.md`.

B5 is not complete. This checkpoint accepts direct nullable callback-record fields and
populated starts, and repairs defects found during review.

## Accepted in this checkpoint

- Direct nullable callback-record fields: population, replacement, clearing, sibling
  independence, nested paths, `null` variant, optional fields, ordinary siblings, and a
  right operand that reads the old field. Composition with a scoped callback array.
- Nullable callback-record locals and fields can start populated. Before this
  checkpoint the initializer reported `Expected Holder | undefined, found Holder`.
- A record with a populated nullable field whose closure owns its captures has a
  defined native type and can be returned from a function.
- Unplanned nullable field stores are diagnosed for every binding shape. An indexed
  store such as `boxes[0].saved = ...` previously reached the Rust compiler.

## Defects repaired

Three tests in `tests/scoped_record_composition.rs` failed at the previous accepted
commit `680f1205`. Earlier checkpoints ran subsets and missed them.

1. `removed_record_map_preserves_owned_callback_environment`: an emptied array of
   records with a Map field used the unspecialized declaration as its empty layout.
   `empty_scoped_record_environment` now gives Map fields their empty layout.
2. `cleanup_loop_preserves_populated_entry_before_empty_backedge`: a cleanup control
   dispatch converted Map entries into a layout that the plan marks empty. The dispatch
   now uses the planned empty state.
3. `ordinary_loop_deletion_does_not_scan_unused_released_captures`: commit `62b35d58`
   made every loop with distinct Map edge environments convert the Map on each
   iteration. `map_entry_demand::mutates` now keeps that conversion only when the loop
   can write, move or call through a released capture owner.

Seven suites held rejection rows that gate 181 truthiness and scoped Map storage made
stale: `acceptance`, `loops`, `control_flow_native`, `numbers`, `value_types`,
`nullable_narrowing` and `scoped_record_boundaries`. Each accepted form was checked
natively and against the pinned Node reference, then replaced by a positive witness in
`tests/truthiness.rs`, `tests/loops.rs` or `tests/scoped_record_boundaries.rs`.

The editor test `array_removal_docs_and_nullable_result_views_are_consistent` failed
because documented examples written as whole programs were wrapped without being run.
The example collector now uses the body of such a program.

`support::reject` now prints the source when a rejection row is accepted.

## Evidence

- Whole compiler crate: 2,566 passed, 0 failed, 3 ignored. This run includes the eight
  depth canaries and the unit tests.
- Whole editor crate: 419 passed, 0 failed.
- `scoped_record_composition` suite: 183 tests.
- Registration test for the workload passes. Native and Node-after-tsc verification of
  `scoped_record_composition` passes, untimed.
- Project format: 0 files changed.
- SQLite holds two verification results (`runtime/results/2026-10-09T10-41-00-006Z-verify.json`
  and `...T10-59-17-611Z-verify.json`), the fourteen logs of the previous agent, and two
  whole-crate logs. `tsrust-nullable-fields-whole-compiler.log` is the original run
  with seven failures; `tsrust-nullable-fields-whole-compiler-final.log` is the clean
  run. All were read back byte for byte and the loose copies removed.

## Consumers

The five installed consumers were refreshed from commit `53b53e5b` by copy and atomic
rename, and each `tsrust` binary ran a smoke program that uses a populated nullable
callback record, a truthiness condition and a unary sign.

## Not done

- No timing run and no performance review. Those belong to the B5 family milestone.
- The workload has no helper for populated starts or returned records. Compiler and
  editor tests cover them.
- Every row under "Diagnosed, not supported" and "Not started" in the acceptance map.
- Ledger rows B7 to B13 and D2 to D5 in `docs/compiler-completion-plan.md`.

## Owner decisions still open

- Float and callable field policy for structural record keys.
- Whether a function may return `Record | undefined` when the closure owns its
  captures. It reports `callback-escape` today; the non-nullable form is accepted.
- B14 `switch` fallthrough.
- Whether the formatter may rewrite the protected fixtures that still hold old unary
  spacing.

## Working rules

Follow `AGENTS.md` and `docs/development-workflow.md`. Stage explicit paths. The
fifteen fixtures listed in the previous handoff stay modified and unstaged. Copy them
aside before `node tools/format-project.mjs` and restore any that change. Use
`CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`,
`--offline --locked`, Node 24.11.1 first on `PATH`, and `/opt/homebrew/bin/python3`
for `benchmarks/storage/store.py`. Import runtime results with root
`<repo>/benchmarks` and key `runtime/results/...`; import logs with root `/private/tmp`
and the basename.
