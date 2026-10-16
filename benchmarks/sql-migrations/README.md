# Migration history workload

`TSRUST=/path/to/current/tsrust node benchmarks/sql-migrations/run.mjs --verify`
checks one untimed batch in both engines, using a debug native build. Timing builds
release mode and uses three processes, eight warmups
and 31 samples, with 32 already-applied history checks per batch. Each process owns
its disposable file-backed SQLite database. The independent oracle is the sole
seeded primary key, 1; source and fixture hashes reject stale registrations.

Native SQLx performs its complete migration history path. Node-after-tsc uses a
bounded fixture-specific SQLite history/checksum implementation with SHA-384. This
is not a public Node migration library, arbitrary migration parser, concurrent
migration benchmark, or PostgreSQL/MySQL timing comparison. Initial migration DDL
is outside timing. No performance result is accepted until the S21 milestone.
