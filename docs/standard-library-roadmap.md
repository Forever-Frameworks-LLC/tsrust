# Modules, native standard library and async roadmap

Status: S01–S03 module loading/checking/native CLI accepted with focused evidence; S04 project editing is accepted at gate 185; S05 primitive native bindings/build is accepted on macOS with focused evidence; S06 type foundations are accepted at gate 186; S07 is accepted on macOS after gates 187–190 native bytes, views and file resources; S08 collections is accepted on macOS; S09 synchronous host foundations are accepted on macOS; S10–S21 remain pending. Project-wide multiple-error diagnostics is accepted on macOS; see the [diagnostics acceptance](plans/project-diagnostics-plan.md). User-approved direction: bundled standard library, Rust package backends, Tokio-backed explicit async/await and native threads. No custom goroutine scheduler. This file is the progress ledger and compiler/editor handoff contract. See [module loader acceptance](milestones/modules-01-project-loader.md) and [module checking acceptance](milestones/modules-02-project-linking.md).

[Researched Node and ecosystem coverage inventory](standard-library-coverage.md) maps every pinned Node API family, representative Node/Rust packages, application witnesses and supplemental H-series gates. Its per-export inventory and popularity measurements remain explicit follow-up requirements before broad parity or ranking claims.

## Product contract

Ship supported capabilities with the Tsrust toolchain. Users import std: modules without maintaining Cargo dependencies. Keep stable Tsrust APIs independent of backend crates. Link only selected features; a console-only program must not acquire database, HTTP or async runtime dependencies. Rust crates are implementation dependencies, not arbitrary source-level Rust interoperability.

Preserve JavaScript-compatible globals where their behavior is already supported. Native modules have explicit ownership, errors, limits and lifecycle contracts. No claim of complete Node or Zod compatibility until a published coverage matrix is verified. Existing clone remains an explicit independent value copy; cloning a resource handle is a separate documented operation, never an accidental database or socket duplication.

Proposed namespaces: std:path, fs, env, process, bytes, time, collections, json, schema, task, thread, channel, sync, http, tls, sqlite, sql, cache and queue. Spellings and signatures remain design proposals until their gate closes.

The owner-approved [globals/imports policy and mandatory audit](plans/globals-and-imports-policy.md)
requires exhaustive per-symbol reconciliation before adding new conventions.
The JSON namespace will expose throwing `JSON.parse` and Result-returning
`JSON.tryParse`, preserving native safety and existing imports. Signatures and
IntelliSense must expose semantic differences; runtime dependencies follow actual
usage. Implementation follows the prioritized typed-data imports and aliases and
is required before S10 acceptance. Neither the audit nor the JSON globals are
implemented merely by recording this decision.

## Architecture and compiler handoffs

Keep in-process frontend analysis independent of network, databases, Cargo and native execution. Add project analysis alongside existing check(source), preserving the single-source API. Explicit source identity must accompany project symbols, diagnostics, related spans and edits; do not concatenate files and guess offsets.

A compiler-owned typed native binding registry defines argument/result types, generics, borrowing/mutation/transfer, callback retention and invocation, async behavior, thread-transfer constraints, errors and required runtime features. Library modules consume this registry. The editor must consume public metadata, not independently invent these contracts. Every new method/property still receives complete builtins.json documentation before gate acceptance.

The native build driver resolves an exact pinned dependency/feature graph and cached generated Cargo project. Dependencies are acquired in an explicit toolchain/setup operation; frontend checking and ordinary offline builds never silently fetch packages. Retain lockfiles, licenses, target support and dependency provenance in developer evidence. Only std imports select native features. Preserve direct rustc compilation for programs requiring no external runtime packages where practical.

Shared compiler integration points: parser/type representation, symbol resolution, ownership/loans, checked IR and Rust emission. Modules and host handles cannot land as editor-only conveniences. Keep these changes coordinated with ongoing numbered language gates; do not allocate their gate numbers or alter their running checkpoint.

## Sequenced gates and progress ledger

S01–S09 are accepted within their documented platform and feature boundaries; S10–S21 are pending. A gate closes only with its listed focused evidence plus the common acceptance requirements below. Implementation acceptance and performance/checkpoint acceptance are separate fields.

