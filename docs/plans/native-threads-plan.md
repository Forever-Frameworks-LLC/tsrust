# Native threads: S12

S12 depends on accepted S07; S10 and S11 are also accepted. This plan implements
the existing roadmap scope, not an async runtime or shared-ownership expansion. S12 is not accepted.

## Current backend evidence

`tests/native_threads_backend.rs` proves that an owned byte vector can move into
a distinct OS thread and back through join while preserving its allocation.
Pointer addresses are test observations only, not source handles or lifetime
representations. A bounded channel refuses overflow and returns the unsent owned
value; receiving preserves the original allocation, and sender drop reaches EOF.
A worker panic arrives as join failure. Rust rejects a local borrowed capture and
an Rc capture. Two focused tests pass, with three handwritten Rust compilations.
This does not prove Tsrust source checking, public panic conversion, multicore
speedup, cleanup policy or a stable public API.

SQLite evidence: `tsrust-thread-backend-5c033a7e.log`. No timing measurements.

## Implementation order

1. Infer transfer eligibility from owned capture/result types, using existing
   native thread_transfer metadata and recursively inspecting records, arrays,
   unions and collection payloads. Reject borrowed views and unsupported resource
   handles before emission. Callback captures must transfer explicitly; no hidden
   clone, Arc/Mutex wrapper or erased lifetime.
2. Add typed spawn and consuming join with Result-based spawn/join failures.
   Verify returned values, moved captures, callback throws, backend panics and
   partial failure disposal. Enforce the settled explicit join/detach policy
   before exposing it.
3. Add bounded channel endpoints, capacity validation, FIFO semantics, receiver
   closure and unsent-value recovery. Specify endpoint transfer and sharing;
   do not infer permission for implicit shared ownership.
4. Complete compiler diagnostics, catalog help/examples, editor tests and native
   integration. Register matched workloads with independent result oracles and
   explicit differences between Node workers and native threads.
5. At the family milestone, run complete regressions, measure multicore and
   allocation/resource behavior, review performance and refresh consumers.

## Settled lifecycle decision (updated 2026-10-14)

The owner approved explicit join or detach, with a checker error for abandoned
handles. Do not reopen this as an undecided three-way choice. Automatic join can
deadlock with bounded channels, and silent detach permits work to outlive callers.
Public lifecycle enforcement remains to be implemented. Native threads cannot be
forcibly cancelled safely; future cancellation is cooperative and separately specified.

## Owned-data transfer classifier

`analysis/thread_transfer.rs` now provides a finite graph walk over checked data
types. Owned scalars, arrays, maps, sets and recursive records inspect every
reachable payload. Native values require explicit thread_transfer capability and
must not be borrowed representations. Missing field/type facts reject rather
than silently permitting transfer. Borrowed unions and all callable forms reject
at this boundary; spawn's separate capture analysis remains to be implemented.
The classifier does not grant arbitrary closures Send-like capability.

Two focused tests cover seven checked source shapes: recursive owned arrays,
map payloads and owned Bytes pass; nested ByteView/JsonRaw, File arrays and stored
callbacks reject. Tests load standard-library imports through the real project
loader. The initial missing implementation and a single-file test setup failure
remain preserved. SQLite evidence: `tsrust-thread-transfer-*-dc78ffdd`.
Whole-project formatting changes no files or fingerprints.

This internal helper is not yet connected to a public spawn/channel API. There
is no source feature acceptance, timing result or consumer refresh for this step.
The explicit join/detach policy is settled; public capture/lifecycle checks remain open.

## Capture storage eligibility

The separate `require_owned_captures` helper checks actual checked environment
storage before inspecting payload types. Consumed storage and genuinely copied
values qualify; read and mutable caller loans reject even when the payload is
an integer. Invocation access does not turn an owned environment into a caller
loan. Conversely, moving a record does not make a stored callback transferable.
The helper returns the offending capture index for future source diagnostics.
It does not change capture inference or insert copies.

Five formatted-source cases cover moved strings, borrowed strings, copied
integers, mutably borrowed integers and moved records containing callbacks.
All three transfer tests pass. SQLite evidence:
`tsrust-thread-capture-{red,check,final,format}-de981df0.log`.
This remains internal groundwork; public spawn/join, result checking, lifecycle
checking and native source execution still require implementation and acceptance.

## Generated closure backend checks

`native_threads_backend` now tests compiler-emitted closure environments and
invocation bodies, with only the thread boundary supplied by a Rust harness.
Three successful source cases share one native compilation: consuming an owned
string returns the same allocation; reading an owned environment repeatedly
returns the expected values; throwing an owned string crosses join as the
generated checked error, with its allocation preserved and no backend panic.
Inputs use runtime-owned strings rather than static literals. The worker thread
identity differs from its caller. Two negative compilations reject generated
read-borrowed string and mutable-borrowed integer environments with E0597.

