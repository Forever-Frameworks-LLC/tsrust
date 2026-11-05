<!-- tsrust-docs
{"id":"reference-standard-library-threads","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:threads","name":"Channel"},{"receiver":"std:threads","name":"Sender"},{"receiver":"std:threads","name":"TaskContext"},{"receiver":"std:threads","name":"ChannelError"},{"receiver":"std:threads","name":"SendError"},{"receiver":"std:threads","name":"spawnScoped"},{"receiver":"std:threads","name":"ScopedThread"},{"receiver":"std:threads","name":"spawn"},{"receiver":"std:threads","name":"join"},{"receiver":"std:threads","name":"detach"},{"receiver":"std:threads","name":"Thread"},{"receiver":"std:threads","name":"SpawnError"},{"receiver":"std:threads","name":"ThreadError"},{"receiver":"std:threads","name":"cancel"},{"receiver":"std:threads","name":"cancellationRequested"},{"receiver":"std:threads","name":"ThreadPoolStats"},{"receiver":"std:threads","name":"ThreadPool"},{"receiver":"std:threads","name":"ThreadPoolOptions"},{"receiver":"ThreadPool","name":"task"},{"receiver":"ThreadPool","name":"stats"},{"receiver":"ThreadPool","name":"shutdown"},{"receiver":"Channel","name":"sender"},{"receiver":"Channel","name":"receive"},{"receiver":"Channel","name":"tryReceive"},{"receiver":"Channel","name":"close"},{"receiver":"Sender","name":"send"},{"receiver":"Sender","name":"trySend"},{"receiver":"Sender","name":"close"},{"receiver":"TaskContext","name":"cancellationRequested"},{"receiver":"Thread","name":"join"},{"receiver":"Thread","name":"cancel"},{"receiver":"Thread","name":"detach"},{"receiver":"ScopedThread","name":"join"},{"receiver":"ScopedThread","name":"cancel"},{"receiver":"ScopedThread","name":"detach"},{"receiver":"ThreadPool","name":"taskAsync"}]}
-->
# Threads and channels

Thread APIs describe their transport and cancellation rules. Scoped work does not remove lifetime checking.

`ThreadPool.taskAsync` owns a concrete callback immediately but submits only when
its Task is first polled. Admission is bounded and never waits for capacity:
`SpawnError` reports `full` or `closed`. Completion returns the callback result
or throws the existing `ThreadError<E>`. Starting the task and joining it adds
normal `AsyncError` wrapping. Completion uses asynchronous notification.

An unpolled or rejected task drops its captures exactly once. Cancellation before
worker start skips the callback; after start, work must cooperate through
`TaskContext`. Scope cleanup waits for actual work completion. The lazy task
retains private weak admission identity, so it cannot keep a dropped public pool
owner alive. Pools retain explicit bounded capacity; worker count uses the actual
Rayon backend limit, and capacity must fit the target rather than an artificial
one-million ceiling.
