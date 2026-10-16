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

## Acceptance evidence

Implementation checkpoint: `1b4a1639`; nested cleanup repair: `1eeaac6e`. The final repeat Docker acceptance passed after the cleanup repair, including all three drivers and readable examples; all temporary containers were removed. Release CLI and LSP consumers are refreshed. The installed CLI checks both readable examples; the final LSP bytes match the server that passed the completion matrix. VS Code reports no workspace problems.

- PostgreSQL 18, MySQL 8.4 and SQLite passed the generated-program contract: exact scalar binding (including mixed floating-point and maximum 64-bit integers), duplicate/missing/wrong columns, commit, explicit/drop/exception rollback, cancellation during database work, timeout rollback, acquisition timeout recovery and rows outliving their pool.
- All three direct runtime contracts passed, including PostgreSQL SMALLINT, INTEGER and REAL widths. Statement validation and connection-range tests passed too.
- Ordinary SQL acceptance: three tests passed, one database test ignored. These check examples/formatting, exact integer emission and rejected ownership violations, including a pending commit/rollback retaining its pool borrow.
- Existing behavior: 123 native-view regressions, 24 async task regressions, 12 Tokio integration tests and ten futures runtime tests passed. The tenth reproduces cancelling a drain after it has collected nested cleanup; the repaired drain retains that work until settlement.
- S17 compatibility: five native integration tests, five SQLite runtime tests and four SQLite editor tests passed.
- Editor: three SQL tests, two checker-agreement tests, the three-test completion matrix and eight consistency tests passed. The completion matrix also passed against the installed release server, and VS Code workspace diagnostics cleared after restart.
- The exported SQL package built offline with an empty Cargo home and printed `1: Ada` and `2: Grace`.
- Default Cargo runs passed five database-free checks and skipped the two database suites.
- Whole-project formatting changed zero files and zero fingerprints at the implementation checkpoint.

Closed evidence is stored in `benchmarks/records.sqlite`. The first import byte-restored 56 logs and four benchmark records, preserved all 55132 pre-existing record bindings and passed SQLite integrity/foreign-key checks. Log record names start with `tsrust-s18-`; failures are retained alongside repairs.

## Benchmark status

Native and Node-after-tsc prepared-batch checksums match on PostgreSQL, MySQL and SQLite: 16256 for 128 inserted rows. The existing synchronous SQLite workload also passes after the shared dependency change. These are untimed verification runs, not throughput measurements. Full timing remains a separate feature-family milestone.

Exact records:

- `benchmarks/async-sql/prepared-sqlite-2026-10-15T21-29-26.683Z-verify.json`
- `benchmarks/async-sql/prepared-postgres-2026-10-15T21-37-39.264Z-verify.json`
- `benchmarks/async-sql/prepared-mysql-2026-10-15T21-39-17.920Z-verify.json`
- `benchmarks/sqlite/prepared-2026-10-15T21-38-55.987Z-verify.json`

## Default test policy and limits

Driver coverage is opt-in (`#[ignore]`), independent of ordinary compiler and editor checks. SQL runtime, SQLx dependency, task lifecycle, ownership lowering and SQL emission changes require explicitly rerunning all three drivers with `node tools/check-async-sql.mjs --docker`. Unrelated changes do not require Docker.

SQLx 0.9.0 requires one compatible SQLite native library. The shared dependency is now rusqlite 0.39.0 / libsqlite3-sys 0.37.0 / SQLite 3.51.3, replacing rusqlite 0.40.2 / SQLite 3.53.2. Historical S17 measurements remain unchanged.

Scope is bounded pools, prepared parameter binding, owned scalar rows and transactions. A transaction returned through `begin().timeout(...)` is conservatively rejected because that adapter does not transport its returned native borrow; await begin directly and configure acquisition timeout. Migration orchestration, streaming and compile-time database schema checking remain deferred. Cancellation preserves cleanup but may wait for an in-flight database statement; it is not an immediate server-side query abort. A cancelled or disconnected commit has an indeterminate server outcome.

Tested images: PostgreSQL `sha256:74935e72241653ca55e0414067e6d8763aceb8a810eb51b452253ec3dcfc4336`; MySQL `sha256:24989179f58856db763f418010c1169fecc944e3ec1ea7e921572d83eb070f70`.

Final evidence includes `tsrust-s18-cleanup-docker-acceptance.log`, `tsrust-s18-final-futures-runtime.log`, `tsrust-s18-final-default-policy.log`, `tsrust-s18-installed-editor-matrix.log`, `tsrust-s18-final-consumer-refresh.log` and `tsrust-s18-final-evidence.json` in SQLite.

## Pool cursor streaming — focused acceptance

`Pool.cursor(sql, parameters?)` returns a pool-borrowing Cursor. `next()` yields
one owned Row or undefined; `close()` consumes it. EOF, terminal errors and
close/drop release the result stream. One pending next task exclusively borrows
the cursor. Returned rows remain independent. Dropping a pending read task does
not itself dispose the cursor; close/drop it when abandoning the query.

SQLx owns the SQL and bound values. Metadata preparation and result streaming
can acquire different connections from the same pool. This is client-side row
streaming with driver buffering, not a named server cursor or transaction snapshot.
Transaction cursors, fetch-size controls and scroll/reposition operations are
unsupported. SQL can include writes with RETURNING; cursor disposal is not a
rollback boundary. Existing exact parameter conversions and column checks remain active.

Cursor runtime checks pass PostgreSQL, MySQL and SQLite: EOF releases the pool
connection while the cursor remains live; explicit close, terminal errors and
cancelled reads followed by close permit independent pool use. Owned rows survive
cursor disposal. Generated three-driver ownership/cleanup programs, the cursor
example, SQL/HTTPS coexistence, retained HTTP pools and offline migration execution
also pass. Exact-domain checks passed in the initial run and were reused.

The initial generated optional-row comparison failed because an await wrapper
converted an already borrowed union into a second view. The emitter now preserves
the child's view. The repaired runner resumes with
`--resume-from=sqlite --reuse-domain-evidence`; it skips the already accepted runtime
and domain groups. Original failures remain retained alongside repaired evidence
under `tsrust-cursor-*` in SQLite. Three focused editor catalog checks, live
hover/completion and all four typing modes pass. The matched SQLite cursor workload
passes untimed native/Node verification with 128 ordered rows and checksum 16,256.
It also executes a cursor returned through an async helper, preserving its pool loan.
The record is `cursor-sqlite-2026-10-16T17-12-53.193Z-verify.json`. No timing or
full-family regression run is claimed.
