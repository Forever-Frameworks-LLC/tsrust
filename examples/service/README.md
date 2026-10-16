# Authenticated notes service

This S21 acceptance service combines a file-backed SQLite database, versioned
migrations, bounded HTTPS handlers, structured redacted events, opaque sessions,
forms, binary multipart uploads and graceful Unix signal shutdown. It is an
executable contract witness, not a general authentication framework.

Set `DATABASE_URL=sqlite:/absolute/path/service.db?mode=rwc`, `SERVICE_ADDRESS`,
`SERVICE_ORIGIN` (an exact HTTPS origin without a trailing slash), `SERVICE_CERT`
and `SERVICE_KEY` (PEM files). Generate `SERVICE_ADMIN_TOKEN` as 32 random bytes
encoded as 64 lowercase hexadecimal characters. Keep this bootstrap credential
out of logs and command lines. Run from the repository root so migrations resolve.
The Linux runner copies the migration assets into the same relative location.

`POST /session` authenticates `Authorization: Bearer TOKEN` and issues a random
one-hour `__Host-session` cookie with Secure, HttpOnly, SameSite=Strict and Path=/.
Expiry explicitly floors the current instant with startOfSecond before exact
millisecond conversion; the session can expire less than one second before a
full elapsed hour. Exact date conversion never silently truncates precision.
Only SHA-256 digests of session tokens reach SQLite. Session expiry is checked on
every request; logout uses `DELETE /session` and deletes that digest. At most 1024
unexpired sessions are retained. This is one service identity, not users, password
hashing, OAuth, JWT, permissions, refresh tokens or key rotation. Bootstrap tokens
must have high entropy. SHA-256 alone must never hash user passwords.

Authenticated `GET /notes` reads one note. `POST /notes` accepts exactly one
URL-encoded `body` field up to 4096 UTF-8 bytes. Duplicate fields and malformed
encoding reject. `POST /upload` accepts one binary multipart `attachment` and
returns its byte count; it deliberately does not persist client-named files.
Mutating cookie-authenticated requests require the exact configured Origin.
CORS never reflects arbitrary origins or combines wildcard origins with credentials.
`GET /` serves one embedded HTML asset with a restrictive CSP; there is no path
lookup, directory listing, content sniffing or filesystem fallback. `/health`
returns readiness only after migrations and required-column queries succeed.

Limits: 64 connections, 32 active handlers, 64 KiB total request body, five-second
handler deadline, three-second TLS handshake and five-second shutdown grace.
A global 100-arrival fixed one-second window returns 429 when full; it is not
per-IP, per-user or distributed rate limiting. Concurrency overload returns 503.
A handler deadline returns 504 and requests cancellation. Non-yielding native work
is still cooperative. Response streaming after headers has no new total deadline;
shutdown still bounds connection draining. The kernel backlog is outside the
application connection count. Multipart collection preserves bytes and order;
there is no resumable or disk-streaming upload API.

Logs contain only server-generated correlation IDs, fixed operation/site names,
phase, status and elapsed milliseconds. Headers, bodies, query strings, credentials
and raw database errors never cross the logging boundary. Fixed-cardinality counters
report started/completed/failed/tasks/incomplete on shutdown. Hard cancellation can
leave an incomplete trace; source `finally` is not promised for dropped futures.
The separate observability witness returns a trace through an owned child task to
verify parent/child correlation. SQL errors are reported as a fixed source site
and 503, without exposing SQL or credentials. Logs are not a general tracing SDK.

`std:signals.listen()` registers SIGINT/SIGTERM before readiness. A signal stops
accepting connections, drains/cancels handlers, releases retained pools and flushes
the bounded telemetry channel. Dropping a listener does not restore Tokio's
process-wide signal disposition. Non-Unix targets return an explicit unsupported
error. The service requires Unix for this deployment contract.

After implementation, run `tools/check-service.mjs` with `TSRUST` and the shared
`CARGO_TARGET_DIR`. `--linux` exports the package, builds inside the pinned Rust
container with an empty Cargo home and no network, then checks real HTTPS, sessions,
restart persistence and both shutdown signals. The runner owns and removes its
container, scratch databases, keys and native executable. SQL driver/migration
acceptance is reused; this does not repeat the three-driver suite.

Implementation and acceptance status are recorded separately in the service
readiness plan. Neither this README nor the presence of a runner claims it passed.
