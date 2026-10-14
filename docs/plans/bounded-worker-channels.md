# Bounded worker channels and explicit task context

Focused implementation acceptance passed. The full compiler/LSP regression pass
and final-source throughput measurement are pending.

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
