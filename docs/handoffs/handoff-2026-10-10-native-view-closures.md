# Gate 189 ordinary closure invocation lifetimes

Read AGENTS.md, docs/development-workflow.md, docs/plans/resources-bytes-plan.md and
docs/milestones/native-byte-views-implementation.md first. This checkpoint follows b34bc4ee.
Gate 189 and S07 remain open; public view signatures remain guarded.

native_view_calls::Target distinguishes Function, Closure and Method identities.
Closure summaries map inner parameter binding IDs through checked parameter lists to
actual argument positions. This matters because visible capture slots precede source
parameters. Cache entries are replaced or removed on re-analysis. No per-expression
loan vector or source-span key is introduced.

Ordinary arrow invocation now transports input view origins, including alternative
inputs with a scalar capture. Shared native signatures name closure input/result
lifetimes independently of self. The original native red demonstrated both an invalid
receiver-bound return and a caller returning a reference to its local closure. O0/O3
native pointer checks pass after repair.

Specialized closure method dispatch is connected to the same summary/signature route,
but the full specialization/callback matrix is not yet accepted. Captured native view
returns, receiver lifetimes, arbitrary stored families and abstract callback lifetime
contracts remain pending. capture_context projects captures as parameters and seeding
creates carried environment slots, but argument-only transport rejects returned
capture slots. Native capture storage/emission and outer provenance still need explicit
integration. Keep native_binding_types' public guard.

Checks passed: 48 view tests, nine existing callable signature tests, 89 depth
canaries and 16 native editor tests. No failed or aborted targets. Whole-project
formatting changed zero files and fingerprints.

Exact original red, repairs and check logs are in records.sqlite under
runtime/results/native-view-closures-2026-10-10-checks.json. No timing/full regression.
Installed consumers remain f569190a. Warm compiler rebuilt for project formatting.
Use one Cargo target with CARGO_INCREMENTAL=0, Node 24.11.1 first on PATH and
/opt/homebrew/bin/python3 for store.py. One agent/tree.

## Continuation prompt

Continue gate 189 from docs/handoffs/handoff-2026-10-10-native-view-closures.md. Complete native
capture/receiver lifetime and provenance transport, specialized callbacks and stored
callable families. Add safe source/native witnesses, then finish collection/control-flow
loan transport and real factories. Preserve actual Rust slices without copies or hidden
owners. Keep public factories guarded until the complete required matrix agrees.
Finish editor/docs and matched benchmark support before gate acceptance. Full
regressions and timing belong to S07 completion.
