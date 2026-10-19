# Compiler continuation: UI resource lifetimes, data types, then S19–S21

## Current owner direction: complete UI application workflow — 2026-10-18

The owner stopped local tuning in favor of a researched, coherent UI system.
Follow [the application-system contract](../plans/ui-application-system.md),
UI_VISION.md sections 6/18/19 and the existing ownership decisions. Themes,
configurable editable component source, immutable bundled offline installation,
compatibility/lock records, shared styles and LSP are one implementation section.
They are missing product requirements, not optional later polish. Keep Caveman
updates concise and Ponytail's minimum complete solution; no repeated planning
approval is needed. Do not reopen the selected GPU or bounded Wasm decisions.

The application section now implements project-owned light/dark theme JSON,
semantic token scales, editable local Button/Card source, exact bundled v1/v2
payloads, offline dependency installation and compatibility/provenance locks.
`new --ui` uses that same installer. JSX prop spread transfers owned callbacks and
children; direct signal-reading `disabled={count.get() < 5}` predicates use weak
captures and refresh after handlers. No public lifetime plumbing is required.
See [the application guide](../guides/ui-applications.md) for supported utilities,
range checks, source preservation, explicit upgrades and unsupported cases.

Compiler and editor acceptance covers derived conditions, owned prop forwarding,
configured/unsaved theme errors, lock incompatibility, inherited props/variants,
class completion/hover, definitions, checker agreement and editor consistency.
Installer witnesses cover exact dependency closure, custom destinations, edited
source/lock preservation across compatible compiler changes and refusal to
silently overwrite upgrades. Shared native layout/paint and theme precision checks
pass. The untimed Node-after-tsc oracle agrees with generated native HTML tokens
and component sizes. Existing lifetime, cancellation, grammar and SQL evidence is
reused; no full regression or performance matrix was started.

Real Chromium acceptance passes derived disabled updates (0, 5, 105), stable DOM,
dark semantic colors, local Button size/padding, rounded keyboard focus, focus-over-
hover priority, stable font/height across 640/800/480/640 widths and disposal.
The generated Wasm is 396,689 bytes plus 15,619 bytes of JavaScript glue,
uncompressed; these are payload observations, not speed measurements. UI package
export now permits the pure JSON module needed by theme serialization while still
rejecting native service/file/task modules in the bounded browser tool.
Generated native GPU acceptance passes the same state transition through actual
Accessibility button actions and closes the window cleanly. Pixel inspection is
not claimed. The first new support build took 270.7 seconds; the standalone build
reused renderer support and spent 3.3 seconds in native compilation/linking. These
are diagnostic samples, not controlled benchmarks. Installed compiler/LSP consumers
are refreshed; real standalone protocol checks include local props, classes, clean
diagnostics, invalid setter reporting and diagnostic clearing. The standalone
GPU app independently passes 0–5–105–104 while the second counter stays at 0;
its application-owned screen test and formatting/check commands also pass.

Runtime artifacts no longer have implicit disk size/count quotas. Explicit
`cache info`/`cache clean`, identity checks, atomic publication and private copies
remain. Earlier 255.7-second cold/115-ms warm/3.586-second edited-build observations
are historical diagnostics, not controlled benchmarks or current measurements.

Closed records are byte-verified in canonical SQLite under `tsrust-ui-contract-*`.
One earlier theme-agreement rerun accidentally overwrote its loose original log;
only a retrieved diagnostic excerpt survives for that attempt. The archive labels
that excerpt explicitly. Earlier failures and subsequent complete repaired logs
remain intact. Do not claim full original-byte preservation for that one attempt.

The standalone project retains its owner's Decrement handler, two independent
counters and exact derived disabled condition. Continue with the editable-list
family (string/boolean state, text editing/IME, selection, keyed insertion/removal
and deterministic disposal), then persistent layouts and typed routing. These are
required product work, not optional counter polish. The GPU and bounded Wasm
choices remain settled. No full UI-readiness claim is made.


## Standalone UI project and installed editor — accepted 2026-10-18

Created `/Users/seancotter/Desktop/dev/learning/tsrust-counter` with `tsrust new`.
The generated config retains `src/main.tsr` and `target/tsrust-counter`; the source
now uses the accepted paired counter, with a rendering test and local editor
settings selecting installed executables. Opened this directory as a separate
VS Code root. The installed LSP passes real protocol checks for clean `.tsrx`
diagnostics, Signal get/set completion, setter hover, an invalid setter argument,
and diagnostic clearing after correction. The editor has one idle server for this
root. Whole compiler-fixture workspace scanning had delayed open-file diagnostics;
the speculative scan opt-out was discarded, not shipped.

The standalone test exposed the same typed-record formatting borrow defect in the
`std:testing` callback adapter. The shared adapter now borrows the error payload.
The focused throwing-record regression and actual standalone screen test pass.
Installed compiler refreshed. Existing UI, browser and native acceptance is reused;
no full regression, SQL or timing suites rerun. Closed original failures and repaired
checks are verified in SQLite under `tsrust-state-*`. Continue the unfinished UI
scope listed below; this small project is not a full UI-framework readiness claim.


## Direct UI state bindings and captures — accepted 2026-10-18

`Signal` is used directly in JSX and callback bodies. The compiler creates
non-retaining bindings at use and closure creation; the returned component tree
retains its single state owner. Public `add`, `handle`, `SignalHandle`, `SignalText`
and `ownState` are absent. State exposes `get()` and `set(value)`; the `.tsrx`
application handler calculates the increment. Disposed access throws `UiError`.
Checked integer overflow occurs before mutation and preserves the previous value.
The formatter preserves conventional parenthesized multiline JSX returns.

Focused acceptance passes: compiler UI witnesses, generated nested callbacks and
overflow preservation, repeated bindings, branch/loop captures, parameter shadowing,
private API rejection, nine runtime witnesses (including independent destruction),
JSX layout/idempotence, editor hover, UI completion at all four typing modes,
Signal checker agreement and editor consistency. The generated Wasm counter passes
real Chromium pointer/keyboard, independent-state, disabled, stable-node, targeted
text, disposal and late-click checks against the updated TypeScript oracle.
Generated native GPU presentation and bounded close also pass. Native OS input
injection and screen-reader sessions remain unverified. Wasm is 231,840 bytes plus
15,601 bytes of JavaScript glue, uncompressed; this is not timing evidence.

