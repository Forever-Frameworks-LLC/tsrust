# Standard-library coverage and ecosystem research

Research date: 2026-10-03. Planning evidence, not implemented coverage. Parent: [gate roadmap](../../plans/standard-library-roadmap.md). This inventory covers every section of the repository's pinned Node API index, plus representative application ecosystems. Package names are capability/design references, not promises that arbitrary npm JavaScript can execute as native Tsrust.

## Evidence and selection limits

The [pinned Node API index](https://nodejs.org/download/release/v24.11.1/docs/api/) supplies the exhaustive baseline of API families. The [current Node index](https://nodejs.org/api/) also lists newer families such as FFI, benchmark runner, iterable streams and virtual filesystem; track these separately rather than silently changing parity baseline.

Primary project docs were reviewed for Express, Fastify, Nest, Next, Undici, Zod, Ajv, Prisma, Drizzle, BullMQ, ioredis, Socket.IO, Pino, Axum, SQLx, rusqlite, serde_json, simd-json, Moka, reqwest, tracing, clap, Rustls, Rayon, redis and regex. Sources appear beside the relevant coverage below. Other named alternatives are discovery candidates requiring API review before selection.

This is a representative capability survey, not a measured ranking of the most popular packages or application workloads. Do not claim an exhaustive npm/crates.io ranking. At each backend/API selection gate retain dated npm direct/dependent/download and crates.io recent-download/dependent evidence, maintenance activity, license, target/toolchain support and measured fit. Downloads include transitive dependencies and CI, so rank within capability families and distinguish direct application choices from build infrastructure. Framework docs establish supported patterns, not global usage percentages. Refresh selection evidence when implementing rather than pinning unverified latest releases now.

## Node API family disposition

Every family remains pending except existing capabilities explicitly evidenced elsewhere. Rows group related sections but preserve their identities. 'Native equivalent' means useful behavior with documented differences, not exact API compatibility. Every family's implementation gate must expand into a per-export/per-overload ledger with defaults, errors, platform conditions, lifecycle, ownership, tests and IntelliSense.

| Node index families | Planned treatment | Gate/dependency |
|---|---|---|
| Buffer; String decoder | Owned bytes, bounded slices, encodings, incremental UTF-8 decoding; clarify copies versus views | S07 plus H01 |
| File system; Path | Sync and async files, directories, metadata, watch, links, streams and platform paths | S05/S09; async S14; H02 |
| Environment Variables; OS; Process | Env/config, system info, argv/exit, signals/std streams and lifecycle | S09 plus H03 |
| Child processes; Cluster | Argument-safe spawn, stdio streaming, exit/signal/cancellation; multi-process service supervision as native equivalent | H03; S14 for async |
| Console | Preserve existing implementation and tracked difference ledger | existing console track |
| Errors | Typed native errors and Result; source stacks; exception model separately | S06/S07; H04 |
| Events | Typed event/subscription lifecycles, synchronous ordering, async event streams and listener cleanup | H04; S14 |
| Timers | Monotonic deadlines, intervals, sleep, cancellation and runtime lifetime | S09/S14 |
| Stream; Web Streams API | Bounded readable/writable/transform streams, pipeline/cleanup, async iteration and adapters | H01; S14 |
| HTTP; HTTP/2; HTTPS | Client/server, headers/cookies, pooling, bodies, streaming, abort, graceful shutdown; HTTP/2 separate acceptance | S15/S16; H05 |
| Net; UDP/datagram; DNS | TCP/Unix sockets, UDP, lookup/resolve and cancellation/platform behavior | H05; S07/S14 |
| TLS/SSL | Client/server trust and identities, secure defaults, handshake lifecycle and errors | S16 |
| URL; Query strings | URL parsing/formatting, URLSearchParams/form encoding; legacy querystring separately | H06 |
| Crypto; Web Crypto API | Entropy, hashes/HMAC, authenticated encryption, signing/verification and key handling; per-algorithm gates | H07; S07 |
| SQLite | Native synchronous database and explicit compatibility matrix | S17 |
| Worker threads | Native threads/task pools, transfer, channels and safe explicit sharing; no JS isolate assumption | S12/S14 |
| Asynchronous context tracking; Async hooks | Request/task context propagation; native instrumentation equivalents; do not reproduce JS hook execution model by assumption | H08; S14 |
| Diagnostics Channel; Performance hooks; Trace events; Report | Structured instrumentation, monotonic timings, tracing, metrics and diagnostic reports | H08 |
| Assertion testing; Test runner | Assertions, test discovery, fixtures, deterministic timers and concurrency tests; CLI integration | H09 |
| Readline; TTY; REPL | Terminal input, line editing, color/detection and interactive tooling; REPL depends on execution strategy | H10 |
| Utilities | Inspection, formatting, comparisons and type predicates; promisify/callbackify only if callback bridges are supported | H04/H10 |
| Zlib | Streaming compression/decompression, format compatibility and decompressed-size limits | H11; H01 |
| Internationalization | Locale/number/date/collation APIs; explicit data packaging, Unicode and timezone policy | H12 |
| Modules: ECMAScript modules; Packages; node:module API | Static modules/std resolution first; package manifests and distribution later; reflective loaders separate | S01–S05; H13 |
| Modules: CommonJS modules; TypeScript | CommonJS dynamic loader and execution of arbitrary JS/TS not supplied by static native modules | deferred interoperability decision H13 |
| Globals | Audit each pinned global: fetch/URL/AbortController/EventTarget/TextEncoder/Decoder, structuredClone, web objects and scheduling; language globals remain separate compiler track | H01/H04/H06; S14/S15 |
| Permissions | Explicit capability/permission model design and host resource boundaries; avoid implying full sandbox from wrapper checks | H14 |
| Command-line options; Single executable applications; Usage/example; About docs | Native CLI/build/package equivalents with clear supported targets; no Node flags promise | S05/S21; H13 |
| Debugger; Inspector | Native debugger/source maps and editor integration; JS inspector protocol not automatically applicable | H15 |
| V8; VM | Engine-specific APIs have no direct native parity; isolated execution requires separate sandbox/interpreter design | explicitly deferred H15 |
| C++ addons; C/C++ addons with Node-API; C++ embedder API | Native FFI/addon boundary is separate; no promise to load Node addons | deferred H16 |
| WASI | Separate target/runtime capability roadmap and cross-platform resource model | deferred H16 |
| Domain; Punycode; Deprecated APIs | Track individually; prefer modern error/context and URL equivalents; legacy compatibility only with demonstrated need | deferred legacy ledger |
| Newer Node FFI; Virtual File System; Iterable Streams API; Benchmark runner | Supplemental discovery beyond pinned baseline; native FFI/VFS/streams/benchmark equivalents scoped independently | H16/H02/H01/H09 |

## Application-pattern acceptance suite

Use these as coverage targets, not claims about measured market share. Each needs an executable multi-file application and failure/load tests, not only API unit tests.

1. CLI/config/data transformation: argv/env, files/globs, JSON/YAML/TOML adapters, validation, terminal output, child processes and graceful signals.
2. REST/JSON service: routing, request/response schema, errors, CORS/security headers, authentication hooks, rate limits, database pool, tracing and graceful shutdown.
3. Streaming service: uploads/multipart, downloads/static files, proxy/client pooling, bounded pipelines and compression; early disconnect cleanup.
4. Realtime service: WebSocket and SSE, heartbeats, bounded fan-out and reconnect; Socket.IO is a distinct protocol adapter.
5. Worker/jobs: queue producers/consumers, concurrency bounds, cancellation, retries/backoff, delayed/recurring jobs, idempotency and failure recovery.
6. Data service: migrations, prepared queries, transactions, pagination, row streaming, cache invalidation and connection failure.
7. CPU-heavy processing: safe parallel mapping/batching with thread pools alongside responsive async I/O.
8. Testing/operations: unit/integration tests, fixture lifecycle, deterministic clocks, metrics/tracing/logging and deployment packaging.
9. Full-stack/SSR/serverless: survey Next-style server routes/rendering and deployment boundaries, but React/JSX/browser execution, bundling and platform SDKs remain separate product decisions.

## Ecosystem capability matrix

Rows specify behavior worth covering; backend selection stays subject to toolchain and focused prototype acceptance. Shipped modules may have optional adapters so this breadth does not inflate every binary.

| Capability | Node references/candidates | Rust references/candidates | Scope/acceptance |
|---|---|---|---|
| Routing/middleware | [Express](https://expressjs.com/), [Fastify](https://www.npmjs.com/package/fastify), [Nest](https://docs.nestjs.com/); Hono discovery | [Axum](https://docs.rs/axum/latest/axum/), Tower/tower-http discovery | S15 + H17: route params, middleware order, typed state, hooks, errors, CORS, limits, static files, compression, uploads and OpenAPI; decorators/DI framework not required for core |
| HTTP client | [Undici](https://www.npmjs.com/package/undici); Axios/Got discovery | [reqwest](https://docs.rs/reqwest/latest/reqwest/), Hyper discovery | S15: reusable clients/pools, redirects, proxies, forms/multipart, streaming, TLS, abort and body limits |
| Validation | [Zod](https://zod.dev/api), [Ajv](https://ajv.js.org/); Joi/Valibot discovery | Typed generated validators; jsonschema discovery | S11 + H18: inferred schemas plus JSON Schema import/export subset; explicit coercion/default/union/error semantics |
| JSON | Native JSON and schema-based serialization patterns in Fastify | [serde_json](https://docs.rs/serde_json/latest/serde_json/), [simd-json](https://docs.rs/simd-json/latest/simd_json/) | S10: typed/fused path, dynamic path, raw fields and retained allocation/throughput measurements; no unchecked fastest claim |
| Database access/ORM | [Prisma](https://www.prisma.io/docs/orm), [Drizzle](https://orm.drizzle.team/docs/overview); pg/mysql2/better-sqlite3 discovery | [SQLx](https://docs.rs/sqlx/latest/sqlx/), [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/); Diesel/SeaORM discovery | S17/S18 + H19: transactions/pools/migrations/typed rows first; query builder/schema generation/ORM separate acceptance |
| Redis/cache | [ioredis](https://github.com/redis/ioredis); node-redis/lru-cache discovery | [redis](https://docs.rs/redis/latest/redis/), [Moka](https://docs.rs/moka/latest/moka/); lru discovery | S19 + H20: local versus distributed, bounds/TTL/stampede, pooling, scripts/pubsub, reconnect/cluster semantics independently |
| Durable jobs/scheduling | [BullMQ](https://docs.bullmq.io/); cron/p-queue discovery | BullMQ Rust binding discovery; broker/cron adapters individually researched | S20 + H21: persistence/ack/retry/delay/recurrence/recovery tests; never infer BullMQ compatibility from a Redis client |
| Realtime | [Socket.IO](https://socket.io/docs/v4/); ws discovery | tokio-tungstenite/Axum WebSocket discovery | H22: WebSocket/SSE first; Socket.IO handshake/rooms/acks/reconnect as distinct later protocol |
| Logging/observability | [Pino](https://github.com/pinojs/pino); Winston/OpenTelemetry discovery | [tracing](https://docs.rs/tracing/latest/tracing/); OpenTelemetry discovery | H08: structured logs, redaction, request/task context, spans/export, metrics and bounded transport |
| CLI/config | commander/yargs/dotenv discovery | [clap](https://docs.rs/clap/latest/clap/); config/TOML/YAML discovery | H10: argv parsing, typed env/config and useful terminal diagnostics; avoid whole-package cloning |
| CPU parallelism | worker_threads; Piscina discovery | [Rayon](https://docs.rs/rayon/latest/rayon/), Rust threads | S12 + H23: bounded pools and safe parallel iterators, cancellation limitations and scheduling isolation |
| TLS/security | crypto/WebCrypto; jose/bcrypt/argon2/helmet discovery | [Rustls](https://docs.rs/rustls/latest/rustls/); RustCrypto/argon2 discovery | S16/H07/H17: reviewed algorithms, password hashing, token verification and security middleware; auth providers later |
| Regex/text/time/IDs | RegExp; lodash/date-fns/dayjs/uuid discovery | [regex](https://docs.rs/regex/latest/regex/); time/chrono/uuid discovery | H12/H24: explicit regex differences, Unicode/timezone and identifier semantics; existing array methods before utility duplication |
| Full-stack frameworks | [Next](https://nextjs.org/docs), Nest | SSR/template engines and framework candidates unselected | Separate framework product track; cover server primitives without promising JSX/React compatibility |

## Supplemental gates

All pending; no implementation numbering is reserved. Dependencies refer to parent S-series. Split large families into separately accepted subgates before coding.

| Gate | Deliverable | Dependencies and required evidence |
|---|---|---|
| H01 | Streaming/encoding/global web adapters | S07/S14; bounded pipelines, split encoding boundaries, cleanup on abort and declared Web API subset |
| H02 | Extended filesystem/VFS/watch | S09/S14; platform fixtures, watch behavior, traversal/glob limits and descriptor cleanup |
| H03 | OS/process/subprocess | S07/S09/S14; stdio backpressure, exit/signal tests, process reaping and portable argument handling |
| H04 | Events/errors/utilities | S06/S14; ordering/lifetime/error-path tests and callback bridge policy |
| H05 | Transport/DNS/HTTP2 | S14–S16; loopback protocol tests and cancellation; HTTP2 stream limits |
| H06 | URL/form/query | S07; parser conformance, percent encoding and Unicode/domain policy |
| H07 | Cryptography/identity/auth primitives | S07; algorithm vectors, key lifecycle, secure entropy and independent dependency review |
| H08 | Observability/task context | S14; async context propagation, redaction, bounded logging and instrumentation overhead |
| H09 | Tests/assertions/benchmark CLI | S03/S14; native fixture discovery, failure source mapping, deterministic clocks and reproducible measurements |
| H10 | CLI/config/terminal tooling | S09; parsing/default errors, precedence and TTY/non-TTY tests |
| H11 | Compression/archive adapters | H01; format vectors, corruption and expansion limits |
| H12 | Time/locale/Unicode | S09; calendar/timezone fixtures, data packaging and platform consistency |
| H13 | Package/build/distribution | S05/S21; manifest/lock/cache/version contracts, offline build and platform installers |
| H14 | Host capability model | S07; explicit trust boundary and negative resource-access tests |
| H15 | Native debug/source maps | S03; source-level stepping/stacks; V8/VM replacement unresolved separately |
| H16 | FFI/WASI/embedding | S07/H13; ABI/ownership/target safety design before implementation |
| H17 | HTTP application layer | S11/S15/S16; middleware/auth hooks/multipart/rate limits/static serving and OpenAPI witnesses |
| H18 | Extended schema standards | S11; JSON Schema dialect/conformance and explicit unsupported keyword errors |
| H19 | Database tooling/query builder | S18; migrations/offline query metadata, rollback fixtures and schema drift checks |
| H20 | Redis/external cache | S14/S19; actual service integration, reconnect, expiry, transactions and cluster scope |
| H21 | Durable jobs | S20/H20 or accepted SQL adapter; restart/recovery, retry/idempotency and stated delivery contract |
| H22 | Realtime protocols | S15; disconnect/backpressure/WebSocket/SSE tests; Socket.IO separate subgate |
| H23 | CPU pool/parallel collections | S08/S12; safe capture/alias rejection and parallel workload verification |
| H24 | Text/regex/utility adapters | S08; semantic matrix and bounded regex behavior; no JS regex parity assumption |

## Research closure and completeness checks

Before a family enters implementation, enumerate its exports from the pinned primary documentation and classify each as exact behavior, native equivalent, later scope or intentionally unavailable with rationale. Record dependencies and acceptance fixtures for every included overload. Nothing disappears because it is obscure or inconvenient. Compare current Node additions separately. Keep deprecated and engine-specific entries visible instead of pretending they fit native compilation.

Before broad coverage claims, expand these family tables into machine-readable inventory and test registry; verify every included item has implementation, IntelliSense, executable witnesses and evidence. Each unimplemented item stays visible. For ecosystems retain selected package versions and reviewed APIs, dated popularity signals, reasons for selecting/rejecting backends and realistic application witnesses. Full ecosystem compatibility and running arbitrary npm packages require separate language/runtime/interop decisions; useful capability coverage is the present goal.

## Accepted lexical path increment

The bounded primitive `std:path` family adds dirname, extname, isAbsolute and normalize alongside basename. Compiler/native execution, imported editor help, executable examples, matched benchmark registration and scoped native/Node-after-tsc measurements are accepted on the library branch. See [family acceptance](../../milestones/stdlib-lexical-path.md) and integration handoff.

Public integration is verified on macOS; explicit-platform helpers cover POSIX/Windows reference behavior. Windows native integration and the advancing combined checkout still require their own checks. This primitive-only increment does not accept filesystem resources, the wider host family, or the rest of the standard library.

## Accepted bounded OS identity slice

`std:os.platform()` and `std:os.arch()` are implemented through existing zero-argument string bindings. [The contract](../../milestones/os-identity-contract.md) freezes exact supported target metadata mappings and explicit unsupported-target errors. Native acceptance is macOS ARM64; 16 cross-target metadata cases do not establish native acceptance on those targets. Complete compiler/editor acceptance and untimed matched workload support are required by [the OS identity plan](../../plans/parallel-os-identity-plan.md). Identity timing remains deferred because the wrappers can constant-fold. Other OS exports remain unimplemented.

## Accepted variadic composition and byte order

`std:path.join(...paths: string[]): string` adds lexical composition through a reusable homogeneous borrowed string rest bridge. Aliases, stored values, array spread, concrete rest Function adaptation, evaluation order and failure cleanup are covered. `std:os.endianness(): string` adds compiled BE/LE metadata with explicit unsupported-selector errors. [The contract](../../milestones/path-join-contract.md) and handoff preserve compiler/editor/native acceptance, independent POSIX/Windows reference coverage, scoped composition measurements and constant identity timing limits. Actual native acceptance remains macOS ARM64; main integration and native Windows execution remain separate. No wider path, OS or resource family is accepted.

## Relative path increment

`std:path.relative(from: string, to: string): string` adds borrowed-pair lexical resolution with owned results and checked cwd failures. [The contract](../../milestones/path-relative-contract.md) distinguishes host execution from deterministic Windows helper/reference coverage. Combined compiler/editor regressions, native examples and matched measurements are accepted. Source, immutable SQLite evidence and five rebuilt consumers are integrated into main; see the handoff. No further path or OS family is included.

## S07 resources and bytes

S07 is accepted on macOS: typed owned bytes, checked mutation/copy/UTF-8 operations,
real borrowed views, recursive view storage and callbacks, and owned read-only file
open/read/close with deterministic cleanup. [Byte/view behavior](https://forever-frameworks-llc.github.io/tsrust/reference/library/bytes/)
and [file behavior](https://forever-frameworks-llc.github.io/tsrust/reference/library/fs/) describe supported boundaries. This is not
general Uint8Array, Node filesystem or buffered-stream compatibility; broader
S09 host operations remain pending.

The [family record](../../milestones/s07-resources-bytes.md) binds complete regressions,
17 native/Node workloads, allocation profiles, two direct Rust reference workloads,
performance review and six refreshed executable copies including the PATH CLI.
Coverage tracks these APIs individually and keeps later S08–S21 work pending.

## S11 accepted scope (2026-10-13)

S11 is accepted on macOS for inferred reusable validation schemas, owned/JSON input, fused text decoding, bounded typed issues, pipelines/collections/recursion and scoped Draft 2020-12 export. Complete compiler/LSP regressions, all 31 matched measurements and consumer refresh are recorded in the acceptance handoff. H18 extended dialect/import conformance remains pending. Native is slower in 28 of the 31 handwritten-reference workloads; preserve that measured limitation. Compiler nesting architecture is the first next-agent task before S12.
