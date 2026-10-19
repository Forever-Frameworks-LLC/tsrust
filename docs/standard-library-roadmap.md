<!-- Current continuation takes precedence over historical scheduling below. -->

Current UI application contract (2026-10-18): owned theme JSON, shared semantic
utilities, editable local components, exact offline installation/locks and LSP
are implemented. Direct signal-reading disabled predicates refresh after handlers.
See [the application guide](guides/ui-applications.md) and current handoff for
focused acceptance and explicit limits. Required next UI work is editable controls,
string/boolean state, keyed lists and persistent routing; historical UI pause and
resource-dependency statements below are superseded.

Exact decimal focused acceptance is complete on macOS: `std:decimal.parseDecimal`,
checked arithmetic/conversions, editor/examples, offline export and untimed matched
money workload. Calendar dates also pass focused compiler/native/editor, offline
export and untimed date-fns acceptance. Time of day, UTC instants, IANA/fixed zones
and exact durations also pass focused acceptance, including explicit DST and
precision failures. Exact UUID text/bytes and fallible random generation now pass
focused native/editor/export acceptance. Explicit Rust adapters and all three SQL
domain mappings pass focused acceptance. Data-type family regressions and targeted
integration repairs and all five accumulated measurements pass. Continue S19–S21. Date/time API design follows the owner's date-fns preference.
See [current data-type coverage](data-type-coverage.md) and the
[current handoff](handoffs/compiler-to-roadmap-continuation.md).

# Modules, native standard library and async roadmap

Current S21 implementation status (2026-10-16): the scoped Router-over-Axum API,
typed application/request state, cookie helpers and declarative service routes
pass targeted compiler/native/editor acceptance. The revised integrated service
passes on macOS and offline Linux aarch64, including toolchain relocation and
both shutdown signals. Authentication remains application policy. Installed
consumers are refreshed. Full-family regressions and new performance measurements
remain a separate pending checkpoint; no unmeasured workload is described as timed.
See [service readiness](plans/service-readiness-plan.md) for exact boundaries.


