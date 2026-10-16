# Practical service readiness: S21

## Inventory at the S19/S20 checkpoint

S15/S16 already provide Axum routes and middleware, owned request/response/body
values, streaming forwarding, TLS, verified clients and bounded shutdown. S18
provides owned SQLx pools, prepared parameters, typed rows and checked transactions.
S19 supplies local LRU/TTL and a bounded serialized loading pattern; S20 supplies
bounded producer/consumer composition using the existing Channel and task APIs.
Reuse their accepted ownership, cancellation and cleanup evidence.

The following are concrete gaps, not accepted service capabilities:

- Retained read-only HTTP state passes focused acceptance below. SQL pools remain unique owners retained by handlers; local caches remain
  deliberately non-transferable. Shared cache integration now has focused acceptance below.
- HTTP body, connection and shutdown limits are fixed. There are no scoped form,
  multipart, cookie, CORS, rate-limit or static-serving helpers.
- SQL migration implementation and recovery are described below. Transactions
  continue to reject DDL; driver-specific recovery remains explicit.
- Console/JSON, environment access, UUIDs and exact dates exist, but there is no
  structured redaction/correlation/metrics integration or authentication primitive.
- Server.wait and shutdown exist, but no source signal API completes deployment
  shutdown. Offline package evidence exists for HTTP/TLS/SQL separately; it does
  not establish an integrated Linux service build or toolchain distribution.

## Implementation order

The combined S19/S20 regression and measurement milestone, retained handler state,
shared cache and migration recovery are accepted. Continue observability and bounded
HTTP contracts, maintained-library authentication and integrated deployment acceptance.
Each section includes native ownership/failure witnesses, compiler/editor agreement,
readable executable examples and relevant untimed benchmark support before checks.
The final witness must combine these sections in an authenticated persistent service,
including overload, cancellation, shutdown and independent resource observations.

Keep mutable cache operations serialized, configuration immutable, pool shutdown
explicit and asynchronous loans checked. Do not add a general-purpose shared
mutable language model or claim JavaScript reference semantics. Durable jobs,
external caches, arbitrary crate compatibility and a polished demo remain outside
this acceptance.

## Environment observation

Docker Desktop is running on 2026-10-16 and reports Linux aarch64. Disposable SQL
containers exercise database boundaries; integrated Linux service acceptance still
requires an actual service build and execution. macOS or generated Docker files do
not substitute for it.

## Retained HTTP state implementation

Routes, middleware and fallback callbacks can own read-only retained data. The
emitted request future owns an Arc of the callback before borrowing its receiver;
the request task may suspend with that read loan. Registration proves that captures
are owned, invocation does not mutate or consume them, and all reachable payloads
are supported concurrent data. Caller loans, invocation-local returned loans,
program state, nested callbacks/tasks, local caches and request resources reject.
This permission is limited to direct HTTP callbacks. Ordinary return-escape checks
remain unchanged. There is no public general shared-mutable type.

Configuration examples cover three concurrent requests, captured middleware and
fallback, completion cloning and explicit request-input preparation. The checker
conservatively treats `clone` as potentially throwing; preparing all URLs before
starting jobs avoids abandoning already-started work on that path. This section
does not weaken task accounting or claim that string cloning throws in practice.
The pool example transfers a SQL pool into a handler and borrows it during requests;
server shutdown drains requests before releasing the handler owner. Independent
native Drop observation checks retained state through success and cancellation.
HTTP errors contain independent owned scalar data and now permit checked cloning.

The matched retained-state workload uses 128 loopback requests per batch and retains
configuration in both engines. Its full timing protocol remains three processes,
eight warmups and 31 samples. Focused verification uses one untimed batch; S21 timing
is deferred until its feature-family milestone.

A build-memory audit found that native support builds removed `CARGO_BUILD_JOBS`
along with artifact-changing Cargo settings. The scheduling limit is now preserved;
profile, target, compiler and wrapper overrides remain filtered to protect cache
identity. Existing SQL evidence is reused except for the explicitly required
three-driver run following the pool-lifetime change.

Focused evidence so far is stored byte-exactly in SQLite under `tsrust-http-state-*`:

- `prepared.log`: four compiler cases and two editor cases pass, including native
  routes/middleware/fallback; the database test stays ignored by default.
- `drop-repaired.log`: independent retained-state destruction passes for success
  and cancellation. `stateless.log`: existing loopback/streaming/shutdown passes.