Original failures and repaired checks are retained in SQLite under `tsrust-state-*`.
The native callback error adapter repair borrows typed error payloads while
formatting them; the generated-button witness prevents recurrence. Earlier static,
SQL and full-family evidence is reused. No broad regression or performance run was
triggered. State remains integer-only; constructor aliases, state factories, state
creation inside anonymous callbacks, general reactive props, dynamic trees,
text input/IME and async UI remain open. Named local functions retain ordinary
checked capture rules; automatic non-retaining capture currently applies to arrows.


## Paired interactive counter — 2026-10-18

The owner selected the bounded Wasm browser backend. The generated counter now
passes real Chromium pointer, disabled-button, Tab, Enter and Space checks, two
independent instances, stable node identity, targeted text updates, explicit mount
disposal and late-click rejection. Generated native GPU presentation/close passes;
shared state, native keyboard/action dispatch, AccessKit actions and independent
capture destruction have focused Rust witnesses. Native OS input automation and
screen-reader sessions were not performed. These limits are not visual parity claims.

The owner rejected separate reactive components and manual state attachment.
Ordinary `Text` now accepts mixed literal/SignalHandle children. Public `SignalText`
and `ownState` are absent. Returning a component subtree automatically transfers
its direct local signal owners through ordinary checked moves. Multiple owners and
branch returns are covered; premature moves and rebindable owners reject. Constructor
aliases/anonymous callback state are explicitly rejected. Broader state factories,
reactive properties, text input/IME, dynamic trees and async UI remain open.

See `examples/ui-counter/README.md` and `tools/build-ui-web.mjs`. The Wasm artifact
is 217,823 bytes plus 15,601 bytes of JavaScript glue, uncompressed. These are
artifact sizes, not timing measurements. The typed independent counter oracle,
source hashes and original repair logs are retained in SQLite under
`tsrust-counter-*`. Compiler UI tests (4), runtime witnesses (8 plus independent
capture-drop witness) and focused editor help pass. Reuse static/JSX and SQL evidence.
The browser harness fixes concern target selection, Chromium key text and profile
cleanup; their original failures remain recorded. No full compiler/LSP, SQL or
performance matrix was rerun.


## Generated static UI and counter continuation — 2026-10-18

`std:ui` now connects checked `.tsrx` components to the shared owned renderer.
`Text`, `View`, disabled `Button`, `document` and `run` have public documentation.
HTML-only output excludes native GPU dependencies. Native exports use the existing
pinned renderer dependencies. The shared screen passes generated semantic HTML
checks and an actual bounded GPU presentation/close. This does not claim pixel,
screen-reader or interactive acceptance. Node ownership/private-runtime checks and
JSX highlighting checks pass; editor hover acceptance is recorded with this section.

The owner selected WebAssembly for the paired interactive counter, not as a
permanent browser-backend commitment. The Wasm target and digest-verified matching
wasm-bindgen tool are prepared. Continue component-owned state, retained checked
handlers, weak handles, targeted bindings, pointer/keyboard input, disabled behavior
and cleanup on both targets. The static renderer and syntax evidence should be
reused. No SQL or full regression rerun is required for this section.


## JSX compiler integration — 2026-10-18

Named JSX components now lower into ordinary checked calls with props records and
ordered children arrays. `.tsrx` discovery, configured CLI execution, import
resolution, dev watching and editor attribute help are integrated. Focused native,
editor and matched TypeScript witnesses pass; see [JSX integration](../plans/jsx-integration.md).
The formatter preserves JSX islands verbatim. This is syntax acceptance, not paired
rendering or counter acceptance. Continue checked renderer bindings, generated
static output and the paired interactive counter without another go-ahead.


## GPU-first UI foundation — 2026-10-18

The owner selected Winit + Taffy + Vello. `crates/tsrust-ui` now provides a bounded
owned static tree with shared utility meanings, HTML escaping, native text/layout,
GPU window rendering and AccessKit semantics. Native API checks, three focused
checks and a bounded real window presentation/close pass. See its README for exact
limits. The Rust-constructed example does not count as generated Tsrust UI.
Continue `.tsrx` parser/checker/lowering, project/CLI/editor integration, then the
paired generated static fixture. Preserve the static/interactive distinction and
accepted lifetime evidence. Renderer selection is resolved; JS/WASM selection is
still unnecessary for this noninteractive slice.


## Named HTTP query helpers — 2026-10-18

`c.req.query(name)` and `c.req.queries(name)` return owned optional text/arrays
without consuming the request body. Scope and Hono differences are explicit in
[acceptance](../plans/http-query-acceptance.md). Continue remaining ordinary HTTP
helpers/composition and the paired static `.tsrx` UI slice. The UI review remains
read-only; this thread remains the sole writer/build owner. The owner selected GPU-first Winit + Taffy + Vello on 2026-10-18. Continue the
paired static slice; static output does not require choosing JS/WASM yet.


## Process arguments — 2026-10-18

`std:env.args()` now exposes owned arguments with checked Unicode errors. Native,
editor, executable documentation and untimed native/Node witnesses pass. See
[acceptance](../plans/process-arguments-acceptance.md). Continue ordinary HTTP query
and response helpers, composition/error hooks, then the open UI milestones.
Generic inheritance and CLI argument acceptance do not establish framework parity.


## Generic interface inheritance — 2026-10-18

Explicit generic interface bases now support local/imported definitions, aliases,
multiple/chained bases, optional and nested fields. Wrong arity, cycles, duplicate
fields and unknown unused arguments reject; nominal ownership is unchanged.
Compiler-derived editor fields include project and incomplete-member recovery.
Focused acceptance: six interface tests, six retained-project-session tests,
three interface editor tests, agreement for InheritedItem, its four-mode completion
row, and 17 editor consistency checks. The unchanged interface oracle passes eight
native/Node untimed cases; no timing or SQL rerun. See
[acceptance](../plans/generic-interface-inheritance-acceptance.md).
Continue CLI usability, ordinary HTTP APIs and UI foundations under the owner's
readiness correction below; this closes only generic interface inheritance.


## Owner correction — language and framework readiness, 2026-10-18

Continue basic language completeness, CLI usability, Hono-style API coverage and
UI implementation. These requirements are not conditional on selecting another
service workload. Accepted S21 sections do not establish overall readiness.
See [the current source-backed inventory](../plans/language-framework-readiness.md)
for concrete gaps and implementation order. Reuse accepted evidence; batch targeted
verification by complete section. Preserve Router naming and application-owned auth.



## Current continuation — SQL cursors and application jobs, 2026-10-16

Checked `Pool.cursor`, `Cursor.next` and `Cursor.close` now pass focused runtime,
generated execution, ownership, editor and untimed native/Node acceptance. The
explicit SQL runner passes all three drivers and downstream SQL/HTTP/migration
checks; accepted runtime and exact-domain evidence was reused on repair. The await
emitter no longer applies a second borrowed-union conversion. See
`docs/plans/async-sql-acceptance.md` and SQLite evidence under `tsrust-cursor-*`.

