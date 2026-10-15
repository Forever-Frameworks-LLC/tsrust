# Async SQL acceptance

S18 uses SQLx through `std:sql`. PostgreSQL, MySQL and SQLite are accepted separately. Public operations use the existing lazy Task model, ordinary await error propagation, and checked receiver borrows.

## Contract

- `connect(url, maxConnections?, acquireTimeout?)` returns `Task<Pool>`; defaults are five connections and 30000 milliseconds. Explicit bounds are validated before connecting.
- `pool.query(sql, parameters?)` returns owned rows; `pool.execute` returns affected rows. Arguments use positional bindings, not string substitution. PostgreSQL uses `$1`; MySQL and SQLite use `?`.
- `pool.begin()` returns a transaction retaining a borrow of its pool. Transaction query/execute borrow it exclusively. Commit and rollback consume it. Drop or cancelled owning tasks initiate rollback; cleanup completion and connection reuse require database evidence.
- Pool close consumes the owner and waits for connection shutdown. Live transaction or operation borrows prevent closing it.
- Rows provide strict named accessors. Duplicate columns require aliases. Unsupported database-specific types must be explicitly cast in SQL.
- SQL text uses one prepared statement per call. Raw transaction/session control and transaction DDL are excluded so callers cannot defeat lifecycle tracking or trigger MySQL implicit DDL commits. No identifier interpolation or sanitization is implied.

## Acceptance matrix

Ordinary compiler/editor tests remain enabled. Database tests are opt-in and must fail clearly if explicitly selected without configured database URLs.

For every driver: connect/defaults/ranges; bind integer, finite number, string, blob and null; missing/wrong columns; duplicate columns; affected rows; commit; explicit rollback; dropped transaction rollback; exception rollback; cancellation rollback; exclusive borrow rejection; pool close with live loans; use-after-close/commit; bounded acquisition timeout and recovery; rows outlive database; formatted readable examples.

PostgreSQL and MySQL run against isolated Docker containers bound only to loopback. SQLite uses a temporary file or private memory database. Test scripts clean their own resources on both success and failure and never touch existing databases.

Run affected compiler metadata, native emission, async ownership, SQLite compatibility, editor agreement/completion/consistency, and exported offline native build checks. Preserve evidence in SQLite. Database tests default to skipped after acceptance; rerun explicitly whenever SQL runtime, SQLx dependencies, task lifecycle, SQL ownership lowering or native SQL emission changes.

## Active acceptance

Implementation is not accepted yet. Direct runtime contracts passed on PostgreSQL 18, MySQL 8.4 and SQLite. Generated-program cancellation and cleanup checks remain in progress.

Completed affected checks: 123 native-view regressions, 24 async task regressions, 3 SQL editor tests, 4 SQLite editor tests, the 3-test completion matrix and 8 editor consistency tests. SQLite's new prepared-batch workload passed an untimed native/Node checksum comparison; this is not a throughput measurement.

Driver coverage is opt-in (`#[ignore]`), independent of ordinary compiler and editor checks. SQL runtime, SQLx dependency, task lifecycle, ownership lowering and SQL emission changes require explicitly rerunning all three drivers. Unrelated changes do not require Docker.

SQLx 0.9.0 requires one compatible SQLite native library. The shared dependency is now rusqlite 0.39.0 / libsqlite3-sys 0.37.0 / SQLite 3.51.3, replacing rusqlite 0.40.2 / SQLite 3.53.2. S17 compatibility checks must pass before acceptance; historical measurements remain unchanged.