| Gate | Depends on | Concrete deliverable and exit evidence | State |
|---|---|---|---|
| S01: module semantics | none | Commit import/export, entry main, type identity, resolution, cycles and diagnostic API decisions with positive/negative fixtures; no unresolved first-release semantics | accepted: module loader evidence |
| S02: project loader | S01 | Explicit relative .tsr imports, canonical identities, deterministic graph, missing/duplicate export and cycle diagnostics; diamond loads once; no std/package implicit file search | accepted: module loader evidence |
| S03: module checking/emission | S02 | Cross-file functions/types/ownership, private symbols, name collisions and source order; only selected entry main auto-runs; imported main ordinary; native two-file and diamond programs pass | accepted: [library/CLI/native witnesses](milestones/modules-02-project-linking.md); scaling measurements pending checkpoint |
| S04: project editor | S03 | Unsaved imported buffers, dependent invalidation, cross-file hover/definitions/diagnostics/versioned fixes, rename/removal and stale result tests; single-file behavior preserved | accepted: gate 185 ([record](milestones/185-project-editor.md), [map](b9-project-editor-acceptance-map.md), [guide](project-editor.md)); closed-file fixes are guarded by disk text; help survives unfinished text and imports; import names complete from the module exports; packaged-extension run of the gate 185 additions is pending |
| S05: std bindings/build | S03 | Registry-backed std import and one small path operation; exact feature lock/cache, offline warm build, friendly backend errors, unused feature exclusion and installed CLI/editor witness | accepted on macOS: [requirement audit](milestones/stdlib-01-bindings-build.md); primitive std:path, native function values within current language boundaries, cache/offline/copied CLI/editor witnesses; registered measurements pending checkpoint |
| S06: generics/results | S05 | Generic functions/containers, bounded specialization, tagged Result success/error, optional types integration, exhaustiveness and typed error propagation with ownership/drop tests | accepted: [generic and callable foundations](milestones/stdlib-02-types-results.md) and [tagged Result, exhaustive statement match and explicit propagation](milestones/186-result-match.md); shorthand propagation is separate |
| S07: resources/bytes | S06 | Owned opaque handles, explicit close plus deterministic drop, use-after-close rules, byte buffers/views, bounds and native errors; cleanup on early return and failure | accepted on macOS: gates 187–190, complete regressions, 17 matched measurements, direct Rust review and refreshed consumers; [evidence](milestones/s07-resources-bytes.md) |
| S08: collections | S06 | Map, Set, ordered map/set, deque and heap; key equality/hash/order, borrowing iterators, invalidation and explicit clone; boundary and native reference tests | accepted on macOS: [family regression, measurement, optimization and consumer evidence](milestones/s08-collections.md) |
| Dev-loop performance: owner priority | S03, S05, new/dev CLI | Filesystem events, retained compiler state, separate native runtime compilation, bounded caches; clean-build equivalence, lifecycle tests, repeated scaling measurements and installed acceptance | accepted on macOS, Linux watcher-module checks accepted; installed lifecycle, full regressions and matched dev-loop measurements retained; [acceptance plan](plans/dev-loop-performance-plan.md) |
| S09: synchronous host | S07 | Path/fs/env/time foundations, file read/write and buffered streams, size limits, monotonic clock; filesystem sandbox fixtures and cleanup | accepted on macOS: 3,207 compiler and 532 LSP tests pass; nine matched measurements and allocation profiles, final installed checks and editor restart pass; see [host acceptance](milestones/s09-host-measurements.md) |
| S10: JSON | S07, S08 | Typed decoding/encoding and explicit JsonValue, correct number/Unicode/property rules, malformed/depth/size errors, JSONC/JSONL, bounded file streaming and allocation baseline; see JSON contract | in progress: bounded dynamic JSON/JSONC, owned JSONL readers and offline package export pass focused checks; typed JSON/JSONC imports and aliases pass focused compiler/native/editor checks; installed CLI/editor refreshed from fd213e26; typed runtime decoding, borrowed adapters, public writers and family benchmark acceptance pending; see [JSON plan](plans/json-plan.md) |
| S11: schemas | S06, S10 | Typed reusable schema API, structured issue paths, safeParse, object/union/optional/refinement/transformation coverage and fused JSON validation; see schema contract | pending |
| S12: native threads | S07 | Spawn/join with explicit transferred captures, Send-like restrictions, panic/failure conversion, bounded channels; multicore witness and compile-time invalid sharing tests | pending |
| S13: async compiler | S06, S07 | async functions/await/task result types, async main selection, suspend-safe captures/loans/drop, correct control flow and diagnostics; generated native futures execute | pending |
| S14: Tokio tasks | S12, S13 | Runtime bootstrap, spawn/task groups, timers, bounded async channels, cancellation/timeouts and shutdown; blocking work separated; no leaked tasks/resources | pending |
| S15: HTTP | S10, S11, S14 | Client/server request/response, typed validated bodies, limits, streaming/backpressure, disconnect and graceful shutdown; loopback integration and cancellation tests | pending |
| S16: TLS | S15 | HTTPS client/server, certificate/key configuration, hostname verification and trust roots, handshake errors; local certificate positive/negative fixtures | pending |
| S17: SQLite | S07, S10 | rusqlite-backed synchronous connection, prepared statements, typed rows/parameters, transactions and rollback/drop; SQLite integration without async requirement | pending |
| S18: async SQL | S11, S14, S17 | SQLx-backed SQLite/Postgres/MySQL adapters incrementally; bounded pools, typed binding/rows, transactions, timeout/cancellation and migrations; each driver accepted separately | pending |
| S19: caches | S08, S09 | Owned bounded LRU/TTL cache, eviction/expiration contracts and fake-clock tests; concurrent cache/loading and stampede policy follow S14 | pending |
| S20: queues | S08, S14 | Bounded in-memory work queue, producer/consumer ownership, backpressure, close/drain, cancellation and worker shutdown; durable/distributed adapters are separate gates | pending |
| S21: integrated release | S04, S15–S20 | Validated HTTP service using database, cache and queue; graceful shutdown, overload, leak checks, complete IntelliSense and reproducible toolchain distribution | pending |

