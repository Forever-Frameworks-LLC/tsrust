# S17 SQLite acceptance

Status: implementation complete; remaining validation and consumer refresh in progress.

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

Completed before final consumer refresh: 5 SQLite runtime checks; 221 native
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
