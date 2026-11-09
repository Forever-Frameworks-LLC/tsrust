<!-- tsrust-docs
{"id":"reference-standard-library-futures","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:futures","name":"delay"},{"receiver":"std:futures","name":"yield"},{"receiver":"std:futures","name":"race"},{"receiver":"std:futures","name":"allSettled"},{"receiver":"std:futures","name":"Job"},{"receiver":"std:futures","name":"AsyncError"},{"receiver":"std:futures","name":"Channel"},{"receiver":"std:futures","name":"Sender"},{"receiver":"TaskMethods","name":"start"},{"receiver":"TaskMethods","name":"timeout"},{"receiver":"Job","name":"cancel"},{"receiver":"Job","name":"timeout"},{"receiver":"AsyncChannel","name":"sender"},{"receiver":"AsyncChannel","name":"send"},{"receiver":"AsyncChannel","name":"receive"},{"receiver":"AsyncChannel","name":"close"},{"receiver":"AsyncSender","name":"send"},{"receiver":"std:futures","name":"all"},{"receiver":"std:futures","name":"Interval"},{"receiver":"std:futures","name":"MissedTickBehavior"},{"receiver":"std:futures","name":"CancellationToken"},{"receiver":"std:futures","name":"SendError"},{"receiver":"std:futures","name":"ChannelError"},{"receiver":"Interval","name":"tick"},{"receiver":"CancellationToken","name":"share"},{"receiver":"CancellationToken","name":"child"},{"receiver":"CancellationToken","name":"cancel"},{"receiver":"CancellationToken","name":"isCancelled"},{"receiver":"CancellationToken","name":"cancelled"},{"receiver":"AsyncChannel","name":"trySend"},{"receiver":"AsyncChannel","name":"tryReceive"},{"receiver":"AsyncSender","name":"trySend"},{"receiver":"AsyncSender","name":"close"}]}
-->
# Tasks and async queues

Tasks and async channels preserve ownership, cancellation and cleanup. Use the documented send/receive and join contracts; do not assume resources can cross task boundaries.

Cancelling a started Job drops its suspended future and owned resources. Source `finally` statements run when control reaches them; dropping a future does not execute them. Put explicit cancellation cleanup in the owner that cancels and awaits the Job. Use `CancellationToken` when the task should handle cancellation cooperatively.

Use `all` when every result is needed. Results keep input order; the first observed
failure cancels siblings and waits for cleanup. `all([])` returns `void[]`;
`Task<void>` inputs also produce `void[]`. `allSettled` retains every outcome.

Call `all(...)` directly or use a renamed import such as `import { all as gather }`.
The polymorphic operation itself is not a storable function value: `const collect = all`
is rejected. Concrete Task/Job inputs and the Task returned by the call can be stored.

`new Interval(periodMs, missedTickBehavior?)` requires an active async runtime
and positive target-representable milliseconds. The first tick is immediately
ready. The default `burst` catches up missed ticks; `delay` shifts the schedule;
`skip` omits missed ticks. Await one exclusive `tick()` at a time.

Cancellation tokens are cooperative. Use `share()` for another handle to the same
signal and `child()` for a descendant. Cancelling a child leaves its parent
unchanged. `cancelled()` waits without cancelling work itself. Handle drop does
not signal cancellation.

`trySend` never waits; both `full` and `closed` failures own the exact unsent
message. `tryReceive` exclusively borrows the channel and returns `empty` or
`closed` when no buffered message exists. Closing one sender releases only that
endpoint; other senders continue working.

**Send-error migration:** awaited `send` now throws `SendError<T>` with `code` and
`value`, matching `trySend` and the synchronous channel identity exported from
`std:threads`. It previously threw `AsyncError<undefined>` and discarded the
message. Starting a send task produces `Job<void, SendError<T>>`; joining adds
the ordinary `AsyncError` wrapper. Cancelling a pending send disposes its message
once rather than returning a closed-send error.

Delay and timeout no longer impose a one-year ceiling. Invalid negative or
unrepresentable deadlines still fail with code `range`. Async channel capacity
must be positive, fit the target and satisfy Tokio's semaphore limit.

`delay` and `yield` also support renamed imports and direct calls, but the operations themselves are not storable function values. Their concrete returned Tasks can be stored or forwarded; an explicit async wrapper supplies a callable.
