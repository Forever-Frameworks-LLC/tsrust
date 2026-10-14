# Shared and scoped worker pools with cooperative cancellation

Focused implementation acceptance passed on 2026-10-14. Channels remain the
next section; this document does not claim their implementation or a new full
compiler/LSP family regression.

`pool.task(callback)` submits immediately to the global Rayon pool. `spawn`
starts an OS thread for long-lived or blocking work. Import `pool` from `std:threads`. Both return the same owned
`Thread<T, E>` and retain scope joining, explicit join and explicit detach.
`spawnScoped` remains a compatibility spelling of dedicated spawn.

Call `pool.configure({ workers, capacity })` before the first pooled spawn. Workers range
from 1 through 256; outstanding capacity ranges from 1 through 1,000,000. Defaults
are available CPU parallelism clamped to 2..256 and capacity 1024. Configuration is
permanent. Configuration may run at module scope; it returns void and throws a
typed SpawnError for invalid limits, repeated configuration or worker creation
failure. No unwrap is needed. Pool creation failure does not freeze an invalid
configuration. Independent construction also returns Pool directly and throws
SpawnError; ordinary try/catch handles both.

`new Pool({ workers, capacity })` returns an independent `Pool` with the same limits
and validation. It never initializes or changes the shared default. `jobs.task`,
`jobs.stats` and `jobs.shutdown` operate on that pool only. Limits are fixed at
creation. The unique owner may move between caller functions or be borrowed as
`ReadOnly<Pool>`; it cannot be cloned, captured by callbacks, stored in containers
or returned across a worker boundary. This prevents an owner from draining itself.
Scope exit closes admission and drains queued/running work, including detached
work. Explicit shutdown is idempotent. Retained task results remain joinable after
the owner leaves scope. The executor is reclaimed after its owner and retained
task handles release their references. The shared default intentionally lives
until explicit shutdown or process exit.

Cleanup acceptance includes repeated independent creation/destruction with weak
scheduler references, exactly-once captured/result/error disposal, cancelled
queued work, backend panic, detached tasks and retained completion handles.
These are lifetime witnesses, not a claim of universal leak-sanitizer coverage.
`examples/scoped-pools/main.tsr` demonstrates independent pools and caller borrowing.

Admission returns `SpawnError` with code `full` instead of waiting for capacity.
Completed tasks free their worker but keep a slot until join or owner cleanup.
Detached tasks keep their slot until completion. `pool.stats()` snapshots queued,
running, completed and outstanding counts plus configured workers and capacity.
Dedicated threads do not contribute. A nested joining worker helps execute Rayon
jobs, allowing one-worker parent/child progress. Dependency cycles and arbitrary
blocking callbacks can still prevent progress; use dedicated workers for blocking
services. Running task count can exceed worker count while parents help children.

`pool.shutdown()` closes admission permanently and drains queued/running tasks.
Handles keep their results and can still be joined. Repeated calls succeed; calls
from workers return code `worker` instead of waiting on themselves. Dedicated
threads remain available. There is no forced timeout or implicit cancellation.

`cancel(worker)` borrows its handle. It returns true for the first request before
published completion, otherwise false. `cancellationRequested()` reports the
current worker's flag and is false outside a worker. Long-running code polls and
returns promptly. Queued callbacks are skipped when scheduled and captured owners
are disposed exactly once. A request winning the completion race discards the
callback result/error and yields `ThreadError` code `cancelled`, with no payload.
An already published completion wins over cancellation. Cancellation is a request,
not proof of termination: join and cleanup still wait; native calls cannot be
forcibly interrupted. Nested execution restores the parent's cancellation context.

Rayon 1.11.0 is pinned in compiler and bundled native dependencies. Pool submission
bridges owned source callbacks, results, typed errors and backend panics into Rayon;
source loans remain rejected. Offline native builds and exported packages must
include its locked transitive source closure. No parallel iterator API is added.
The backend uses [Rayon's pool yield operation](https://docs.rs/rayon/1.11.0/rayon/struct.ThreadPool.html#method.yield_now)
while joining from a pool worker, without holding completion locks.

`examples/worker-pool/main.tsr` demonstrates configuring, joining and cancelling.
The existing dedicated startup benchmarks retain their semantics through dedicated `spawn`. A new `pool-join` workload compares two reusable workers
with Node worker-message dispatch. The independent checksum is 140. Preserve three
processes, eight warmups and 31 samples when measuring; quick acceptance uses
`node benchmarks/threads/run.mjs --pool --verify` without timing claims.

## Acceptance evidence

The current section passes 12 runtime tests, all 16 native thread integration
cases (15 original passes plus the repaired Pool-only storage rejection), 64
native declaration tests, 9 expression-fact tests and 3 package export tests.
The package test relocates JSON, shared-pool and scoped-pool examples and builds
each offline in debug and release with an empty Cargo home.

Editor acceptance passes checker agreement (2), the full typing-speed completion
matrix (3), editor consistency (5) and worker documentation/protocol tests (10).
The catalog examples execute after formatting in debug and release. Shared native
call-result facts now reach the editor, including PoolStats locals. Both namespace
aliases and independent Pool receiver methods have completion coverage.

macOS `leaks --atExit` reports 0 leaks / 0 leaked bytes for the 11-test cleanup
runtime corpus; the twelfth test independently proves two pool workers overlap
before either is joined. Weak scheduler references expire across 32 repeated pool
lifetimes. Captures, success values, typed errors, rejected admissions, cancellation,
panics and detached work have explicit disposal witnesses. This is bounded evidence,
not proof about all possible programs or nonterminating callbacks.

SQLite retains exact `tsrust-scoped-*.log` evidence, including failed attempts and
the affected repairs. The original native suite exposed missing Pool-only storage
validation; the repaired guard rejects it without changing ownership rules.
The initial macOS leak attempt lacked sandbox task-port access; the unrestricted
inspection of its own spawned test process succeeded.

Pooled timing is recorded in `benchmarks/threads/pool-join-2026-10-14T22-25-15-491Z.json`:
3 processes, 8 warmups and 31 samples, with checksum 140 for every sample. Median
native/Node-after-tsc times are 0.014625/0.070583 ms per eight-task batch. These are
small-task dispatch costs, not an application speedup claim. Dedicated startup and
scoped-join workloads passed current untimed checks. Their earlier timings remain
unchanged. The registration guard passes with current source fingerprints.


## Configuration ergonomics amendment

The shared singleton remains available. Module-scope `pool.configure({ workers,
capacity })` returns void; `new Pool({ workers, capacity })` returns an independent
owner. Both propagate typed SpawnError through normal exception handling, without
an unwrap call. PoolOptions supplies contextual integer checking and reusable
configuration values. Task submission and completion retain their Result types.

The native thread suite now has 18 cases. Focused amendment checks cover
ordered, once-only dependency initialization,
initialization failure before main, configuration limits, independent construction,
ownership rejection, aliases, completion and executable catalog examples. The
operation-only runtime selection regression covers configuration, statistics and
cancellation queries without a Thread value. Existing module-state checks pass.
Offline relocated packages are checked in debug and release.

The earlier pooled timing record uses the prior configuration syntax. The current
source has an untimed checksum verification record; no new timing claim is made.
Exact `tsrust-pool-options-*.log` records retain the failed attempts and repairs.
