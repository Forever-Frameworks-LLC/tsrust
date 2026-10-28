---
title: Work with async tasks
description: Create lazy tasks, await their results, and understand ownership and errors.
---

Use an `async` function for work whose result becomes available through `await`.
Declare its return type as `Task<T>`, or `Task<void>` when it returns no value.
An async program entry point uses `async function main(): Task<void>`.

## Create and await a task

Save this complete example as `main.tsr` after
[preparing native build tools](../../tools/rust-setup/):

```tsrust
async function greeting(name: string): Task<string> {
  return `Hello, ${name}`;
}

async function main(): Task<void> {
  const message = greeting('Ada');
  console.log('created');
  console.log(await message);
}
```

Run the program with `tsrust run main.tsr`. It prints:

```text
created
Hello, Ada
```

Calling `greeting` evaluates its arguments and defaults, transfers owned inputs,
and creates a lazy task. Its body has not run yet. Awaiting the task starts that
body and produces its result. `await` is valid only inside an async function.

## Use each task once

Await consumes a task. You cannot await it twice, clone it, or call it as a function.
Moving or returning a task transfers responsibility to its new owner.

Discarding a never-started task on a reachable path produces an `unused-task`
warning. This includes failure paths where an earlier operation throws before the
await. The warning identifies creation and abandonment locations. It does not
start the task for you.

## Handle errors at the right point

An error evaluating an argument or default occurs when you call the async function.
An error from its body reaches the await site, where ordinary `try`/`catch` handles
it. Normal completion, return and throw retain ordinary `finally` behavior.

Dropping or cancelling suspended work releases owned state, but does not promise
to execute arbitrary source `finally` statements after the suspended await.
Do not depend on those statements as a cancellation cleanup hook.

## Keep borrowed inputs alive

A task taking `ReadOnly<T>` or `Mutable<T>` holds that loan until it completes or
is disposed. Conflicting mutation, transfer or disposal of the owner is rejected
while the loan remains live. After awaiting completion, the owner is usable again.

A task cannot return a child task that retains borrowed data. Await the child and
return its result instead. Nested tasks containing only owned data are supported.
Local awaiting does not require inputs to be transferable across worker threads.

## Start concurrent work

Plain async calls are lazy; they do not schedule background work. To schedule an
owned task on the runtime worker pool, call `.start()` and await the returned job.
Replace `main.tsr` with this program and run it with the same command:

```tsrust
import { delay } from 'std:futures';

async function answer(): Task<int> {
  await delay(10);
  return 42;
}

async function main(): Task<void> {
  const job = answer().start();
  console.log(await job);
}
```

This prints `42`. A task borrowing local data can be awaited locally, but cannot
be started across workers. A started job must be awaited or transferred to an
owner that settles it; requesting cancellation alone does not settle it.

The runtime also provides [timers, task groups and channels](../../library/futures/). Consult
IntelliSense for the `Task`, `Job` and `std:futures` operation contracts. These
are Tsrust APIs, not JavaScript `Promise` compatibility.

For asynchronous service code, continue with [HTTP routing](../../backend/http/)
or [testing](../../tools/testing/).
