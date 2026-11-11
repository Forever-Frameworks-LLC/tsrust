# Realtime backend admission — dependency-only gates passed

Base965de83f; no public Tsrust/compiler/LSP modifications. Rust dependency
resolution, archive materialization and metadata-only gates have passed; no
build, Rustc invocation, native test or timing has run here. Backend choice and
streaming/upgrade ownership remain unaccepted.
`admission.json` records primary tagged sources, licensing/MSRV/features,
known advisory references, defaults and outstanding target/closure admission.

The candidate uses Axum0.8.9 SSE/ws, matching Tokio Tungstenite0.29, Tokio1.53.3,
and the existing Rustls0.26.6 stack. The standalone manifest keeps SSE independent
of optional WS/TLS activation. Node24.11.1 plus ws8.18.3 is a private comparison
control; optional addons are disabled explicitly and recorded. There is no
manual WebSocket codec, automatic reconnect, facade message queue or hidden pool.

`src/main.rs` exercises backend SSE framing, absent/explicit keepalive, bounded
producer backpressure and body-drop→producer settlement. `src/websocket.rs`
exercises text/binary bytes, standard owned split during a pending receive,
automatic pong, malformed/UTF-8/explicit frame/message limits, write backpressure
and close races. A separately joined cancellation control aborts an actually Pending send and verifies peer EOF after both halves drop. Small capacities, bounds and timeouts are explicit fixtures.
Default backend config is observed separately; ordinary defaults are preserved.

The private network server runs auth/origin fixture hooks before upgrade. Its
explicit16-session admission permit survives through socket destruction; an
active registry waits for actual callback destruction, including failed upgrade.
Cancellation selects against the entire read/write loop so a blocked send cannot
hide shutdown. A caller-supplied grace bounds backend close/drain before dropping
the socket; this models reuse of B10's existing shutdown policy, not a new default.
Public integration must use the existing B06 task scope/cleanup and B10 listener
owner. Axum's internally spawned upgrade callback alone is insufficient proof.
The network SSE row is a direct stream backend floor; typed channel cancellation
is a separate control and is not claimed to prove a generated facade bridge.

Two SSE semantics need measurement and a reviewed decision before public code:
Axum source appears to omit data for an empty string and to prefix CR and LF
separately. Fixtures retain exact backend bytes AND an independent WHATWG parser.
The runner reports semantic differences prominently; it does not quietly declare
an empty-event/CRLF facade accepted or replace the backend protocol implementation.
`node-control.mjs` encodes canonical SSE lines independently.

## First exclusive admission lease — NOT run

Work from this managed tree; use pinned Node24 and Rust1.99.1, `TSRUST_REALTIME_NODE=/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node`, one absolute
`CARGO_TARGET_DIR=/Users/seancotter/Desktop/dev/learning/tsrust/target`,
`CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=1`. First inspect live processes.
Rust package materialization/lock generation is complete under its dependency-only
lease. Node materialization and all builds still need separate authorization;
no guessed cached hashes or unreviewed global dependency update is allowed.

1. Materialize exact candidate archives and Node package; resolve standalone
   locks while preserving compatible versions already selected by the accepted
   HTTP graph. Review every added/changed package and feature before proceeding.
   No root production manifest/lock changes belong to this admission task.
2. Capture `cargo metadata --format-version=1 --offline --locked
   --filter-platform=aarch64-apple-darwin --features websocket-rustls` as JSON.
   Build standalone `Cargo.toml` with `--release --offline --locked
   --features websocket-rustls --message-format=json`, under `/tmp/tsrust-watch.py`
   (5GB/30minutes). Capture terminal success and actual artifact paths.
3. `python3 closure.py METADATA ARTIFACTS Cargo.lock CLOSURE` verifies normal/build
   identities, runtime optimization, archives against lock SHA256 and license data.
   Build-time authored identity is embedded in the binary and checked before execution. Runtime and binary features must exactly match metadata. Build-time proc-macro/build-script artifacts may be unoptimized; runtime
   packages need an optimized artifact. Do not weaken guards to accept mismatch.
4. `python3 node-package.py WS_TARBALL package-lock.json node_modules/ws NODE_PROOF`
   independently verifies the installed Node library bytes against npm lock integrity.
   Review full resolved-closure licenses, source archives, current advisories and
   selected-platform features. Known historical fixes are not a full audit.
   Regenerate only reviewed authored source identities after any source repair.
5. With pinned Node, `node profile.mjs --artifacts=ARTIFACTS --closure=CLOSURE
   --node-proof=NODE_PROOF --output=RESULT` runs correctness then matched backend timing/resource groups.
   Rust Cargo.lock is now an actual input; Node lock remains pending. No runner
   performs a build or package fetch.

The runner reuses the existing HTTP CPU protocol and telemetry:3 processes,
8 warmups,31 timed batches,128 complete events/echoes per batch,1/16 clients,
one server worker. A separate31-batch resource pass samples RSS every50ms; it
never overlaps timing. Nearest-rank p99 uses11,904 request observations per engine/case, not batch
percentiles: WS send to full typed echo, or SSE request to its full six-event
body. Individual SSE dispatch latency remains a later public-workload gate. RSS delta is whole warmed process memory, not heap/leak
proof. Raw partial/failed groups are persisted before moving to the next group.
The prepared matrix totals142,848 request-latency samples and36 resource rows;
watchdog fit must be estimated from the first completed group, preserving full
3×31 protocol. Split complete case groups with `--only=sse-c1` (or text/binary-c1/c16) if
necessary; each selection retains all3×31 samples. Do not reduce samples.
No prototype timings are ingested as public generated workload measurements.

