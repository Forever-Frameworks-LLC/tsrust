# S15 HTTP acceptance

Status: accepted, 2026-10-15. Compiler, runtime, editor, examples, matched
measurements and consumer refresh are complete.

## Delivered scope

Axum-backed `std:http` provides App routes/middleware, owned Request/Response/Body/
Next/Server, a reusable Reqwest Client, and text/html/json/request/stream helpers.
Named async handlers pass directly. Typed body validation uses the existing schema
API. Standalone services await Server.wait(); controlled shutdown consumes Server.
See [HTTP guide](../guides/http.md) and [examples](../../examples/http).

Fixed limits are 1 MiB cumulative body reads/forwarding, 128 server connections,
a 30-second client deadline and five-second graceful shutdown. Streaming forwards
frames on demand. Captured handler state, configurable limits, producer streams,
WebSocket/SSE, request providers and UI integration are outside this slice. TLS
listener configuration and certificate fixtures remain S16.

## Correctness and ownership

Acceptance combines the complete milestone runs with focused repairs, without
counting reruns twice: **4,111 compiler tests pass**, with seven existing ignored
tests; **700 editor tests pass**. The compiler milestone covered 339 targets.
The original run's three failed targets remain in SQLite alongside passing repairs:
a shutdown assertion now verifies socket rebinding, a native-binding test no longer
expects std:http to be nonexistent, and a package-management cold-build timeout
passes in an isolated rerun without increasing its timeout. An affected async CLI
check also timed out during concurrent builds and passed on rerun.

Six HTTP runtime witnesses cover routes, 404/405, middleware order and early return,
encoded parameters, header validation, cumulative limits, streamed bodies, generic
500 responses, disconnect, stalled shutdown and cancellation of Server.wait().
Nested request tasks and owned resources are disposed exactly once. Seven source
integration tests cover examples, catalog examples, ownership rejection, runtime
cache reuse, standalone waiting/cancellation and consecutive body reads. Body and
Next cannot be duplicated; listening consumes App; pending reads retain their loans.
Offline HTTP packages build and execute in debug and release after relocation.

Editor completion, checker agreement, consistency, hover, signatures and aliases
pass. All HTTP catalog examples execute natively. The full editor milestone was
supplemented with focused Server.wait() coverage and final HTTP/agreement checks.
Formatting checks cover the entire project, token preservation and idempotence;
source execution fixtures also verify formatted behavior.

## Async review

HTTP exposed three shared async defects, now repaired:

- Owned tasks whose bodies only read their environments emitted borrowed receivers,
  incorrectly requiring Send-only bodies to be Sync. Tasks now consume their owned
  environments, with matching capture metadata.
- Lazy native operations lost retained loans when leaving a synthetic lowering
  block. Those loans survive until binding, awaiting or disposal. Discarded lazy
  tasks release temporary loans at statement and comma-expression boundaries.
- Returned tasks borrowing multiple inputs omitted native lifetime arguments.
  Task signatures now name those lifetimes using the existing signature planner.

All 24 async ownership tests and 12 Tokio lifecycle tests pass. Additional HTTP
witnesses reject overlapping body reads while allowing consecutive reads, chained
responses and discarded lazy reads. Ownership rules were not relaxed.

Whole-body text collection appends borrowed transport chunks directly, eliminating
one intermediate allocation/copy per chunk. No scheduler redesign, hidden clones
or claimed measured speedup accompanies this change. Cancellation remains
cooperative; it does not promise source finally execution or interrupt CPU work
that never suspends.

## Measurements and evidence

Untimed native Axum/Reqwest and Node-after-tsc HTTP verification passes with checksum
5376. The matched workload performs 128 sequential keep-alive requests per batch;
it measures the combined client/server stack, not isolated server or multicore
throughput. Timing retains three processes, eight warmups and 31 samples per process.
Final source af0cbba8 produced 93 samples per engine for each workload:

- HTTP: native median 5.390333 ms, Node-after-tsc 7.993125 ms per 128 requests,
  approximately 1.48 times faster in this combined loopback workload. Native
  process medians are 5.340791–5.417042 ms; Node 7.903750–8.645500 ms.
