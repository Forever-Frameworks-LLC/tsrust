# Current continuation: practical performance and product features

## Owner priority, updated 2026-10-14

The owner approved a bounded practical-performance review and a return to product
features. Extreme-nesting optimization is deferred. This supersedes earlier
instructions to finish every nesting audit, eliminate every quadratic path, or
achieve universal benchmark parity before S12. Historical measurements remain
valid for their original inputs; they are not current acceptance requirements.

Do not turn this continuation into an open-ended compiler cleanup campaign.
Fix correctness failures, crashes at practical depths, and measured costs that
block a representative workload. Keep existing depth witnesses and diagnostics.
Do not raise worker stacks or silently remove resource checks.

## Accepted work and current evidence

- S01–S11 are accepted within their documented scope. The
  [S11 acceptance](2026-10-13-s11-acceptance.md) retains its historical evidence.
- Parser/checker/emitter worklists, disposal, constructor and spread repairs are
  implemented across multiple checkpoints. Use the
  [pipeline history](../plans/compiler-nesting-pipeline.md) and
  [source layout](../guides/compiler-source-layout.md) for implementation details;
  old open-item statements must be checked against later commits.
- Schema `safeParse`/`safeParseJson` expose `{ success, data } | { success, error }`.
  Compiler narrowing, editor hover/completion, examples and workload migration are
  accepted. Do not restore the old Result-match surface.
- Program-scoped test registry: `7c7b5bc0`. Callbacks own eligible local captures
  and borrow module state only while invoked. Registry disposal precedes module
  storage disposal. [Design and acceptance](program-scoped-test-registry.md).
- Nested concat overflow: `fc82d9db`. Receiver and argument work is scheduled on
  the shared expression stack; earlier operand loans stay active in source order.
  Depth-100 string/array witnesses execute on normal stacks; depth 200 preserves
  the existing admission diagnostic. All 18 affected compiler tests, 10 editor
  agreement tests and 20 native/Node correctness cells pass.
- Allocation fixtures/adapters: `263ff644`. Seven migrated fixtures, 19 adapter
  tests and 189 native/direct-Rust/Node executions pass. Native allocation and
  disposal records are in `tsrust-allocation-syntax-acceptance-2026-10-14.json`.
- Rust warnings: `00f996c8`. Obsolete resolver entry points/import removed;
  test-only query facades remain test-only. Live generated-runtime APIs retain
  behavior and gain meaningful backend coverage. Workspace/all-target checking
  reports no warnings/errors; 219 affected test executions pass. VS Code reports
  No Problems. Evidence: `tsrust-rust-warnings-acceptance-2026-10-14.json`.

Original acceptance and failed attempts are stored in `benchmarks/records.sqlite`.
Registry, concat and consumer refresh records use respectively
`tsrust-program-registry-acceptance-2026-10-14.json`,
`tsrust-concat-acceptance-2026-10-14.json` and
`tsrust-consumer-refresh-2026-10-14.json`.

## Practical performance decision

See [the bounded performance review](../reviews/2026-10-14-practical-schema-performance.md)
for current representative timings, allocation evidence, comparability limits and
priorities. The historical 31-row S11 review has 27 slower native medians, not the
28 claimed in its prose. Some references omit validation work, return existing
objects, or construct simpler errors. They are handwritten references, not Zod
measurements and not a universal language-performance ranking.

Use fused `safeParseJson` when validating raw JSON directly. The separate owned
`safeParse(JSON.parse(...))` path intentionally constructs an intermediate dynamic
tree. Preserve both APIs and their ownership contracts. Do not remove safety checks,
add hidden clones/shared wrappers, or weaken reference observations to win a ratio.

Keep timing and implementation acceptance separate. No new optimization is
required solely because a specialized reference wins a microbenchmark. Profile
representative applications before changing general schema machinery.

## Deferred work and trigger to reopen

