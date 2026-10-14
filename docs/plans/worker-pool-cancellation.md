# Global worker pool and cooperative cancellation

Implementation in progress, following the accepted default cleanup slice. Channels
remain the next section; this document does not claim their implementation.

`pool.task(callback)` submits immediately to the global Rayon pool. `spawn`
starts an OS thread for long-lived or blocking work. Import `pool` from `std:threads`. Both return the same owned
`Thread<T, E>` and retain scope joining, explicit join and explicit detach.
`spawnScoped` remains a compatibility spelling of dedicated spawn.

Call `pool.configure(workers, capacity)` before the first pooled spawn. Workers range
from 1 through 256; outstanding capacity ranges from 1 through 1,000,000. Defaults
are available CPU parallelism clamped to 2..256 and capacity 1024. Configuration is
permanent. Pool creation failure does not freeze an invalid configuration.

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
