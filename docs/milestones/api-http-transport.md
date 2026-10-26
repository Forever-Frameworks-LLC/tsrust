# API HTTP client transport

Status: implementation in progress from `0075eb3e`; native dispatch is not yet
implemented. This section does not establish the end-to-end service prototype.

The generated package adds a separate `./http` export using the host Fetch API.
The plain entry keeps its dependency-free, DOM-free declarations. HTTP callers
provide an absolute endpoint and optional static/async headers or host Fetch
function. Requests post `{operation,input}` and identify the caller's checked
contract through `x-tsrust-contract`. This identity is not an exact-equality
compatibility policy; additive rolling-client support remains server work.

The transport resolves credentials per call, controls JSON/protocol headers,
omits cookies, rejects redirects, and never retries. Non-2xx structured public
failures survive; success envelopes on failing HTTP statuses reject. JSON MIME
and result codecs prevent an HTML fallback or malformed output from becoming
application data. Network exceptions are projected without URLs/headers/host
exception messages. Aborted calls report `api-cancelled` without asserting
rollback or native cleanup completion. Response text is buffered before parsing;
streaming memory limits and backend body limits are not claimed here.

String codecs now reject lone UTF-16 surrogates. Valid surrogate pairs remain
valid strings. This aligns JavaScript inputs with native UTF-8 JSON. Negative
zero becomes zero through ordinary JSON serialization.

## Current verification

`tools/verify-api-http.mjs` is called by the full client verifier and uses a real
temporary loopback HTTP server. It checks rotating auth, protocol headers,
structured errors, MIME/JSON/type failures, no redirects/retries, pre-aborted and
in-flight abort, disallowed-operation rejection and Unicode behavior.

The source runtime was assembled against the already accepted example manifest
in a disposable `target/api-http-runtime-*` directory and passed the loopback
checks with normal localhost permission. The sandbox network failure is retained
as a permission limitation. An initial test-cleanup duplicate end was fixed;
the repaired test passed. The later full generated-package run also passes the
positive HTTP Unicode witness. No native HTTP dispatch or mobile acceptance is inferred.

The first focused compiler integration command completed successfully in session `76127`:
`cargo test --offline --locked -p tsrust-compiler --test api_contract -- --test-threads=1`,
with `CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=1`, output
`target/api-http-focused.log`: five tests pass. The library/CLI build took about
ten minutes; observed CLI compilation used approximately 2,518 MiB private memory.
Source metadata and test assertions were completed while this first build was
running, so a final-source rerun of the same affected target is now live in
session `89309`, with output `target/api-http-final-focused.log`. Poll this handle;
do not launch another build or restart solely because its redirected log is quiet.

`node tools/verify-api-client.mjs` passes against the actual first rebuilt CLI,
including generated HTTP exports, DOM consumer type checking with native fetch
and AbortController, DOM-free plain consumers, the positive Unicode round trip,
all loopback failures/cancellation cases, output ownership and linked existing-app
consumption. Output: `target/api-http-client-first.log`.

Pending: final-source Cargo completion, repeat generated-package acceptance for
that final binary, local checkpoint and installed-consumer refresh. The full
verifier needs normal localhost and temporary junction permission on this Windows
host. Whole-project formatting passes twice with zero files/fingerprints changed;
`git diff --check` passes and owner edits remain
untouched. No broad regressions, performance measurements, public publication,
remote CI or interactive desktop control ran.

Fetch behavior was checked against [MDN's Fetch API guide](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch).
