# Prepared SQLite transaction

Run with Node v24.11.1 first on PATH and TSRUST pointing to the verified compiler:

```sh
node benchmarks/sqlite/run.mjs --verify
node benchmarks/sqlite/run.mjs
```

Each batch deletes previous rows, inserts 128 `(id, value)` pairs through one
prepared statement in an IMMEDIATE transaction, commits, and reads the aggregate.
The independent oracle is `sum(2*i, i=0..127) = 16256`. Both versions use an
in-memory database, the same SQL and operation order. Native reads the aggregate
as exact int; Node returns a number, exact for this range.

Verify mode runs one process and sample per engine without retaining timing.
Measurement uses three alternating process pairs, eight warmups and 31 samples
per process. Both generated programs are optimized (native release and Node after
tsc). Durations are milliseconds per 128-insert transaction, including statement
preparation, binding, commit and the aggregate query; startup and logging are outside
the timed batch. The runner records source/compiler/harness hashes and versions.
Native SQLite is 3.53.2 through rusqlite 0.40.2; Node v24.11.1 embeds SQLite 3.50.4.
Results compare these complete stacks, not language overhead in isolation.

Closed JSON records belong in `benchmarks/records.sqlite` after byte verification;
do not retain duplicate loose measurement records here.
