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

## Async testing implementation boundary

Implement async callbacks inside the existing registry, suite traversal and error
reporter. A registered callback owns its environment; an invocation lends mutable
program/registry references to a future whose lifetime ends before invocation
returns. Do not require shared program state to be Send, cloned, or static.
Use the existing multithreaded native async runtime at the synchronous terminal
runner boundary. One scope must cover setup, all tests and teardown; creating a
runtime for each callback would invalidate beforeAll resources. Reuse the existing
runtime with a child scope when runTests executes from async main; drain that child
before returning without cancelling unrelated application jobs. Pure synchronous tests
must retain their dependency-free runtime path.

The runtime implementation lives in stdlib/testing/runtime/async.rs and the
stdlib/futures runtime. Compiler registration adaptation is a separate pending
part of this section; these internals alone do not make public test/it/hooks accept
async functions. Preserve ordinary synchronous callbacks and typed error rendering.
Check inline async arrows, named functions, aliases and typed callback values;
evaluate callback expressions during registration, and do not change a reusable
factory's Task return type elsewhere in the program. Suite registration stays
synchronous. Reject wrong task result types, escaping local loans and registration
or recursive execution during an awaited test. Keep completion, hover, signatures,
CLI multi-file execution and builtins.json aligned before public acceptance.

Focused runtime controls must prove awaited failure/panic reporting, continued
teardown/next-test execution, resource lifetime across callbacks, non-Send borrowed
state, actual async network I/O, child-task cleanup and existing-scope reuse.
Native generated-program controls cover both pure synchronous tests and a runner
called from async main. These controls do not replace public compiler/LSP/native
async callback acceptance or the remaining HTTP SDK/body work.

Runtime foundation acceptance, 2026-10-28: six runner cases plus twelve reused/new
futures controls passed (18/18). Generated native synchronous/async-main boundary
regression passed for both variants. Backend and workspace all-target checks had
zero warnings/errors. See agent-progress.md for source/consumer evidence and
repaired fixture/sandbox issues. Public async callback registration subsequently
passed hosted run 40672007920 on c5b3d4a9: eleven compiler/native/CLI tests, three
LSP tests and all-target diagnostics on Windows, Linux and macOS.

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

Implementation decision, 2026-10-28: Request.json() returns owned JsonValue and
accepts no type arguments. Typed DTOs use the existing schema.parse(await req.json())
composition. This preserves checked schema failures and avoids unchecked generic
casts or body-backed views. Compiler rejection witnesses cover generic arguments,
wrong result types, request reuse and context mutation while a read is pending;
native witnesses cover successful DTO validation and rejected shapes. Acceptance
remains pending until the current reader suite and matched checks pass.

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
