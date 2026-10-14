# Owned ThreadPool workers with cooperative cancellation

The active API is `ThreadPool`; the global `pool` singleton and the `Pool` name
have been removed. This section preserves the settled ownership and lifecycle
rules. Dedicated-worker channels remain separate work.

```tsrust
import { ThreadPool, join } from 'std:threads';

function main() {
  const jobs = new ThreadPool();
  const task = unwrap(jobs.task((): int => 42));
  console.log(unwrap(join(task)));
}
```

`new ThreadPool()` uses available CPU parallelism clamped to 2..256 workers,
with fallback 2, and outstanding capacity 1024. Explicit options require both
`workers: int` (1..256) and `capacity: int` (1..1000000). Construction returns the
owner directly and throws typed SpawnError (`range` or `spawn`) on failure.
There is no configure operation or hidden application-wide instance. Share one
owner through caller borrows or module-owned helper functions when needed.

`jobs.task(callback)` submits an owned zero-argument callback immediately; work
may queue behind busy workers. Admission never blocks and returns `full` when
capacity is exhausted or `closed` after shutdown. Completed results retain a
capacity slot until join or handle cleanup. Detached work releases its slot after
completion. `jobs.stats()` reports this instance only. `spawn` still starts a
dedicated OS thread and uses the same completion/cancellation handles.

`jobs.shutdown()` permanently closes admission and drains running and queued
work, including detached tasks. It is idempotent; it does not cancel work or
consume retained results. Create a fresh ThreadPool for a new lifecycle. Scope
exit performs the drain automatically. Pool owners remain unique, non-cloneable,
non-transferable to workers, and forbidden in containers. Caller functions may
borrow `ReadOnly<ThreadPool>`. Retained task handles remain joinable after their
pool owner leaves scope; scheduler references are reclaimed after all owners and
completion handles are released.

Cancellation is cooperative for dedicated and pooled workers. A queued cancelled
callback is skipped when scheduled; running callbacks poll cancellationRequested.
Join acknowledges completion with code `cancelled` if the request won the race.
Published completion wins over later cancellation. Native calls cannot be forcibly
interrupted, so cleanup can wait indefinitely for nonterminating callbacks.

Rayon 1.11.0 remains pinned. Owned callbacks/results/errors bridge to the backend;
there is no parallel iterator API. Nested joins yield to the same scheduler without
holding completion locks and preserve cancellation context.

Examples: `examples/worker-pool/main.tsr` uses defaults and cancellation;
`examples/scoped-pools/main.tsr` shows independent owners and borrowed submission.
The matched pool benchmark keeps construction outside timed batches and the
original 3-process, 8-warmup, 31-sample protocol. Its checksum remains 140.

## Prior accepted evidence

The following records describe the earlier shared/scoped API and remain immutable.
They do not establish acceptance of the ThreadPool migration; current evidence is
recorded separately below.


The current section passes 12 runtime tests, all 16 native thread integration
cases (15 original passes plus the repaired Pool-only storage rejection), 64
native declaration tests, 9 expression-fact tests and 3 package export tests.
The package test relocates JSON, shared-pool and scoped-pool examples and builds
each offline in debug and release with an empty Cargo home.

Editor acceptance passes checker agreement (2), the full typing-speed completion
matrix (3), editor consistency (5) and worker documentation/protocol tests (10).
The catalog examples execute after formatting in debug and release. Shared native
call-result facts now reach the editor, including PoolStats locals. Both namespace
aliases and independent Pool receiver methods have completion coverage.

macOS `leaks --atExit` reports 0 leaks / 0 leaked bytes for the 11-test cleanup
runtime corpus; the twelfth test independently proves two pool workers overlap
before either is joined. Weak scheduler references expire across 32 repeated pool
lifetimes. Captures, success values, typed errors, rejected admissions, cancellation,
panics and detached work have explicit disposal witnesses. This is bounded evidence,
not proof about all possible programs or nonterminating callbacks.

