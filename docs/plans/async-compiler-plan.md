# Async compiler: S13

S13 depends on accepted S06 and S07. It remains separate from S14's Tokio runtime
and from the proposed desktop integration. The older async vision document is
research context, not an approved scheduling or cancellation contract.

## Backend evidence

`tests/native_async_backend.rs` compiles bounded, dependency-free Rust controls
with the pinned backend. A future borrows its own owned string across a real
Pending/Ready transition, retaining the string allocation on both successful
and error completion. Its owned resource drops exactly once on completion,
failure, cancellation before the first poll and cancellation while suspended.
A counting waker verifies wake registration; no sleeps or timing assumptions
drive the checks. Arc appears only in test counters and the waker, not as a
proposed source ownership representation.

Two negative native compilations reject destroying an owner while a future
retains its borrow and transferring a future containing suspended Rc state to
a Send boundary. Suspend safety and cross-thread transfer are separate checks:
this does not justify imposing Send on all local futures.

Two focused tests pass. SQLite evidence: `tsrust-async-backend-b0c021e3.log` and
`tsrust-async-format-b0c021e3.log`. These are handwritten backend controls, not
source async support, generated-future acceptance or performance measurements.

## Remaining implementation and acceptance

1. Settle the task-result spelling and execution contract before publishing the
   syntax. Familiar Promise spelling must not silently imply JavaScript's eager
   execution or event-loop behavior. Define when an async body starts, which
   operations consume the result, and how errors reach await.
2. Add parsed and checked async functions, await expressions and task-result
   types together. Preserve source spans, project imports and optional main
   selection. Reject await outside the supported async context with useful
   diagnostics rather than deferring failure to the native compiler.
3. Carry capture ownership and source provenance through suspension. Retain real
   lifetimes for views. Prevent owner mutation, transfer or disposal while a
   suspended future retains a loan. Check Send-like constraints at a thread/task
   transfer boundary, separately from ordinary local await.
4. Lower checked bodies to native futures with correct return, throw, catch,
   finally, break and continue behavior. Specify cancellation's interaction with
   finally before exposing it: dropping a Rust future frees owned state but does
   not execute arbitrary source statements after a suspended await. The backend
   resource-drop witness must not be presented as proof of source finally.
5. Complete async entry execution, diagnostics, IntelliSense, executable examples
   and source/native regressions. Keep synchronous programs independent of async
   dependencies. Runtime bootstrap and task scheduling belong to S14.
6. Register matched workloads and independent result checks during implementation.
   At the family milestone, run complete regressions, measure affected workloads,
   review performance and refresh installed consumers. Do not claim backend
   feasibility as S13 acceptance.