All four backend tests pass, including the prior raw backend controls. Every
Tsrust source undergoes formatting, idempotence checking and normal analysis
before emission. SQLite evidence: `tsrust-thread-generated-a80cc17a.log` and
`tsrust-thread-generated-format-a80cc17a.log`. This closes the emitted-closure
feasibility question without exposing a source thread API or deciding unjoined
handle semantics. Multicore throughput remains unmeasured.

## Completion transfer eligibility

`require_owned_completion` applies the same owned-data rules to the worker's
result and every explicit thrown payload. It also checks the materialized runtime
error type when the checked error contract includes runtime failures. Opaque
error effects or missing runtime error types reject until resolved; an unknown
contract is never treated as an empty error set. Failures distinguish returned
data, thrown data and unresolved error contracts for subsequent diagnostics.

Five formatted-source cases exercise owned string returns, string throws,
runtime index failures, returned callbacks and thrown callbacks. Separate
controls reject opaque and missing-runtime contracts. These checks concern
transfer eligibility only; they do not decide how public join exposes failure.
The focused transfer suite has five passing tests. SQLite evidence:
`tsrust-thread-completion-{red,check,final,format}-d4facb53.log`.
Public source integration and lifecycle acceptance remain outstanding.


## Owner decision (2026-10-12)

Approved: every native thread handle requires explicit join or detach. The checker must reject abandoned handles; do not inherit automatic detach-on-drop.

## Rust-module integration design dependency

Implement the design review for the owner-approved [local Rust module direction](../interop-and-targets.md#local-rust-modules-and-native-workers-owner-discussion)
before freezing the S12 public worker interface. Importing Rust and running work on
a thread are separate capabilities. The owner approved automatic default pool management for threaded interop and
supported package wrappers. Exact syntax remains pending; arbitrary Rust crate
compatibility is not assumed. Include pool integration and its documented
acceptance cases in S12 planning, with Rayon evaluated for CPU work. Preserve the approved explicit
join/detach contract and ensure a future native callable can use the same checked
transfer boundary without payload serialization.


## Public owned-worker slice (2026-10-14)

The earlier public-integration pending statements above are historical. The
`std:threads` spawn/join/detach slice now connects the transfer classifier, concrete
callback lowering, typed Result completion and explicit lifecycle checks. See
[owned worker acceptance](owned-thread-acceptance.md) for current acceptance and
storage/control-flow limits. Bounded channels and Rust/pool integration remain open.
Direct OS-thread spawning does not select or freeze a future pool contract.

The [default cleanup update](default-worker-cleanup.md) makes fallback joining part
of `spawn` and exposes global `unwrap`. Moves transfer cleanup responsibility.
The older scoped spellings remain compatibility forms.

## Owner clarification (2026-10-14)

Configurable default thread pools must be available as a standard-library option,
not only as an interop implementation detail. Evaluate Rayon for CPU work and
compatible pool reuse. Keep dedicated OS-thread spawning available. The scoped
cleanup pass does not implement or settle pool sizing, queue admission, nested
join scheduling or shutdown; those remain required S12 follow-up work. No measured
pool benefit is claimed before matched workload acceptance.

## Owner API direction (2026-10-14, pending implementation)

Use one public `spawn` with automatic joining when its owned handle leaves scope,
including return and error exits. Explicit `join` observes the result; `detach`
opts out of waiting. Remove the need to choose `spawnScoped`. Make `unwrap` a global
operation like `clone`; preserve typed error propagation and ownership checks.

Prioritize a configurable global default pool. A completed but unjoined task holds
an outstanding-task slot, not a worker thread. Workers become reusable when work
finishes. Bound outstanding tasks separately from worker count, report capacity
exhaustion through `spawn` rather than blocking admission, and expose queued, running
and completed-but-unjoined task counts. Define detach accounting and shutdown,
including nested worker joins, before implementation acceptance. Evaluate Rayon
against these requirements. This is required follow-up, not an accepted API yet.

Current round prioritizes value-producing matches, literal patterns and named
record variants with ownership, runtime and editor acceptance. Thread API
simplification follows; pool work remains high priority if it cannot fit this round.

Owner addition (2026-10-14): cancellation is required for both dedicated and pooled
workers. Rayon bridging and dedicated-worker channels are also required S12 work.
See [cancellation and integration requirements](default-worker-cleanup.md#additional-owner-requirements-cancellation-rayon-and-channels).
These requirements remain pending implementation and must not disappear behind the
accepted match/default-cleanup slice. Cancellation must preserve ownership cleanup
and distinguish a request from acknowledged completion.
