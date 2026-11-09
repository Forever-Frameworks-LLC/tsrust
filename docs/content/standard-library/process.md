<!-- tsrust-docs
{"id":"reference-standard-library-process","group":"standard-library","order":22,"status":"partial","targets":["native"],"api":[{"receiver":"std:process","name":"CommandOptions"},{"receiver":"std:process","name":"Child"},{"receiver":"std:process","name":"ExitStatus"},{"receiver":"std:process","name":"Output"},{"receiver":"std:process","name":"ProcessError"},{"receiver":"std:process","name":"spawn"},{"receiver":"std:process","name":"run"},{"receiver":"Child","name":"id"},{"receiver":"Child","name":"takeStdin"},{"receiver":"Child","name":"takeStdout"},{"receiver":"Child","name":"takeStderr"},{"receiver":"Child","name":"wait"},{"receiver":"Child","name":"tryWait"},{"receiver":"Child","name":"kill"}]}
-->
# Child processes

`std:process` launches an executable directly, with separate UTF-8 argument
strings. It never inserts a shell. Executable lookup, argument quoting, cwd and
environment behavior follow the host backend. Per-child settings do not mutate
the parent environment or working directory.

`spawn(program, args, options?)` immediately returns `Result<Child, ProcessError>`
and requires an active async runtime. Its default stdin, stdout and stderr are
inherited. A Child is unique and nonclonable. `id()` observes the live process ID;
`tryWait()` polls, and `wait()` lazily waits for an exit snapshot. A retained Child
survives cancellation of its borrowed wait. `kill()` forcefully terminates and
awaits reaping of the direct child. A completed wait/kill clears the observed ID.

Set stdin/stdout/stderr to `pipe` and transfer each endpoint once with
`takeStdin`, `takeStdout`, or `takeStderr`. Missing or already-taken endpoints are
`undefined`. The returned Reader/Writer uses `std:io`; close stdin to deliver EOF.
Drain both output pipes concurrently when a child may fill either pipe. Child
waiting does not automatically drain endpoints transferred to the caller.

`run(program, args, options?)` returns a lazy `Task<Result<Output, ProcessError>>`.
It defaults to null stdin and captured stdout/stderr, concurrently feeds owned
input, closes stdin, drains both outputs and waits. Output owns binary stdout and
stderr Bytes and an ExitStatus; nonzero exit is a successful operation with
`status.success === false`. ExitStatus also has optional `code` and Unix-only
`signal`. Null or inherited output streams yield empty captured Bytes.

CommandOptions accepts optional `cwd`, `env: EnvVar[]`, `clearEnv`, stdin/stdout/
stderr modes (`inherit`, `null`, `pipe`), owned `input: Bytes | Reader`,
`deadlineMs`, and `maxOutputBytes`. Environment is inherited unless clearEnv is
true; supplied entries override child values. Input implies piped stdin and
cannot combine with null/inherit. Run rejects stdin pipe without input because
there is no returned writer. Spawn rejects input/deadline/output-limit settings.

There is no implicit timeout or total-output cap. Explicit deadlineMs is a
nonnegative int checked against the backend Instant range; zero uses cooperative
polling and does not preempt a ready operation. maxOutputBytes is nonnegative and
must fit the target. It bounds the combined captured stdout/stderr; exceeding it
returns output-limit without partial Output. Explicit timeout or an I/O failure
terminates and reaps the owned direct child before returning. Destructor cleanup
is scoped and awaited on cancellation; kill-on-drop is a backstop outside ordinary
scope drainage. This does not supervise descendants. Descendants retaining pipe
handles can keep EOF pending, including after the direct child exits.

ProcessError owns `code`, `operation`, `message` and optional `osCode`. Categories
include not-found, permission-denied, invalid-input, broken-pipe, io, runtime,
option, range, timeout, output-limit and allocation. Keep the original OS message
and code when diagnosing failures; message text is not a stable discriminator.

Run consumes options/input when the Task is created. Stable program and argument
owners remain borrowed through settlement, including stored/forwarded calls;
mutating or moving them while pending is rejected. Inline temporaries move into
the Task. Dropping an unpolled Task drops owned input without spawning. Started
work must move its own resources into an async factory instead of retaining caller
loans. Spawn borrows its arguments only during the immediate call.

See `examples/process/main.tsr` and `examples/process/pipes.tsr` for portable
self-child examples with captured output and one-time pipe ownership.
