---
title: Schedule tasks and exchange messages
description: Use timers, task groups, cancellation and bounded async channels.
---

Use `std:futures` to coordinate asynchronous work. First read
[async task ownership](../../language/async/) and
[prepare native build tools](../../tools/rust-setup/). Each example below is a
complete replacement for `main.tsr`; run it with `tsrust run main.tsr`.

## Start work on the runtime

Calling an async function creates a lazy `Task<T>`. Await runs it locally.
`.start()` consumes an owned task and schedules it on the runtime worker pool,
returning a `Job<T, E>`. Await or transfer that job to an owner that settles it.
A task borrowing local data can be awaited locally, but cannot cross workers.

```tsrust
import { delay } from 'std:futures';

async function answer(): Task<int> {
  await delay(1);
  return 42;
}

async function main(): Task<void> {
  const job = answer().start();
  const result = await job;
  console.log(result);
}
```

This prints `42`. There is no implicit conversion between jobs and dedicated
thread, `ThreadPool` or Rust/Rayon worker handles.

## Timers and deadlines

- `delay(milliseconds)` returns a lazy task that waits for the duration and returns
  no value.
- `yield()` returns a lazy task that lets other ready work run, without a fixed
  delay or guaranteed ordering.
- `task.timeout(milliseconds)` consumes the task and returns a lazy deadline
  wrapper. The clock starts when the wrapper is awaited.

Durations are integers from `0` through `31536000000` milliseconds, inclusive.
An out-of-range duration throws `AsyncError` with code `range` when awaited.
A deadline that expires throws code `timeout`. Cleanup can extend the time before
that error is returned, so a timeout is not a strict upper bound on elapsed time.
A started job continues running even if its deadline wrapper has not been awaited.

Ordinary asynchronous I/O already yields while waiting. Explicit `yield()` is
useful in otherwise uninterrupted loops. It does not make blocking work safe on
runtime workers; use dedicated threads or the existing blocking-work APIs.

## Wait for a group

`allSettled` consumes its inputs and waits for every outcome. It returns an array
of `Result` values in input order, including failures without stopping at the first
one. An empty group returns an empty array.

```tsrust
import { allSettled, delay } from 'std:futures';

async function calculate(value: int): Task<int> {
  await delay(1);
  return value * 2;
}

async function main(): Task<void> {
  const outcomes = await allSettled([calculate(3), calculate(5).start()]);
  for (const outcome of outcomes) {
    console.log(unwrap(clone(outcome)));
  }
}
```

This prints `6`, then `10`, regardless of completion order. `unwrap` extracts a
successful result or throws its error; this example deliberately propagates errors.

`race` also consumes its inputs, but returns the first completion, including a
failure. It cancels and drains remaining work before returning. Ready ties prefer
input order. An empty race throws code `empty`.

Both operations start lazy inputs when the group is awaited. All inputs must
produce the same result type. Mixed Task/Job inputs use an array literal at the
group call; a stored array contains one input type. A group holding started work
must itself settle. Wrapping a job in an unawaited group does not discharge it.

## Cancel and settle a job

`job.cancel()` requests cancellation and returns whether this request won before
completion. It does not settle the job: await afterward, normally in `try`/`catch`.

```tsrust
import { delay } from 'std:futures';

async function waitForWork(): Task<void> {
  await delay(60000);
}

async function main(): Task<void> {
  const job = waitForWork().start();
  job.cancel();
  try {
    await job;
  } catch (error) {
    console.log(error.code);
  }
}
```

This prints `cancelled`. Cancellation does not forcibly interrupt synchronous
code. It releases owned resources and waits for nested child cleanup before
completion. It does not execute arbitrary source `finally` statements after a
suspended await. Ordinary completion, return and throw keep normal `finally`
behavior.

Started-job responsibility follows record fields, arrays and closures. Job-bearing
array `splice` is rejected because removed and replacement owners are not tracked
separately; transfer the whole array to a group instead.

## Exchange messages through a channel

`new Channel<T>(capacity)` creates a queue with capacity from `1` through `1000000`.
Full sends and empty receives suspend instead of blocking a runtime worker.
`sender()` returns an independently owned endpoint suitable for transfer to a task.

```tsrust
import { Channel, Sender, allSettled } from 'std:futures';

async function produce(sender: Sender<int>): Task<int> {
  await sender.send(21);
  return 0;
}

async function consume(queue: ReadOnly<Channel<int>>): Task<int> {
  const value = await queue.receive();
  console.log(value * 2);
  return 0;
}

async function main(): Task<void> {
  const queue = new Channel<int>(1);
  const outcomes = await allSettled([produce(queue.sender()).start(), consume(queue)]);
  for (const outcome of outcomes) {
    unwrap(clone(outcome));
  }

  queue.close();
}
```

This prints `42`. `send(value)` moves the message immediately into a lazy task;
await that task to send it. The endpoint remains borrowed until completion or
disposal. `receive()` similarly holds a channel borrow and returns the next message.
Messages cannot contain worker handles or channel endpoints.

`close()` rejects new sends and permits queued messages to drain. Receiving from
an empty closed channel, or sending after closure, throws code `closed`.

## Handle runtime failures

Awaiting a job, deadline, race, timer or channel operation can throw `AsyncError`.
Its `code` identifies the cause:

| Code | Meaning |
| --- | --- |
| `worker` | Started work threw an error; `cause` preserves its typed payload. |
| `cancelled` | Work was cancelled. |
| `panic` | A runtime worker failed with a panic. |
| `timeout` | A deadline expired. |
| `range` | An argument was outside its permitted range. |
| `empty` | A race had no inputs. |
| `closed` | A channel operation could not continue after closure. |

For codes other than `worker`, `cause` is undefined. Directly awaiting a source
Task preserves its original throw type instead of wrapping it as a worker error.
Use [project tests](../../tools/testing/) to verify error and cleanup behavior in
your application.
