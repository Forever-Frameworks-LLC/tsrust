# Native static web mounts

Status: focused native/static/editor acceptance passes, including real React CRUD
served by the native process. Baseline web checkpoint is `6ceb7ccc`. Installed CLI
is refreshed; active installed editor image replacement remains blocked by the
Windows executable lock. Full prototype acceptance remains incomplete.

The new source API is Router.serveStatic(prefix, directory, fallback?). It uses
Tower HTTP ServeFile for streaming, content types, ranges and date conditionals.
Tower HTTP 0.6.11 was already pinned transitively; this section enables its fs
feature explicitly in compiler development and the bundled runtime manifest.
Both locks add http-range-header 0.4.2 and mime_guess 2.0.5; the runtime lock also
adds unicase 2.10.0, already present in the root lock. Runtime dependency names
remain reserved against handwritten-binding replacement. No renderer dependency
is added. Direct upstream source in the Cargo registry was inspected after the
docs.rs endpoint was unavailable.

Mount prefixes must be literal non-root paths and cannot overlap. Files stay
within canonical public roots. SPA fallback is explicit, mount-local, HTML-only
and unavailable for missing paths with extensions. Global middleware and earlier
routes retain normal order. Root deployment assets must not have concurrent
untrusted writers; canonical resolution is not a filesystem capability sandbox.
The API does not take ownership of caller configuration strings.

`examples/backend-tasks/web-mounts.tsr` exposes a source-level array configuration
helper and optional TASK_WEB_ROOT/TASK_ADMIN_ROOT convenience mounts. Production
main and the native test witness share it. The witness additionally accepts
test-only address/origin overrides so real browser same-origin serving can be
tested. Existing defaults remain unchanged. The React app now defaults to `/api`
at the service origin; Vite accepts TSRUST_WEB_BASE to build for a chosen prefix.

Focused witnesses:

- `http_static` compiler integration: formatted successful source, emitted native
  call, arity/type and readonly receiver rejection.
- LSP http_context static help/completion, existing Router typing-speed matrix
  expanded with serveStatic, and current checker agreement/consistency.
- `tools/verify-task-static.mjs`: two distinct roots, isolated HTML fallback,
  missing assets/methods/API routes, binary/MIME/range/HEAD/304, raw malformed
  paths, symlink escape, stream cancellation, native SQLite coexistence, then
  actual built React CRUD from native-served assets. Builds one task witness and
  offers --run-only for retained execution. Temporary databases/roots clean up;
  browser remains headless. Set TSRUST_WEB_BASE=/web/ for the app build first.

Initial offline full lock regeneration encountered an existing yanked historical
clipboard dependency. A targeted Tower HTTP update succeeded while retaining
those locked versions. Cargo also rewrote unrelated Windows dependency edges;
those edges were restored from the source checkpoint. Current lock diff is
limited to required HTTP feature dependencies and direct roots. This is dependency
resolution evidence, not runtime acceptance or an advisory audit.

Next: complete the focused compiler/LSP build using one Cargo job, no incremental,
shared target. Run the static native verifier and relevant editor rows, repair
only affected failures, update evidence, whole-project format and checkpoint.
Full family regressions/performance remain a later milestone requirement.

Build is live in session `36943`, log `target/http-static-focused-build.log`:

```sh
cargo test --offline --locked -p tsrust-compiler -p tsrust-lsp \
  --test http_static --test http_context --test completion_matrix \
  --test checker_agreement --test editor_consistency \
  --bin tsrust --bin tsrust-lsp --no-run
```

At 08:24 local, the same rustc process was still building the compiler library
with one Cargo job; recent inspection showed about 1.2 GiB private memory and CPU
progress. Do not start a competing Cargo consumer. The independent React build
for `/web/` passes (`target/task-static-web-build.log`), and the new verifier's
JavaScript syntax check passes. Runtime/static checks must wait for the new CLI.

Matched static-file benchmark support is prepared in `benchmarks/http`: native
and Node-after-tsc sources, exact manifest, independent 5376 checksum and shared
asset fixture, runner `--static` registration and source/protocol guard. The guard
passes; one untimed matched runtime check remains pending. Timing has not run.
Its existing sample counts remain unchanged. No routine output enters SQLite.

