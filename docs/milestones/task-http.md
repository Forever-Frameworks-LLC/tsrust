# Native SQLite task HTTP application

Status: focused Windows native HTTP acceptance passes. Repository
native correctness/persistence evidence remains in `task-repository.md`.

The application reuses the generated Context-aware dispatcher and existing HTTP,
SQL, schema, signals and futures libraries. No parallel RPC server is added.
`application.tsr` exposes an ordinary owned Router factory with bounded resource
settings and API routes. `main.tsr` reads explicit environment configuration,
supports optional TLS files, restricts cleartext binds to loopback, migrates the
configured SQLite database, emits typed readiness and drains after its signal
listener completes. TLS/process lifecycle has source checking only so far.

The application-owned access policy compares a bearer token digest through the
existing constant-time helper. Browser origin matching is exact; native clients
may omit Origin. Preflight requires the configured origin and POST, and allows
the generated client's content/auth/contract headers. No cookies are required.
Response middleware sets no-store and nosniff. Private Context resources remain
outside the four public operation schemas. Domain failures remain tagged values;
transport/access/protocol failures use generated ApiError behavior.

## Completed checks

- Task API generation succeeds with four operations, contract
  `fe81030f7c957d23a19a3b466e5a0e1a84fc9320e4fba0772938e37284a73fa7`,
  `target/task-http-generation.log`.
- `api generate --check` succeeds, `target/task-http-generation-check.log`.
- Process entry project checking succeeds, `target/task-http-main-check.log`.
- Test entry project checking succeeds, `target/task-http-named-stop-check.log`.
- Whole-project formatting completes with a stable zero-change second pass;
  no benchmark fingerprints change. JS verifier syntax checking succeeds.

The test entry uses a reusable callback that invokes a named async stop handler
with its retained sender explicitly readonly. The first anonymous async capture
was correctly rejected for consuming retained HTTP state. No sharing/ownership
guard was weakened. The production entry has no fixture stop route.

## Completed native acceptance

Completed session `29751`, output `target/task-http-acceptance.log`:

```text
Task HTTP: generated named CRUD, validation, access policy/CORS, parameterized SQL, orderly drain and process-restart SQLite persistence passed.
```

The verifier generated and stale-checked the four-operation client/private server,
checked the production entry, compiled the native test executable, then completed
both real native processes with exit 0. All CRUD, UTF-8 title bounds, SQL-looking
parameter values, cursor/domain failures, unauthorized/invalid-origin requests,
strict/stale requests, preflight/headers and persisted rows matched their expected
values. The test ran from the temporary database directory, using an absolute
migration path; the second process reused that same database. Its known temporary
directory was removed successfully after both processes drained. No fixture
process remains live and no test data was imported into analytics.

TLS, production signal delivery, concurrent cancellation/drain, packaged execution,
web mounts and frontend targets are not established by this fixture. Those goal
requirements remain pending. This is correctness evidence, not a timing run.

## Native witness protocol

`tools/verify-task-http.mjs` is ready to drive real generated named calls against
the test executable. It uses a disposable SQLite file containing a space in its
path and a working directory outside the repository. The explicit migration
directory is reused on restart. Coverage includes CRUD, trimmed byte-limited
titles, Unicode boundaries, SQL-looking parameter data, cursor bounds, tagged
failures, denied auth/origin, strict/stale requests, CORS preflight, response
headers, orderly drain, process restart persistence and cleanup of its own files.

The completed run used one active native Cargo consumer, the shared compiler
target/native cache, and incremental compilation disabled. Retain its completed
checks and rerun affected cases after changes. Never infer TLS, production signal
or package acceptance from this HTTP witness. Serving web mounts and frontend
clients remain separate goal requirements.
