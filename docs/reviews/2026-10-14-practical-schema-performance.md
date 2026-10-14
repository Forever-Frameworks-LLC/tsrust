# Practical schema performance review — 2026-10-14

## Decision

Return to product features. Defer quadratic emission for extremely deep expressions
and quadratic nested array-literal spread checking. Reopen them when actual generated
source shows material compile-time or memory impact, or an explicit compiler-service
requirement makes adversarial input relevant. Large applications do not necessarily
contain individual expressions tens of thousands of levels deep.

This bounded review does not establish a blocker requiring a broad schema rewrite.
It does identify collection/error-heavy loops as the first runtime profiling target
if a real application spends substantial time there. Keep checks, ownership and
structured errors intact. No compiler or runtime behavior changed in this review.

## Current measurements

Source commit `00f996c8`; Apple M3, arm64 macOS; Rust 1.96.1, Node v24.11.1,
TypeScript 5.9.3. Release compiler build passed. The existing execution-only protocol
ran three fresh processes, eight warmups and 31 samples per process and engine.
All 60 correctness cells passed (six workloads × two engines × five cases).
All 1,116 measured durations passed the harness checksum/observation checks.
Compilation and startup time were not measured.

Each timing sample executes 2,000 workload iterations. Values below are pooled
median milliseconds per batch, not request latency. Each iteration performs two
operations, except pipeline (three) and collections (five). These small fixed inputs
include frequent failures; collections includes two collision failures per iteration.
This is a targeted microbenchmark review, not an application throughput guarantee.

| Workload | Native ms | Node ms | Native / Node | Native process-median range ms |
| --- | ---: | ---: | ---: | ---: |
| export | 1.854 | 0.645 | 2.88× | 1.847–1.858 |
| recursive | 3.399 | 1.447 | 2.35× | 3.372–3.429 |
| json_array | 0.705 | 0.405 | 1.74× | 0.702–0.726 |
| json_object | 0.919 | 0.716 | 1.28× | 0.896–0.959 |
| pipeline | 0.786 | 0.353 | 2.23× | 0.765–0.799 |
| collections | 4.484 | 0.864 | 5.19× | 4.482–4.490 |

Native costs span approximately 0.35–2.24 microseconds per workload iteration.
The ratios are investigation signals, not equal-work language rankings. In particular:

- **json_array:** Node checks lower bounds on an asserted number[]; native also validates input shape, exact integer types, and resource budgets. Same fixed fixtures, unequal general validation.
- **json_object:** Node reference is specialized to the fixed object corpus and mutates parsed input; native produces owned typed output with full checks.
- **collections:** Node collision failures return only a code; native builds structured issues. Both construct collection outputs.
- **recursive:** Node returns original parsed tree after validation; native constructs an owned typed tree. Both inspect the tested failure path.
- **pipeline:** Same fixed transformations, issue observation, and retry outcome; native includes general schema plans and reversible ownership machinery.
- **export:** Node constructs literal objects for two fixed plans; native supports general plan traversal. Export is commonly reusable outside request loops.

These are handwritten Node references, not Zod measurements. The current run is a
new baseline after source/API migration; it does not prove a regression from S11.
Historical `tsrust-s11-measurement-review.json` contains 27 slower native medians
out of 31 rows, despite its prose saying 28. Preserve that original record; this
review corrects the interpretation without rewriting evidence.

## Allocation evidence and practical choices

The previously accepted `tsrust-allocation-syntax-probes-repaired.json` covers seven
fixtures, sizes 0, 1 and 1,000, three samples each, and native/direct-Rust/Node results.
All 189 executions passed. Native/direct-Rust final live allocated bytes are zero
in those probes; this does not prove every application is leak-free.

For 1,000 integers, fused `safeParseJson` requests 16,192 bytes versus 16,352 for
handwritten Rust; both retain 8,192 bytes for the output. The separate owned
`safeParse(JSON.parse(...))` path requests 97,952 bytes, about six times the fused
path, because it constructs an intermediate dynamic tree. Requested bytes include
reallocation requests; they are not peak memory or whole-application memory.

Use the fused API when starting from raw JSON. Keep the owned API for callers that
already have a dynamic value. Reuse schema exports outside hot loops when appropriate.
The reusable union probe does speculative branch work absent from its direct-Rust
array reference; its ratio is not an equivalent-work comparison.

## Next work and acceptance boundary

Proceed to S12 public owned spawn and consuming join, including capture/result
transfer checks, explicit join/detach enforcement, failure cleanup, editor help,
examples and benchmark readiness. Existing internal thread helpers are groundwork,
not public feature acceptance. Complete that coherent slice before broadening scope.

Run consolidated compiler/LSP regressions at the next meaningful feature-family
milestone. Recent focused acceptance remains valid; this review did not run a fresh
full regression or new compiler scaling suite. The extreme-nesting limits remain
known and deferred, not fixed. See the [current handoff](../handoffs/compiler-to-roadmap-continuation.md).

## Durable evidence

Original bytes and derived timing samples are retained in `benchmarks/records.sqlite`:

- `runtime/results/2026-10-14T18-08-10-796Z.json`: complete current run, source hashes,
  toolchain/machine identities, checksums and individual samples.
- `tsrust-practical-review-2026-10-14.json`: acceptance and measurement summary.
- `tsrust-practical-review-audit.json`: historical count correction, reference
  comparability audit and retained allocation observations.
- `tsrust-practical-review-stats.json`: pooled and per-process medians for both engines.
- `tsrust-practical-review-build.log`, `tsrust-practical-review-runtime.log` and
  `tsrust-practical-review-format.log`: current build, run and whole-project formatting.

Closed loose evidence is removed only after exact-byte restoration and SQLite
integrity checks. Generated workload binaries are disposable and rebuilt from source.
