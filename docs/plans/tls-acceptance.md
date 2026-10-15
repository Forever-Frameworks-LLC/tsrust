# S16 TLS acceptance

Status: accepted on macOS, 2026-10-15. Source checkpoints: `b0656298` and
`7510b254`. Compiler/runtime/editor checks, offline exports, matched timing and
consumer refresh are complete.

## Delivered scope

The existing Axum-backed `std:http` API now accepts `app.listen(address, { cert,
key })` for HTTPS and `new Client({ ca })` for a client with explicit private trust
roots. Plain HTTP and zero-argument clients retain their behavior. `TlsOptions`
and `ClientOptions` are ordinary checked records with documented fields.

TLS options move into the lazy listener task immediately; dropping that task
releases its owned inputs. Client construction borrows CA options synchronously.
TLS startup validates PEM credentials and key matching before binding. Explicit
client roots replace default roots only for that client. Chain, hostname and time
verification remain mandatory. The runtime negotiates HTTP/1.1 and HTTP/2.

TLS connections use the existing connection task set, 128-connection cap, request
scope, cancellation and five-second graceful shutdown. Incomplete handshakes have
a five-second deadline and are interrupted immediately on shutdown. No detached
accept loop or independently owned cleanup path was introduced.

The direct tokio-rustls dependency was already present transitively through
Reqwest. Both lockfiles retain their existing versions. Native exports include it;
package management reserves its name alongside other managed runtime dependencies.

## Acceptance evidence

**59 affected compiler/runtime/export tests and 15 editor tests pass.** This
includes nine HTTP runtime tests, ten HTTP compiler tests, 24 async ownership tests,
12 Tokio lifecycle tests, three native-binding contract tests and one relocated
HTTPS export test that builds and executes in both debug and release. Editor
agreement, completion while typing, consistency, hover and signatures pass.
All HTTP catalog examples execute natively, and whole-project formatting is
idempotent. The S15 complete regressions remain the preceding full milestone;
this pass reran affected checks rather than the complete unrelated matrix.

Runtime witnesses cover
valid private trust, default-root rejection, wrong hostname, expired/not-yet-valid
certificates, malformed credentials, mismatched keys, HTTP versions, repeated
requests, stalled handshakes, socket release and exactly-once nested task disposal.
The readable example is [HTTPS](../../examples/http/https.tsr).

The HTTPS benchmark retains the 3-process, 8-warmup, 31-sample protocol, with 128
sequential requests per batch and an independent checksum. Both sides use HTTP/2,
private CA verification and reused connections. Certificate loading is outside the
timed interval. Fixture/source/toolchain hashes accompany the original records.

Median time per 128-request batch: **5.978666 ms native, 6.784292 ms Node after
TypeScript compilation** (about 1.13× native throughput by median duration).
Each engine has 93 measured samples; all checksums are 5,376. This is the first
TLS baseline. Cleartext HTTP/1.1 S15 timings are not a transport-identical
regression comparison. No further optimization was justified by this result.

Exact logs, initial attempts, consumer hashes and the acceptance summary are stored
in `benchmarks/records.sqlite` under `tsrust-s16-*`. Original verification and timing
records retain their `benchmarks/http/https-2026-10-15T18-27-42.241Z-verify.json` and
`benchmarks/http/https-2026-10-15T18-37-51.173Z.json` paths. The measured original also
has 186 queryable samples and two throughput summaries in the existing analytics
tables. Imports are restored byte-for-byte and verified before loose copies are
removed. SQLite integrity and foreign keys pass. The generic archive-wide path
validator still rejects 758 pre-existing absolute historical paths; no S16 record
has that issue, and historical bindings were left unchanged.

The installed compiler and both LSP consumers were atomically refreshed and hash
verified. VS Code was restarted; the HTTPS example shows the optional TLS signature,
current ownership help and no workspace problems. The shared Cargo target was
released to the B4 repair agent after builds and timings; it remains active for
that agent rather than being pruned during its validation window.

## Limits

No mutual TLS, certificate reload, ACME, insecure verification mode, TLS over Unix
sockets, or configurable handshake/connection limits. Server certificate validity
and trust are checked by clients; the server does not claim to establish its own
public trust chain. Acceptance is on macOS; other operating systems are unmeasured.
