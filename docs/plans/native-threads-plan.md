# Native threads: S12

S12 depends on accepted S07 and can proceed independently while S10's raw output
policy remains pending. This plan implements the existing roadmap scope, not an
async runtime or shared-ownership expansion. S12 is not accepted.

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
   partial failure disposal. Decide unjoined-handle policy before exposing it.
3. Add bounded channel endpoints, capacity validation, FIFO semantics, receiver
   closure and unsent-value recovery. Specify endpoint transfer and sharing;
   do not infer permission for implicit shared ownership.
4. Complete compiler diagnostics, catalog help/examples, editor tests and native
   integration. Register matched workloads with independent result oracles and
   explicit differences between Node workers and native threads.
5. At the family milestone, run complete regressions, measure multicore and
   allocation/resource behavior, review performance and refresh consumers.

## Pending public decision

Unjoined handles cannot inherit Rust's detach-on-drop behavior accidentally.
Owner question: require explicit join/detach, automatically join at scope exit,
or detach at scope exit. Recommendation: explicit join/detach with a checker
error otherwise. Automatic join can deadlock with bounded-channel ownership;
silent detach allows work to outlive its caller. No option is implemented or
assumed approved. Native threads cannot be forcibly cancelled safely; any future
cancellation is cooperative and separately specified.

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
Unjoined-handle semantics and subsequent capture/lifecycle checks remain open.