- Lazy await: native 0.008583 ms, Node 0.143625 ms. S14 native was 0.009625 ms;
  current process medians span 0.008541–0.010375 ms. This variability does not
  establish an optimization gain.
- Scheduled batch: native 11.866791 ms, Node 0.783625 ms for 4,096 tiny jobs.
  Against S14, native is 5.4% slower and Node 4.4% slower; their ratio changes
  about 1%. One run does not isolate a runtime regression from machine variation.
  Native remains about 15 times slower for tiny scheduled jobs. Node microtasks
  do not provide the same worker parallelism. No speculative scheduler change
  was justified by this signal.

Both async workload hashes match S14. All observations match their independent
checksums. Original protocol and samples are retained; these are not multicore
throughput measurements. Records are stored in SQLite under:

- benchmarks/http/loopback-2026-10-15T17-58-12.019Z.json
- benchmarks/async/lazy-await-2026-10-15T17-58-36.548Z.json
- benchmarks/async/scheduled-batch-2026-10-15T17-58-51.418Z.json

The dirty-tree flag records acceptance documentation and unrelated UI handoffs;
compiler source was committed. The final release CLI and both configured language
servers were atomically refreshed. VS Code was restarted, open HTTP files have
no Tsrust diagnostics, and Server.wait hover was checked in the editor. One stale
rustc dead-code diagnostic remains in the editor; current runtime tests directly
exercise that method and final Cargo builds emit no warnings.

CLI SHA256: 57fd08aad9358fa367434bb5f5f036885b0148a7c0994939addea8f230aceca3.
LSP SHA256: ee730502c2c1d368704c828643d2a208765f115dcd11d9d9877cc83334dd8bd9.

Source checkpoints include 8f4154f3, 2ba94b6d, c3443972, 838ae928, 562c7dc1 and
af0cbba8. Closed tsrust-http-* logs and benchmark records live byte-exact in
benchmarks/records.sqlite. Original failures are retained. New evidence uses
relative record paths; historical absolute paths are not migrated by this gate.
UI work and its handoff files remain separate.

Final storage validation passes SQLite integrity, foreign keys and all 33,311
pre-final-import blob hashes/sizes; subsequent imported records were restored and
compared byte-for-byte. Closed loose records were removed after verification.
Obsolete outputs in the shared Cargo target were pruned from roughly 21 GB to
269 MB, retaining current release consumers. Disk space is 80 GiB free. The next
Cargo build will be cold; active editor-managed workspace caches remain intact.

## Application-produced bodies — focused acceptance, 2026-10-16

`streamChannel(channel, status = 200)` transfers a `std:futures` `Channel<Bytes>`
into a response. The existing bounded channel supplies backpressure; the bridge
starts no task. All external senders dropping ends the stream after queued chunks
are drained. Response disposal closes the receiver and wakes pending sends with
`closed`. Capacity bounds items, not bytes: applications must bound chunk size.
The normal response `text()`/`next()` read limit remains 1 MiB.

Producer tasks retain their existing owners. In particular, a request-scoped job
cannot outlive its handler merely because a streaming response was returned.
Application-owned producers can feed responses, and caller-owned producers can
feed directly consumed responses. Callback-owned streaming production is not yet
supported. Producer errors require an application protocol; sender disposal alone
signals EOF. No error frames, SSE protocol, or detached task semantics are implied.

The executable example is `examples/http/producer-stream.tsr`. Matched bounded
production/collection support is `benchmarks/http/producer-stream.{tsr,ts}`;
`node benchmarks/http/run.mjs --producer --verify` performs untimed verification.
This workload measures in-process production and collection, not network throughput.

Focused native backpressure/EOF/disposal checks, formatted source execution,
wrong-payload/use-after-transfer rejections, live hover and four-mode import
completion pass. The compiler now rejects mismatched async channel send payloads
before Rust generation. The example joins its producer in `finally`, preserving
cleanup on read/assertion failure. Native and Node pass one untimed 128-body batch.
Original failures and repaired checks are retained in SQLite as `tsrust-producer-*`;
the matched record is `benchmarks/http/producer-stream-2026-10-16T16-33-24.653Z-verify.json`.
No SQL/task-lifecycle behavior changed; accepted three-driver evidence is reused.
No full regression or timing pass was run.
