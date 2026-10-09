# B5 nullable fields: review and continuation handoff

## Status and reason for handoff

The owner requested a candid progress assessment and offered to move the work to a stronger model. B5 is still incomplete. Recent development has made real progress, but has become too fragmented: neighboring ownership and native carrier cases repeatedly expand each checkpoint. Review the architecture and remaining requirements before continuing incremental patches. Do not equate another narrow passing checkpoint with completion of B5.

Accepted HEAD is `680f120566b2849b3edef9e742e6d0216d00b0b8`. Current nullable-field work is uncommitted and not accepted. There are no live tool sessions or background checks from this agent. The whole-project formatter completed successfully after the latest benchmark edits: one workload formatted, one fingerprint refreshed. No source commit was made for this work.

Read `AGENTS.md`, `docs/development-workflow.md`, `docs/indexed-capture-release-plan.md`, `docs/scoped-callable-aggregate-plan.md`, `docs/map-entry-capture-release-plan.md` and `docs/handoff-2026-10-08-b5-error-restoration.md`. Historical plans contain accepted and pending sections; use current source and explicit tests as authoritative.

## Owner decisions

Keep existing ownership rules. No implicit clone. The owner prefers Rust semantics when JavaScript behavior introduces ambiguity or side effects. Proposed record Set/Map keys should use structural equality and hashing, transfer ownership on insertion, permit explicit clone, protect stored keys against equality-changing mutation, and diagnose unsupported fields. Float and callable field policies are unresolved. Record key implementation has not started in this diff.

Do not spawn agents in this tree. Preserve unrelated edits and use explicit Git paths. Formatting may change layout but must not change their semantics.

## Current source changes

- `analysis.rs`: registers the new `nullable_record_fields.rs` module. Record literal analysis gives an absent optional callback-record field a concrete empty environment and accepts the corresponding specialized nullable contract.
- `nullable_record_storage.rs`: retains the already accepted local absent-initializer implementation. The newer field helpers were moved out so local annotation handling remains isolated.
- `nullable_record_fields.rs` (untracked; include it in any eventual commit): discovers nested nullable record fields, restores point-specific field types after record carrier wrapping, plans direct field stores, and diagnoses unplanned branch/loop/indexed/borrowed stores and whole-record replacement.
- `record_array_carriers.rs`: nullable contracts consult the canonical record declaration for scoped facts, so empty concrete environments retain the nominal callback contract.
- `record_branch_carriers.rs`: field contracts recover the current concrete nullable field type while maintaining the declaration contract.
- `array_storage_epochs.rs`: includes records with nullable callback fields even without scoped arrays; exports nullable field leaves across ordinary blocks; plans direct field writes; merges point-specific nullable fields with array slot carrier types. A nullable sibling store no longer makes scoped array storage opaque.
- `array_storage_epoch_emit.rs`: direct result replacement resolves flattened record field leaves rather than assuming a root native variable exists. It evaluates the RHS, drops the old leaf, then binds the new environment.

Whole-record replacement is deliberately still a pending implementation requirement. Flattened native records have field locals rather than one root variable. Correct replacement must dispose only live old leaves after evaluating the RHS, respecting already moved fields. Dropping an assembled old record would read moved leaves; shadowing without explicit disposal prolongs capture lifetimes. Do not remove the new diagnostic without implementing and testing live-leaf disposal.

Review the diagnostic guard: it distinguishes declarations from later root stores and allows normal direct field assignments. Indexed stores into deeper container paths may need broader discovery; the current field-path helper traverses records, not arbitrary array/Map/union compositions. These are pending coverage requirements, not proven support.

## Verified evidence for the current work

Before extracting the field module, these four tests passed together:

- `nullable_record_fields_release_independent_capture_owners`: first population, different-origin replacement, independent sibling fields, clearing, exact disposal at O0/O3, ordinary nested block transport and whole-record move.
- `nullable_record_fields_compose_with_scoped_array_storage`: optional field alongside callback array; removing the array entry releases its capture while the optional field remains usable.
- `nullable_record_field_branch_stores_have_source_diagnostics`.
- `nullable_record_whole_replacement_has_a_source_diagnostic_until_drop_planned`.

