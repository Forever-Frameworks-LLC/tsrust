# Native task repository

This is the SQLite domain layer for the backend prototype. It does not yet expose
HTTP operations or claim application access-policy acceptance. A private borrowed
pool is never wire data; the generated Context-aware dispatcher is still pending.

Task writes use parameterized SQL and atomic `RETURNING` results. Listing uses a
positive limit of at most 100 and an exclusive nonnegative id cursor. Titles are
trimmed and must contain 1–200 UTF-8 bytes, matching Tsrust string length and the
SQLite constraint. For example, fifty rocket emoji fit and fifty-one reject.
Web/mobile validation must count UTF-8 bytes rather than UTF-16 string units.
Positive ids stay in the JavaScript-safe
integer range. Missing tasks and invalid domain inputs return named data failures.
Database/runtime errors must be projected by the later server boundary without
exposing private SQL details. There is no implicit retry or rollback claim.

`witness.tsr` requires a fresh disposable disk `TASK_TEST_DATABASE_URL`. Run it once
without `TASK_TEST_PHASE`, then again with `TASK_TEST_PHASE=verify` using the same
database to prove persistence across executable restarts. The migration path is
currently relative to the repository root; standalone server packaging must provide
its own migration path. Run `node tools/verify-task-repository.mjs` after other
Cargo work finishes; it compiles once and drives both phases against a temporary
disk database, then removes the database. `--run-only` reuses that executable for
affected execution checks. The formatted source passes Tsrust project checking.
Native CRUD and validation pass, including UTF-8 byte boundaries, SQL-looking
parameter values, cursor ordering and persistence across executable restarts.
See [acceptance evidence](../../docs/milestones/task-repository.md). HTTP access
policy and generated dispatch remain pending.
