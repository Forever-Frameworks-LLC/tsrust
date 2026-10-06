# Milestone 59: array reverse search

Status: complete. Four focused groups, one 593-group regression, required Rust checks and fresh O0/O3 release checks pass. Eight default and bounded comparisons across eight runtimes, 756 TypeScript observations, eighteen unique Node harness groups and 24 native allocation profiles complete acceptance. Every output and final disposal check reaches zero live bytes.

## Primitive search contract

Support `array.lastIndexOf(value, fromIndex?)` over the same dense primitive arrays as existing `includes` and `indexOf`: int, Number, boolean and native UTF-8 strings. Return the last matching array index or -1. Default to the final live element. Clamp nonnegative `int` bounds to the final index; normalize negative bounds relative to length, returning no match when the normalized bound is below zero. Empty arrays return -1 after argument evaluation. Handle both signed integer extremes safely.

Use strict equality for searching: NaN never matches, and positive and negative Number zero compare equal. Strings use ordinary exact native equality. Nested-array and record reference identity are outside the current primitive-search contract; this gate must not introduce artificial identities or silently substitute structural equality.

## Implementation and acceptance

Use standard Rust reverse iteration, bounded skipping and position search over the existing live elements. Reuse current containers and reverse iterators. Read the receiver and search value without consuming owners, preserve receiver and selected-parent loans during argument evaluation, and evaluate the receiver and arguments once in source order. Bounds failure while selecting a receiver precedes argument evaluation.

Focused checks cover duplicates, misses, every primitive leaf, empty/singleton/fixed/inline/heap arrays, unused storage, both bound directions and signed extremes, Number NaN and signed zero, dynamic Unicode strings, source reuse, argument effects and loan diagnostics. Focused allocation instrumentation confirms that searching allocates no storage; normal and unwind disposal checks pass. The full regression ran once after focused acceptance.

Eight comparisons separate default and bounded searches for int, Number, boolean and UTF8 strings. Every workload invocation builds a 129-element receiver once, then retains all 2,048 search results in an int array as index + 1. This maps -1 through 128 to 0 through 129 without losing information; subtracting one recovers each original index. Exact complete-array encoding and comparison run outside timing. Receiver setup, needle and bound selection, searching and output retention are timed, so these measurements describe complete search iterations. Native string setup owns independent copies while JavaScript may share immutable references.

The independent oracle scans forward and retains the greatest eligible match, without calling a search builtin. Actual TypeScript checks cover 756 cases, twenty-one mutation and domain negatives, fourteen independent hand cells, twelve signed-boundary cells and complete receiver/needle/bound evaluation order. Queries cover duplicates, misses, empty and Unicode strings, NUL, combining characters, NaN, signed zero, infinities and signed integer extremes. JavaScript Number rounding of extreme i64 literals leaves the same clamped search range.

Native preflight records 34 O0/O3 builds, eighty complete outputs, sixteen untimed allocation/disposal checks and two complete order outputs. Timing records 192 process cells and 5,952 measured batches, publishing 64 min/average/max rate cells. Twenty-four allocation profiles retain raw counters and report min/average/max allocations, reallocations and requested, retained and peak bytes per operation across three samples. Input adaptation stays separate. Peak memory is the retained-batch peak divided by 2,048 operations, not an isolated-call peak. All live bytes reach zero after output and final disposal. All 338 earlier runtime tables, nine primitive memory tables and auxiliary startup/compiler baselines remain byte-exact; no earlier measurements were repeated.

Compiler evidence: [593-group regression and fresh release checks](../../benchmarks/runtime/diagnostics/2026-10-02T14-09-00-000Z-array-last-index-of-compiler-proof.json). Source evidence: [actual TypeScript and independent complete-output checks](../../benchmarks/runtime/diagnostics/2026-10-02T14-17-00-000Z-array-last-index-of-source-validation.json). Runtime evidence: [timing, native output and normalized allocation acceptance](../../benchmarks/runtime/diagnostics/2026-10-02T14-22-23-000Z-array-last-index-of-runtime-proof.json).

Native leads Node after tsc on 8 of eight rows and the fastest scripting runtime on 8.
