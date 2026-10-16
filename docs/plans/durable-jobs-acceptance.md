# Application-level SQL jobs

The owner's 2026-10-16 decision selects an application example using existing SQL
pools, not a `std:jobs` library. Implementation lives in
`examples/durable-jobs/jobs.tsr`; the readable executable and README describe the
supported SQLite scope. There are no new compiler types, native methods or editor
catalog entries. Existing Pool/Transaction IntelliSense and checked asynchronous
loans apply unchanged.

The section adds stable-ID enqueue deduplication, atomic claims, 30-second leases,
monotonic generations, bounded linear backoff, three-attempt exhaustion, stale-worker
fencing and retained terminal states. The example's SQL effect and acknowledgement
share a transaction. External side effects still require independent idempotency.
Lease expiry can cause repeated deliveries; exactly-once processing is not promised.

Focused acceptance must execute the formatted multi-module example against a fresh
file-backed SQLite database. It checks conflicting payloads, delayed retries,
reopening the database, expired lease recovery, stale acknowledgement/retry rejection,
duplicate acknowledgement and enqueue, poison-job exhaustion and final-attempt
abandonment. An independent connection then observes persisted terminal rows and
exactly one committed effect. The suite skips database execution by default.

The matched `benchmarks/async-sql/run.mjs --jobs --verify` workload executes 128
SQLite jobs and checks 128 effects in native and Node-after-tsc implementations.
It pins the actual application helper source. It measures no persistence or concurrent
throughput: the workload is in-memory and untimed verification is distinct from
future 3-process/8-warmup/31-sample timing. PostgreSQL/MySQL dialects, lease renewal,
remote brokers and distributed clock guarantees remain unsupported.

The formatted multi-module example and native execution pass. Independent SQLx
inspection of the closed database confirms three attempts/generations for each job,
failed states for both poison and abandoned jobs, and exactly one receipt effect.
Evidence is retained under `tsrust-durable-jobs-*`. The matched native/Node
workload also passes one untimed batch: 128 jobs and 128 committed effects, recorded
in `jobs-sqlite-2026-10-16T17-13-42.637Z-verify.json`. Cursor editor checks cover the
unchanged SQL methods. Installed CLI and both editor servers are atomically
refreshed and hash-verified. The restarted active editor shows the job example
with checking enabled and no workspace problems. Rust-analyzer remains disabled. Reuse the cursor section's SQL runtime evidence because this
example introduces no runtime changes.