SQLite retains exact `tsrust-scoped-*.log` evidence, including failed attempts and
the affected repairs. The original native suite exposed missing Pool-only storage
validation; the repaired guard rejects it without changing ownership rules.
The initial macOS leak attempt lacked sandbox task-port access; the unrestricted
inspection of its own spawned test process succeeded.

Pooled timing is recorded in `benchmarks/threads/pool-join-2026-10-14T22-25-15-491Z.json`:
3 processes, 8 warmups and 31 samples, with checksum 140 for every sample. Median
native/Node-after-tsc times are 0.014625/0.070583 ms per eight-task batch. These are
small-task dispatch costs, not an application speedup claim. Dedicated startup and
scoped-join workloads passed current untimed checks. Their earlier timings remain
unchanged. The registration guard passes with current source fingerprints.


## Configuration ergonomics amendment

The shared singleton remains available. Module-scope `pool.configure({ workers,
capacity })` returns void; `new Pool({ workers, capacity })` returns an independent
owner. Both propagate typed SpawnError through normal exception handling, without
an unwrap call. PoolOptions supplies contextual integer checking and reusable
configuration values. Task submission and completion retain their Result types.

The native thread suite now has 18 cases. Focused amendment checks cover
ordered, once-only dependency initialization,
initialization failure before main, configuration limits, independent construction,
ownership rejection, aliases, completion and executable catalog examples. The
operation-only runtime selection regression covers configuration, statistics and
cancellation queries without a Thread value. Existing module-state checks pass.
Offline relocated packages are checked in debug and release.

The earlier pooled timing record uses the prior configuration syntax. The current
source has an untimed checksum verification record; no new timing claim is made.
Exact `tsrust-pool-options-*.log` records retain the failed attempts and repairs.

Release CLI and LSP consumers were rebuilt from `33c4d741` and atomically
installed with matching hashes. Both installed-CLI examples produced their
expected output. Restarting the primary VS Code server cleared all three stale
diagnostics; its Problems panel reports no problems. The secondary window was not
rechecked. Final release, installation and example logs are retained in SQLite.

## ThreadPool migration acceptance

Compiler acceptance passes all 18 native thread cases (17 initial passes and one
fixture repair), 64 native declaration cases, 10 module-state cases and 9
expression-fact cases. The fixture repair reads a scalar count before a boolean
short-circuit expression, preserving the existing ownership restriction. The
three package-export tests pass, including relocated debug/release builds with
an empty Cargo home.

All 12 runtime tests pass. Current macOS leak inspection reports 0 leaks / 0
leaked bytes across this corpus, including default construction, fresh independent
lifecycles, repeated scheduler disposal, cancellation and detached work. This is
bounded evidence, not a guarantee for nonterminating callbacks.

Current source verification is recorded in
`benchmarks/threads/pool-join-2026-10-14T22-58-41-368Z-verify.json`; native and
Node-after-tsc both produce checksum 140. Prior timing records remain historical.
Editor checks pass: checker agreement (2), editor consistency (5), native thread
hover/signature/catalog tests (10), and the affected typing-speed pool rows (3
harness tests). Catalog examples execute in debug and release after formatting.

Current measurement:
`benchmarks/threads/pool-join-2026-10-14T23-03-02-577Z.json`. Each engine retains
93 samples, with 3 processes, 8 warmups and 31 measured samples per process.
Every checksum is 140. Native/Node medians are 0.014875/0.068792 ms per eight-task
batch. Native is 1.7% above the prior 0.014625 ms median; this small difference
provides no basis for another optimization pass. These are dispatch
microbenchmarks, not application speedup claims. Original records are retained.

Exact `tsrust-threadpool-*.log` evidence and both new benchmark records are stored
in SQLite after byte-for-byte restoration checks. Release consumer refresh follows
the accepted source checkpoint.
