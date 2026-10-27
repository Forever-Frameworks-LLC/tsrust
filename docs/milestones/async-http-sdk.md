# Asynchronous HTTP SDK foundation

2026-10-27, Windows x64. Supersedes the synchronous Rust transport groundwork at
0470ac5d. It does not complete async std:testing, request JSON convenience, original
UI dependency isolation, Cargo-cache migration or installation/update work.

The Rust generator now emits awaitable named calls, future-returning custom
transports and async reqwest HTTP. Credential providers may await refresh.
CLI description acquisition also uses async reqwest with a terminal Tokio runtime
boundary; no blocking HTTP feature is selected. Bounds, no redirects/retries,
environment header inputs, output ownership and retained wire profiles remain.
The generated Rust model tests cover optional/missing/null distinctions, safe
integers, numeric/literal/tuple handling, bad results, errors and Rust identifier
collisions. Initial tuple support is capped at 16 elements with generation rejection.

OpenAPI's default POST path is corrected to /api, matching the starter. The live
test derives its call URL from the specification, so matching two generated copies
cannot hide a wrong route. Custom mounts require application-owned specification
path/server adaptation; arbitrary OpenAPI import is not supported.

## Executed evidence

- target/async-http-sdk-tests.log: API contract tests 11/11 passed. The first Rust
  execution hit the unchanged 120s cold Cargo-build limit; the resulting executable
  ran directly and passed. No timeout was widened. The affected warm rerun in
  target/async-http-sdk-recheck.log passed Rust 2/2 and scaffold 3/3.
- The owner requested normal test libraries instead of standalone assertion
  programs. Generated Rust cases now run with cargo test using tests/client.rs.
  target/async-sdk-standard-tests.log records 2/2 outer Rust tests passing, including
  generated-crate test execution and default-HTTP compilation.
- Fresh actual CLI output: generated-projects/backend-async, with generated/client,
  generated/server and an explicitly generated generated/rust-client. Backend check,
  native std:testing greeting test 1/1 and native server build passed.
- target/async-sdk-http-normal-tests-final.log: node:test 1/1 passed. This runs the
  actual native backend; fetches live/saved TS/Rust descriptions; verifies OpenAPI-
  derived calls; builds an external Rust consumer with cargo test; checks header
  providers, unsafe URL/redirect/size rejection and compiler rejection of wrong Rust
  argument/result types. On one Tokio thread, two requests must both reach the
  server before either receives a response. All owned servers are stopped afterward.
- target/async-sdk-editor.log: real LSP entry/operation diagnostics clean; deliberate
  in-memory type error detected and cleared after repair, disk source preserved.

## Remaining testing cleanup

Native test registration is currently synchronous: target/testing-async-audit.log
shows an async callback rejected. Add real async std:testing/CLI execution before
converting the retained async backend executable assertions. Keep only necessary
external-process fixtures for language/process boundaries and benchmark measurement
drivers. Preserve historical evidence and ordinary compiler rejection tests.

Request text/form/multipart already exist. Checked JSON and contextual streaming
ergonomics are tracked in plans/http-request-body-readers.md. Installation/cache/
update source inventory and upstream research remain in the distribution plans.
No installed tool replacement, full matrix, performance measurement or remote
release/publication is claimed by this milestone.
