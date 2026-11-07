# CPU work, request latency and server resources

The headline example is a CPU rendering handler: POST a varying ID, compute a
96 × 64 Mandelbrot tile with up to 256 iterations per pixel, then return the
checksum/escape count as JSON. Tsrust, TypeScript after tsc, and independently
written Rust execute the same f64 arithmetic. A Python complex-number oracle
prepares expected outputs outside timing. Servers never read those oracle outputs.

All engines use one server process and one executor worker. Tokio's worker count
is fixed with TOKIO_WORKER_THREADS=1; Node executes its JS kernel on one event-loop
thread. This is a single-worker comparison, not a multicore scaling claim.
One and 16 keep-alive connections are measured separately with the common client.

The original protocol remains three processes, eight warmup batches and 31 timed
128-request batches per engine/case. Request start-to-complete-body durations
supply 11,904 observations for nearest-rank p50/p99. They are not batch percentiles.
The closed-loop, same-machine client can bottleneck and 16 connections introduce
queueing; these are benchmark latencies, not production SLA estimates.

A separate warmed 31-batch pass measures server CPU time and RSS. CPU/req is
total process CPU milliseconds divided by completed requests. macOS ps time has
10 ms resolution; RSS is sampled every 50 ms. Peak sampled RSS includes server
runtime/shared pages, excludes the client, and is not heap usage or a guaranteed
high-water mark. Resource polling never overlaps throughput/latency samples.
CPU/RSS sampling currently requires macOS. The compiler/server sources remain
portable. No startup, compilation or oracle preparation is timed.

From the repository root:

```sh
TSRUST="$PWD/target/debug/tsrust" CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 node benchmarks/http/requests-run.mjs --cpu --verify
TSRUST="$PWD/target/debug/tsrust" CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 node benchmarks/http/requests-run.mjs --cpu --prepared
python3 benchmarks/http/cpu-oracle.py --check
node --test benchmarks/http/cpu.test.mjs
npm run store:ingest --prefix benchmarks/runtime
```

Source/protocol/oracle hashes are pinned in cpu-manifest.json. Results retain only
aggregate request percentiles, sample counts, CPU/request and peak sampled RSS
alongside existing minimal measurement histories. Raw requests/resource samples
are disposable after exact import verification. Public selection lives in
website/tools/performance-runs.json. Existing I/O controls below are reused;
adding this CPU example does not rerun their baselines.

# HTTP request comparisons

The current suite compares **Tsrust**, **Node.js after tsc**, and an independently
written **Rust Axum/serde/SQLx server**. It runs four scenarios at both **one and
16 keep-alive connections**:

- GET `/text`: return UTF-8 `héllo 🦀`.
- GET `/json`: construct and serialize an `{id, label, value}` record per request.
- POST `/parse`: read/validate a JSON record, append `!` to its label, increment
  its value and serialize the result. IDs and values vary across the batch.
- POST `/sqlite`: read/validate a JSON ID, run a parameterized SQLite query, and
  serialize the row. The private in-memory database contains 128 distinct rows.

One common Node HTTP client drives all three servers. Each batch completes 128
requests and checks status, content type and the entire response body against an
independent expected byte sequence. Invalid JSON/types/Unicode, invalid IDs and
missing database rows are verified before timing. Request payloads, database
creation/seeding, builds and server startup are outside measurement. Response
encoding, request parsing, database queries, transport and client verification are
inside it. These rates measure the complete loopback stack, not an isolated
server's maximum throughput. The SQLite test uses warm in-memory reads and one
connection; Tsrust/Rust use SQLx asynchronously, Node uses built-in synchronous
SQLite with a prepared statement. SQLite versions are recorded per engine.

From the repository root, with Rust 1.96.1 and Node v24.11.1 prepared locally:

```sh
TSRUST="$PWD/target/debug/tsrust" CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 node benchmarks/http/requests-run.mjs --verify
TSRUST="$PWD/target/debug/tsrust" CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 node benchmarks/http/requests-run.mjs --prepared
```

