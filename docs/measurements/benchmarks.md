# Benchmark methodology

[Documentation index](../README.md) · [Proposed roadmap](../roadmap.md) · [Source section map](../plans/section-map.md)

## Workstream scope

**Status:** Initial runtime comparison suite measured at compiler gate 31; broader design/research remains open. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Groundwork; ongoing.

**Scope:** Define independent equivalent workloads, runtime metrics, and fair comparisons.

**Planning deliverable:** A reproducible workload specification and baseline plan before performance claims.

**Dependencies to coordinate:** [product-vision](../plans/product-vision.md)

## Current measurements

SQLite is the persistent analytics source of truth. The existing [BENCHMARKS.md](../../BENCHMARKS.md) is a historical snapshot; future Markdown reports are optional exports. The [compact storage tools](../../benchmarks/storage/README.md) preserve exact original records alongside derived queryable measurements for a future official documentation site with charts and tables.

The [pinned local harness](../../benchmarks/runtime/README.md) separates correctness, startup, compilation, warmed execution, semantic caveats and raw samples. Full regressions and accumulated performance comparisons run every five gates. Their purpose is to detect bugs and performance problems early, with generated native execution versus Node after TypeScript compilation as the primary signal. Investigate slower workloads and regressions in comparable measurements before more features depend on their implementation; runtime rankings alone do not establish a compiler bug. Preserve other runtimes as context and allocation metrics as normalized counters.

Each new measurable operation needs an accurate TypeScript counterpart or an explicit non-comparability explanation. The existing [frontend analysis measurements](analysis-performance.md) remain separate. HTTP, JSON, RSS, async and service tail latency remain future work. See [the development workflow](../development-workflow.md) for checkpoint acceptance and storage requirements.

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-35"></a>

## 35. Performance Philosophy

Avoid promising universal “Rust performance.”
Instead define performance goals.
Benchmark:
* HTTP requests/sec
* p50/p95/p99 latency
* idle RAM
* peak RAM
* startup time
* JSON serialization
* JSON parsing
* database throughput
* string processing
* CPU-bound workloads
* async task overhead
* shared-state contention
Compare against:
* Node.js
* Bun
* Deno
* Go
* Rust
* Java/Kotlin where relevant
Measure generated code rather than relying on assumptions.

<a id="web-10"></a>

## 10. Benchmarking Philosophy

Do not benchmark by downloading competing projects’ benchmark repositories and incorporating their code into this repository.
Instead define language-neutral workload specifications.
Then independently implement those workloads in each language.
Example specification:
Accept an HTTP POST containing a JSON array of 100 users, validate required fields, normalize names, calculate a score, serialize a JSON response, and return HTTP 200.
Implement equivalent versions in:
* this language
* Rust
* Node.js
* Bun
* Deno
* Go
* relevant TypeScript-to-native competitors where practical
If a competitor can be executed through its normal public toolchain, it may eventually be benchmarked externally, but its source implementation should remain separate from this project’s source tree.
Do not vendor competitor code.

<a id="web-11"></a>

## 11. Benchmark Categories

Measure at minimum:
HTTP
* plaintext
* JSON
* routing
* path parameters
* query parameters
* middleware
* JSON validation
* server-rendered HTML
* streaming response
* WebSocket where supported
Runtime
* startup time
* idle RSS
* peak RSS
* binary size
* executable startup latency
CPU
* string transformation
* JSON parsing
* JSON serialization
* hashing
* sorting
* collection pipelines
* regex
Memory
* allocation count
* bytes allocated
* peak heap
* reference-count operations if measurable
* clone count
* temporary allocation
Async
* task creation
* task switching
* timer workload
* concurrent HTTP requests
* database-style simulated I/O
* channels
Tail latency
Always measure:
p50
p90
p95
p99
p99.9 where practical
Average throughput alone is insufficient.
A major project thesis is predictable latency without GC.

<a id="web-12"></a>

## 12. Benchmark Against Rust Fairly

Do not expect to consistently outperform carefully written Rust.
Rust should function as an approximate performance ceiling/reference.
Measure:
this language vs idiomatic Rust
and report the gap honestly.
Potential performance target:
Ordinary backend code should remain within a practically acceptable range of idiomatic Rust while requiring significantly less ownership-specific syntax.
Do not establish an arbitrary percentage until representative prototypes exist.

<a id="web-13"></a>

## 13. Benchmark Against Node Fairly

Node is highly optimized and should not be treated as slow by definition.
Compare:
* CPU-heavy work
* high-concurrency I/O
* startup
* memory footprint
* p99 latency
* multithreaded workloads
* long-running server behavior
The proposition is not simply:
native is faster.
It is:
native compilation + deterministic memory + stronger static guarantees may offer better operational predictability while preserving familiar development ergonomics.

<a id="web-31"></a>

## 31. Benchmark Web Workload

Create a canonical web benchmark implemented independently across languages.
Example endpoints:
GET /plaintext
GET /json
GET /user/:id
POST /validate
GET /html
GET /stream
POST /validate should:
1. Parse JSON.
2. Validate a structured object.
3. Normalize strings.
4. Perform collection transformations.
5. Serialize a typed response.
GET /html should:
1. Render nested JSX-style components.
2. Escape dynamic strings.
3. Loop through typed data.
4. Produce HTML.
This benchmark can later compare:
* this language
* Rust/Axum
* Node/Hono
* Bun/Hono
* Deno/Hono
* Go
* RustScript if practical
* other relevant native languages

<a id="web-32"></a>

## 32. Never Optimize Solely for Benchmarks

Benchmark workloads must guide engineering rather than define semantics.
Reject changes that make microbenchmarks faster while making ordinary application code harder to reason about.
Primary order:
correctness
safety
predictability
developer ergonomics
performance
benchmark score

[Checkpoint 91](../../benchmarks/runtime/diagnostics/2026-10-03T18-25-41-000Z-checkpoint91-completed-proof.json) accepts 68 comparisons against compiled Node and 204 disposed native profiles. Native is faster on 34/68. SQLite contains 622 historically measured workloads among 623 registered sources; the separate peer path row remains unmeasured. Original raw records were imported, restored byte-exactly and archived; no Markdown report was required.
