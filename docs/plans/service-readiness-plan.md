# Practical service readiness: S21

## Inventory at the S19/S20 checkpoint

S15/S16 already provide Axum routes and middleware, owned request/response/body
values, streaming forwarding, TLS, verified clients and bounded shutdown. S18
provides owned SQLx pools, prepared parameters, typed rows and checked transactions.
S19 supplies local LRU/TTL and a bounded serialized loading pattern; S20 supplies
bounded producer/consumer composition using the existing Channel and task APIs.
Reuse their accepted ownership, cancellation and cleanup evidence.

The following are concrete gaps, not accepted service capabilities:

- HTTP callbacks reject all retained state. SQL pools are transferable owners but
  have no public shared handle. Local caches are deliberately non-transferable.
  Shared handler state needs an explicit checked lifetime and concurrency contract;
  simply removing the callback rejection would be unsound.
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

Finish the combined S19/S20 regression and measurement milestone first. Then
implement shared handler state; migration recovery; observability and bounded HTTP
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

Docker CLI and the desktop-linux context are installed. On 2026-10-16 the Docker
socket was absent and the daemon was not running. Linux/container acceptance is
pending actual execution; macOS or generated Docker files do not substitute for it.