S17 can proceed before async once S07/S10 land. S08/S09 and thread research can proceed independently after shared prerequisites. S01–S05 are the immediate concurrent track alongside built-in methods. Compiler integration is coordinated work in shared files, not safe unlimited concurrent edits.

## Module decisions for S01

First scope: explicit named imports/exports and explicit relative .tsr paths; std: reserved for shipped modules. No CommonJS, dynamic imports, bare package discovery, glob imports or implicit extension/index search. Resolve relative to importer, normalize filesystem identities, specify symlink/case behavior and report ambiguous identities. Exported types retain defining-module identity.

First scope has no executable top-level initialization and rejects import cycles with an import-chain diagnostic. Constants and re-exports require separate explicit semantics before inclusion. Each file may define its own main; only the chosen entry's zero-argument procedure runs automatically. No imported main auto-run. Later initialization/cycles/package gates must amend this contract explicitly.

Project API returns source-indexed checked symbols/contracts and diagnostics. Source provider supports disk and unsaved buffers without frontend I/O. CLI selects entry, LSP identifies project roots and dependency snapshots. Build caching keys include source graph, toolchain, std feature graph and lock identity.

## JSON correctness and efficiency contract

Immediate S10 priority: owner-approved typed JSON/JSONC file imports and project
path aliases, including extensionless `.tsr`/`index.tsr` resolution and shared
compiler/editor data types. These precede further JSON API and benchmark expansion.
See [typed data imports and aliases](plans/typed-data-imports-and-aliases-plan.md)
for behavior, implementation order and acceptance. Both additions remain pending.

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

Next steps: S10 JSON, then S11–S21 in dependency order. S09 is accepted on macOS; see [host acceptance](milestones/s09-host-measurements.md). The required [dev-loop performance milestone](plans/dev-loop-performance-plan.md) is accepted on macOS with Linux watcher-module evidence: filesystem events, retained compiler state, separate runtime compilation, bounded caches, installed lifecycle checks and matched measurements are complete. S08 collections and the user-requested [project diagnostics milestone](plans/project-diagnostics-plan.md) are accepted. The owner-prioritized [new/dev CLI workflows](plans/project-cli-workflows-plan.md) are accepted on macOS with installed CLI checks. Continue S10–S21 in dependency order. S07 direct Rust review and final consumer refresh are accepted. Recursive views, callback support and gate 190 resource lifecycle have focused acceptance. S07 complete regressions and 17 matched measurements are recorded in `milestones/s07-resources-bytes.md`. S04 is accepted at gate 185. S05 frontend scaling and its separately registered runtime measurements remain pending their own checkpoint.

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
