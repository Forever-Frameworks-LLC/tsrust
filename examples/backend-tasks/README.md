# Native task backend

The SQLite domain layer and generated four-operation HTTP API pass focused Windows
native acceptance. The production entry passes source checking; TLS and its
signal lifecycle still need runtime acceptance. A private borrowed pool
is never wire data. The application owns bearer authentication and exact-origin
browser access policy; the generated dispatcher validates the public boundary.

## HTTP application under validation

Use the current built development compiler (the installed accepted compiler has
not been refreshed for the new server configuration yet):

```powershell
target/debug/tsrust.exe api generate --config examples/backend-tasks/tsrust.api.json
target/debug/tsrust.exe run examples/backend-tasks/main.tsr
```

Set these process environment variables before running the service:

| Setting | Meaning |
| --- | --- |
| TASK_DATABASE_URL | Persistent SQLite URL, for example `sqlite:tasks.sqlite?mode=rwc`. |
| TASK_MIGRATIONS | Migration directory; use an absolute path when running outside this repository. |
| TASK_TOKEN | A private, randomly generated 64-character lowercase hexadecimal bearer credential. |
| TASK_ORIGIN | One exact browser origin, including its port and without a trailing slash. |
| TASK_ADDRESS | Listener address, for example `127.0.0.1:0` for an assigned loopback port. |
| TASK_TLS_CERT / TASK_TLS_KEY | Optional pair of PEM file paths. Both are required for TLS. Cleartext binds are restricted to loopback. |

The entry prints one typed readiness JSON object containing address/protocol, and
requests orderly shutdown when its signal listener receives a signal. These entry
paths are source-checked, not yet demonstrated as standalone/package lifecycle
acceptance. Tokens are not printed in readiness output.

`application.tsr` builds an ordinary Router. Users can extend its routes and
later add separately configured web mounts. Only `/api` is registered for the
generated operations; its browser preflight permits POST with authorization,
content-type and x-tsrust-contract headers from the configured origin. Cookies
are not required. Responses use no-store and nosniff headers.

`node tools/verify-task-http.mjs` generates/checks the API, checks the process
entry, builds a separate native test entry, then drives the real named TypeScript
client through CRUD, validation, authorization/CORS, orderly drain and a second
process using the same temporary SQLite file. It runs outside the repository as
its working directory, with explicit migration paths. Its authenticated stop
route exists only in the test entry. `--run-only` reuses its built executable.
This verifier passes on Windows, including both native processes exiting 0 and
cleanup of its temporary database. It does not publish or import test data into
analytics. See [HTTP evidence](../../docs/milestones/task-http.md).

## Accepted repository behavior

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
policy and actual generated dispatch execution now pass the HTTP witness. The
built compiler generates and stale-checks the server configuration successfully; the installed
compiler has not been refreshed yet.