The first command builds release servers offline and verifies real requests.
`--prepared` reuses servers only when source/compiler identities still match.
Omit `--prepared` to rebuild. Add `--engines=native` to verification and timing
commands, including `--cpu`, to refresh Tsrust while reusing cached Node.js and
hand-written Rust measurements. The default selects all three engines. Selection
controls server execution; preparation still builds all three servers, and every
measurement uses the common Node HTTP client. Results record the selected engines.
Timing keeps the original **three processes × 31
samples after eight warmups** (93 samples per engine/workload), rotating server
order and waiting for a quiet machine. All servers bind only to loopback port
zero and are shut down after each process. The Rust counterpart links the same
prepared Axum/Tokio/SQLx/serde artifacts as Tsrust; generated Rust is never
relabeled as hand-written Rust. Builds share the repository Cargo target.

`requests-manifest.json` pins protocol/source identities. `requests-contract.mjs`
prepares independent expected responses. Run guards with
`node --test benchmarks/http/requests.test.mjs`. Measurement JSON is disposable:

```sh
npm run store:ingest --prefix benchmarks/runtime
```

Verify the imported medians, all 93 samples, source identities and SQLite
integrity before removing raw JSON. Accepted HTTP selections live alongside the
other performance-page selections in `website/tools/performance-runs.json`.
Language/library summary counts remain separate from these eight HTTP cases.

# Legacy HTTP loopback workloads

`--readers` selects paired JSON/schema and contextual stream request handling.
Each of 128 iterations posts `{"value":40}` and the two-byte UTF-8 string `é`,
checks both responses, and contributes 40 + 2 to the independent 5376 checksum.
Native routes use checked `req.json()` plus a schema and `req.body()` plus text
collection, retaining their Context for responses. Node uses strict UTF-8,
JSON.parse and an explicit integer/shape check. Both enforce a 1 MiB body limit.
The workload measures the combined loopback stack, not parser-only throughput.
Run `node benchmarks/http/run.mjs --readers --verify` for one untimed batch;
measurement retains three processes, eight warmups and 31 samples. Current source
identities are in readers-manifest.json. Both native and Node-after-tsc passed
the untimed batch on macOS arm64 at source 1ca135b1, checksum 5376 each.
No timed measurement is claimed yet.

Run with Node v24.11.1 first on PATH and TSRUST pointing at the verified compiler:

```
node benchmarks/http/run.mjs --verify
node benchmarks/http/run.mjs
```

Each batch makes 128 sequential keep-alive GET requests to its own loopback server,
checks status 200 and body `42`, and sums to the independent oracle 5376. Native
uses Axum/Reqwest; TypeScript uses Node HTTP/fetch after tsc. This measures the
combined client/server stack, not isolated server throughput or multicore scaling.
The timing protocol is three processes per engine, eight warmups, 31 samples,
alternating engine order. Verification checks one untimed sample per engine.

Both servers bind port zero and close after sampling. The legacy runner emits disposable JSON records. The current comparison runner
above stores execution measurements in benchmarks/measurements.sqlite; legacy
records are not silently substituted for the new HTTP protocol.

TLS: `node benchmarks/http/run.mjs --tls --verify` verifies the matched HTTPS
workload; omit `--verify` for the original 3-process, 8-warmup, 31-sample protocol.
Both implementations trust only the fixture CA and reuse connections. The Node TLS
counterpart uses `node:http2` with a persistent session and an explicit private CA, matching
the native client/server HTTP/2 negotiation.
Certificate files are read before timing. Run from the repository root.

`--service` measures 128 in-process authenticated form boundaries: SHA-256,
equal-length token comparison, explicit UTC second flooring, duplicate-preserving URL-form decoding and fixed
structured JSON output. Expected checksum is 128 × (32 + 7 + 2 + 1) = 5376.
Node uses its crypto primitives, Request and URLSearchParams after TypeScript
compilation. Native uses checked bytes, HTTP forms and JSON. Both use valid UTF-8;
Node's permissive malformed-percent behavior is not claimed as parity. This is
not network, database, session lookup or distributed-service throughput. Those
boundaries retain their existing matched workloads and integrated acceptance.
`--verify` builds debug and runs one untimed batch. Measurement retains the
three-process, eight-warmup, 31-sample release protocol; timing remains deferred.

The clock boundary floors a pre-epoch instant to -1000 milliseconds. Native input
retains nine fractional digits; Node Date input has three. Both lie in the same
second, so the independent expected floor is equal without claiming nanosecond
Date precision in Node.

