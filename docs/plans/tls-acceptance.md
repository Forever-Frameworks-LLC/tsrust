# S16 TLS acceptance

Status: validation in progress.

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

Pending final compiler/editor/export and benchmark results. Runtime witnesses cover
valid private trust, default-root rejection, wrong hostname, expired/not-yet-valid
certificates, malformed credentials, mismatched keys, HTTP versions, repeated
requests, stalled handshakes, socket release and exactly-once nested task disposal.
The readable example is [HTTPS](../../examples/http/https.tsr).

The HTTPS benchmark retains the 3-process, 8-warmup, 31-sample protocol, with 128
sequential requests per batch and an independent checksum. Both sides use HTTP/2,
private CA verification and reused connections. Certificate loading is outside the
timed interval. Fixture/source/toolchain hashes accompany the original records.

## Limits

No mutual TLS, certificate reload, ACME, insecure verification mode, TLS over Unix
sockets, or configurable handshake/connection limits. Server certificate validity
and trust are checked by clients; the server does not claim to establish its own
public trust chain. Acceptance is on macOS; other operating systems are unmeasured.