The owner selected application-level durable jobs using existing SQL pools, not
`std:jobs`. `examples/durable-jobs` implements SQLite claims, leases, bounded retry,
deduplication, stale-worker fencing and recovery. Native execution and independent
persisted-state inspection pass, as does the matched 128-job native/Node workload.
See `docs/plans/durable-jobs-acceptance.md` and `tsrust-durable-jobs-*` evidence.
This example supports SQLite only; external effects still need their own idempotency.
Source checkpoint: `dd85a30d`. Installed CLI and both editor servers are refreshed;
the active job example has no workspace problems with checking enabled.
Rust-analyzer remains disabled. Consumer hashes and cleanup proof are retained
under `tsrust-sql-jobs-*` in SQLite. Cleanup reclaimed 37.78 GiB of obsolete
workspace, cross-target, test and native outputs while preserving all five current
consumers. Reuse the single shared target; deleted caches are intentionally cold.

The concrete cursor/job follow-ups are implemented. Keep remaining scope practical:
The owner correction above supersedes the earlier conditional scope for streaming,
HTTP/UI and language work. Redis/brokers remain demand-driven. The S21
full-family/performance checkpoint remains separate and unrun under the owner's
current targeted-verification direction. Do not repeat accepted SQL/S18 or S19/S20
work. Preserve Router naming and application-owned authentication policy.


## Current continuation — producer stream bridge, 2026-10-16

`std:http.streamChannel(Channel<Bytes>, status = 200)` now transfers bounded
application-produced chunks into a response. Focused runtime, compiler, editor,
formatting and untimed native/Node verification pass; evidence is in SQLite under
`tsrust-producer-*` and the HTTP producer record. This bridge preserves producer
scope: it does not detach request children. Callback-owned producer lifetimes and
SQL cursors remain the next streaming work. See `docs/plans/http-acceptance.md` for
bounds and unsupported cases. Reuse accepted S21 and SQL evidence; no full-family
regression/performance run was added.

## Current acceptance and continuation — 2026-10-16

Checkpoint `c82daa25` adds typed Router application/request state, cookie helpers
and the service route rewrite. The native metadata/body boundary is repaired:
body ownership remains exclusive while request/response metadata can be shared
across suspension. All 16 affected HTTP runtime checks, five ownership rejection
witnesses, focused editor checks and integrated macOS service acceptance pass.
Old App retained-state examples are preserved; the new example is
`examples/http-state/router.tsr`. The v3 native/Node workload passes untimed verification. The revised service
and relocated compiler/editor package pass offline Linux aarch64 acceptance.
Installed CLI and both LSP consumers are refreshed; the active editor reports no
workspace problems with checking enabled. Rust-analyzer remains disabled.
Implementation acceptance is complete for this scoped Router/service section.
The S21 performance/full-family checkpoint remains separate and unrun. Original failures and intermediate
checks are retained byte-for-byte in SQLite under `tsrust-router-*`.

The public router is named `Router`, not `Hono`; Hono is only the API reference.
The existing server lifecycle is accepted. Ordered/scoped middleware, multiple
callbacks, inline async arrows and conditional short-circuits now execute in the
focused native example. The matched untimed workload passes. Original evidence
is in SQLite under `tsrust-hono-*`; the Unicode runtime witness was corrected to
observe Unicode through a response body instead of an ASCII-only header accessor,
and that affected witness now passes its repaired rerun. No full regressions or
new timing were run. The typed state, cookie and service rewrite continuation is described above. Keep application authentication policy out of the framework.
Installed CLI and both editor servers now include Router/async arrows; the active
editor reports no Tsrust errors. The cookie-loop TextMate bug is repaired and
visually verified after reload. Rust-analyzer was restored to disabled for this
workspace. Context form/multipart bridges pass integrated macOS service acceptance.

Owner correction supersedes the authentication wording below: implement a
Hono-style public HTTP API over the existing Axum backend. Prioritize request
context, routing/middleware, request and response headers, cookie helpers, checked
request-local state and safe shared application state. Authentication, sessions,
user models and authorization policy belong to application code. The existing
service authentication example is an application witness, not a framework API or
a substitute for these primitives. See `docs/http-and-hono.md` and the current
scope in `docs/plans/service-readiness-plan.md`. Preserve accepted ownership,
cancellation and cleanup rules. Finish this API section before further deployment
verification; reuse completed evidence and batch targeted checks.

SQL migrations and recovery now pass focused acceptance. `Pool.migrate` keeps its
checked pool borrow, validates versioned files and history, and closes failed or
cancelled migration connections through tracked cleanup. All three drivers pass
failure/recovery and independent lock-release observation; ephemeral SQLite inputs
reject before migration. The full explicit SQL runner, relocated offline package,
editor checks and matched untimed history workload pass. Exact failures, repairs,
consumer hashes and cleanup proof are in SQLite under `tsrust-migrations-*`.
Installed consumers are refreshed; the restarted active editor reports no problems.
Continue S21 observability and scoped HTTP/security/authentication, then actual
Linux/offline/signal and integrated service acceptance. Do not repeat completed
SQL or S19/S20 suites without a relevant change. See
`docs/plans/service-readiness-plan.md` for contracts and remaining work.

Shared cache integration now passes focused native/compiler/editor acceptance:
`SharedLruCache`, explicit `share`, owned `getCloned`/`peekCloned`, unchanged local
cache loans, retained HTTP access and independent destruction. Constructor catches
now receive their missing failure checkpoint. Matched scalar checks pass untimed
in both engines. CLI/LSP consumers are refreshed and the restarted editor reports
no problems. Exact original failures and repaired passes are in SQLite under
`tsrust-shared-cache-*`. Continue remaining S21 observability, HTTP/security/authentication and actual Linux/offline/signal
acceptance. Do not repeat completed S19/S20 regressions or timing.

S21 retained read-only HTTP state now passes focused acceptance: owned configuration
and SQL pools, routes/middleware/fallback, suspension and exact-once destruction,
compiler/editor agreement, all three SQL drivers, and matched untimed HTTP state
checks. CLI and both editor consumers are refreshed; the active editor reports no
problems after restart. Native support builds now preserve `CARGO_BUILD_JOBS` to
respect the caller's memory limit. Exact logs and workload records are in SQLite
under `tsrust-http-state-*` and `benchmarks/http/retained-state-*-verify.json`.
Continue remaining S21
requirements in `docs/plans/service-readiness-plan.md`. Full S21 acceptance and
its timing milestone remain open. Do not repeat accepted S19/S20 regressions.

