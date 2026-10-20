# Documentation startup and retained lazy routes

Implementation: `c3d57f41`, `585e7f65`, `6e468b4d` and `cdafceb8`.
Compressed transport: `583c097d`; independent live protocol and throttled-browser acceptance passes.

## Observed cause and repair

A fresh Chrome profile reproduces a 2,196 ms timer gap and application readiness at
2,218.9 ms. Streaming Wasm instantiation takes 14.7 ms. The application constructs
every article, API tree and preview before showing the landing page. Repeated
validation of the immutable default theme dominates sampled work. Hidden DOM alone
does not avoid constructing these retained native models.

The app now constructs article content on the first matching visit. A transparent
Route retains visited Nodes, state, field instances and scroll positions; inactive
content leaves layout, input and accessibility. It never acquires the location
owner, rebinds to a replacement owner or evicts visited content implicitly. Factory
captures release after construction, including failure; a failed factory does not
retry. Ordinary conditional branches keep their previous disposal rules.

The default theme is parsed and validated once, then cloned into independent
editable values. Custom theme parsing and validation remain unchanged. A mutation
witness verifies that one returned default cannot change another.

The old 1,625,343-byte escaped-JSON catalog program is replaced by a 47,723-byte
typed dispatcher plus readable, page-sized data factories and one shared model.
The compiler checks all generated modules before publication. Per-file fingerprints
cover caching, dev snapshots, tamper repair and removal of unchanged obsolete files.
An explicit full snapshot remains available for tooling; the app loads navigation,
selected page/API/example data and the search index separately. Search text loads
on the first nonempty query. The full JSON export is build/tooling data, not an
application startup dependency. This is lazy construction, not Wasm code splitting.

## Focused acceptance

Six native model checks cover retained route construction, identity, inactive input,
weak location lifetime, factory captures and existing conditional branches. A seventh
checks failure cleanup without retry. The independent Node-after-tsc lifetime oracle
agrees. The formatted documented Route example executes natively; required factory,
return type and exact-value contracts reject invalid calls. Content publication,
unchanged reuse, generated-file tamper repair and obsolete-output cleanup pass.
LSP completion excludes an already supplied path, supplies remaining props, documents
retention/ownership and diagnoses a missing required factory. Initial fixture setup
and overly literal documentation assertions are repaired without weakening behavior.

The live browser passes actual wheel input, first navigation, retained article DOM
and scroll, field DOM/value, landing counter, full-text/code/API search, mode changes,
direct deep entry and disposal. Deep entry does not build the landing subtree. The
actual native AspectRatio documentation route builds, remains alive for five seconds,
emits no errors/output and is reaped. No native pixel or OS gesture claims are added.

The same fresh-browser profile after the repair records a 108.3 ms maximum timer gap
and application readiness at 172.1 ms. Instantiation takes 29.9 ms. These are diagnostic
cold-page observations, one per version, not a controlled performance benchmark or
latency guarantee. Reused profile records pass explicit 250 ms normal and 750 ms
throttled event-loop-gap guards without repeating the browser runs. The debug Wasm payload remains 5,423,715 raw bytes before transport
compression. The live watcher reports a 26,571 ms initial build with warm dependencies;
frontend/build latency and browser startup are separate measurements.

The new dev transport precompresses assets once per successful publication and
negotiates gzip/identity without altering decoded bytes or application/wasm MIME.
It preserves revision checks, HEAD lengths and previous assets on failed builds.
Independent wire/decompression checks pass for gzip/identity preferences, wildcards,
q=0, invalid quality, MIME, HEAD, 406 and mixed revisions. The 5,423,715 decoded
Wasm bytes transfer as 1,855,037 gzip bytes, with exact independent decompression.
The installed-server LSP witness also passes after the final consumer refresh.

At four-times CPU slowdown, 1.6 Mbps download and 150 ms latency, a fresh browser
loads without errors, records a 416.7 ms maximum event-loop gap and becomes ready at
10,309.7 ms. The compressed Wasm transfer takes 9,525.2 ms. Streaming's asynchronous
wait includes that transfer; it is not a nine-second main-thread pause. Cold network
latency remains visible until Wasm is ready. HTML-first rendering and executable
code splitting remain separate required framework work, not claims of this repair.

The final real dev supervisor remains at http://127.0.0.1:4317/ and reports a 19,749 ms
initial build with warm dependencies. Five consumers are refreshed from `583c097d`;
the active editor maps the new executable inode and exact settings/mtime are restored.
Canonical evidence uses `tsrust-startup-*` in SQLite, including original failures and
CPU profiles. Closed originals are independently restored and compared by exact
bytes, length and SHA-256 before loose copies are removed. No SQL, full regression
or timed benchmark matrix is repeated.

## Sources and continuation

The runtime already uses streaming instantiation. The browser profile identifies
application construction as the blocking work; moving DOM rendering into a worker
would require a different architecture. Browser loading guidance is available in
[WebAssembly performance patterns](https://web.dev/articles/webassembly-performance-patterns-for-web-apps).
The ordinary shared compiler and renderer remain the application implementation;
this section adds no server-side JavaScript engine or optional JS bridge.

Keep the live preview and editor current. Reuse accepted editing, native layout,
resource and SQL evidence. Resume generic reactive props, base composition, overlays
and the required pinned component library after startup acceptance. Broader binary
splitting and HTML-first hydration are not established by these checks.