TLS entry point: with websocket-rustls, binary `tls-client URL [CA.pem]` uses
verified default roots or an explicitly supplied test CA. Node server uses
`TSRUST_REALTIME_CERT/KEY` fixtures. Required future controls: trusted CA succeeds;
default roots reject that private CA; wrong hostname/untrusted roots reject.
No verification bypass exists. Certificate fixture materialization and these
cross-process TLS controls remain pending; the profile runner does not claim them.

The driver adds untimed Rust/Node shutdown with an active paused reader and an
observed Pending transport send; it requires server exit and active0. Failed server
startup kills and joins the child. The explicit4096-frame fixture budget is not a public cap.

Before target/public acceptance still require remaining full/half disconnect and
client-close/server-close/cancel races,
header/auth/origin rejection under real HTTP upgrade, exact TLS controls,
Windows/Linux/macOS execution, relocated export and optional-unused activation.
No WIP checkpoint until root grants formatter/checkpoint lease.


## Dependency-only gate (2026-11-06)

Actual Cargo.lock now preserves compatible accepted B10 pins (d6392571), including
rustls-webpki 0.103.16. Eight offline/locked actual metadata graphs cover minimal
and websocket-rustls on aarch64/x86_64 macOS, x86_64 Linux and x86_64 Windows.
Minimal graph: 44 active identities including this package (43 Windows); full:
88 (87 Windows). Union90 registry identities:69 exact common B10 checksums and21
new identities, recorded in admission.json. No production/canonical lock changed.
B11 uses webpki-roots0.26.11 forwarding to1.0.11, not B10 webpki-root-certs.

All90 reachable archives match lock checksums; every extracted source file matches
archive bytes and unexpected source files are rejected. All have SPDX/license
and notice files. Maximum declared MSRV1.85;20 crates do not declare MSRV, so
metadata cannot prove their compiler compatibility. RustSec database26fd2d43
whole-lock audit found zero vulnerabilities and zero warnings. Root Finder
.DS_Store metadata was preserved; no advisory sources ignored or deleted.
Actual Cargo caches lack some .cargo-checksum.json sidecars; closure.py now uses
checksum-pinned archive bytes directly, retaining optional sidecar validation.
No Cargo build/check/test, rustc, native/library execution, protocol/network
fixture or profile was run. Node ws package materialization and actual compiled
features/TLS defaults/cross-platform behavior remain unproved.

## Public prepared-native protocol hook (unmeasured)

The default invocation and Rust/Node controls remain private admission. Supplying
`--native-proof=/absolute/prepared.json` adds exactly the public native engine to
the same six network cases, 3 processes, 8 warmups and 31 samples. It does not
compile anything. The stable public server source is `benchmarks/http/realtime.tsr`.
Private protocol KATs and pending-send controls still belong to their original
backend engines; the public registered suite supplies facade ownership and TLS
coverage. No public acceptance is inferred from private controls.

The prepared JSON has kind `prepared-public-realtime`, absolute `binary`, `source`,
`manifest`, `rustSource` and `artifacts` paths, and `files` mapping each path to its
actual SHA-256. `artifacts` is the actual terminal successful Cargo JSON stream for
the already emitted release package. The runner verifies exact normal binary
source/manifest/executable identity, optimized profile, shared target placement,
selected `http-websocket` feature, and every input before/after each whole group.
Preparation must emit the registered public source with the current verified CLI
and record that source/compiled package relationship; a hand-authored proof is
not evidence that compilation occurred. No locks, binary copies or new build
framework are created by this hook. Historical private source seals/results stay
authoritative for their original run; this changed runner needs a refreshed root
crate identity before another invocation.

Public measurement remains false until this three-engine mode actually completes.
SSE framing differences remain the explicitly selected Axum contract, not a claim
of universal JavaScript encoder byte parity.

Public mode writes `<output>.common.json` using the existing measurement ingestion
schema. Engine labels are native, rust-hand and node-ws (JavaScript ws control,
not TypeScript-after-tsc). Each timing row has its exact process/case CPU/RSS
counterpart and all 3968 request latencies. Stable IDs are
`http_realtime_{sse,text,binary}_c{1,16}`.

`--control-reference=/absolute/reference.json` reuses unchanged passed Rust/Node
controls and times only native. The reference has kind
`realtime-control-reference`, absolute `result` and `scope`, their actual SHA-256
values as `resultSha256`/`scopeSha256`, original `timestamp` equal to the successful
historical scope start in ISO UTC, and actual `binarySha256`/`nodeProofSha256`.
The original terminal scope, original compiled identity, dependency/artifact
closure, Node package proof, fixtures, control sources and complete timing/resource
matrices must match. A source or binary change rejects reuse. The public driver
and reviewed metadata can change without pretending the original control binary
was rebuilt. Existing private mode still checks its current full source seal.

Reused histories are exported separately as `<output>.controls-reference.json`
with their ORIGINAL timestamp; the new common file contains only newly measured
native execution rows and its current timestamp. Ingest both files together only
when the old control history is not already present. Never relabel old durations
with a new run date. Combined protocol reports can include both for comparison.
Public String materialization/revalidation differs from the direct Rust control
that retains Utf8Bytes; timing comparisons do not assert identical conversion work.
