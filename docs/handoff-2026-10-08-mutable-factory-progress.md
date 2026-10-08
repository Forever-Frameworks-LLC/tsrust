# Mutable factory implementation checkpoint

This is a source checkpoint before further array-flow changes, not feature-family acceptance. The callable replacement plan remains active. Read AGENTS.md, docs/development-workflow.md and docs/callable-replacement-plan.md before continuing.

## Implemented and verified

Sequential owned factories retain point-sensitive capture state. A capture moved out and restored at every checked completion can keep reusable invocation without adding a callable variant or cloning its native payload. Restoration remains conservative for implicit failure operations. Optional native capture slots support both reusable and consuming outer environments, including disposal witnesses at optimization levels 0 and 3.

Producing declaration paths distinguish factories created through a helper at different caller sites. Recursive declaration re-entry collapses the path instead of growing with recursion depth. Concrete owned callbacks can be returned through ordinary callable storage. Nested generic results from the same declaration retain independent construction origins while explicitly sharing registered native storage. Compatibility metadata survives origin compaction.

A dedicated array-flow module preserves exact local element provenance through push, unshift, pop and shift. The straight-line retained factory witness transfers two callbacks out of an array and observes independently selected methods with output `1 2 9`. Its editor signature help exposes the selected two-argument method.

Latest bounded evidence: 46 capture-effect tests pass with one retained loop witness excluded; 81 generic-call controls pass; 103 callable controls passed before the subsequent array-flow addition. The added editor witnesses pass individually. All eight depth canaries were rebuilt and pass on the checkpoint source. Workspace all-target checking reports no warnings. Whole-project formatting changes zero files and zero fingerprints. No complete compiler/LSP regression or timing was run for this checkpoint.

## Remaining requirement

`loop_factory_storage_keeps_independent_mutable_environments` remains red. Factories created at a repeated loop call site are retained in an array, transferred by shift and mutated independently. The later two-argument callback invocation incorrectly receives a one-argument contract.

Exact element joins currently lose shape when lengths differ. Add a bounded sequence or region model that retains provenance through loop joins without growing element vectors indefinitely. Distinguish retained allocation multiplicity from completed iteration-local instances. An extraction must retain the captured state of the transferred runtime instance; rebuilding it blindly from initial allocation records would be unsound after mutation. Direct indexed callbacks are readonly views, so this witness deliberately transfers ownership with shift. No implicit clone is permitted.

Complete failure, recursive lifetime and retained-instance controls; update matched benchmarks, editor behavior and the guide; format the whole project; store and verify evidence; then close the focused gate. Broader family regression and performance review remain separate requirements. Do not describe this checkpoint as acceptance.

## Evidence and integration

SQLite holds exact original logs for mutable factory, restored capture, producing context, nested storage/result families and array environment flow. Closed remaining loose records were imported and read back before removal. The thirteen-observation workload's five native and Node-after-tsc cases are preserved in `runtime/results/2026-10-08T06-09-41-487Z-verify.json`; this is untimed verification from before the subsequent aggregate changes and must be rerun when benchmark support is updated. Earlier observation protocols remain immutable.

Live executable consumers remain on standard-library source `31e367f`; this checkpoint does not refresh them. The parallel path.relative worker must obtain exclusive main ownership before integrating. Main ownership remains with this agent. Fifteen protected owner fixture edits stay uncommitted and must never be staged. Use explicit Git paths.

## Continue prompt

Continue Tsrust from docs/handoff-2026-10-08-mutable-factory-progress.md. Read AGENTS.md, docs/development-workflow.md and docs/callable-replacement-plan.md first. Finish retained mutable factory state through bounded array joins, ownership transfer and distinct runtime instances. Keep the existing red loop witness and all restoration, disposal, nested result and editor contracts. Use one warm Cargo target, disable incremental compilation, use pinned Node and Python for evidence storage, and preserve all fifteen owner fixture edits. This checkpoint is implementation progress, not acceptance. Finish benchmark/guide/evidence support and depth checks before closing the gate; perform broad regressions and measurements at the completed family milestone. Coordinate exclusive main ownership before any parallel-worker integration.