A separate test passed: `nullable_record_field_replacement_preserves_sibling_capture_loans`. Clearing the left field does not release the right field's capture.

After module extraction, the focused editor test passed:
`nullable_record_fields_preserve_independent_capture_diagnostics` in `crates/tsrust-lsp/tests/intelligence.rs`. It checks the supported source has no diagnostics and a sibling mutation reports `capture-conflict`.

The debug compiler rebuilt successfully after module extraction and benchmark source edits. `node tools/format-project.mjs` then completed successfully. The matched benchmark, registration suite and depth canaries have NOT been run on this diff. Do not report this checkpoint as accepted.

Current concise logs remain under `/private/tmp/tsrust-nullable-fields-*.log`. They include red failures and passing repair checks. Import them into SQLite, read back exact bytes, and remove verified loose copies at checkpoint closure. Some earlier logs contain generated Rust diagnostics, including the repaired block transport and mixed array field failures. Preserve those original failures as evidence.

Accepted HEAD evidence remains in SQLite: `runtime/results/2026-10-09T09-44-51-321Z-verify.json`, nineteen diagnostic logs, five native/Node-after-tsc correctness cases, 125 observations, no timing. Accepted local nullable storage also passed five nullable tests, five comma tests, 31 affected array tests, focused editor diagnostics and eight canaries. These older checks do not prove the current field changes.

## Benchmark changes, not yet verified

The existing `scoped_record_composition` workload now appends two helpers to both matched sources:

- `nullableFieldReplacement(value)`: two independent optional fields, different-origin replacement and clearing. Independent expected result is `5 * value + 18` (23 and 28 for values 1 and 2).
- `nullableFieldArray(value)`: optional field alongside an array, removal, capture mutation and clearing. Expected result is `3 * value + 7` (10 and 13).

The workload should now have 127 observations. Oracle formulas, pinned output tail, source contract snippets, mutation rejection controls and two literal expected rows were updated. Manifest hashes were changed as text before formatting; formatter refreshed the native fingerprint after proving token preservation. No manifest reserialization.

Files: both `benchmarks/runtime/workloads/scoped_record_composition` sources; `scoped-record-composition-oracle.mjs`; `scoped-record-composition-source-contract.mjs`; `scoped-record-composition-registration.test.mjs`; `manifest.json`. Existing runner/profile/report hooks are reused. Verify all of these rather than assuming the edits are correct.

## Next concrete steps

1. Review the current diff for the field planner/emitter invariants above. Add focused ownership witnesses for old destination protection during the RHS, nested field paths, null variants and ordinary callable siblings. Repair defects found.
2. Run affected compiler tests after module extraction. Run registration/source mutation checks. The editor check is already green; rerun only if relevant source changes follow.
3. Run eight depth canaries because shared analyzer record literal and contract dispatch changed.
4. Run the five matched native and Node-after-tsc correctness cases untimed. Keep and poll the same live session if the release build takes time.
5. Finish developer records and guide accuracy; import and byte-verify all closed evidence in SQLite. Whole-project format again before the explicit source checkpoint. Do not commit unrelated fixtures.
6. Before continuing narrow patches, write a bounded remaining B5 acceptance map. Prefer shared storage/lifetime planning that closes several required forms rather than proliferating special cases. Whole-root replacement, branch/loop joins, aggregate composition, function parameters/returns and arbitrary comma finals remain open. Structural record keys are another separate requirement.
7. Run complete regressions, timing/performance review and optimization at the meaningful completed B5 family milestone. Refresh installed consumers then. Do not launch routine heavy regressions merely to close this narrow checkpoint.

## Environment and commands

Use one warm Cargo target and disable incremental compilation:

