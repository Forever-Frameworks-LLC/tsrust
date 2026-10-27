# Initial Rust analysis baseline

[Compiler architecture](../guides/language/compiler-and-modules.md) · [Benchmark workstream](benchmarks.md) · [Compiler API](../../crates/tsrust-compiler/README.md)

The compiler and future language server are Rust applications. This baseline measures the reusable analysis library separately from generated-program compilation. Rust implementation alone does not establish efficiency or accuracy; both need evidence as the language grows.

## Reproduce

```sh
cargo run --offline --release --example analyze_bench
```

The dependency-free harness is in `crates/tsrust-compiler/examples/analyze_bench.rs`. It generates a string-record declaration, an empty entry point, and the stated number of independent functions that read a record field and return `trim().toLowerCase()`.

Each size gets five warmups and 31 measured iterations. Each iteration calls `tsrust_compiler::check` in process. The result stays live until after timing ends. Measurements include lexing, parsing, name/type checks, and ownership analysis; they exclude source construction, process startup, result destruction, Rust emission, `rustc`, disk I/O, and editor transport. The table reports medians from one local run.

## Recorded result

September 30, 2026; local macOS arm64 environment; `rustc 1.96.1 (31fca3adb 2026-06-26)`; Cargo release profile.

| Generated functions, excluding main | Source bytes | Median analysis time |
| --- | ---: | ---: |
| 10 | 800 | 19.8 µs |
| 100 | 7,640 | 172.4 µs |
| 1,000 | 76,940 | 1,389.6 µs |

The largest measured case takes approximately 1.39 ms in this run. These are synthetic straight-line, single-file examples with no control-flow joins, closures, async, generics, or imports. There is no production latency guarantee, comparison against another compiler, or claim about native build time.

## Gate 26 measurements

The [recursive harness](../../crates/tsrust-compiler/examples/recursion_bench.rs) uses the same five warmups, 31 measured checks, and median timing method. Reproduce it separately from native execution:

```sh
cargo run --offline --release --example recursion_bench
```

September 30, 2026; the same local macOS arm64 environment and pinned Rust 1.96.1 release profile; one run after the recursive-function gate. The tree fixture exercises shared recursive reading and exclusive recursive mutation. Permuted workloads rotate owned parameters and return the first on the base path, requiring consuming access to propagate through every argument position. The scalar depth parameter is excluded from the owned-parameter count.

| Recursive workload | Owned parameters per function | Source bytes | Median analysis time |
| --- | ---: | ---: | ---: |
| Tree traversal and mutation | 1 | 560 | 109.5 µs |
| Permuted owner forwarding | 4 | 165 | 62.1 µs |
| Permuted owner forwarding | 16 | 309 | 243.8 µs |
| Permuted owner forwarding | 64 | 933 | 1,481.4 µs |

The current implementation copies the parsed program for recursive inference passes and increases finite parameter access summaries until stable, then runs fresh definitive validation. It does not cache source components or incrementally invalidate affected functions. Acyclic checking retains its existing path without speculative syntax copies. Re-running the original straight-line harness in the same session gives:

| Generated functions, excluding main | Source bytes | Median analysis time |
| --- | ---: | ---: |
| 10 | 800 | 27.3 µs |
| 100 | 7,640 | 241.8 µs |
| 1,000 | 76,940 | 1,990.4 µs |

These are local synthetic measurements with scheduler and system-load variability. They neither attribute differences from the initial historical baseline to one change nor establish production editor latency, native compilation performance, or general recursive-program throughput. Representative files with captures, callbacks, joins, and mixed dependency graphs still need measurements.

## Gate 27 measurements

The [recursive callback harness](../../crates/tsrust-compiler/examples/recursive_callback_bench.rs) uses five warmups, 31 measured checks, and the same median timing method. It measures successful frontend analysis, including stable effect inference and definitive concrete type-vector validation. Source generation, result destruction, emission, and native compilation remain outside the timed interval.

```sh
cargo run --offline --release --example recursive_callback_bench
```

September 30, 2026; local macOS arm64 environment, pinned Rust 1.96.1 release profile; one run after the recursive-callback gate. Distinct workloads exchange the first two callbacks or rotate all positions, generating 24 and 120 concrete type vectors. The repeated workload uses seven factory results with different captured integers but one environment type.

| Callback workload | Callback parameters per recursive function | Source bytes | Median analysis time |
| --- | ---: | ---: | ---: |
| Borrowed tree traversal callback | 1 | 459 | 48.0 µs |
| Distinct callback permutations | 4 | 490 | 95.6 µs |
| Distinct callback permutations | 5 | 553 | 137.5 µs |
| Repeated environment permutations | 7 | 560 | 135.8 µs |

