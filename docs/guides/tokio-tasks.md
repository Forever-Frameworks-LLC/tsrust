# Tokio tasks

An async function still returns a lazy, uniquely owned `Task<T>`. Calling it
evaluates arguments and transfers inputs; `await` runs it locally. `task.start()`
consumes it and schedules owned work on Tokio's worker pool. The resulting
`Job<T, E>` must be awaited or transferred to an owner that settles it. A local
borrowed task can be awaited, but cannot be started across workers.

```tsrust
import { delay } from 'std:futures';

async function answer(): Task<int> {
  await delay(10);
  return 42;
}

async function main(): Task<void> {
  const job = answer().start();
  console.log(await job); // 42
}
```

`delay(milliseconds)` and `yield()` return lazy tasks. Timers accept integer
milliseconds from 0 through 31536000000. `task.timeout(milliseconds)` creates a
lazy deadline wrapper; the clock starts when the wrapper is awaited. Started
jobs continue running while their deadline wrapper waits to be awaited.

`race([first(), second().start()])` starts lazy inputs when awaited, returns the
first completion (including failure), and cancels and drains the remaining work.
Ready ties prefer input order. `allSettled(...)` waits for every outcome, preserving
input order in an array of Results. Inputs must produce the same result type.
Mixed Task/Job inputs use an array literal at the group call; stored arrays hold
one input type. Both groups consume their inputs. A group holding started work must itself settle.

`job.cancel()` requests cancellation and returns whether this request won before
completion. It does not settle the job: await afterward, usually in try/catch.
A successful cancellation does not forcibly interrupt synchronous code. Keep CPU
and blocking work on dedicated threads, `ThreadPool`, or the existing Rust/Rayon
bridges. There is no implicit conversion between these worker handles and Jobs.

Awaiting a Job, deadline, race, timer or channel operation can throw `AsyncError`.
Its `code` is `worker`, `cancelled`, `panic`, `timeout`, `range`, `empty`, or `closed`.
Worker failures preserve their original typed payload in `cause`; otherwise cause
is undefined. Directly awaiting a source Task preserves its original throw type.

Cancellation releases owned resources and waits for nested child cleanup before
reporting completion. It does not execute arbitrary source `finally` statements
after a suspended await. Ordinary completion, return and throw retain normal
`finally` behavior. Cleanup can exceed a requested timeout.

Import `Channel` and `Sender` from `std:futures` for asynchronous messages.
`new Channel<T>(capacity)` permits 1..1000000 queued messages. Full sends and empty
receives suspend instead of blocking a worker. `sender()` creates an independently
owned endpoint to move into a task. `send(value)` eagerly transfers the message
into a lazy Task; its endpoint remains borrowed until completion or disposal.
`receive()` similarly retains a channel borrow. `close()` rejects new sends while
allowing queued messages to drain; an empty closed channel throws `closed`.
Messages cannot contain worker handles or channel endpoints.

Examples: [timers](../../examples/tokio-tasks/timers.tsr),
[groups](../../examples/tokio-tasks/groups.tsr),
[cancellation](../../examples/tokio-tasks/cancellation.tsr), and
[channels](../../examples/tokio-tasks/channels.tsr).

`await yield()` gives the scheduler a chance to run other ready tasks, then
resumes without a fixed delay or guaranteed ordering. Ordinary async I/O already
yields while waiting; explicit yielding is useful in otherwise uninterrupted loops.

Started-job responsibility also follows record fields, arrays and closures.
Cancelling a job or wrapping it in an unawaited group does not discharge that
responsibility. Job-bearing array `splice` is rejected until its removed and
replacement owners can be tracked separately; transfer the whole array to a group.
