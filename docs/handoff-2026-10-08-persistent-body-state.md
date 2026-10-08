# Handoff: persistent body state extraction accepted

Source checkpoint 3a263b5 extracts generic_call_body_state.rs. The module retains graph, per-node partitions and completion values, duplicate-suppressed node queue and accumulated call facts. The ordinary one-shot evaluator delegates to it. A test-only original evaluator remains the reference; branch, loop and try/catch/finally outcomes match, and a second idle drive returns the same result.

Extraction acceptance: 95 generic-flow controls pass, with the explicit pending nested replay witness excluded and the full diagnostic ignored. All eight depth canaries, 23 affected native contracts and 57 checked editor projects pass. Three representative emitted programs remain byte-identical to the previous accepted compiler. No full matrix or new runtime timing is claimed for this isolated refactor.

Five consumers are atomically refreshed from 3a263b5 and pass combined CLI/native/LSP smoke checks for aggregate/operator returns, caught assertions, path/OS imports and independent callable state. Exact evidence uses tsrust-body-state-* identities in SQLite. Closed records are imported and read back byte for byte before loose removal; closed test executables are pruned. Fifteen protected owner fixture changes remain untouched and uncommitted.

## Next gate

Follow docs/resumable-callable-summaries-plan.md gate 3. Engine entries still recreate body state; no cross-call resumption exists yet. Record parent request plus graph node/partition dependency coordinates, save each node's pre-evaluation input and requeue only affected nodes when callee summaries grow. Multiple calls inside one expression can replay that expression but must not replay earlier statements. Preserve parameter-entry ordering, capture/version invalidation, exceptions/finally and recursive fixpoints through subsequent gates. Do not describe the general nested visit budget or larger compilation bottleneck as solved.

Read AGENTS.md and docs/development-workflow.md first. Keep the pending nested independent-call witness red until its behavior is implemented. Use explicit Git paths, pinned Node/Python, one Cargo target and CARGO_INCREMENTAL=0. Parallel std:path.relative integration requires exclusive main ownership and latest-checkpoint reconciliation, including the intentional pending witness. Give a physical handoff at the next checkpoint.
