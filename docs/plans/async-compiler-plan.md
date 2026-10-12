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

## Proposed source contract for review

Recommend `async function work(...): Task<T>` with a uniquely owned, lazy result.
Calling the function evaluates arguments and constructs the task; its body starts
when awaited or explicitly spawned. This does not promise JavaScript Promise
eager execution, shared settlement or repeated awaiting. Use Task rather than
Promise so the spelling signals that distinction.

Await consumes the task once and yields T. Body throws propagate at the await
site through the existing checked error model; argument-evaluation errors still
occur at the call site. Tasks cannot be cloned or implicitly shared. Ownership
transfers and retained parameter borrows happen when constructing the task, with
real lifetimes retained until completion or disposal. Local await does not impose
thread-transfer capability; a spawn boundary does. An async main is driven by
the selected runtime, while synchronous main keeps the synchronous build path.

An unpolled task can release its owned inputs without running its body. Diagnostics
must make accidentally unused tasks visible. This proposal does not settle
cancellation of a running task or finally execution during cancellation; those
remain explicit S14 lifecycle decisions. Ordinary completion, return and throw
must retain existing finally behavior in S13. Backend future drop is not a
substitute for that source control-flow implementation.

The decision requested before parser/type integration is the lazy, single-use
Task surface. This section records a recommendation, not owner approval or
implemented source support. Runtime/library APIs and cancellation must not be
inferred as accepted from approval of the Task spelling/execution contract alone.


## Owner decisions (2026-10-12)

Approved: lazy, uniquely owned Task<T>; await consumes the result once; concurrency
requires an explicit scheduling operation. Arguments transfer and retained loans
begin at task construction. Ordinary calls do not start the async body.

Cancellation does not promise execution of source finally blocks. Owned resources
must still be released exactly once. Ordinary completion and throwing retain
normal finally behavior. Cancellation triggers, propagation and scheduling races
remain S14 implementation work; this decision does not permit leaked resources.

Unused tasks produce warnings in the checker and LSP. Track task ownership through
control flow: warn when an unstarted task is discarded, overwritten or leaves
scope on a reachable path without being awaited, returned or transferred to an
operation that takes responsibility for it. Returning a task or storing it in a
live owner is not itself an unused-task error. Arbitrary whole-program reachability
is not promised. Diagnostics should identify creation and abandonment sites and
explain conditional paths where practical. Never auto-start tasks to silence a
warning. Tests must cover branches, loops, early return, throw, moves and shadowing.

The owner approves explicit scheduling but questions the name spawn for async
work. Proposed spelling: task.start(); final spelling remains open. Do not freeze
a public spawn function for tasks. Native thread/process naming remains separate.
JSON completion has priority before this source integration. These decisions
supersede the proposal's pending execution/cancellation statements above.


## Scheduling names approved (2026-10-12)

The owner approved `.start()` and `.cancel()` naming. `.start()` schedules a lazy
task explicitly. `.cancel()` is the cancellation operation on running work;
the proposed awaited completion, completed-job races and result/error contract
still require specification. Do not infer approval of the illustrative race/timer
APIs from the example. Keep ordinary try/catch as the initial error-handling
recommendation; Promise-style `.catch()` chaining is not part of this approval.