The 128-vector guard bounds each recursive callback function rather than runtime call count. These measurements do not establish a universal complexity bound, allocation profile, editor latency target, or native compile-time guarantee. Whole-file inference still copies speculative syntax and repeats checking; incremental analysis remains separate work.

## Gate 28 measurements

The callback harness now also measures owned recursion, including template analysis, iterative concrete body checking, and canonical arrow type lookup. September 30, 2026; local macOS arm64 environment, pinned Rust 1.96.1 release profile; one run with the same five warmups, 31 measured checks, and exclusions described above.

| Owned callback workload | Callback parameters per recursive function | Source bytes | Median analysis time |
| --- | ---: | ---: | ---: |
| Retained counter through recursive moves | 1 | 455 | 55.4 µs |
| Distinct owned permutations, 24 specializations | 4 | 514 | 345.8 µs |
| Distinct owned permutations, 120 specializations | 5 | 583 | 1,557.8 µs |
| Repeated owned environment permutations | 7 | 602 | 95.9 µs |
| Fresh closed inline arrow replacement | 1 | 193 | 40.0 µs |

Owned permutations check a concrete body per cached owned vector, while ordinary callback permutations validate vectors against a shared generic body. Different captured values of one environment type reuse a specialization. Fresh closed arrows inside owned specializations reuse their source origin and capture layout, so identical environments do not spuriously produce an unbounded chain of native types.

These local measurements include whole-file speculative inference and syntax/template copies. They do not establish incremental editor latency, memory use, native build performance, or a general throughput guarantee. The historical gate 27 measurements remain a separate recorded run.

## Gate 29 measurements

The callback harness now includes eager and lazy recursive callable-result workloads. September 30, 2026; local macOS arm64 environment, pinned Rust 1.96.1 release profile; one run with five warmups and 31 measured checks. Timings include speculative origin discovery, stable effect inference, definitive checking, and metadata cleanup. Source construction, result destruction, emission, and native compilation remain excluded.

| Recursive callable result | Source bytes | Median analysis time |
| --- | ---: | ---: |
| Returned mutable counter | 330 | 33.5 µs |
| Returned closure invoking its factory | 194 | 25.6 µs |

These small workloads establish reproducible local examples, not editor latency, allocation/peak-memory bounds, general throughput, or native build performance. Discovery clones candidate syntax and can revisit unresolved functions before origins propagate. Whole-file analysis and incremental invalidation remain separate concerns. Earlier tables retain their historical measurements.

## Gate 31 measurements

The [field move harness](../../crates/tsrust-compiler/examples/field_move_bench.rs) measures a nested extraction/restoration fixture and generated independent functions that move a string field, restore it on both branch paths, transfer the restored record, and return another owned field. Reproduce it with:

```sh
cargo run --offline --release --example field_move_bench
```

September 30, 2026; local macOS arm64 environment, pinned Rust 1.96.1 release profile; one run after field move implementation. Each workload receives five warmups and 31 measured checks; medians include parsing, names/types, partial availability, branch joins, and ownership checking. Source construction, result destruction, Rust emission, native compilation, process startup, and disk I/O remain excluded.

| Partial-field workload | Functions excluding main | Source bytes | Median analysis time |
| --- | ---: | ---: | ---: |
| Nested profile extraction/restoration | 1 | 510 | 16.4 µs |
| Branch restoration | 10 | 1,843 | 66.0 µs |
| Branch restoration | 100 | 17,773 | 664.1 µs |
| Branch restoration | 1,000 | 177,973 | 5,727.5 µs |

The unchanged straight-line field-read harness measured 48.3, 381.2, and 2,162.8 µs for 10, 100, and 1,000 functions in the same session. These are local observations with load/frequency variability; historical runs do not isolate the cost of one change. Ordinary field reads before any field move avoid owned projection-path allocation. Missing-field state uses frontend maps and deterministic joins; generated programs use native places/drop flags without runtime field-state maps or repair clones. Allocation/peak-memory profiling, changed-file reanalysis, and editor latency still require separate evidence.

## Accuracy evidence and remaining measurements

The current Rust library tests cover source contracts, related error locations, Unicode handling, bounded nesting, and 1,500 deterministic malformed token streams. The black-box suite verifies actual native output and rejected source through the Rust CLI. Neither constitutes a formal soundness proof or sustained fuzz campaign.

Before setting a language-server performance target, add representative projects and measure changed-file reanalysis, dependency invalidation, incomplete-source diagnostics, allocation/peak memory, cancellation responsiveness, and end-to-end editor latency. Keep native `rustc` build timings separate from frontend analysis timings.
