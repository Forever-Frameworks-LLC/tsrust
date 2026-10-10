# Gate 189 private native view storage checkpoint

Read AGENTS.md, docs/development-workflow.md, docs/resources-bytes-plan.md and
docs/native-byte-views-implementation.md first. This checkpoint follows 6ca0cb11.
Gate 189 and S07 remain open. No public byte view factory is implemented yet.

## Implemented

Private native view records emit safe Rust slice lifetimes. Nonrecursive wrappers,
arrays and Result payloads retain the actual slice types through generic positions.
Readonly Result argument emission supplies every required data argument. Native view
storage is distinct from callback-only record borrowing bridges. Small-record layout
selection excludes native view storage.

The isolated native_view_emit module owns a cached reverse reachability index and
private AST/native witnesses. Its graph handles cyclic type dependencies and nested
arrays without recursive traversal. Programs without views take an empty-index path.
The executable witness preserves source pointer identity and mutable write-through
at O0 and O3. It also compiles a readonly Result parameter and checks formatted-source
idempotence before constructing private native identities.

## Remaining work

Keep native_binding_types' borrowed-signature guard. Do not publish view/viewMut
before source loans and the backend agree. Native factories must retain source roots
through Result success, moves, branches, returns and stored function values. Err paths
must release success loans. Infer precise parameter lifetime relationships; reject
escaping local owners, consuming parameters and dangling temporaries. Check conflicts,
reborrows and all control-flow exits. Recursive wrappers and callback/view composition
remain unverified and require implementation before their source forms are enabled.

Then publish readable/mutable byte contracts, full IntelliSense, source examples and
matched view benchmark support. Run normal-stack canaries after analyzer dispatch
changes. Full regressions and timing remain the completed S07 family milestone.

## Evidence

Focused outcomes and original failures are retained in benchmarks/records.sqlite at
runtime/results/native-view-storage-2026-10-10-checks.json. This includes the first
unsupported export test setup, the intended empty-representation failure, generated
Rust failures in callback bridge routing, and the recursive-layout test setup repair.
Final counts and command output belong to that record; this is not a whole-crate
regression claim. No timing ran. Five installed consumers remain at f569190a because
this checkpoint has no published source behavior.

## Continuation prompt

Continue Tsrust gate 189 from docs/handoff-2026-10-10-native-view-storage.md. Read
AGENTS.md and docs/development-workflow.md first. Implement source loan provenance
and precise function lifetime relationships for genuine byte views. Preserve real
slice storage, no implicit copy or hidden shared owner. Keep source exports guarded
until analyzer and emitter agree. Use focused TDD, editor parity and ready matched
benchmark support; gate 189 is not accepted and S07 remains unmeasured.