- `three-drivers.log`: the explicit runner passes all five tests, including each
  driver's rollback/cancellation, exact domains, SQL/TLS coexistence and retained
  pools. Its disposable containers were removed and an empty filtered inventory
  verified cleanup.
- `job-limit.log`: scheduling inheritance and artifact override filtering pass.
- Original failed checks and their repaired focused passes remain available. No
  compiler/LSP family regression was repeated for this section.

The untimed retained-state workload passes with checksum 5,376 in both engines.
`benchmarks/http/retained-state-2026-10-16T12-03-43.758Z-verify.json` is retained in
SQLite; no timing claim is made. Installed CLI and both LSP consumers were replaced
atomically and hash-verified. The active VS Code server was restarted; workspace
diagnostics report no problems. The installed CLI checks both new examples.
This read-only state section is accepted. Shared cache acceptance follows below;
remaining S21 requirements are open. Continue implementation without a permission checkpoint.

## Shared cache integration

`std:cache.SharedLruCache<K,V>` explicitly shares one bounded LRU/TTL cache through
`share()`. `getCloned` and `peekCloned` return independent owned values or undefined;
`set` and `remove` preserve the existing owned-entry and rejected-input contracts.
Local `LruCache` borrowing and resource support are unchanged. Shared values require
transferable, independently clonable data, so opaque resources, endpoints, callbacks
and tasks cannot be retained. A handle is not independently clonable. Final-handle
destruction disposes its entries, while returned copies remain independently owned.

A synchronous mutex serializes individual operations. Arguments are evaluated before
locking, including expressions that access the same cache. No user callback or await
runs under the lock. Read access permits synchronized resource methods; it does not
freeze shared entries. Separate reads/writes are not a transaction, atomic update,
or single-flight loader. Capacity counts entries, not bytes, and expiry remains
operation-driven. A poisoned lock fails closed instead of repairing unknown state.

Focused compiler/native acceptance passes: 12 backend checks, nine local/cache
integration checks including all 22 catalog examples, and three shared-cache checks.
The shared example proves nested-copy independence, explicit sharing, concurrent
writes, retained HTTP route/fallback access, invalid TTL input recovery and checked
constructor catches. Native Drop observation proves final-handle and independent-copy
release. Five editor checks cover both local behavior and aliased shared methods,
completion, hover and parameter help. Constructor failure witnesses exposed a missing
exception checkpoint that previously left catch bodies unchecked; native constructors
now record their own failure point without entering ordinary call dispatch.

Original failures and repaired focused passes are retained under
`tsrust-shared-cache-*` in SQLite. The matched scalar workload passes checksum 97,936
in both engines with one untimed batch. Its timing remains pending at the S21 family
milestone. This compares serialized native operations with a single-threaded Node
Map LRU; it does not measure parallel throughput. Migration recovery acceptance follows below.

Shared-cache CLI/LSP consumers are atomically refreshed and hash-verified; installed
CLI checks pass and the restarted active editor reports no problems.

## SQL migrations and recovery

`Pool.migrate(directory)` is a lazy Task that borrows its pool. It loads trusted
UTF-8 files named `VERSION_description.sql`, rejects duplicate/nonpositive/overflowing
versions and unsupported reversible/no-transaction files, then uses the existing
SQLx backend's migration ledger and checksums. Files are deployment inputs, not
implicitly embedded assets. No ordinary transaction DDL/session-control permission
was widened. Examples and recovery instructions are in `examples/sql-migrations`.

PostgreSQL and MySQL use advisory migration locks; SQLite uses per-migration
transactions. Deploy with one migration owner before request serving. On success
the unlocked connection returns to its pool. Errors close the connection through
tracked cleanup and await that close. Cancellation registers the same cleanup even
when its enclosing operation is dropped. An independent second pool must acquire
the migration lock after cancellation; the witness also verifies that cancelled
DML is not applied twice. Source loading happens before acquiring a connection.

SQLite migration acceptance requires an actual backing file, checked through its
own database metadata. Parsing a URL alone is insufficient: encoded filenames and
SQLite URI parameters can conceal an ephemeral database. Rejection leaves the
existing pool usable for ordinary queries. This also avoids a
SQLx URL-rendering panic exposed by the initial attempt; the original failure is
retained rather than counted as acceptance.

Changed/missing applied SQL reports history drift. Out-of-band schema changes are
separate: startup queries verify required columns, and the witness deliberately
removes a column to prove this check fails. No whole-schema/index/constraint
comparison is claimed. PostgreSQL/SQLite roll back the failed migration, while
MySQL DDL can persist partially with a dirty ledger. The fixture restores its exact
pre-migration schema before an operator-style ledger reconciliation and retry;
there is no automatic repair, skip, downgrade or database backup facility.

