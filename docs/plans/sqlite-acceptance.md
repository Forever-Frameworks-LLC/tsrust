# S17 SQLite acceptance

Status: accepted. Compiler/runtime/editor checks, executable examples, matched measurement and consumer refresh are complete.

## Scope

`std:sqlite` adds `open(path, busyTimeout?)`, owned Connection, borrowing Statement
and Transaction, owned Row, and SqliteError. Connections prepare statements and
start IMMEDIATE transactions. Statements bind typed positional parameters and
execute writes or materialize read-only queries. Rows expose strict named readers
for signed 64-bit integers, finite reals, UTF-8 text, blobs and SQL NULL checks.
Application record mapping is an ordinary checked function, demonstrated in
`examples/sqlite/people.tsr`.

Transactions exclusively borrow the connection; statements keep their parent
alive. Explicit close/finalize/commit/rollback consume their owner. Drop and error
propagation finalize statements and roll back uncommitted transactions. Failed
commit attempts rollback. Failed rollback prevents further database operations
except close. SQLite-triggered rollback is detected before later statement work.
An authorizer blocks raw transaction/savepoint SQL from bypassing ownership.
Queries reject writes and duplicate column names; execute rejects result columns
before side effects. Parameters are bound afresh on every invocation.

Two existing compiler boundaries needed adjustment: SQLite native calls accept a
borrowed handle through their lifetime-aware input ABI without returning one, and `unwrap` must preserve loans carried
by its success payload. Return contracts remain validated. Both SQLite early-close
rejections and an existing byte-view/resize rejection cover the latter change.

## Validation

Closed logs are imported and byte-restored from `benchmarks/records.sqlite` under
`tsrust-s17-*`; only active checks retain loose logs. Earlier failed checks remain
alongside their repairs. Source checkpoint: `91e6b937`. B4's reviewed callable
specialization repair is integrated at `e33af7db`; its separate evidence is under
`runtime/diagnostics/ui-b4-repair/4beded639e8735186a4f97df3c0f8d209bd9afa0/`.

Completed checks: 5 SQLite runtime checks; 221 native
regression checks (220 initially passed, the affected returned-borrow guard and
all 9 contract checks passed after repair); 46 error-effect checks; 91 affected
compiler integration checks; and one relocated offline export check covering
both debug and release. Groups overlap and are not a unique combined test count.

The editor repair prevents an unknown indexed receiver such as `rows[0]` from
being mistaken for an array literal. Inferred SQLite handles are checked through
the project path and exact member labels, including a typing-speed matrix row.
The workspace panic reported while opening empty/comment-only files came from
B4 callable access summaries during workspace checking, not empty-file parsing.
The combined fixture checks those files alongside the repaired named handler.

- Runtime: commit/drop/explicit rollback/unwind, deferred-constraint commit failure,
  SQLite automatic rollback, exact data conversion, malformed UTF-8, binding count,
  raw control SQL, column/type errors, lock contention and file cleanup.
- Compiler: readable examples, typed record mapping, stored `open` callable,
  rows surviving finalized/closed parents, loan conflicts, consumed handles and
  byte-view `unwrap` provenance.
- Editor: native metadata drives inference; completion, hover, signature help,
  imported aliases and inferred locals are checked with agreement/consistency suites.
- Export: relocated source packages build and execute offline with an empty Cargo
  home in both debug and release.
- Benchmark support: `benchmarks/sqlite/prepared.{tsr,ts}` and `run.mjs` implement
  the same 128-row prepared transaction. Independent checksum is 128 × 127.
  Verify mode is untimed acceptance; timing uses 3 processes, 8 warmups and 31 samples.

## Boundaries

No async database pools, ORM, schema inference, migrations, savepoints, extensions
or write-query RETURNING. Read-only query results materialize in memory; callers
should use LIMIT for large results. Synchronous work blocks the calling thread.

Pinned backend: rusqlite 0.40.2 with bundled SQLite. The adapter uses public safe
APIs; no unsafe lifetime extension or shared connection wrapper. Reference:
[rusqlite](https://docs.rs/rusqlite/0.40.2/rusqlite/).

## Final acceptance and consumers

Combined source checkpoint: `b255009e`. B4's 2 compiler execution checks and the
package reservation check pass. All 18 affected editor checks pass: checker
agreement (2), typing-speed completion matrix (3), consistency (8), SQLite (4),
and the named-handler/empty/comment-file workspace fixture (1). Full recursive
workspace checking completes without panic or errors. One existing `unused-task`
false positive flags `console.log(await message)` in `examples/async-tasks/main.tsr`;
it reproduces with the pre-S17 installed compiler and remains separate follow-up.
No full unrelated compiler/LSP regression matrix was repeated after S15.

Installed CLI SHA256: `3551e7be8a852c9222782327e4036ad91f2311784654a94f74890bc31195db96`.
Installed and configured editor LSP SHA256:
`8432e8da93bf38da826d91872f4282c53a3070cf9393a1f2cd0dba82c3cf97f6`.
The active VS Code server was restarted through its status item. SQLite `open`
hover shows the current signature, defaults and ownership behavior. Installed CLI
checks of the previously crashing handler and SQLite examples have no diagnostics.

Matched in-memory prepared transactions used 3 processes, 8 warmups and 31 samples
per process, retaining all 186 samples. Median per 128 inserted rows plus transaction
and aggregate costs: native 0.031084 ms, Node-after-tsc 0.105708 ms (3.40× ratio).
All 186 checksums equal 16256. Native SQLite 3.53.2 and Node SQLite 3.50.4 differ;
this is a complete-stack comparison, not isolated language overhead or disk I/O.
No comparable earlier SQLite measurement exists, so this establishes a baseline.
No optimization was needed for acceptance.

Exact raw record: `benchmarks/sqlite/prepared-2026-10-15T19-50-10.637Z.json`, stored
in SQLite with SHA256 `4cfde74134a01db79333ffbcc4ef812e5d76f2e10a31a0a73518235cfc118d10`.
The matching proof and normalized run/workload/sample/summary rows are retained.
Untimed verification is separately stored under
`benchmarks/sqlite/prepared-2026-10-15T19-31-48.654Z-verify.json`.