`--context` selects the Hono-style context and middleware workload. It retains
configuration in Router application state, copies its body into a typed request
variable, reads that variable in the handler, and sets headers before and after
delegation. Every request checks both headers and the `42` response body.
The Node counterpart uses an independent explicit context/continuation over Node
HTTP; it is not a measurement of the Hono npm package. Both sides use the existing
3-process, 8-warmup, 31-sample, 128-request protocol. `--verify --context` is one
untimed correctness batch. Source identities are pinned in context-manifest.json.
The v3 state workload passes matched untimed verification; its exact record is in
SQLite. Timing remains unmeasured. Earlier v2 records retain their original
closure-only semantics and are not v3 performance baselines.

`--producer` selects bounded application production and body collection. Each of
128 bodies contains six seven-byte chunks through a capacity-one channel; the
independent checksum is 128 * 6 * 7 = 5376. Node uses a highWaterMark-one object
stream and validates the same complete body. This is in-process streaming, not
network throughput or a claim of identical scheduler costs. The usual timing
protocol is registered; run `--producer --verify` for one untimed batch.

`--static` selects streamed static-file HTTP, with an independent Node core HTTP
and filesystem stream counterpart. Both serve the same `value.txt` containing
UTF-8 `42`; every response checks status/body and contributes 42 to the independent
128 * 42 checksum. Filesystem metadata, open, streaming, loopback transport and
body collection are inside the batch. Build/startup are outside it; repeated
requests use a warm filesystem cache. This is not backend computation throughput.
The harness seeds the common external asset root and pins both source hashes in
static-manifest.json. It preserves 3 processes, 8 warmups, 31 samples and 128 requests
per batch; `--static --verify` uses one untimed correctness batch. This new epoch
pins Node v24.19.0; older workloads retain their original v24.11.1 requirement.
Run `node benchmarks/http/static-registration.test.mjs` for source/protocol guards.
Both counterparts pass a separate one-batch untimed check with checksum 5376;
timing remains pending. Registration/correctness is not performance measurement.


## Client-owned upload comparison (source prepared; unvalidated)

`requests-run.mjs --uploads` uses the existing HTTP runner with native,
handwritten same-stack Rust and Node-after-tsc controls. Six workload rows cover
owned bytes, true body streams and bounded channel producers at concurrency1/16.
Every128-request batch cycles64/4096/65537-byte public payloads. The destination
checks every byte in order; the common client independently checks the complete
length/checksum response using BigInt arithmetic. Rust and native channels have
capacity1; Node uses demand-driven Readable production, not an identical queue
representation. No stream is accumulated to enable replay.

The original3processes,8warmup batches and31timed128-request batches remain.
Request latencies provide p50/p99;31separate resource batches measure CPU/request
and RSS without overlapping timing. One executor worker is used. Build, startup
and common-client payload preparation are excluded. Upload buffer generation,
frontend client transport, backend validation and response consumption are timed.
These rows have no accepted correctness or measurements yet.

After a root-granted55-minute complete validation/timing lease:

```sh
node --test benchmarks/http/client-uploads.test.mjs
TSRUST=/absolute/shared-target/release/tsrust CARGO_TARGET_DIR=/absolute/shared-target CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 node benchmarks/http/requests-run.mjs --uploads --verify
TSRUST=/absolute/shared-target/release/tsrust CARGO_TARGET_DIR=/absolute/shared-target CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 node benchmarks/http/requests-run.mjs --uploads --prepared
```

Use pinnedNode24.11.1 andRust1.96.1. CPU/RSS sampling requires macOS; untimed
correctness can run on other supported targets. No prototype/admission results
are ingested as accepted public measurements.

Form and multipart upload modes use the same original 3×31-after-8 protocol and semantic binary corpus. Form peers base64-encode one ordered `payload` field using maintained codecs and URL encoders. Multipart peers send one binary file part with backend boundary/framing. Payload generation, base64/encoding, and server-side decoding/validation are included; only outer fixtures are excluded. Node uses URLSearchParams and FormData/Blob (Blob snapshots its input); the Rust/native multipart paths move byte owners. These additional four workload rows remain unmeasured.