1. **Quadratic fragment assembly in extremely deep expressions.** Recorded example:
   roughly 4.8 seconds at 32,768 nested Set constructors. Programs containing many
   ordinary functions scale linearly in the recorded checks. Reopen for measured
   compile-time/memory impact in actual generated source or an explicit untrusted
   compiler-service requirement. This is not a blocker for S12.
2. **Quadratic nested array-literal spread checking.** Retain witnesses; reopen
   when representative source makes it material.
3. **Broader architecture audits and representation limits.** Preserve known
   limitations and diagnose practical failures. The historical audit inventory is
   not an instruction to exhaust every possible shape before feature work.

The match/default-cleanup milestone completed consolidated compiler (4,008 passed,
7 ignored) and LSP (679 passed, 1 ignored) regressions. Initial failures and affected
repairs remain in SQLite. Do not repeat this full pass for routine edits. Differential
suites require pinned Node v24.11.1 first on PATH.

## S12 owned workers

The owned `spawn`/consuming `join`/explicit `detach` slice is implemented across
compiler, runtime, editor and benchmark support. See [scope and acceptance](../plans/owned-thread-acceptance.md)
for the direct-handle lifecycle boundary and evidence. Broader S12 remains open:
Bounded channels and multicore throughput are accepted; restricted local
Rust/pool integration is implemented. See [bridge acceptance](../plans/rust-rayon-bridges.md)
for its supported boundary and current checks. Broader Rust package mapping remains
separate work. Do not repeat public spawn/join groundwork or treat untimed checks as timings.

Preserve already settled decisions:

- `spawn` starts immediately. `join` observes completion; otherwise owning-scope
  cleanup joins and discards completion. `detach` explicitly opts out. No hidden
  clone, automatic detach or implicit shared ownership. Global `unwrap` propagates
  the original Result error through ordinary try/catch.
- Local Rust modules and supported wrappers belong alongside S12 before worker
  APIs harden. Automatic lazy default pooling for explicit threaded interop is
  approved; evaluate Rayon and reuse compatible pools. Direct calls stay direct.
  Queue/admission/shutdown contracts remain material design work. Arbitrary
  crates/macros/traits/lifetime-heavy APIs are not implicitly authorized.
- Async tasks are lazy, uniquely owned and single use. Calling transfers inputs;
  awaiting consumes. Start/cancel/timeout direction is approved. Never-started
  tasks warn; abandoned started jobs error. Cancellation alone does not settle a
  job or guarantee source finally execution. Detailed combinator contracts remain
  open; `std:futures` naming remains a recommendation.
- Cache reads borrow. Explicit clone creates independent supported values.
  Expiration occurs during operations; outstanding used borrows block mutation.

Later dependency order remains S13 async checking, S14 runtime/task lifecycle,
S15–S16 HTTP/TLS, S17–S18 SQLite/SQL pools, S19 cache, S20 queues, S21 integrated
service acceptance. This review does not implement or accept those features.

## Work discipline and consumers

Follow `AGENTS.md` and [development workflow](../development-workflow.md). One
agent works in this tree. Inspect status and preserve owner UI/interop plans.
Use Git checkpoints, not archives or snapshots. Format the entire project before
source checkpoints; preserve tokens, idempotence and executable fixture behavior.

Reuse `/private/tmp/tsrust-gate100-editor-target` with `CARGO_INCREMENTAL=0` and
`CARGO_BUILD_JOBS=1`. Keep normal worker stacks. Preserve exact original evidence
in SQLite, verify restoration/integrity, then remove closed loose copies. Prune
obsolete test/native outputs without deleting owner editor caches.

Five executable consumers are the installed CLI/LSP, the workspace override
`target/editor/debug/tsrust-lsp`, and shared-target debug CLI/LSP. Consumer refresh
now uses formatter source checkpoint `9fb8036f`, with atomic installed
release replacements and matching SHA-256 hashes. The primary VS Code server was
restarted and its workspace reports No Problems. The second `tsrust_test` window
was not rechecked in this pass; reload it if it still uses an older server.
Request both `--bin tsrust` and `--bin tsrust-lsp` when selecting Cargo binaries
explicitly.