Exact decimals now pass focused macOS acceptance: `std:decimal.parseDecimal`,
exact bounded arithmetic, checked conversions, explicit cloning and worker
transport, editor agreement/help, 13 native catalog examples, offline package
export and matched untimed money checks. See `docs/data-type-coverage.md`.
Installed CLI/LSP consumers are refreshed; open decimal examples report no
problems after the active server restart. SQLite holds `tsrust-decimal-*` evidence.
Current CLI SHA-256: `e1678a081b74795801123e9da220e818bff42a1a0358ef2c2632fbb97f0fa7d7`.
Current LSP SHA-256: `49b5cb23b43f9169af491876e522563bbd6fa123486e93b2576e8fdc891c0ca1`.
The `std:date` calendar section now passes focused acceptance: validated Date,
parseDate/formatDate, date-fns-style immutable arithmetic, bounded formatting,
editor agreement, 15 catalog examples, offline export and matched untimed date-fns
workload. The original fixture failure and repaired pass remain in SQLite under
`tsrust-calendar-*`. Installed consumers are refreshed; VS Code shows no problems.
Temporal values also pass focused acceptance: Time, Instant, Duration and TimeZone,
exact nanoseconds, checked epoch conversion, explicit DST gaps/folds and named/fixed
zones. Chrono-TZ 0.10.4 pins IANA 2025b; later zone-rule changes are not included.
Four compiler/export checks, 31 native docs examples, runtime/editor checks and
matched millisecond workload pass. Current installed consumers include this section.
UUID focused acceptance also passes: strict canonical text, independent exact bytes,
fallible entropy, ownership/worker transport, nine native documentation examples,
offline export and matched untimed workload. Installed consumers include UUID.
Explicit checked Rust width/byte/Option/tuple adapters now pass native/compiler
and editor acceptance. Direct unsupported signatures still reject. SQL domain
witnesses now pass all three drivers, including exact decimal, date/time, UUID,
JSON and nullable storage. Pinned Rust/date-fns/Decimal.js declaration inventories
are recorded. Full data-type family regressions and targeted integration repairs
now pass; see `docs/data-type-coverage.md` for retained original failures and final
evidence. The five accumulated workloads are measured at `b911a378`: 930 samples
with checksum agreement, exact records and derived summaries in SQLite. Native
medians are lower in these scoped first baselines. Do not repeat this full pass.
The local `std:cache` API now passes focused compiler/runtime/editor acceptance.
Normal reads handle undefined without `!`; ownership, resource destruction and
checked TTL/capacity errors pass. See `docs/milestones/s19-local-cache-acceptance.md`
for the conservative lifetime boundaries and exact evidence. Matched LRU
verification passes; timing is not yet performed. Installed consumers include the
cache API and the restarted active editor has no problems. The serialized loading
integration now passes focused cancellation, closed-client, independent drop and
Node counterpart checks. Cache matrix/agreement gaps and two native emission
defects are repaired; see the current section in `docs/plans/cache-plan.md`.
Bounded loading/shutdown now passes with pre-registered clients, explicit completion,
closed-reply handling and a group deadline. Owned Task capture metadata and deferred
exception assignment are repaired with focused ownership/drop witnesses. S19
implementation acceptance is complete within its serialized-loading limits.
S20 bounded work queues now pass focused native/compiler/editor acceptance, including
backpressure, exact cancelled-message destruction and close/drain. Mixed task groups
exposed repaired empty-failure and record-origin compaction defects; see
`docs/plans/work-queue-plan.md`. Installed consumers are refreshed and the active
editor reports no problems after restart. The combined S19/S20 owned-coordination
milestone now passes: 4,194 compiler and 731 editor tests, separate doctest drivers,
and 372 measured cache/queue samples. All 18 sandbox socket failures passed exact
unsandboxed reruns; do not repeat the full suites. See
`docs/milestones/s19-s20-owned-coordination.md`. Continue S21 implementation now
using `docs/plans/service-readiness-plan.md`. No new permission checkpoint is needed. The owner prefers date-fns: use small
named immutable functions and familiar arithmetic/formatting names, while keeping
checked ranges, precision and explicit DST ambiguity handling. The bounded Rust/SQL
boundary witnesses are accepted; proceed without another go-ahead.

The first data-type section, `std:numeric` (`4760e58d`), now passes focused compiler/runtime,
editor and matched-workload acceptance. It preserves original int/number union
inputs and rejects loss for toInt/toNumber/toFloat32, validates signed/unsigned
8/16/32/64-bit ranges within int storage, and parses strict exact decimal integers.
Full u64 above i64::MAX and 128-bit values remain explicitly unsupported. See
`docs/data-type-coverage.md` for measured workload scope. SQL REAL precision repair
also passes explicit three-driver acceptance, including precision/underflow
rejection and unchanged automatic rollback. S19 is next; no new go-ahead is required.


The checked `std:testing` resource facility and synchronous UI lifetime contract
are accepted on macOS arm64. Compiler source is `3c23f7b8`; the UI representation
repair is `5540a836`. See [the current L01–L26 mapping](../../experiments/ui-tsrust-owner-handles/CHECKED-RESOURCES.md)
and `release-matrix.json::checkedContinuation`. Historical external projected Drop
rejection remains intentional. Do not restore a separate public testing-resources
module or treat synchronous success as async/worker UI readiness.

Passed: 7 resource compiler tests, 56 testing-library tests, 4 callback-error tests,
the native-effect unit test, all existing UI feasibility/identity checks, and final
native demo exit 0. Added subtree-removal/FIFO-payload coverage passes its affected
formatted native rerun. Final editor suites pass: checker_agreement 2,
completion_matrix 3 and editor_consistency 9. The initial agreement run imported
all bundled testing code into every unrelated probe; its original log/sample are
retained. Scoping that import to the resource row preserves every receiver and
member check; the repaired full agreement suite passes in 234.55 seconds.

Installed CLI `~/.cargo/bin/tsrust` and both installed/configured LSP consumers were
atomically refreshed. VS Code's Tsrust server was restarted; the already-open
resource example and workspace Problems panel report no problems. Installed CLI
execution and installed LSP resource hover/signature help pass independently.
The user's rust-analyzer workspace extension remains disabled.