Exact installed crate manifests declare Tower HTTP, http-range-header and
mime_guess MIT; unicase is MIT OR Apache-2.0. The
[Tower HTTP RustSec page](https://rustsec.org/packages/tower-http.html) and
[RUSTSEC-2022-0043](https://rustsec.org/advisories/RUSTSEC-2022-0043.html) were
reviewed. That Windows traversal advisory is patched in >=0.2.1, including the
selected 0.6.11. Searches for the helper crates did not identify a package-specific
advisory; this limited review is not a complete current Cargo advisory audit.
Raw Windows drive/alternate-stream and encoded separator rejection witnesses
are included independently of the upstream fix.

Initial build `36943` passed in 12m49s. The compiler fixture initially called the
raw parser on imports; project-aware emission exposed a missing shared binding
registration. ROUTERSTATIC is now included in native/bindings.rs, and the fixture
also asserts lookup directly. The focused test now includes the exact bundled
HTTP/futures Rust runtime, checking atomic configuration rejection and actual
streamed GET/fallback/isolation behavior without the unrelated Windows lib-test
compile gap. Repair build `61878` is live, log
`target/http-static-registration-repair-build.log`. No runtime pass is claimed yet.

Initial LSP help, Router typing-speed matrix and Router checker agreement pass
(`target/http-static-lsp-help.log`, `target/http-static-matrix.log`,
`target/http-static-agreement.log`). These did not catch the missing binding;
project-aware compiler/native execution remains mandatory. HTTP consistency now
includes Router's complete member catalog. Preserve old passes and rerun affected
checks after the registration repair.

Final build `51941` passes after correcting the direct test's Rust API names.
Two focused compiler/runtime tests pass (`target/http-static-compiler-final.log`),
as do final help, Router typing-speed matrix and checker agreement plus HTTP
member consistency. Logs end in `-final.log`, with
`target/http-static-consistency.log` for reachability. No broad family pass is
claimed. The first real application attempt found a test-only optional ternary
narrowing mismatch; a named explicit-if setting helper resolves it. Native
application build/acceptance is live in `92096`, log
`target/task-static-source-repair.log`; do not start a duplicate build.

The independent Node-after-tsc counterpart passes one untimed 128-request batch,
checksum 5376 (`target/http-static-node-verify.log`). Its source is unchanged;
the native counterpart remains pending. No timing or analytics import occurred.

Application attempt `92096` built and launched the native server, then failed the
asset nosniff assertion before browser acceptance. Pending Context headers were
copied by context.text/html/json builders but were lost for independently
returned Responses, including static files. The repair shares header application
between builders and ContextNext response installation. It preserves post-next
middleware overrides and replaces/appends each configured header's complete
value list consistently. A direct runtime witness now tests both static and raw
Responses with entry/exit headers and nosniff. The external security assertion was
not relaxed. Header repair build `10875` is live in
`target/http-static-header-repair-build.log`; full native app/browser acceptance
still remains pending. Existing editor passes are unaffected by this runtime-only
repair. Do not reuse the old task-static executable to claim the repair accepted.

## Final acceptance

Header repair build `10875` completed, exit 0. Seven affected checks pass: two
static compiler/runtime cases, two existing Hono middleware/order cases and three
existing Router state/cookie/metadata cases. These use the exact bundled HTTP and
futures source in the integration target, avoiding the unrelated Windows lib-test
compile gap. Logs: `target/http-static-header-focused.log`,
`target/http-static-header-hono.log`, `target/http-static-header-router.log`.
Earlier final editor help, typing-speed matrix, agreement and HTTP reachability
passes are retained; the header-only repair does not alter their inputs.

`node tools/verify-task-static.mjs` rebuilt its native executable and passes all
assertions, `target/task-static-header-acceptance.log`. It proves two roots with
isolated fallbacks/API routes; raw traversal/encoded separators/device paths and
outside-root directory-junction rejection; MIME/binary/range/HEAD/304; cancellation
of a file response; retained application nosniff; actual native-served React CRUD;
and orderly exit with temporary SQLite/assets removed. The headless native-served
screenshot `target/task-native-web.png` was inspected. No desktop input occurred.

Native and Node-after-tsc static benchmark counterparts separately pass one
untimed batch of 128 requests, both checksum 5376. Native source also passes
format/idempotence. Logs are `target/http-static-native-verify.log` and
`target/http-static-node-verify.log`. Node's earlier pass was preserved. Original
timing protocol/sample counts remain unchanged; no timing or analytics import ran.

Installed CLI was atomically refreshed, SHA-256
`dc1e0480b107dfc30ca41824346f1d84d553495bc356da5eb26ba5fd9c67750b`;
installed source checking and task API stale checking pass. The installed LSP is
held open by editor processes 32876 and 32540. Both standard atomic replacements
were denied by Windows; temporary siblings were removed, and no editor process
was stopped. Debug LSP SHA-256 is
`5bccffaba56857c0106108d3022ebf208d26adfb6e33d3d3810b6a8405f3df18`.
Headless editor acceptance used that rebuilt executable. Active-editor refresh
and live document verification are pending; do not claim those consumers updated.

The serveStatic IntelliSense example also passes formatting/idempotence and
native execution with the exact documented output,
`target/http-static-doc-example.log`.

Next priority is the owner's SDK version/replica compatibility steering. Keep the
larger Electron, Expo, cancellation, convenience/migration and family milestone
requirements active. This section does not settle hosted rolling-fleet behavior.
