# Bounded worker channels and explicit task context

Implementation and acceptance are complete at source checkpoint `33703a9e`, with
test guard repairs at `3823c349` and `8c40ffb5`.

The full compiler/LSP pass completed 389 targets: 4,715 passed, two failed and eight
were ignored. Both failures were test integration guards: the recursive clone
fixture expected pre-formatting whitespace, and the catalog witness registry
omitted worker/channel families. The repaired recursive-storage suite passed all
six tests; the repaired catalog witness passed. No runtime assertion was weakened.
The complete pass emitted no Rust warnings. Original failures and affected reruns
are retained in SQLite (`tsrust-channels-family-regression.log`,
`tsrust-channels-recursive-storage-repair.log`, and
`tsrust-channels-catalog-witness-final.log`).

Release CLI and LSP consumers were atomically refreshed and hash-verified. The
active VS Code Tsrust server was restarted; its open channel examples have no
errors. Restarting Rust Analyzer cleared its stale runtime warnings. Installed CLI
examples returned `hello\ngoodbye`, `204`, and `42\ncancelled\n0`.

Focused evidence covers 16 runtime tests, four native channel tests, 18 existing
native thread tests, 64 native declaration tests, three offline package exports,
and affected editor completion, hover, signature, agreement and consistency checks.
The runtime corpus reports zero leaks under macOS `leaks --atExit`. Whole-project
formatting preserves tokens and is idempotent; channel fixtures execute formatted
source. Original attempts and repaired checks are retained in SQLite under
`tsrust-channels-*.log`. These checks do not establish arbitrary protocol deadlock
freedom or a performance claim.

`new Channel<T>(capacity)` owns a single receiving endpoint. Capacity is required,
integer 1..1000000; invalid capacity throws RangeError. `sender()` creates an
explicitly owned, transferable Sender<T>. Create senders before receiving: a receive
with no senders closes the channel permanently. Endpoints are not cloneable. Messages
must satisfy the same owned-transfer classifier as worker captures and results.
Messages cannot contain channel endpoints, including nested record/collection fields,
because an endpoint queued inside its own channel could create an ownership cycle.

`send(value)` waits for space; `trySend(value)` returns full immediately when full.
Both return Result<void, SendError<T>>; every failure preserves the unsent owned
value in `error.value`. Other failure codes are closed and cancelled.
`receive()` waits for a value; `tryReceive()` returns empty immediately when open
but empty. Both return Result<T, ChannelError>; closed and cancelled remain distinct
from a successfully received value. The queue is FIFO; concurrent senders interleave.

Closing a sender affects that sender only. Last-sender close/drop closes admission.
Closing the receiver stops admission but permits draining. Receiver drop disposes
unread values. Closure wakes waiters. Blocking operations register with the current
worker cancellation state; cancellation wakes them without silently detaching.
Cancellation takes precedence when observed before an operation commits a transfer.
Successful transfer before cancellation is not rolled back.

Worker callbacks may take one inferred TaskContext (`task.cancellationRequested()`),
or retain their zero-argument form. Handles expose join, cancel and detach methods;
existing free functions remain compatible. Contexts are not transferable to another
worker. Cancellation flags are specific to each invocation, including pooled work.

Close communicating endpoints in finally before their outer worker handles leave
scope when caller errors can interrupt the protocol. Automatic join alone cannot
finish a receiver while a still-live sender promises more messages.

Blocking channel operations occupy the executing OS worker. Do not fill a pool
with consumers whose producers are queued behind them, or join a producer while
its bounded output is full and nobody receives. This API does not promise arbitrary
deadlock prevention or silently expand the configured pool.

Rayon remains the pool backend. Its owned callback/result/error bridge is in scope
for regression checks, including nested join, cancellation and panic conversion.
General user-authored Rust-module imports and parallel iterator APIs are separate
integration surfaces; using Rayon internally does not establish those APIs.

Examples: `examples/channel-worker` and `examples/channel-batch`. The throughput
workload uses four equal CPU jobs with serial, two-worker and four-worker variants,
plus matched Node worker execution. Construction is outside timing. Retain the
3-process, 8-warmup, 31-sample protocol and independent BigInt checksum oracle.


## Measured throughput

SQLite record `benchmarks/threads/channel-throughput-2026-10-15T00-24-43-676Z.json`
retains three processes and 93 samples per engine/worker count on Apple M3 (eight
logical CPUs). Every checksum is 4791373295. Native median batch times are
3.511500 ms serial, 1.735416 ms with two workers, and 1.052875 ms with four workers:
2.02x and 3.34x speedups. Node-after-tsc medians are 27.922833, 14.795958 and
7.765750 ms: 1.89x and 3.60x speedups. This demonstrates multicore execution for
this CPU workload, not general application speedup. Native versus Node differences
include integer arithmetic and transport implementation costs.

The affected existing tiny-task pool workload was also remeasured with unchanged
source hashes and 93 samples per engine. Record
`benchmarks/threads/pool-join-2026-10-15T00-25-18-930Z.json` has native/Node medians
0.015458/0.099500 ms. Native is 3.9% above the prior 0.014875 ms, while the unchanged
Node reference is 44.6% above its prior 0.068792 ms. This run does not isolate a
compiler regression and does not justify a speculative optimization. Original
records and protocols remain intact.


## Handoff

Exact closed logs and measurement records are imported and byte-verified in SQLite;
`PRAGMA quick_check` passed. Removed 388 completed regression executables (13.31 GiB),
28 generated editor example directories after retaining their nonempty logs, and
disposable native execution caches. Retained the shared Cargo dependency target,
current CLI/LSP binaries and the owner's Rust Analyzer target. Future tests may
relink removed harnesses; cleanup does not invalidate retained evidence.

Final VS Code inspection reports “No problems have been detected in the workspace”
and Tsrust checking enabled. Remaining API limits are explicit above: cancellation
is cooperative, protocol deadlocks remain possible, and general Rust-module imports
and parallel iterators are not implemented by this channel milestone.
