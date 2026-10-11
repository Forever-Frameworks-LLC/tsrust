# Gate 189 source call origin transport

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md and
docs/native-byte-views-implementation.md first. This checkpoint follows 38ac7505.
Gate 189 and S07 remain open; source view exports remain guarded.

native_view_calls retains checked function return summaries in each context, keyed
by checked function identity. It transports formal carried slots to actual argument
origins through the existing iterative value walker. It preserves selected fields,
alternative inputs, nested calls and sibling output paths. Storage dependencies retain
the actual source place plus inherited origins. Unchanged summaries are reused; changed
or absent summaries replace or remove previous entries.

Direct calls and saved named functions register their checked summary. A saved named
function adapter now uses the target's named result lifetime; its old anonymous result
incorrectly bound to self. Generated native witnesses verify both forwarding forms at
O0 and O3, preserving pointer identity. No extra per-expression field was added.

Recursive view-returning calls without stabilized summaries and returned hidden capture
slots are diagnosed. Arbitrary stored callable families, callback parameters, closure
methods/captured returns, collection transport, recursive storage and completion
reconciliation remain pending. Native factory/storage-return transport still needs
end-to-end witnesses. Do not remove native_binding_types' borrowed-signature guard.

Checks passed: 46 view tests, nine callable-signature tests, 89 depth canaries and
16 native editor tests. Formatting changed zero files and fingerprints.

The earlier returns checkpoint undercounted editor checks: its original log proves
16, not 14. Its total is 136, not 134. Current evidence records the correction without
overwriting that immutable record.

Original red, adapter failure, repair and check logs are in records.sqlite under
runtime/results/native-view-calls-2026-10-10-checks.json. No timing or whole-crate run.
Installed consumers remain f569190a. Warm compiler rebuilt for project formatting.
Use the single Cargo target with CARGO_INCREMENTAL=0, Node 24.11.1 first on PATH and
/opt/homebrew/bin/python3 for store.py. One agent/tree.

## Continuation prompt

Continue gate 189 from docs/handoff-2026-10-10-native-view-calls.md. Complete stored
callable-family and callback/closure lifetime contracts, then collection and control-flow
loan transport. Add real factory/storage-return source/native witnesses before enabling
public view APIs. Preserve genuine Rust slices without copies or hidden owners. Finish
the complete editor/docs and matched benchmark support before gate acceptance. Full
regressions and timing belong to S07 completion.
