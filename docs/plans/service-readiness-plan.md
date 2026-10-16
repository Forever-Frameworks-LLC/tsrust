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
  deliberately non-transferable. Shared mutable cache integration is still open.
- HTTP body, connection and shutdown limits are fixed. There are no scoped form,
  multipart, cookie, CORS, rate-limit or static-serving helpers.
- SQL has no versioned migration orchestration or drift detection. Transactions
  deliberately reject DDL; migration recovery must respect each driver's behavior.
- Console/JSON, environment access, UUIDs and exact dates exist, but there is no
  structured redaction/correlation/metrics integration or authentication primitive.
- Server.wait and shutdown exist, but no source signal API completes deployment
  shutdown. Offline package evidence exists for HTTP/TLS/SQL separately; it does
  not establish an integrated Linux service build or toolchain distribution.

## Implementation order

The combined S19/S20 regression and measurement milestone is accepted. Implement
retained handler state; migration recovery; observability and bounded HTTP
contracts; maintained-library authentication; and integrated deployment acceptance.
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
This read-only state section is accepted; shared cache and the remaining S21
requirements are still open. Continue implementation without a permission checkpoint.
