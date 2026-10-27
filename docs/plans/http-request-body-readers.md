# HTTP request body readers

Owner follow-up, 2026-10-27. Complete after the async SDK section, before declaring
the backend HTTP API ergonomic/accepted. This does not revive frontend work.

Testing prerequisite: the owner explicitly wants std:testing instead of the older
executable assertion programs. Current test registration accepts synchronous
callbacks; target/testing-async-audit.log confirms an async callback is rejected.
Add proper async test/hook execution and CLI runner integration with resource/error
semantics preserved, then express the request-reader/native backend cases as normal
tests. Rust SDK cases use cargo test; HTTP cross-language checks use node:test and
only the minimum external server fixture. Keep benchmark drivers separate.

## Existing behavior verified in source

- Request.text and contextual c.req.text asynchronously read bounded UTF-8 text.
- Request.form / c.req.form parse application/x-www-form-urlencoded into fields,
  with a bounded body and 128-field limit.
- Request.multipart / c.req.multipart use the existing async Axum multipart parser,
  bounded body and 128-part limit. Reuse them; do not implement parallel parsers.
- An owned Request.body transfers an unread streaming Body; Body.next is async.
  Acquiring the stream performs no network I/O. Native string length is UTF-8 byte
  length, so do not assume JavaScript UTF-16 units when reviewing existing limits.
- Contextual body streaming has an ergonomic ownership gap: the focused current
  compiler check in target/http-context-body-audit.log shows c.req.body() transfers
  req out of Context, preventing the later c.text response. Add contextual transfer
  that preserves request metadata and Context usability without cloning the body.
- Request.json is missing. Context.json already exists as a response helper; keep
  request/response completion, signatures and documentation distinct.

## Coherent implementation and acceptance

Add an async JSON request API using existing checked JSON/schema decoding, not an
unchecked generic cast or a second parser. Settle no-type-argument JsonValue versus
explicit DTO/schema overloads using executable type witnesses. Preserve typed
decode failures, unsupported-resource rejection and borrowing restrictions: a
decoded view must not outlive the body storage. Generated RPC dispatch may retain
its specialized raw JSON codec path when necessary for ownership/performance;
ordinary routes should not need manual text-to-JSON plumbing.

Keep byte limits, UTF-8/media handling, malformed input and consumed-body errors
explicit. All readers consume the body once, including failure behavior; metadata
and response helpers remain usable. Test repeated/mixed reads, malformed/oversized
JSON, non-ASCII boundaries, form/multipart regressions and real async server/client
execution. Test contextual streaming followed by a response. Do not merely rename
text() or bypass validation to make a generic signature compile.

Complete compiler checking/lowering/native behavior, LSP signatures/completion/
hover/parameter help, builtins.json behavioral examples, formatted fixtures and
focused regression evidence together. Prepare matched TypeScript/Node benchmark
support and independent oracles; run representative untimed checks before the gate
and reserve full regressions/timing for the backend milestone.

XML is a separate parser-backed extension. Track its media types, encoding,
namespace/tree representation, limits and entity policy before selecting an
upstream parser. Do not claim XML support or introduce an unchecked text cast.
Generic REST/OpenAPI import, new encodings and a universal body-decoding framework
are not implied by this request. No XML runtime acceptance is claimed.