Prior SQL CLI SHA-256: `296f5d1ba4c6ac93a49925d7ba97870b2d36179e8dd9e1241dd47d74d984c19e`.
Prior SQL LSP SHA-256: `8cf43bc480b94629114fccafdf0e9bbaebd072d9faeb0c5bd7065d541599a0b7`.
Both consumers were refreshed again for numeric acceptance. The active editor
server was stopped when inspected, then restarted through its status item.
Both already-open numeric fixtures initially reported no workspace problems.
Subsequent consumer inspection found repeated background-thread stack overflows
in the active editor (SIGABRT at 19:20:25, 19:31:47 and 19:35:36). The faulting stack identifies workspace scanning of nested HTTP calls, not the
numeric fixture. A real-server regression reproduced the failure using the
existing HTTP loopback example. The single analysis worker now has a named,
bounded 8 MiB stack instead of the platform default. Four real-server fixture
checks, five workspace-scan checks and six server lifecycle/version checks pass;
installed-server fixture checks also pass. The editor was restarted and open
documents again show no problems. This fixes the witnessed finite call nesting;
it is not an unlimited-recursion guarantee. Resume the domain types below.
Reuse `/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0` and
`CARGO_BUILD_JOBS=1`. The restart made the initial build cold; no separate target
was introduced. Closed diagnostic records are byte-verified in
`benchmarks/records.sqlite` under `tsrust-resource-*`, including original failures,
repairs and interrupted runs. Reuse accepted S18 SQL evidence; SQL/task lifecycle
code did not change in the resource section.

Continue the [data-type coverage inventory](../data-type-coverage.md), then S19–S21,
without another go-ahead. Discovery now includes pinned TypeScript 5.9.3 and a
bounded standalone rust-analyzer 0.3.3081 run; the latter shuts down after probing
and does not re-enable the editor extension. The Tsrust inventory handles native
aliases and explicitly identifies the internal non-importable TestRegistry.
The bounded domain/width coverage is accepted; unsupported wider source types
remain explicitly documented.
SQL REAL narrowing is now checked: finite values that round or underflow reject
with precision. All three drivers passed with retained/resumed evidence; no need
to repeat this suite until subsequent SQL or task-lifecycle changes require it.

One writer/build owner remains in this tree. Preserve the owner's six unrelated
untracked UI plan files. The UI review chat may review readiness read-only until
an explicit write/build window is coordinated.

## Restart checkpoint — 2026-10-15, resource observer implementation

The owner requested a restart-safe stopping point during resource acceptance.
Resume the priority order below; no data-type, S19, S20 or S21 implementation has
started in this pass. The practical-service requirements below are durable.

Implemented `TrackedResource`, `trackedResource()`, `resourceId()`,
`closeResource()`, `releaseCount()`, `releaseOrder()` and `liveCount()` in
**std:testing**, as requested by the owner. Native bindings remain private in
std:testing-runtime; do not restore the temporary std:testing-resources namespace.
See [resource contracts](../guides/resource-lifetimes.md) and
`examples/resource-lifetimes`. Native Drop releases an allocation before recording
its integer identity; the observer never retains a resource. Record replacement,
array removal, callback capture, explicit close, local async suspension, failure
unwinding and abandoned lazy-task cleanup pass. Worker, channel and started-task
transport reject explicitly. External Rust projected-data Drop rejection remains.

Focused results: 6 tracked-resource tests, 6 Rust-bridge tests and 3 native-resource
lifecycle tests pass. Final focused editor completion (four typing modes) and
hover/signature-help tests pass. The full checker-agreement suite passed against
the earlier temporary namespace; the final std:testing full editor pass was stopped
at the owner's restart request. Do not claim final full editor acceptance. Resume
checker_agreement, completion_matrix and editor_consistency on the final source;
retain completed evidence. A mistaken direct invocation selected an older editor
test binary and ran zero tests; the correct newest binary then ran and passed the
new help test. This is recorded rather than counted as acceptance.

**Demonstrated UI blocker:**
`experiments/ui-tsrust-owner-handles/witnesses/tracked-release.tsr` passes active
callback self-removal and exact-once release, then fails the assertion
`automatic child release must precede parent`. Owner scope destruction bypasses
its explicit child-first cleanup; current registration insertion order destroys
parent captures first. No owner representation repair has been applied. The UI
review chat confirmed that reversing handlers alone is insufficient because drain
removes/re-appends entries. Group component state and captures into a lifetime unit,
with an extractable callback slot if state must remain accessible during dispatch;
maintain descendant-first destruction independently of display order and FIFO.
Monotonic descending component IDs are a candidate only while reparenting is absent.
Add post-delivery, nested-mount, same-key replacement and sibling-reorder cases.
Resource-bearing state and queued payload acceptance remains pending; define rejected
enqueue ownership and stale/consumed/owner-drop payload cleanup explicitly. Do not
claim B2 or async UI readiness from the primitive. Preserve existing identity rules.

This is a source checkpoint, **not a completed resource acceptance gate**. Remaining:
final editor suites, testing-library/error-effect cross-checks, full resource/UI
acceptance, benchmark applicability review, and consumer refresh after acceptance.
No timed workloads or S18 three-driver suites were rerun. The changed shared error
metadata is limited to new private testing functions; SQL/task lifecycle is unchanged.
Installed CLI/LSP and configured editor consumers remain at the prior accepted
checkpoint. Shared debug binaries contain this work; rebuild from Git if needed.

Original logs are byte-verified in `benchmarks/records.sqlite` under
`tsrust-resource-*.log`; interrupted passes and original failures are preserved.
Whole-project formatting passed, then changed zero files/fingerprints on repetition.
Six unrelated untracked owner plans are preserved. Active test processes were stopped
for restart. Reuse the shared Cargo target with incremental compilation disabled;
removed test artifacts may need rebuilding. Do not launch a full historical pass.

## Current handoff — 2026-10-15, after S18 acceptance

This section supersedes all historical “next”, “pending”, consumer-version and build-window
statements below. Follow the [current roadmap priorities](../standard-library-roadmap.md).

1. Finish checked resource lifetimes for UI, including a real destructor and independent
   non-retaining release audit. Read the roadmap acceptance contract and the B2
   `experiments/ui-tsrust-owner-handles/RESOURCE-RELEASE.md`, `release-matrix.json` and
   `RESOURCE-RELEASE-REVIEW.md`. Preserve Drop rejection for projected external records.
   Coordinate readiness with “Review UI vision and roadmap”; compiler acceptance alone
   does not close the UI matrix or establish worker readiness.
2. Complete the roadmap's required data-type coverage pass. Map existing support first,
   then implement practical gaps for UI, Rust/SQL boundaries and S19–S21. Include exact
   numerics, dates/times, UUID, JSON, binary and nullable values in the boundary inventory.
   Require explicit conversion/ownership contracts and compiler/native/editor witnesses.
   Record unsupported cases honestly; do not expand into an unlimited type-system audit.