The current pass adds value-producing matches, scalar literal patterns and named
record variants. See [match acceptance](../plans/match-values-acceptance.md).
It also makes worker cleanup the default for `spawn` and exposes global `unwrap`;
see [default cleanup](../plans/default-worker-cleanup.md). Compatibility spellings
remain available. Family checks and all three matched workload measurements pass.

Owned `ThreadPool` instances replace the global `pool` singleton and generic
`Pool` name. `new ThreadPool()` supplies system-based defaults; explicit options
set workers and capacity. Constructor failure throws SpawnError without unwrap.
Task, stats, shutdown, cancellation and cleanup retain the accepted ownership
rules. A shut-down instance cannot restart; a new instance has an independent
lifecycle. See [pool acceptance](../plans/worker-pool-cancellation.md). Migration
compiler/runtime/editor and offline-export acceptance passes, with zero leaks in
the cleanup corpus and current matched benchmark measurement. Release consumers
are installed and hash-verified; the restarted primary VS Code server reports
No Problems. Both installed-CLI examples produce their expected output.

Required next S12 work: dedicated-worker channels. Preserve owned transfer and
typed completion. Cancellation remains cooperative, with join acknowledging
completion; there is no forced thread termination. ThreadPool container storage is
rejected until lifecycle support exists. No async/deferred keywords or parallel
iterator API were added. The full family regression remains separate from this
focused pool/cancellation acceptance.

## Empty-delimiter formatting acceptance

Checkpoint `9fb8036f` keeps empty `{}`, `[]` and `()` inline, including gaps
containing blank lines. Comments retain their previous formatting. Acceptance:
29 formatter tests, 240 lexable fixtures / 84 callable contracts, 5 LSP formatting
tests and 8 fingerprint guards pass. Whole-project formatting changed 65 files
and refreshed 37 token-preserving fingerprints; a repeat changed zero files or
fingerprints. Installed release formatting is idempotent and its native probe
prints `0`. The primary editor server was restarted with checking enabled.
Evidence is stored under `tsrust-empty-delimiters-*.log` in SQLite. Unrelated full
regressions were not repeated.

## S13 async ownership continuation — 2026-10-15

S13 compiler/editor support is accepted: lazy `Task<T>`, consuming `await`, eager
argument/default evaluation, retained ReadOnly/Mutable loans, generic transfers,
source errors and normal finally handling. Readable examples live in
`examples/async-tasks`. Full regression evidence and the matched lazy-task
measurement are in [async acceptance](../plans/async-tasks-acceptance.md).

The next async section is S14 Tokio scheduling/I/O and started-task lifecycle.
S13's local future driver supplies no scheduling, timer or cancellation API.
Dropping a future releases owned resources without promising source finally
execution. Returning a child task retaining borrowed data is currently rejected
with an actionable source diagnostic; await it and return its value instead.
Nested owned tasks remain supported. Preserve these boundaries and the existing
thread ownership/cancellation rules when integrating Tokio.

## S14 Tokio completion — 2026-10-15

S14 adds owned scheduling with `Task.start()`, required Job settlement, cancellation,
deadlines, delay/yield, race/allSettled and bounded asynchronous channels. Compiler,
runtime and editor acceptance passes; see [Tokio acceptance](../plans/tokio-tasks-acceptance.md)
and [usage guide](../guides/tokio-tasks.md). The active editor was restarted with
current Task documentation and no open-document diagnostics.

Keep `await yield()` as settled. No external pause/resume or priority API was
requested. Cancellation remains cooperative; dropping suspended futures releases
resources without promising source finally execution. Dedicated threads, pools
and Rust/Rayon bridges remain separate. S15 HTTP is the next roadmap dependency;
networking and a blocking-work adapter were not added in S14.


## S15 HTTP completion — 2026-10-15