The matched workload performs 32 already-applied SQLite history checks per batch.
Native uses the complete migration path; Node-after-tsc uses a bounded fixture
history/checksum implementation. Initial DDL is outside timing. Untimed checks use
a debug native build; the later S21 timing milestone retains three release-mode
processes, eight warmups and 31 samples. This is not concurrent deployment or
cross-driver migration throughput evidence.

Focused migration acceptance passes: source validation, compiler ownership/arity
checks and two editor checks; the explicit SQL runner's seven tests, including all
three drivers, SQL/TLS coexistence, retained HTTP pools and the relocated package
built offline with an empty Cargo home; and the final migration-only witness for
independent lock release and ephemeral SQLite rejection. A runtime-only runner
option preserves earlier evidence when strengthening migration witnesses. Database
suites remain ignored by default. The original receiver-transfer error, SQLx URL
panic and incorrect MySQL DML expectation are retained alongside their repairs.

Exact records are byte-verified in SQLite under `tsrust-migrations-*`. Both engines
return checksum 1 for the untimed history workload; no migration timing is claimed.
Installed CLI and both LSP consumers are atomically refreshed and hash-verified.
The active editor restart settled with checking enabled and no problems. Cleanup
removed 11 obsolete executable paths (1,003,916,064 logical bytes), the failed
fixture's scratch database, and both runs' disposable database containers. Current
consumers and reusable dependency libraries remain. Continue observability next;
S21's full service and measurement acceptance remain open.

## Remaining service implementation — acceptance pending

The remaining sections are now implemented together rather than validated as
individual package gates. `App.configure(ServerOptions)` controls request bytes,
connection and handler counts, handler/handshake/drain deadlines and a global
fixed-window request rate. Form and multipart collection retain order and reject
malformed/oversized input; binary uploads remain bytes. `std:crypto` uses pinned
getrandom, sha2 and subtle dependencies for OS randomness, SHA-256 and equal-length
constant-time comparison. `std:signals` registers Unix SIGINT/SIGTERM before
readiness and preserves checked exclusive receive loans.

`examples/service` integrates HTTPS, persistent notes, hashed one-hour sessions,
revocation, bounded session count, cookies, exact-origin CORS, an embedded static
allowlist, bounded telemetry and graceful signal shutdown. The observability
witness includes an owned child task and independent correlation assertions.
`examples/service/acceptance.tsr` combines source contracts, observability and the
service into one native compilation. `tools/check-service.mjs --linux` is the
actual offline container runner, not evidence that it has passed.

The service boundary workload adds matched TypeScript/native hashing, form decode
and structured output with an independent checksum. Existing HTTP/TLS, retained
state, shared-cache and migration workload evidence is retained. New timing is
not yet measured. S19/S20 and three-driver SQL acceptance are not being rerun.

Compiler checks are used during implementation. Targeted acceptance is batched
only after the complete sections, following the owner's 2026-10-16 correction.
Do not reinterpret these implementation notes as accepted runtime/editor/Linux
behavior; append actual outcomes after the consolidated run.

### Local integrated acceptance

The combined executable now passes on macOS: source security contracts, owned-child
correlation, TLS authentication, hashed-session revocation, database persistence
across restart, form/multipart validation, exact-origin CORS, body limits, static
allowlisting, redaction and SIGINT/SIGTERM shutdown. Targeted runtime witnesses pass
for crypto known answers and HTTP overload/deadline disposal with an independent
Drop counter. The new compiler ownership/arity cases pass. Focused editor help
passes after correcting public HTTP factory return types from backend Result to
the actual unwrapped Request/Response type.

Integration found that exact millisecond conversion correctly rejects a fresh
nanosecond-resolution `now()`. `startOfSecond` now makes service expiry rounding
explicit; pre-epoch flooring, idempotence and preservation of the original instant
are checked in the combined executable. Exact conversion remains unchanged.
The service's expiry can be less than one second shorter than a full hour.

The original adapter array error, mistaken Content-Length witness and HTTPS client
write failure are retained alongside repairs. The oversized-header witness now
uses Expect: 100-continue, so an early 413 is observed without racing a rejected
payload write. Existing awaited-allSettled `unused-task` diagnostics remain a
known false positive; this is not intentionally abandoned work.

Linux/offline toolchain distribution, remaining focused editor agreement and the
new untimed matched workload still require their consolidated results. No new
performance measurement or full regression pass is claimed.
