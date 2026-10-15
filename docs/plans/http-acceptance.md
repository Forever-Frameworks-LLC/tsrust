# S15 HTTP acceptance

Status: implementation and focused acceptance in progress. Do not treat this file
as final acceptance or as evidence of measured throughput.

Surface: Axum App routes/middleware, owned Request/Response/Body/Next/Server,
Reqwest Client, text/html/json/request/stream helpers. Named async handlers pass
directly. Captured application state is rejected. Fixed limits: 1 MiB bodies,
128 server connections, 30-second client deadline, five-second graceful shutdown.
See [HTTP guide](../guides/http.md) for public behavior and deferred scope.

Runtime loopback witnesses pass routing, middleware, streamed UTF-8 bodies,
404/405, response header mutation, shutdown, invalid routes/status/headers,
cumulative body limits, generic panic responses and nested task disposal.
Disconnect and stalled-shutdown witnesses verify exact-once child disposal.
Additional exact middleware-order and encoded-path witnesses are awaiting the
next runtime run.

HTTP exposed an S13 defect: an owned task whose body only read its environment
emitted a borrowed Rust receiver, incorrectly requiring Send-only HTTP bodies
to be Sync. Task invocation now consumes its environment; owned capture binding
metadata agrees with that receiver. All 23 async ownership integration tests pass,
including borrowed native views and real suspension/drop witnesses. HTTP codegen
also now applies the ordinary expression access wrapper to returned strings.

Whole-body text collection now appends borrowed transport chunks directly,
avoiding one allocation/copy per chunk. Streaming forwarding remains demand-driven.
No task scheduler redesign or claimed speedup accompanies these repairs.

Current readable loopback and schema-validated-body examples execute correctly.
HTTP catalog examples, editor checks, offline package export, full milestone
regressions and benchmark measurements remain pending. New HTTP benchmark
support has matched native/TypeScript sources, a checksum oracle and fixed
3-process/8-warmup/31-sample protocol; no timing has been recorded yet.

Closed tsrust-http-* logs are stored byte-exact in benchmarks/records.sqlite;
failed attempts remain alongside passing repairs. Active logs stay in /private/tmp
until closed, imported and hash-verified. UI work remains separate.
