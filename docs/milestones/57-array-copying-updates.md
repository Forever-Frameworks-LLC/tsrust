# Milestone 57: array copying updates

Status: complete. One 585-group regression, required Rust checks and fresh O0/O3 release checks pass. Actual counterpart validation covers 1,836 TypeScript cases and nineteen unique Node harness groups, retaining passes across metadata and runtime-dispatch repairs. Sixteen comparisons across eight runtimes and 48 allocation profiles are recorded; all profiles fully dispose storage.

## Operations

Support `array.toReversed()` and `array.with(index, value)` across every current dense array leaf and rank, including recursive records. Both return fresh owned arrays and leave the source available. `toReversed` accepts no arguments. `with` requires an `int` index and a replacement of the exact element type; negative indices count from the end. Reject out-of-range indices through the existing source bounds-failure mechanism rather than clamping them. Empty arrays can be reversed but have no valid replacement index.

Evaluate the receiver once, then arguments once from left to right. Preserve existing receiver and selected-parent loans through argument evaluation. Evaluate both `with` arguments before reporting an invalid replacement index. Selected-receiver bounds failures still occur before argument evaluation. Handle signed integer extremes without overflow.

Owned descendants use the existing independent clone semantics. A replacement transfers into the result; explicit `clone` retains its original owner. JavaScript's shallow reference sharing remains an explicit difference for strings with owned storage, nested arrays and records. This gate must not introduce shared cells or a new container representation.

## Native implementation

Use standard Rust iteration, cloning and collection. Reverse copying can use `iter().rev().cloned()`. Replacement copying should move the replacement into its selected position and clone the remaining elements, avoiding a redundant deep clone of the overwritten source element. Operate only on live elements in existing compact arrays. Reuse current ordinary vectors, record clone/drop implementations and representation conversions.

Keep existing fixed primitive array support when its proof applies. Do not extend public layout guarantees or introduce unsafe code to obtain a particular benchmark result.

## Acceptance and comparison

Focused checks cover empty and singleton arrays, both index directions and signed extremes, exact Number bits, every current leaf and rank, recursive fields, source reuse and independent result mutation. Include selected and temporary receivers, evaluation effects, loan and ownership diagnostics, invalid-index failure order, and normal and unwind disposal at optimization levels 0 and 3.

Sixteen matched comparisons cover both operations over `int[]`, `number[]`, `boolean[]`, `string[]`, `int[][]`, ordinary record arrays, `int[][][]` and recursive record arrays. Complete outputs are observed outside timing. Receiver setup, replacement construction and retained output construction are timed. Native owned-copy costs and JavaScript shallow sharing remain disclosed; genuine min/average/max throughput and normalized allocation records are preserved.

The single full regression completed without restarting. Benchmark repairs retained passing groups and reran only affected checks. All 314 earlier runtime tables, nine primitive memory tables and startup/compilation tables remain byte-exact; no earlier measurements were repeated.

Compiler evidence: [585-group regression and final release checks](../../benchmarks/runtime/diagnostics/2026-10-02T12-57-58-000Z-array-copying-updates-compiler-proof.json). Counterpart evidence: [actual TypeScript and independent complete-output checks](../../benchmarks/runtime/diagnostics/2026-10-02T13-02-23-000Z-array-copying-source-validation.json). Runtime evidence: [timing, allocation, failed-attempt preservation and final acceptance](../../benchmarks/runtime/diagnostics/2026-10-02T13-15-38-000Z-array-copying-updates-runtime-proof.json). Native adapter acceptance includes 66 O0/O3 builds, 160 complete-result cases, 32 untimed disposal checks and two complete order outputs.

Native leads Node after tsc on 8 of 16 rows and the fastest scripting runtime on 6. Owned strings, nested arrays and records keep independent copying costs visible; no shared-reference equivalence is claimed.