```sh
export CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target
export CARGO_INCREMENTAL=0
export PATH=/Users/seancotter/.nvm/versions/node/v24.11.1/bin:$PATH
cargo test --offline --locked -p tsrust-compiler --test scoped_record_composition nullable_record_
cargo test --offline --locked -p tsrust-lsp --test intelligence nullable_record_fields_preserve_independent_capture_diagnostics
node --test benchmarks/runtime/scoped-record-composition-registration.test.mjs
node benchmarks/runtime/run.mjs --verify --engines=native,node-tsc --only=scoped_record_composition
TSRUST=$CARGO_TARGET_DIR/debug/tsrust node tools/format-project.mjs
```

The runtime runner builds release itself; `TSRUST` does not override that build. Node 24.11.1 must be first on PATH. Use `/opt/homebrew/bin/python3` for `benchmarks/storage/store.py`. SQLite is `benchmarks/records.sqlite`, ignored by Git. Import runtime records using root `benchmarks` and relative `runtime/results/...` keys; diagnostic logs use root `/private/tmp` and basename keys. Preserve immutable path conflicts under content-addressed keys, not overwritten records.

The eight canaries are:

- closures: `closure_nesting_and_malformed_bodies_are_bounded`
- for_of: `unsupported_iteration_forms_and_deep_loops_have_source_diagnostics`
- owned_arrays: `owned_array_analysis_is_backend_independent_and_depth_bounded`
- nullish_coalescing: `nullish_parser_type_borrow_and_normal_depth_diagnostics_are_precise`
- nullish_assignment: `nullish_assignment_types_loans_spans_and_normal_depth_are_precise`
- higher_order_function_values: `recursion_annotation_depth_and_typed_native_graph_paths_preserve_limits`
- number_operators: `numeric_operator_types_grammar_depth_and_enclosing_loans_remain_checked`
- type_syntax: `type_annotation_nesting_is_bounded_without_a_larger_worker_stack`

Five installed consumers remain at the earlier gate181 checkpoint `976a5fb`; debug scratch compiler has current uncommitted source. Refresh consumers only from a verified checkpoint, using copy to `.refresh-tmp` then atomic rename. Inspect and prune obsolete target/native artifacts at the eventual acceptance handoff; no archives or snapshots. This review handoff deliberately preserves the warm target and active logs for continuation.

## Unrelated fixture edits

These fifteen fixture paths were dirty before this field work. Preserve them, leave them uncommitted, and never stage the whole crates tree:

- array-callback-errors/values.tsr
- array-fields/catalog.tsr
- array-from-collections/documented.tsr
- array-from-collections/values.tsr
- comma-contexts/guards.tsr
- generic-value-adapters/array-parameter-pattern.tsr
- generic-value-adapters/sort-search.tsr
- generic-value-adapters/unused-alias-contracts.tsr
- interfaces/shapes.tsr
- map-for-each/values.tsr
- map-group-by/documented.tsr
- sets/documented.tsr
- set-algebra/precision.tsr
- string-iteration/values.tsr
- type-assertions/assignable.tsr

All are under `crates/tsrust-compiler/tests/fixtures/`.

## Suggested receiving-agent prompt

Continue Tsrust from `docs/handoff-2026-10-09-b5-nullable-fields-review.md`. Read AGENTS.md and docs/development-workflow.md first. HEAD is 680f1205. Current nullable-field work is uncommitted and partially verified; fifteen unrelated fixtures must remain uncommitted. Review the planner/emitter architecture and close the current checkpoint with focused compiler/editor/native checks, eight canaries, matched untimed benchmark support, SQLite evidence and project formatting. Then make a concrete remaining B5 acceptance map and address the missing storage/lifetime forms coherently. Do not treat source rejection guards as completed support or another narrow patch as B5 completion. Rust-style structural record keys are the owner direction; no implicit clone. Keep progress reports tied to completed requirements and remaining requirements. No parallel agent in this tree. Provide a handoff at the next accepted checkpoint.