S15 is accepted: Axum routes/middleware, owned request/response/body/continuation/
server, reusable Reqwest clients, validated bodies, demand-driven forwarding,
disconnect and bounded graceful shutdown. See [acceptance](../plans/http-acceptance.md)
and [guide](../guides/http.md). Source af0cbba8 includes the final async disposal
repair; release CLI and both LSP paths are refreshed and VS Code restarted.

Consolidated acceptance: 4,111 compiler tests and 700 editor tests pass, seven
existing compiler tests ignored. HTTP measures 5.39 ms native versus 7.99 ms Node
per 128 sequential loopback requests. Async review repaired owned receivers,
retained loans, temporary disposal and multi-input task lifetimes; it removed one
body-chunk copy without redesigning scheduling. Exact evidence remains in SQLite.

S16 TLS listener configuration and certificate fixtures are next. Shared handler
state/providers, producer streams, WebSocket/SSE and UI runtime integration remain
separate. Preserve explicit ownership, cooperative cancellation and cleanup limits.


## S16 TLS completion — 2026-10-15

S16 is accepted: `app.listen(address, { cert, key })`, `new Client({ ca })`,
mandatory certificate/hostname/time verification and shared HTTP/TLS cleanup.
Zero-argument Client and cleartext listeners retain their behavior. Source
checkpoints are `b0656298` and `7510b254`; see [TLS acceptance](../plans/tls-acceptance.md).
All 59 affected compiler/runtime/export tests and 15 editor tests pass. HTTPS
measures 5.98 ms native versus 6.78 ms Node per 128 requests over HTTP/2 reuse.
Installed CLI/LSP consumers are refreshed; VS Code shows current TLS help and no
problems. SQLite holds original logs, failed attempts, hashes and all 186 timing
samples. No full unrelated regression matrix was repeated after S15.

The B4 repair agent has the exclusive validation window for
`/private/tmp/tsrust-gate100-editor-target`: `CARGO_INCREMENTAL=0`,
`CARGO_BUILD_JOBS=1`, Node v24.11.1 first on PATH, loopback escalation when needed.
TLS has no remaining builds, timings or consumer refreshes. Preserve the active
shared target until B4 finishes. The UI briefs remain owner work. S17 SQLite is
the next standard-library roadmap item; no work on it started in this pass.


## S17 SQLite and B4 workspace repair — 2026-10-15

S17 is accepted at `91e6b937`, with indexed editor completion repaired at
`b255009e`. B4 is merged at `e33af7db`; its missing callable access-summary entry
caused the workspace panic visible even when an empty/comment-only file was open.
The full workspace now checks without panic or errors. The remaining `unused-task`
warning on an awaited greeting is a pre-existing false positive, verified with the
old installed compiler; do not describe it as intentionally discarded work.

See [SQLite acceptance](../plans/sqlite-acceptance.md) for exact checks, consumer
hashes and measurement provenance. New examples are `examples/sqlite/people.tsr`
and `examples/sqlite/rollback.tsr`. Compiler ownership rejects closing a borrowed
connection, committing with live transaction statements, handle cloning and reuse.
Runtime checks cover scope/error rollback, failed commit, SQLite automatic rollback,
strict data conversion and lock cleanup. Borrow provenance through `unwrap` is fixed
for existing byte views too.

All affected compiler/runtime/export checks and 18 editor checks pass. Installed
CLI and LSP binaries are refreshed; the active VS Code server was restarted and
current SQLite hover verified. Matched prepared transactions measure 0.031084 ms
native versus 0.105708 ms Node per 128 rows; versions differ, this is an in-memory
whole-stack baseline. SQLite retains exact evidence and all 186 timing samples.

UI identity feasibility now has a separate managed worktree and the quiet
check/emit/rustc/format window. It uses the verified `b255009e` release CLI; any
compiler dependency requires a minimized witness and coordinated ownership first.
Preserve the five untracked owner UI plan files. S18 async SQL remains pending;
this pass adds no ORM, database pool, migrations or async SQLite adapter.