Current continuation: [compiler foundations through S12–S21](handoffs/compiler-to-roadmap-continuation.md).
Current owner priority (2026-10-15): S18 core acceptance is complete. Next deliver
[checked resource lifetimes for UI](#owner-priority-compiler-dependency-checked-resource-lifetimes-for-ui),
then [required data-type coverage](#owner-priority-required-data-type-coverage), then
continue S19, S20 and S21 without waiting for another instruction. UI implementation
is paused on the resource dependency; coordinate its resumption when that dependency
is accepted. Read the current continuation first; older next-step statements below
are historical and do not override this order.

Owner-prioritized [diagnostic suppression directives](plans/diagnostic-suppression-plan.md)
are accepted and installed for `check` and LSP reporting. Strict compilation stays
unchanged. S10 is accepted; follow the current priority order above.


Status: S01–S03 module loading/checking/native CLI accepted with focused evidence; S04 project editing is accepted at gate 185; S05 primitive native bindings/build is accepted on macOS with focused evidence; S06 type foundations are accepted at gate 186; S07 is accepted on macOS after gates 187–190 native bytes, views and file resources; S08 collections is accepted on macOS; S09 synchronous host foundations are accepted on macOS; S10 JSON is accepted; S11 schemas and S12–S18 capabilities are accepted within the documented boundaries below; S19/S20 are accepted within the boundaries below; scoped S21 implementation is accepted, with the full-family/performance checkpoint pending. Project-wide multiple-error diagnostics is accepted on macOS; see the [diagnostics acceptance](plans/project-diagnostics-plan.md). User-approved direction: bundled standard library, Rust package backends, Tokio-backed explicit async/await and native threads. No custom goroutine scheduler. This file is the progress ledger and compiler/editor handoff contract. See [module loader acceptance](milestones/modules-01-project-loader.md) and [module checking acceptance](milestones/modules-02-project-linking.md).

[Researched Node and ecosystem coverage inventory](standard-library-coverage.md) maps every pinned Node API family, representative Node/Rust packages, application witnesses and supplemental H-series gates. Its per-export inventory and popularity measurements remain explicit follow-up requirements before broad parity or ranking claims.

## Owner-priority compiler dependency: checked resource lifetimes for UI

Priority recorded 2026-10-15 after review of the B2 investigation. UI implementation
is paused until the compiler dependency below is accepted and available to validate.
S18 is now accepted; this is the next compiler priority. Coordinate exclusive
checkout and build ownership before shared native-type, ownership or emission edits.
This entry records priority and acceptance, not completion or permission to interrupt
an active validation run.

The current external Rust bridge projects public records field by field and rejects
exported types implementing `Drop`. That rejection is intentional. Removing it can
release a native resource during conversion or produce invalid partial moves.
S18 compiler-owned SQL handles do not establish a general external-resource bridge.
See the [B2 report](../experiments/ui-tsrust-owner-handles/RESOURCE-RELEASE.md),
[26-row matrix](../experiments/ui-tsrust-owner-handles/release-matrix.json) and
[review](../experiments/ui-tsrust-owner-handles/RESOURCE-RELEASE-REVIEW.md).

Deliver the smallest checked owned-resource facility that can carry a real tracked
native resource through Tsrust state, retained callbacks and queued payloads without
losing its native identity or destructor. Evaluate a narrow test-resource facility
before committing to a general public opaque adapter API. A source-side cleanup
counter, manually released integer registry, patched emitted Rust, or relaxed Drop
rejection does not satisfy this dependency.

Current checked resource/UI implementation and remaining acceptance are recorded in
[the continuation mapping](../experiments/ui-tsrust-owner-handles/CHECKED-RESOURCES.md).
The automatic child-before-parent destruction defect is repaired. Compiler/native
witnesses, full required editor suites and installed consumers pass on macOS arm64.
The narrow synchronous contract is accepted; async/worker UI readiness remains
separate. Continue the [data-type inventory and gaps](data-type-coverage.md).

Required compiler acceptance:

- Preserve ownership and actual destruction through construction, move, record/array
  storage, replacement/removal, callback retention, explicit close and automatic scope
  exit. Reject use after transfer and unsupported clone/field projection.
- Expose an independent non-retaining release audit. Calibrate live-versus-released,
  exact-once and ordering assertions with deliberately wrong expectations and positive
  controls. Distinguish wrapper destruction from underlying backend cleanup completion.
- Define borrowed inspection, returned-resource lifetime relationships and error paths.
  Prove compiler/native behavior and editor diagnostics, completion, hover and parameter
  help agree. Any new built-in receives complete IntelliSense documentation.
- Preserve the intended async/worker UI architecture: specify transfer eligibility,
  thread affinity, active-use ownership across suspension, cancellation versus cleanup
  completion, and the owner executor's mutation authority. Do not infer thread safety
  from a structural record or assume dropping a future completes backend cleanup.
  Document and test supported contracts; explicitly reject unsupported transport.
- Keep plain-data bridge behavior and S18 transaction/pool borrowing intact. Use focused
  cross-feature regressions for shared surfaces and separately coordinated build windows.

UI resumes with the existing B2 matrix once the checked resource path and observer
are validated. B2 is not closed by implementing the compiler primitive alone: test
state, captures and payloads separately, callback-safe release, automatic owner drop,
descendant-before-ancestor order and repeated resource balance. Automatic cleanup
may reveal an owner representation/finalization dependency. Concurrent runtime identity
allocation, late async completion rejection, cancellation and thread-affine cleanup
remain explicit follow-up obligations; synchronous success is not worker readiness.

## Owner priority: required data-type coverage

After the resource dependency, complete a bounded coverage pass for the data types
needed by UI, Rust adapters, SQL and the remaining service roadmap. Start from current
compiler declarations, built-in metadata, guides and accepted fixtures; do not repeat
accepted implementation or infer support from a backend crate alone.

Maintain one matrix mapping required types to source syntax, representation, operations,
conversion rules, ownership/transport, compiler/native/editor coverage and executable
witnesses. Cover primitives and numeric ranges/precision; strings/Unicode; bytes/views;
null/optional values; records, arrays, tuples and collections; unions/variants/results;
JSON and schema values; callbacks; owned resources, tasks and jobs. Explicitly assess
SQL/Rust boundary requirements for integer widths/signedness, exact decimal values,
date/time/time zones/durations, UUIDs, JSON, binary data and nullable columns.

Implement missing types or mappings needed by concrete UI and S19–S21 examples, with
lossless conversions or explicit checked errors. Reuse existing representations where
they preserve behavior. Do not silently round, truncate, stringify or clone to claim
coverage. Record genuinely unsupported or deferred types with the affected use case
and a safe documented alternative; no universal Rust-crate or database-type claim.
Complete compiler/runtime/editor support and focused acceptance for each coherent
section. This priority is a practical coverage gate, not an exhaustive type-system
redesign. Accepted scalar SQL checks remain valid; new mappings need their own driver
witnesses.

Once both priorities are accepted, continue the existing S19 cache plan, S20 queues
and S21 integrated service acceptance in order. Do not stop at a design checkpoint or
wait for a new “go ahead.” Preserve scoped limitations and evidence, and ask only for
material unresolved decisions or external blockers. Full regressions and measurements
belong at the meaningful family milestones defined by the development workflow.

## Product contract

Ship supported capabilities with the Tsrust toolchain. Users import std: modules without maintaining Cargo dependencies. Keep stable Tsrust APIs independent of backend crates. Link only selected features; a console-only program must not acquire database, HTTP or async runtime dependencies. Rust crates are implementation dependencies, not arbitrary source-level Rust interoperability.

Preserve JavaScript-compatible globals where their behavior is already supported. Native modules have explicit ownership, errors, limits and lifecycle contracts. No claim of complete Node or Zod compatibility until a published coverage matrix is verified. Existing clone remains an explicit independent value copy; cloning a resource handle is a separate documented operation, never an accidental database or socket duplication.

Proposed namespaces: std:path, fs, env, process, bytes, time, collections, json, schema, task, thread, channel, sync, http, tls, sqlite, sql, cache and queue. Spellings and signatures remain design proposals until their gate closes.

The owner-approved [globals/imports policy and mandatory audit](plans/globals-and-imports-policy.md)
requires exhaustive per-symbol reconciliation before adding new conventions.
The accepted JSON namespace exposes throwing `JSON.parse` and Result-returning
`JSON.tryParse`, preserving native safety and existing imports. Signatures and
IntelliSense must expose semantic differences; runtime dependencies follow actual
usage. Implementation follows the prioritized typed-data imports and aliases and
is required before S10 acceptance. The audit and namespace now have
[focused acceptance](milestones/json-namespace.md); timing and consumer refresh
remain part of the S10 family milestone.

## Architecture and compiler handoffs

Required organization checkpoint: after focused JSON namespace acceptance and
before further S10 API expansion, reorganize compiler sources, including strays
inside `stdlib`, into coherent modules. Start with filesystem implementation and
tests, then proceed by dependency family. Preserve public APIs and runtime behavior;
see the [compiler source organization plan](plans/compiler-source-organization-plan.md).

Keep in-process frontend analysis independent of network, databases, Cargo and native execution. Add project analysis alongside existing check(source), preserving the single-source API. Explicit source identity must accompany project symbols, diagnostics, related spans and edits; do not concatenate files and guess offsets.

A compiler-owned typed native binding registry defines argument/result types, generics, borrowing/mutation/transfer, callback retention and invocation, async behavior, thread-transfer constraints, errors and required runtime features. Library modules consume this registry. The editor must consume public metadata, not independently invent these contracts. Every new method/property still receives complete builtins.json documentation before gate acceptance.

The native build driver resolves an exact pinned dependency/feature graph and cached generated Cargo project. Dependencies are acquired in an explicit toolchain/setup operation; frontend checking and ordinary offline builds never silently fetch packages. Retain lockfiles, licenses, target support and dependency provenance in developer evidence. Only std imports select native features. Preserve direct rustc compilation for programs requiring no external runtime packages where practical.

Shared compiler integration points: parser/type representation, symbol resolution, ownership/loans, checked IR and Rust emission. Modules and host handles cannot land as editor-only conveniences. Keep these changes coordinated with ongoing numbered language gates; do not allocate their gate numbers or alter their running checkpoint.

## Sequenced gates and progress ledger

S01–S18 have accepted capabilities within their documented platform and feature boundaries. S19/S20 are accepted within the boundaries below; scoped S21 implementation is accepted, with the full-family/performance checkpoint pending. A gate closes only with its listed focused evidence plus the common acceptance requirements below. Implementation acceptance and performance/checkpoint acceptance are separate fields.

| Gate | Depends on | Concrete deliverable and exit evidence | State |
|---|---|---|---|
| S01: module semantics | none | Commit import/export, entry main, type identity, resolution, cycles and diagnostic API decisions with positive/negative fixtures; no unresolved first-release semantics | accepted: module loader evidence |
| S02: project loader | S01 | Explicit relative .tsr imports, canonical identities, deterministic graph, missing/duplicate export and cycle diagnostics; diamond loads once; no std/package implicit file search | accepted: module loader evidence |
| S03: module checking/emission | S02 | Cross-file functions/types/ownership, private symbols, name collisions and source order; only selected entry main auto-runs; imported main ordinary; native two-file and diamond programs pass | accepted: [library/CLI/native witnesses](milestones/modules-02-project-linking.md); scaling measurements pending checkpoint |
| S04: project editor | S03 | Unsaved imported buffers, dependent invalidation, cross-file hover/definitions/diagnostics/versioned fixes, rename/removal and stale result tests; single-file behavior preserved | accepted: gate 185 ([record](milestones/185-project-editor.md), [map](b9-project-editor-acceptance-map.md), [guide](project-editor.md)); closed-file fixes are guarded by disk text; help survives unfinished text and imports; import names complete from the module exports; packaged-extension run of the gate 185 additions is pending |
| S05: std bindings/build | S03 | Registry-backed std import and one small path operation; exact feature lock/cache, offline warm build, friendly backend errors, unused feature exclusion and installed CLI/editor witness | accepted on macOS: [requirement audit](milestones/stdlib-01-bindings-build.md); primitive std:path, native function values within current language boundaries, cache/offline/copied CLI/editor witnesses; frontend/build and scaling measurements accepted; historical runtime evidence reconciled |
| S06: generics/results | S05 | Generic functions/containers, bounded specialization, tagged Result success/error, optional types integration, exhaustiveness and typed error propagation with ownership/drop tests | accepted: [generic and callable foundations](milestones/stdlib-02-types-results.md) and [tagged Result, exhaustive statement match and explicit propagation](milestones/186-result-match.md); shorthand propagation is separate |
| S07: resources/bytes | S06 | Owned opaque handles, explicit close plus deterministic drop, use-after-close rules, byte buffers/views, bounds and native errors; cleanup on early return and failure | accepted on macOS: gates 187–190, complete regressions, 17 matched measurements, direct Rust review and refreshed consumers; [evidence](milestones/s07-resources-bytes.md) |
| S08: collections | S06 | Map, Set, ordered map/set, deque and heap; key equality/hash/order, borrowing iterators, invalidation and explicit clone; boundary and native reference tests | accepted on macOS: [family regression, measurement, optimization and consumer evidence](milestones/s08-collections.md) |
| Dev-loop performance: owner priority | S03, S05, new/dev CLI | Filesystem events, retained compiler state, separate native runtime compilation, bounded caches; clean-build equivalence, lifecycle tests, repeated scaling measurements and installed acceptance | accepted on macOS, Linux watcher-module checks accepted; installed lifecycle, full regressions and matched dev-loop measurements retained; [acceptance plan](plans/dev-loop-performance-plan.md) |
| S09: synchronous host | S07 | Path/fs/env/time foundations, file read/write and buffered streams, size limits, monotonic clock; filesystem sandbox fixtures and cleanup | accepted on macOS: 3,207 compiler and 532 LSP tests pass; nine matched measurements and allocation profiles, final installed checks and editor restart pass; see [host acceptance](milestones/s09-host-measurements.md) |
| S10: JSON | S07, S08 | Typed decoding/encoding and explicit JsonValue, correct number/Unicode/property rules, malformed/depth/size errors, JSONC/JSONL, bounded file streaming and allocation baseline; see JSON contract | accepted: see [S10 acceptance](milestones/s10-json-acceptance.md); full compiler/LSP regressions, matched measurements, optimization review, installed consumers and cleanup complete within documented scope |
| S11: schemas | S06, S10 | Typed reusable schema API, structured issue paths, safeParse, object/union/optional/refinement/transformation coverage and fused JSON validation; see schema contract | accepted on macOS: inferred reusable schemas, collections/recursion, pipelines, throwing parse, fused validation and scoped Draft 2020-12 export; compiler 3,727 and LSP 645 pass after cleanup; all 31 matched workloads measured separately; consumers refreshed; see [acceptance and handoff](handoffs/2026-10-13-s11-acceptance.md) and [schema plan](plans/schema-plan.md) |
| S12: native threads | S07 | Spawn/join with explicit transferred captures, Send-like restrictions, panic/failure conversion, bounded channels; multicore witness and compile-time invalid sharing tests | accepted within documented scope: dedicated spawn with default cleanup, owned ThreadPool, explicit cancellation, bounded channels and Rust/Rayon bridges; see [pools](plans/worker-pool-cancellation.md), [channels](plans/bounded-worker-channels.md), [bridges](plans/rust-rayon-bridges.md) and [package CLI](plans/package-cli-acceptance.md); external owned resources are the priority dependency above |
| S13: async compiler | S06, S07 | async functions/await/task result types, async main selection, suspend-safe captures/loans/drop, correct control flow and diagnostics; generated native futures execute | accepted; lazy single-use tasks, suspend-safe ownership, native execution and editor agreement pass; see [async acceptance](plans/async-tasks-acceptance.md) |
| S14: Tokio tasks | S12, S13 | Runtime bootstrap, explicit .start()/task groups, timers, bounded async channels, cancellation/timeouts and shutdown; blocking work separated; no leaked tasks/resources | accepted; owned scheduling, timers, groups, bounded channels and cancellation cleanup pass compiler/runtime/editor acceptance; see [Tokio acceptance](plans/tokio-tasks-acceptance.md) |
| S15: HTTP | S10, S11, S14 | Client/server request/response, typed validated bodies, limits, streaming/backpressure, disconnect and graceful shutdown; loopback integration and cancellation tests | accepted; Axum-backed HTTP, ownership/cleanup, editor and matched measurements; see [HTTP acceptance](plans/http-acceptance.md) |
| S16: TLS | S15 | HTTPS client/server, certificate/key configuration, hostname verification and trust roots, handshake errors; local certificate positive/negative fixtures | accepted on macOS; verified HTTPS, scoped private roots, ownership/cleanup, editor and offline exports pass; see [TLS acceptance](plans/tls-acceptance.md) |
| S17: SQLite | S07, S10 | rusqlite-backed synchronous connection, prepared statements, typed rows/parameters, transactions and rollback/drop; SQLite integration without async requirement | accepted; [evidence](plans/sqlite-acceptance.md) |
| S18: async SQL | S11, S14, S17 | SQLx-backed SQLite/Postgres/MySQL; bounded pools, typed binding/rows, transactions and timeout/cancellation; each driver accepted separately | core and migration orchestration accepted within documented recovery limits; [evidence](plans/async-sql-acceptance.md) |
| S19: caches | S08, S09 | Owned bounded LRU/TTL cache, eviction/expiration contracts and fake-clock tests; concurrent cache/loading and stampede policy follow S14 | local LRU/TTL and bounded serialized loading pass focused compiler/native/editor acceptance; explicit completion, cancellation and closed clients are covered; [shared S19/S20 regressions and measurements pass](milestones/s19-s20-owned-coordination.md); see [local acceptance](milestones/s19-local-cache-acceptance.md) |
| S20: queues | S08, S14 | Bounded in-memory work queue, producer/consumer ownership, backpressure, close/drain, cancellation and worker shutdown; durable/distributed adapters are separate gates | focused implementation acceptance passes using existing Channel/task primitives; [acceptance](plans/work-queue-plan.md); [combined S19/S20 regressions and measurements pass](milestones/s19-s20-owned-coordination.md) |
| S21: practical service readiness | S04, S15–S20, data coverage | Authenticated deployable service, shared handler state, migrations/recovery, observability, HTTP/security contracts and Linux/container/offline/signal acceptance; see requirements below | scoped Router/service implementation accepted on macOS and offline Linux; full-family/performance checkpoint pending; polished MVP/demo deferred |

S17 can proceed before async once S07/S10 land. S08/S09 and thread research can proceed independently after shared prerequisites. S01–S05 are the immediate concurrent track alongside built-in methods. Compiler integration is coordinated work in shared files, not safe unlimited concurrent edits.

## Practical service readiness and coverage methodology

Current owner direction: expose a Hono-style HTTP API backed by Axum, including
context, headers, cookie helpers, checked request-local state and safe shared
application state. Authentication and session policy remain application code;
the authenticated witness below does not authorize a framework authentication
library. Complete these HTTP primitives before further deployment verification.

S21 must close on an authenticated, deployable service acceptance witness, not a
minimal demo. Defer the separately polished MVP/demo application until after this
acceptance. Executable service witnesses remain required throughout implementation.
Complete these requirements in coherent, independently accepted sections:

- Safe shared handler state for database pools, cache and configuration.
- Versioned migrations with failure, recovery and schema-drift checks.
- Structured redacted logging, request/task correlation, useful source errors and
  basic metrics/tracing.
- Configurable HTTP limits and timeouts, with required forms, multipart, cookies,
  CORS, rate limiting and static-serving contracts scoped to the concrete service.
- Maintained-library security primitives and session or token integration.
- Actual Linux/container, offline-build, signal and graceful-shutdown acceptance.
- Overload, ownership, cancellation, leak checks, editor agreement and repeatable
  toolchain distribution, preserving the existing acceptance requirements.

Calendar dates are a first-class prerequisite. Inventory calendar dates, time of
day, UTC instants/timestamps, offsets/time zones, durations versus monotonic clocks,
parsing, formatting, arithmetic, comparison, invalid dates and DST ambiguity,
serialization and each SQL driver mapping. `monotonicNow` alone does not establish
calendar-date support. Include numeric widths/signedness, exact decimals, UUIDs,
JSON, binary and nullable values alongside language/container/resource types.
Reuse suitable maintained backends and require explicit checked conversions.

Compare pinned TypeScript language-service/tsserver declarations and rust-analyzer
results on representative projects against Tsrust compiler/LSP results. Record
standard library/crate versions, features and targets. Inventory constructors,
methods, properties, overloads, generics and errors; classify each as supported,
partial, intentionally different, missing or deferred. Rust has no universal crate
API set: pin the selected crates. TypeScript declarations describe types rather
than establish runtime behavior. Completion, hover and signature comparisons find
gaps; declaration inventories and positive/negative compiler/native witnesses prove
behavior and ownership. This methodology is required work, not a claim that the
comparison tooling or runtime parity already exists. Do not pursue arbitrary npm
or Rust-crate compatibility.

After S21, order non-UI follow-up as follows, promoting only concrete service needs:

1. Database cursor streaming and bounded application-produced response channels
   now pass focused acceptance. Callback-owned producer lifetimes remain a separate
   concrete-workload follow-up; see the current handoff.
2. Application-level SQLite durable jobs now cover acknowledgement, retries,
   idempotency, recovery and explicit delivery guarantees. The owner selected an
   example using existing SQL pools, not a new standard library. See
   [acceptance](plans/durable-jobs-acceptance.md).
3. External cache/Redis and WebSocket/SSE as application demand requires.
4. Broader host/subprocess/filesystem-watch/compression/URL-form utilities, curated
   bindings and source debugging.

Query builders and compile-time SQL checks are optional after migrations. Full
ORMs, universal Rust/npm compatibility, distributed queue platforms and extreme
nesting optimizations remain deferred. Reconcile these items against actual code
before adding duplicate work.

## Module decisions for S01

First scope: explicit named imports/exports and explicit relative .tsr paths; std: reserved for shipped modules. No CommonJS, dynamic imports, bare package discovery, glob imports or implicit extension/index search. Resolve relative to importer, normalize filesystem identities, specify symlink/case behavior and report ambiguous identities. Exported types retain defining-module identity.

First scope has no executable top-level initialization and rejects import cycles with an import-chain diagnostic. Constants and re-exports require separate explicit semantics before inclusion. Each file may define its own main; only the chosen entry's zero-argument procedure runs automatically. No imported main auto-run. Later initialization/cycles/package gates must amend this contract explicitly.

Project API returns source-indexed checked symbols/contracts and diagnostics. Source provider supports disk and unsaved buffers without frontend I/O. CLI selects entry, LSP identifies project roots and dependency snapshots. Build caching keys include source graph, toolchain, std feature graph and lock identity.

## JSON correctness and efficiency contract

Immediate S10 priority: owner-approved typed JSON/JSONC file imports and project
path aliases, including extensionless `.tsr`/`index.tsr` resolution and shared
compiler/editor data types. These precede further JSON API and benchmark expansion.
See [typed data imports and aliases](plans/typed-data-imports-and-aliases-plan.md)
for behavior, implementation order and acceptance. Both additions have focused
compiler/native/editor acceptance; family measurements remain pending.

Provide two honest paths: dynamic JsonValue for arbitrary data and typed schema/record decoding for known shapes. Typed decode should parse directly into typed output without first constructing a generic tree; fused validation avoids stringify/reparse and unnecessary intermediate copies. No blanket any or unsafe cast after parsing.

Specify null versus missing versus undefined, duplicate key behavior, key ordering, exact integer limits versus floating values, negative zero, nonfinite encoding, Unicode escapes/surrogates, trailing data and error offsets. Default strict JSON; relaxed syntax requires separate APIs. Resource limits cover input bytes, nesting, collection sizes and streaming buffered bytes. Serialization defines cycles, unsupported values and ownership.

Owner-requested JSONC and JSON Lines support is part of S10 acceptance. Keep strict JSON as the default; expose explicit format APIs. Include bounded synchronous file adapters and record-at-a-time consumption over the accepted buffered file layer, with EOF, line/byte errors and deterministic resource cleanup. See [the JSON variant and file-stream contract](plans/json-plan.md#jsonc-json-lines-and-convenient-file-streams).

Use serde_json as the initial correctness baseline. Borrowed/raw fragments are opt-in with input-lifetime checks; owned results remain independently usable. SIMD parsing is a measured candidate, not a promise: mutable-input requirements, CPU portability, number semantics and UTF-8 validation must fit the API before adoption. Streaming supports bounded incremental input and outputs; chunk boundaries may split UTF-8 and escapes.

Benchmark tiny API bodies, large records, numeric arrays, escaped Unicode, deeply nested data, selective/raw fields, invalid inputs and streaming. Retain payloads and outputs. Measure bytes/sec, p50/p95 latency, allocated bytes/count, peak resident memory and wrapper overhead versus direct Rust baseline with identical output/retention. Separate parse-only, parse+validate and serialize measurements, warm and cold costs. Register workloads at implementation acceptance; publish measurements only at scheduled checkpoints. Performance budget is fixed from retained baseline before optimization, not invented after results. Correctness and allocation evidence precede any claim of extreme efficiency.

## Built-in validation inspired by Zod 4

Ship std:schema, with ergonomic reusable schemas and inferred output types. This is a native implementation of specified behavior; Zod is a test/design reference, not an executable dependency. Publish a method-by-method coverage/difference matrix before advertising parity.

First tranche: strings, numbers/integers/booleans/null; object properties, arrays, literals/enums, optional/nullable/default fields, unions/discriminated unions, unknown-key policies; string length and numeric range checks. safeParse returns a typed success/error Result. Errors have stable codes, property/index paths, expected/observed summaries and bounded issue collection. Do not expose input secrets in errors by default.

Second tranche: refinements, transforms, preprocessing, recursive/lazy schemas, record/map/set schemas, cross-field checks, schema composition and JSON Schema export with explicit representability limits. Async refinements follow S14. Regex/date/format validators wait for their own verified semantics; names must not imply stronger validation than delivered. Explicit coercion and transformation must distinguish input and output types.

Compile schema literals into typed validators where possible; reusable runtime schema objects remain supported under specified lifetime/ownership rules. Freeze a schema plan once, avoid rebuilding it per request. Preserve documented validation/transform order, default evaluation, union diagnostics, receiver immutability and rollback/drop on partial failure. Output construction and clone policy must be explicit. Verify type inference, rejected misuse, native example output, Zod differential cases for claimed equivalent behavior and fused decoder allocations. Validation errors are ordinary data; throwing parse requires a separately accepted language error model.

## Runtime, concurrency and resources

Tokio runs explicit async/await; no hidden suspension in ordinary functions. Async concurrency and OS-thread CPU parallelism have separate APIs. Define transferred owned captures, borrowed lifetimes, Send/Sync-like capability inference and rejection of non-shareable resources. Immutable sharing and synchronization must be explicit; do not automatically clone or wrap values in Arc/Mutex.

Task groups own child lifetimes. Define join, cancellation, failure propagation, detach permission, timeout behavior, cooperative cancellation and runtime shutdown. Cancellation is not guaranteed to interrupt arbitrary synchronous or native blocking work. Await cannot retain an illegal loan or mutex guard across suspension. Channels specify capacity, ordering, closure, lost receivers and whether sends return unsent values. HTTP and queue work must exert bounded backpressure.

Opaque handles specify ownership, permitted transfer/sharing, close idempotence, use-after-close diagnostics and destructor fallback. Errors preserve useful operation/source context without dumping generated Rust or credentials. Fallible APIs use Result first. Exceptions/throw remain a separately coordinated compiler feature.

## Database, cache and queue scope

SQLite first: owned connection, prepared parameter binding, typed rows, transactions, busy/lock errors and explicit close. Document text/blob/integer/null conversion, statement lifetime and rollback on failure. Async SQLite must isolate blocking work appropriately.

SQLx: start with one driver then gate each additional driver. Expose pool limits/acquisition timeout, health/reconnect rules, statement cache, streaming rows and transaction scope. Runtime parameterized queries must work without database access during compilation. Compile-time SQL checking is a later explicit prepare/offline-metadata workflow; ordinary frontend/editor checking never connects to a database. Do not mistake schema validation for SQL injection prevention: bind values and constrain identifier construction.

Caches: separate local cache, concurrent cache and external cache clients. Specify TTL clock, lazy/eager expiration, byte/item bounds, eviction, ownership on get/remove and stampede handling. Cloning a cached value must be explicit. Ordered maps supply tree-backed operations before specialized graph/tree APIs.

Queues: separate deque, async channel/work queue and durable broker clients. In-memory queue has no persistence guarantee. Durable jobs require acknowledgements, retry/backoff, idempotency/deduplication, recovery and explicitly stated delivery guarantees; do not promise exactly-once processing. Redis/broker clients are later separately accepted adapters.

## Coverage backlog

Track individually: filesystem/path/env/process; bytes/encoding/base64; wall/monotonic time and dates; entropy/random/UUID and cryptography; JSON/schema; maps/sets/ordered maps/heaps/deques; tasks/threads/channels/synchronization; TCP/UDP/DNS/HTTP/TLS/WebSocket; SQLite/SQL pools/migrations; caches/work queues/external brokers; compression; regex; structured logging/metrics/tracing and testing helpers. No family is implemented merely because its namespace exists. Generic trees/graphs, distributed queues, ORM, custom scheduler and arbitrary Rust package imports remain later scope.

## Gate acceptance and evidence

Follow development-workflow.md and existing AGENTS requirements. Focused checks per gate; full regressions and performance measurements at meaningful feature-family milestones, not fixed gate counts. This S-series is a dependency ledger, not a replacement numbering system. Before implementation begins, map accepted work to the compiler's actual gate schedule without renumbering another agent's work. Preserve running regressions and passed records.

For each gate record: changed source identities; exact commands/results; positive and negative fixtures; ownership/loan/drop/cancellation witnesses; native execution when output matters; editor contract checks; complete builtins.json signatures/parameter help/behavior/examples; dependency/feature graph; platform coverage; benchmark registrations and separately pending measurements. Library claims require real native integration, not just parser acceptance.

Update this ledger after each gate using: state (pending/in progress/blocked/accepted), implementation evidence link, measured evidence link or pending checkpoint, affected APIs, unresolved dependencies and next step. Blocked means a concrete missing contract, not general difficulty. No gate can close with a known unaddressed correctness regression in its changed behavior.

The earlier S11 stop boundary is superseded by the owner-approved S12 owned spawn/join slice. The full roadmap remains S11–S21 in dependency order. S10 JSON is accepted; see [acceptance](milestones/s10-json-acceptance.md). S09 is accepted on macOS; see [host acceptance](milestones/s09-host-measurements.md). The required [dev-loop performance milestone](plans/dev-loop-performance-plan.md) is accepted on macOS with Linux watcher-module evidence: filesystem events, retained compiler state, separate runtime compilation, bounded caches, installed lifecycle checks and matched measurements are complete. S08 collections and the user-requested [project diagnostics milestone](plans/project-diagnostics-plan.md) are accepted. The owner-prioritized [new/dev CLI workflows](plans/project-cli-workflows-plan.md) are accepted on macOS with installed CLI checks. Continue S11–S21 in dependency order. S07 direct Rust review and final consumer refresh are accepted. Recursive views, callback support and gate 190 resource lifecycle have focused acceptance. S07 complete regressions and 17 matched measurements are recorded in `milestones/s07-resources-bytes.md`. S04 is accepted at gate 185. S05 two-project frontend/build and module scaling measurements are accepted; basename runtime evidence is reconciled with the accepted lexical path family. See the S05 requirement audit for exact record identities and scope.

## Primary references

- [Tokio tutorial](https://tokio.rs/tokio/tutorial): runtime/tasks and asynchronous execution.
- [Axum](https://docs.rs/axum/latest/axum/): candidate HTTP backend integrated with Tokio/Hyper.
- [Serde JSON](https://docs.rs/serde_json/latest/serde_json/), [RawValue](https://docs.rs/serde_json/latest/serde_json/value/struct.RawValue.html): typed/dynamic and raw/borrowed decoding baseline.
- [Zod schema API](https://zod.dev/api), [basic parsing](https://zod.dev/basics): validation behavior reference; no blanket compatibility claim.
- [Rusqlite](https://docs.rs/rusqlite/latest/rusqlite/): synchronous SQLite candidate.
- [SQLx](https://docs.rs/sqlx/latest/sqlx/), [query metadata requirements](https://docs.rs/sqlx/latest/sqlx/macro.query.html): async drivers and explicit compile-time query checking workflow.

Pin exact backend versions and features during S05/individual adapter acceptance after toolchain and license verification. Reference links do not select an unverified latest release.

## Accepted lexical path increment

The bounded primitive `std:path` family adds dirname, extname, isAbsolute and normalize alongside basename. Compiler/native execution, imported editor help, executable examples, matched benchmark registration and scoped native/Node-after-tsc measurements are accepted on the library branch. See [family acceptance](milestones/stdlib-lexical-path.md) and [integration handoff](handoff-2026-10-07-lexical-path.md).

Public integration is verified on macOS; explicit-platform helpers cover POSIX/Windows reference behavior. Windows native integration and the advancing combined checkout still require their own checks. This primitive-only increment does not accept filesystem resources, the wider host family, or the rest of the standard library.

## Relative path increment

`std:path.relative(from: string, to: string): string` adds borrowed-pair lexical resolution with owned results and checked cwd failures. [The contract](path-relative-contract.md) distinguishes host execution from deterministic Windows helper/reference coverage. Focused native/compiler/editor examples and matched workload registration are verified in the isolated worker. Family measurements, combined-main observer verification and required source/evidence/consumer integration remain separate acceptance steps. No further path or OS family is included.


## Owner-approved S12 interop expansion

Design local Rust modules and supported native package wrappers alongside S12,
before public worker APIs harden. Provide automatically managed default pooling
for threaded interop, evaluating Rayon for CPU work, with lazy setup and reuse
across compatible wrappers. Preserve explicit execution/completion and ownership
contracts. Direct calls remain direct. See the [interop contract and pool acceptance](interop-and-targets.md#approved-pool-direction-and-implementation-recommendations).
This supersedes the blanket deferral of this restricted Rust integration above;
arbitrary package imports, traits, macros and lifetime-heavy APIs remain separately
scoped. Public syntax, implementation and measurements are not yet accepted.


## Required cleanup between S11 and S12

After S11 acceptance, complete a dedicated source cleanup before beginning S12.
Organize compiler, schema runtime and editor code into coherent modules; consolidate
duplicated logic; remove obsolete code; resolve Rust warnings without blanket
suppression; and reconcile documentation. Preserve observable behavior and protected
owner files. Batch regressions after the cleanup section is complete. Keep new S11
code organized during implementation rather than relying on this later pass.


Current priority: implement program-owned module let/const and exports with complete
checker/editor integration before resuming S11; see [module state](plans/module-state-plan.md).
Revisit thread-pool API design after async implementation, as requested by the owner.
Do not treat earlier proposed pool APIs or mandatory global pools as approved.

## Compiler traversal depth follow-up

Active work and remaining acceptance are tracked in the
[compiler nesting architecture plan](plans/compiler-nesting-architecture.md).


The current 128-level source/combined-analysis limits are implementation resource
guards, not a language requirement for arrays or functions. S11 regression repairs
reduce retained stack frames; they do not provide arbitrary-depth compilation.
Track replacement of recursive compiler traversals with explicit work stacks,
including parsing, checking, emission, type traversal and recursive tree disposal.
Preserve ownership, evaluation order, diagnostics and bounded handling of hostile
input. Acceptance must cover deep source nesting, deep finite recursive values,
long call-dependency chains and ordinary runtime recursion separately. Increasing
the process stack or deleting guards alone is not acceptance. Owner priority: make this the first implementation task in the S11 handoff,
ahead of S12 and further roadmap features. Do not defer it to S21 hardening.
The handoff must explain the current limitation, preserved failure evidence,
and the explicit-work-stack approach. Runtime call-stack limits and JSON validation depth budgets
are separate contracts and must not be silently changed.
