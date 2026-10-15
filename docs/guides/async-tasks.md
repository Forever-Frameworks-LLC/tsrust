# Lazy async tasks

An async function returns `Task<T>`. Calling it evaluates arguments and defaults,
transfers owned inputs, and creates the task. The body starts when awaited.
`await` consumes the task once and produces `T`; it is valid only inside an async
function. Use `Task<void>` for an operation without a result, including async main.

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

This prints `created`, then `Hello, Ada`. Ordinary calls do not start the body.
Tasks cannot be cloned, called like functions, or awaited twice. Moving a task or
returning it transfers responsibility. Never-started tasks discarded on reachable
paths produce warnings. Warnings include creation and abandonment locations.

Body errors reach the await site through ordinary try/catch. Argument and default
errors occur when constructing the task. Normal completion, return and throwing
retain normal finally behavior. Disposing a future releases its owned state;
it does not promise execution of source finally statements during cancellation.

ReadOnly and Mutable arguments retain their loans while the task owns them. The
owner cannot be changed, transferred or disposed while a conflicting loan remains.
After completion, the loan is released. Local awaiting does not require Send;
thread boundaries keep their separate transfer checks.

S13 provides native futures and a local entry driver. Tokio scheduling, asynchronous
I/O, timers, started jobs and cancellation APIs belong to S14. No start, cancel,
timeout or combinator method is exposed here. Async declarations require an explicit
Task result annotation. Async arrows and JavaScript Promise APIs are not included.

Readable examples are `examples/async-tasks/main.tsr` and
`examples/async-tasks/borrow-and-recover.tsr` (prints `2`, then `Record missing`).
The matched lazy-thunk Node counterpart and independent checksum are under
`benchmarks/async`. `run.mjs --verify` checks correctness without timing claims;
`run.mjs` measures three processes, eight warmups and 31 samples per process.
This measures sequential lazy construction/await, not concurrency or Tokio I/O.

Abandonment warnings include failure paths: a throwing operation before await can
discard an unstarted task. A finally block can await that task on both paths.
