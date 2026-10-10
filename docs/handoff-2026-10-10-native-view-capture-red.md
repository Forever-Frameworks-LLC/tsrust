# Native view capture witness

Continue from `docs/handoff-2026-10-10-native-view-closures.md` at commit
`186916cf`. Gate 189 remains open. No compiler implementation changed in this
continuation.

An uncommitted focused test in `crates/tsrust-compiler/src/native_view_emit.rs`,
`captured_native_view_return_preserves_outer_origin`, formats its source twice,
injects the private ByteView identity and checks a closure that returns its captured
input. The test fails with `native-view-call: Returned closure captures require
capture lifetime transport.` One test ran and failed; an earlier invocation with
an incomplete exact filter ran zero tests and supplies no verification evidence.

The failure occurs in `native_view_calls::remember_native_closure_returns`:
capture binding IDs are searched only in the invocation parameter list. Fixing
this requires a distinct capture source route through the caller's environment,
not interpreting a capture ID as an argument position. The loan value walker
currently accepts only actual arguments for returned summaries.

The backend must also preserve capture storage lifetimes. Its closure signature
planner currently introduces function lifetime parameters for returned capture
roots without connecting those parameters to the closure environment or receiver.
Do not remove the analyzer rejection without a generated Rust pointer-identity
test and a negative escape control. Cover read-only, exclusive and consuming
capture behavior under the existing ownership model. No copying or shared owner
fallback is authorized.

SQLite contains the concise red record at
`runtime/results/native-view-capture-red-2026-10-10.json`, imported and read back
byte for byte. The loose copy was removed. No full regressions, timing, consumer
refresh or source checkpoint was performed. The new witness is intentionally red
and remains uncommitted for implementation next.

## Continuation prompt

Read AGENTS.md, docs/development-workflow.md, the closures handoff and this note.
Implement captured native view return provenance and closure environment lifetime
emission together. Start with the retained red witness, then add generated native
O0/O3 pointer identity and rejected escape cases. Keep public factories guarded
until the required matrix is complete. Follow focused checks, whole-project
formatting and SQLite evidence before committing the implementation checkpoint.
