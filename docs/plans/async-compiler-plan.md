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


## Race/deadline discussion (2026-10-12)

The owner prefers operations on jobs over a standalone race function, and proposed
`await job.race(delay(200))` with cancellation when the timer wins. A dedicated
`job.timeout(200)` was recommended for deadlines, and `job.race(otherJob)` for
competing operations. These are proposals, not approved APIs. The recommendation
was first completion including failure, consumed inputs, and cancellation/cleanup
of the losing job before returning. Same-result-type restrictions, timeout error
spelling and cleanup races remain unspecified. Do not publish the earlier match
variants or standalone race pseudo-code as a supported contract.

## Deadline and combinator direction (2026-10-12)

The owner accepts `.timeout(...)` for deadlines and now prefers imported functions
such as `race([job1, job2])` and `allSettled([job1, job2])` for operations involving
multiple tasks. This supersedes the earlier preference against standalone race.
Keep `.start()` and `.cancel()` as lifecycle methods. These are planned source
APIs, not implemented support; JSON completion remains the first priority.

Recommend `std:futures` rather than `std:promise`, because the approved tasks are
lazy and uniquely owned. The owner suggested both module names; the final module
name has not been separately approved. Familiar combinator names must document
their ownership, scheduling, error and cancellation behavior explicitly.

Recommended initial contracts, pending detailed acceptance:
- `await job.timeout(milliseconds)` yields the job result or reports a timeout,
  cancels timed-out work and waits for resource release. The duration triggers
  cancellation; it is not a hard bound on cleanup time.
- `race([...])` consumes its inputs, returns the first completion (including
  failure), and cancels and drains remaining work.
- `allSettled([...])` consumes its inputs, waits for every outcome without stopping
  on an individual failure, and returns typed success/failure outcomes in input
  order.
- `all([...])` is a possible follow-up, not a requirement inferred from this
  naming decision.

Before implementation, specify cancellation of the combinator itself;
heterogeneous result typing; empty inputs; simultaneous
completion; and timeout argument validation. Preserve single-use ownership and
unused-task diagnostics through combinators. Do not silently introduce eager
execution or shared, repeatedly awaitable results.

## Mixed inputs and strict started-job obligations approved (2026-10-12)

The owner approves accepting both lazy tasks and started jobs in combinators.
Lazy inputs begin together when the combinator is awaited; already started inputs
continue running. Constructing a combinator does not eagerly start its lazy inputs.

The checker must reject abandonment of started work as an error, not a warning.
A started job must be awaited, explicitly cancelled with completion accounted for,
or transferred to an owner that inherits the same settlement obligation. Merely
finishing in the background does not discharge that obligation: its result or
failure still needs handling. The earlier unused-task warning policy applies only
to never-started work without inherited running-job obligations.

This is a static ownership obligation, not a claim that the checker proves eventual
termination. Returning a job transfers responsibility to the caller. Moving it
into a record, collection, closure, timeout or combinator must not erase the
obligation. In particular, constructing and then discarding a lazy combinator that
owns started jobs is an error. No implicit detach or fire-and-forget fallback is
authorized. A cancellation request alone must not count as completed cleanup.

Acceptance must cover discarded expressions, unused bindings, overwrite,
shadowing, conditional branches, loops, early return, throw, aggregate ownership,
function boundaries, double await, use after transfer, and abandoned combinators.
Check diagnostics in both CLI and LSP, including creation and abandonment sites.
Strict build analysis must enforce the rule even when an editor/check suppression
directive hides the diagnostic. Native witnesses must verify that combinator
failure, timeout and parent cancellation cannot silently leave child jobs running.
Cancellation cleanup cannot rely on source finally blocks, which are not promised
to execute. Specify and verify the runtime cleanup mechanism before accepting
S13/S14; do not weaken the error to accommodate backend limitations.