3. Continue S19 caches, S20 queues and S21 integrated acceptance autonomously after the
   priority work. The owner explicitly authorized continuation, not another planning stop.

S21 now requires practical service readiness, not a minimal demo. Follow the
[service requirements and coverage methodology](../standard-library-roadmap.md#practical-service-readiness-and-coverage-methodology):
authentication, safe shared state, versioned migrations/recovery, redacted
observability, scoped HTTP/security contracts and actual Linux/container/offline/
signal/shutdown acceptance. Defer the polished MVP/demo, not executable witnesses.
Calendar/date/time coverage is a first-class prerequisite; monotonic clocks are
not calendar support. Compare pinned TypeScript and Rust language-tool inventories
with Tsrust and prove behavior through native/compiler/editor witnesses. Implement
in coherent sections and retain accepted evidence. The same roadmap section orders
post-S21 follow-up and names deferred scope.

### Completed work: reuse evidence, do not restart

S12 workers/pools/channels/bridges and package management, S13 async checking, S14 Tokio,
S15 HTTP, S16 TLS, S17 SQLite and S18 async SQL have accepted implementations within
linked scope. The roadmap rows and individual acceptance documents define limitations.
No global pool remains: use owned `ThreadPool`; `spawn` stays dedicated.

S18 checkpoints: `1b4a1639` implementation, `1eeaac6e` nested cleanup repair,
`cf3aa83c` acceptance documentation. See [SQL acceptance](../plans/async-sql-acceptance.md)
and [guide](../guides/async-sql.md) for exact checks and limits. PostgreSQL, MySQL and
SQLite passed separately, including cancellation, rollback, acquisition recovery,
owned rows and readable examples. A cancelled drain now retains nested cleanup until
settlement. Preserve this regression and SQL pool/transaction loan behavior.

Compiler/editor focused suites, offline export, installed consumers and untimed
native/Node checksum parity passed. Full regression and timed benchmark matrices were
not rerun for S18. Do not relabel untimed evidence as measurements. Original logs,
failures, repairs and benchmark records are byte-verified in `benchmarks/records.sqlite`;
closed loose copies and temporary Docker containers were removed.

Database suites are ignored by default. Run `node tools/check-async-sql.mjs --docker`
when SQL runtime/dependencies, task lifecycle, SQL borrowing or SQL emission changes;
unrelated documentation or source changes do not require rerunning them. The script
provisions and removes its own three-driver acceptance resources. Cancellation can
wait for in-flight queries; a cancelled/disconnected commit has indeterminate outcome.
Migration orchestration, streaming and compile-time database schema checking remain
deferred. The shared SQLite dependency is now SQLite 3.51.3 via rusqlite 0.39.0 for
SQLx compatibility; synchronous SQLite correctness passed after that change.

### Current consumers and exclusive workspace

Installed CLI `~/.cargo/bin/tsrust` and LSP `~/.cargo/bin/tsrust-lsp`, plus the configured
`target/editor/debug/tsrust-lsp`, were refreshed from `1eeaac6e`. The final LSP bytes
passed the installed completion matrix; the active VS Code workspace showed no problems.
Do not rebuild/restart them merely to resume this handoff. Refresh after new accepted API
changes. Reuse `/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`,
`CARGO_BUILD_JOBS=1`, with Node v24.11.1 first on PATH. Inspect current status and confirm
exclusive checkout/build ownership before edits; historical UI validation windows
below are no longer current reservations.

Preserve all unrelated owner UI plans, including the six currently untracked files:
`ui-b4-compiler-repair-first-task.md`, `ui-identity-enforcement-first-task.md`,
`ui-lifetime-contracts-resource-observation-first-task.md`, `ui-resource-release-first-task.md`,
`ui-state-lifecycle-first-task.md` and `ui-tsrust-owner-handles-first-task.md` in `docs/plans`.
Follow AGENTS.md and the development workflow for whole-project formatting, focused
acceptance, source checkpoints, evidence import and cleanup. No archives or snapshots.

## Historical continuation and acceptance notes

The following dated notes preserve decisions and evidence. Their old scheduling,
consumer hashes and unfinished-work statements are superseded by the current handoff.

## Owner priority, updated 2026-10-14

The owner approved a bounded practical-performance review and a return to product
features. Extreme-nesting optimization is deferred. This supersedes earlier
instructions to finish every nesting audit, eliminate every quadratic path, or
achieve universal benchmark parity before S12. Historical measurements remain
valid for their original inputs; they are not current acceptance requirements.

Do not turn this continuation into an open-ended compiler cleanup campaign.
Fix correctness failures, crashes at practical depths, and measured costs that
block a representative workload. Keep existing depth witnesses and diagnostics.
Do not raise worker stacks or silently remove resource checks.

## Accepted work and current evidence

- S01–S11 are accepted within their documented scope. The
  [S11 acceptance](2026-10-13-s11-acceptance.md) retains its historical evidence.
- Parser/checker/emitter worklists, disposal, constructor and spread repairs are
  implemented across multiple checkpoints. Use the
  [pipeline history](../plans/compiler-nesting-pipeline.md) and
  [source layout](../guides/compiler-source-layout.md) for implementation details;
  old open-item statements must be checked against later commits.
- Schema `safeParse`/`safeParseJson` expose `{ success, data } | { success, error }`.
  Compiler narrowing, editor hover/completion, examples and workload migration are
  accepted. Do not restore the old Result-match surface.
- Program-scoped test registry: `7c7b5bc0`. Callbacks own eligible local captures
  and borrow module state only while invoked. Registry disposal precedes module
  storage disposal. [Design and acceptance](program-scoped-test-registry.md).
- Nested concat overflow: `fc82d9db`. Receiver and argument work is scheduled on
  the shared expression stack; earlier operand loans stay active in source order.
  Depth-100 string/array witnesses execute on normal stacks; depth 200 preserves
  the existing admission diagnostic. All 18 affected compiler tests, 10 editor
  agreement tests and 20 native/Node correctness cells pass.
- Allocation fixtures/adapters: `263ff644`. Seven migrated fixtures, 19 adapter
  tests and 189 native/direct-Rust/Node executions pass. Native allocation and
  disposal records are in `tsrust-allocation-syntax-acceptance-2026-10-14.json`.
- Rust warnings: `00f996c8`. Obsolete resolver entry points/import removed;
  test-only query facades remain test-only. Live generated-runtime APIs retain
  behavior and gain meaningful backend coverage. Workspace/all-target checking
  reports no warnings/errors; 219 affected test executions pass. VS Code reports
  No Problems. Evidence: `tsrust-rust-warnings-acceptance-2026-10-14.json`.

Original acceptance and failed attempts are stored in `benchmarks/records.sqlite`.
Registry, concat and consumer refresh records use respectively
`tsrust-program-registry-acceptance-2026-10-14.json`,
`tsrust-concat-acceptance-2026-10-14.json` and
`tsrust-consumer-refresh-2026-10-14.json`.

## Practical performance decision

See [the bounded performance review](../reviews/2026-10-14-practical-schema-performance.md)
for current representative timings, allocation evidence, comparability limits and
priorities. The historical 31-row S11 review has 27 slower native medians, not the
28 claimed in its prose. Some references omit validation work, return existing
objects, or construct simpler errors. They are handwritten references, not Zod
measurements and not a universal language-performance ranking.

Use fused `safeParseJson` when validating raw JSON directly. The separate owned
`safeParse(JSON.parse(...))` path intentionally constructs an intermediate dynamic
tree. Preserve both APIs and their ownership contracts. Do not remove safety checks,
add hidden clones/shared wrappers, or weaken reference observations to win a ratio.

Keep timing and implementation acceptance separate. No new optimization is
required solely because a specialized reference wins a microbenchmark. Profile
representative applications before changing general schema machinery.

## Deferred work and trigger to reopen

1. **Quadratic fragment assembly in extremely deep expressions.** Recorded example:
   roughly 4.8 seconds at 32,768 nested Set constructors. Programs containing many
   ordinary functions scale linearly in the recorded checks. Reopen for measured
   compile-time/memory impact in actual generated source or an explicit untrusted
   compiler-service requirement. This is not a blocker for S12.
2. **Quadratic nested array-literal spread checking.** Retain witnesses; reopen
   when representative source makes it material.
3. **Broader architecture audits and representation limits.** Preserve known
   limitations and diagnose practical failures. The historical audit inventory is
   not an instruction to exhaust every possible shape before feature work.

The match/default-cleanup milestone completed consolidated compiler (4,008 passed,
7 ignored) and LSP (679 passed, 1 ignored) regressions. Initial failures and affected
repairs remain in SQLite. Do not repeat this full pass for routine edits. Differential
suites require pinned Node v24.11.1 first on PATH.

## S12 owned workers

The owned `spawn`/consuming `join`/explicit `detach` slice is implemented across
compiler, runtime, editor and benchmark support. See [scope and acceptance](../plans/owned-thread-acceptance.md)
for the direct-handle lifecycle boundary and evidence. Broader S12 remains open:
Bounded channels and multicore throughput are accepted; restricted local
Rust/pool integration is implemented. See [bridge acceptance](../plans/rust-rayon-bridges.md)
for its supported boundary and current checks. Broader Rust package mapping remains
separate work. Do not repeat public spawn/join groundwork or treat untimed checks as timings.

Preserve already settled decisions:

- `spawn` starts immediately. `join` observes completion; otherwise owning-scope
  cleanup joins and discards completion. `detach` explicitly opts out. No hidden
  clone, automatic detach or implicit shared ownership. Global `unwrap` propagates
  the original Result error through ordinary try/catch.
- Local Rust modules and supported wrappers belong alongside S12 before worker
  APIs harden. Automatic lazy default pooling for explicit threaded interop is
  approved; evaluate Rayon and reuse compatible pools. Direct calls stay direct.
  Queue/admission/shutdown contracts remain material design work. Arbitrary
  crates/macros/traits/lifetime-heavy APIs are not implicitly authorized.
- Async tasks are lazy, uniquely owned and single use. Calling transfers inputs;
  awaiting consumes. Start/cancel/timeout direction is approved. Never-started
  tasks warn; abandoned started jobs error. Cancellation alone does not settle a
  job or guarantee source finally execution. Detailed combinator contracts remain
  open; `std:futures` naming remains a recommendation.
- Cache reads borrow. Explicit clone creates independent supported values.
  Expiration occurs during operations; outstanding used borrows block mutation.

Later dependency order remains S13 async checking, S14 runtime/task lifecycle,
S15–S16 HTTP/TLS, S17–S18 SQLite/SQL pools, S19 cache, S20 queues, S21 integrated
service acceptance. This review does not implement or accept those features.

## Work discipline and consumers

Follow `AGENTS.md` and [development workflow](../development-workflow.md). One
agent works in this tree. Inspect status and preserve owner UI/interop plans.
Use Git checkpoints, not archives or snapshots. Format the entire project before
source checkpoints; preserve tokens, idempotence and executable fixture behavior.

Reuse `/private/tmp/tsrust-gate100-editor-target` with `CARGO_INCREMENTAL=0` and
`CARGO_BUILD_JOBS=1`. Keep normal worker stacks. Preserve exact original evidence
in SQLite, verify restoration/integrity, then remove closed loose copies. Prune
obsolete test/native outputs without deleting owner editor caches.

Five executable consumers are the installed CLI/LSP, the workspace override
`target/editor/debug/tsrust-lsp`, and shared-target debug CLI/LSP. Consumer refresh
now uses formatter source checkpoint `9fb8036f`, with atomic installed
release replacements and matching SHA-256 hashes. The primary VS Code server was
restarted and its workspace reports No Problems. The second `tsrust_test` window
was not rechecked in this pass; reload it if it still uses an older server.
Request both `--bin tsrust` and `--bin tsrust-lsp` when selecting Cargo binaries
explicitly.

The current pass adds value-producing matches, scalar literal patterns and named
record variants. See [match acceptance](../plans/match-values-acceptance.md).
It also makes worker cleanup the default for `spawn` and exposes global `unwrap`;
see [default cleanup](../plans/default-worker-cleanup.md). Compatibility spellings
remain available. Family checks and all three matched workload measurements pass.

Owned `ThreadPool` instances replace the global `pool` singleton and generic
`Pool` name. `new ThreadPool()` supplies system-based defaults; explicit options
set workers and capacity. Constructor failure throws SpawnError without unwrap.
Task, stats, shutdown, cancellation and cleanup retain the accepted ownership
rules. A shut-down instance cannot restart; a new instance has an independent
lifecycle. See [pool acceptance](../plans/worker-pool-cancellation.md). Migration
compiler/runtime/editor and offline-export acceptance passes, with zero leaks in
the cleanup corpus and current matched benchmark measurement. Release consumers
are installed and hash-verified; the restarted primary VS Code server reports
No Problems. Both installed-CLI examples produce their expected output.

Required next S12 work: dedicated-worker channels. Preserve owned transfer and
typed completion. Cancellation remains cooperative, with join acknowledging
completion; there is no forced thread termination. ThreadPool container storage is
rejected until lifecycle support exists. No async/deferred keywords or parallel
iterator API were added. The full family regression remains separate from this
focused pool/cancellation acceptance.

## Empty-delimiter formatting acceptance

Checkpoint `9fb8036f` keeps empty `{}`, `[]` and `()` inline, including gaps
containing blank lines. Comments retain their previous formatting. Acceptance:
29 formatter tests, 240 lexable fixtures / 84 callable contracts, 5 LSP formatting
tests and 8 fingerprint guards pass. Whole-project formatting changed 65 files
and refreshed 37 token-preserving fingerprints; a repeat changed zero files or
fingerprints. Installed release formatting is idempotent and its native probe
prints `0`. The primary editor server was restarted with checking enabled.
Evidence is stored under `tsrust-empty-delimiters-*.log` in SQLite. Unrelated full
regressions were not repeated.

## S13 async ownership continuation — 2026-10-15

S13 compiler/editor support is accepted: lazy `Task<T>`, consuming `await`, eager
argument/default evaluation, retained ReadOnly/Mutable loans, generic transfers,
source errors and normal finally handling. Readable examples live in
`examples/async-tasks`. Full regression evidence and the matched lazy-task
measurement are in [async acceptance](../plans/async-tasks-acceptance.md).

The next async section is S14 Tokio scheduling/I/O and started-task lifecycle.
S13's local future driver supplies no scheduling, timer or cancellation API.
Dropping a future releases owned resources without promising source finally
execution. Returning a child task retaining borrowed data is currently rejected
with an actionable source diagnostic; await it and return its value instead.
Nested owned tasks remain supported. Preserve these boundaries and the existing
thread ownership/cancellation rules when integrating Tokio.

## S14 Tokio completion — 2026-10-15

S14 adds owned scheduling with `Task.start()`, required Job settlement, cancellation,
deadlines, delay/yield, race/allSettled and bounded asynchronous channels. Compiler,
runtime and editor acceptance passes; see [Tokio acceptance](../plans/tokio-tasks-acceptance.md)
and [usage guide](../guides/tokio-tasks.md). The active editor was restarted with
current Task documentation and no open-document diagnostics.

Keep `await yield()` as settled. No external pause/resume or priority API was
requested. Cancellation remains cooperative; dropping suspended futures releases
resources without promising source finally execution. Dedicated threads, pools
and Rust/Rayon bridges remain separate. S15 HTTP is the next roadmap dependency;
networking and a blocking-work adapter were not added in S14.


## S15 HTTP completion — 2026-10-15

S15 is accepted: Axum routes/middleware, owned request/response/body/continuation/
server, reusable Reqwest clients, validated bodies, demand-driven forwarding,
disconnect and bounded graceful shutdown. See [acceptance](../plans/http-acceptance.md)
and [guide](../guides/http.md). Source af0cbba8 includes the final async disposal
repair; release CLI and both LSP paths are refreshed and VS Code restarted.

Consolidated acceptance: 4,111 compiler tests and 700 editor tests pass, seven
existing compiler tests ignored. HTTP measures 5.39 ms native versus 7.99 ms Node
per 128 sequential loopback requests. Async review repaired owned receivers,
retained loans, temporary disposal and multi-input task lifetimes; it removed one
body-chunk copy without redesigning scheduling. Exact evidence remains in SQLite.

S16 TLS listener configuration and certificate fixtures are next. Shared handler
state/providers, producer streams, WebSocket/SSE and UI runtime integration remain
separate. Preserve explicit ownership, cooperative cancellation and cleanup limits.


## S16 TLS completion — 2026-10-15

S16 is accepted: `app.listen(address, { cert, key })`, `new Client({ ca })`,
mandatory certificate/hostname/time verification and shared HTTP/TLS cleanup.
Zero-argument Client and cleartext listeners retain their behavior. Source
checkpoints are `b0656298` and `7510b254`; see [TLS acceptance](../plans/tls-acceptance.md).
All 59 affected compiler/runtime/export tests and 15 editor tests pass. HTTPS
measures 5.98 ms native versus 6.78 ms Node per 128 requests over HTTP/2 reuse.
Installed CLI/LSP consumers are refreshed; VS Code shows current TLS help and no
problems. SQLite holds original logs, failed attempts, hashes and all 186 timing
samples. No full unrelated regression matrix was repeated after S15.

The B4 repair agent has the exclusive validation window for
`/private/tmp/tsrust-gate100-editor-target`: `CARGO_INCREMENTAL=0`,
`CARGO_BUILD_JOBS=1`, Node v24.11.1 first on PATH, loopback escalation when needed.
TLS has no remaining builds, timings or consumer refreshes. Preserve the active
shared target until B4 finishes. The UI briefs remain owner work. S17 SQLite is
the next standard-library roadmap item; no work on it started in this pass.


## S17 SQLite and B4 workspace repair — 2026-10-15

S17 is accepted at `91e6b937`, with indexed editor completion repaired at
`b255009e`. B4 is merged at `e33af7db`; its missing callable access-summary entry
caused the workspace panic visible even when an empty/comment-only file was open.
The full workspace now checks without panic or errors. The remaining `unused-task`
warning on an awaited greeting is a pre-existing false positive, verified with the
old installed compiler; do not describe it as intentionally discarded work.

See [SQLite acceptance](../plans/sqlite-acceptance.md) for exact checks, consumer
hashes and measurement provenance. New examples are `examples/sqlite/people.tsr`
and `examples/sqlite/rollback.tsr`. Compiler ownership rejects closing a borrowed
connection, committing with live transaction statements, handle cloning and reuse.
Runtime checks cover scope/error rollback, failed commit, SQLite automatic rollback,
strict data conversion and lock cleanup. Borrow provenance through `unwrap` is fixed
for existing byte views too.

All affected compiler/runtime/export checks and 18 editor checks pass. Installed
CLI and LSP binaries are refreshed; the active VS Code server was restarted and
current SQLite hover verified. Matched prepared transactions measure 0.031084 ms
native versus 0.105708 ms Node per 128 rows; versions differ, this is an in-memory
whole-stack baseline. SQLite retains exact evidence and all 186 timing samples.

UI identity feasibility now has a separate managed worktree and the quiet
check/emit/rustc/format window. It uses the verified `b255009e` release CLI; any
compiler dependency requires a minimized witness and coordinated ownership first.
Preserve the five untracked owner UI plan files. S18 async SQL remains pending;
this pass adds no ORM, database pool, migrations or async SQLite adapter.
