# Milestone 80: String.substr

Status: implementation and checkpoint 81 accepted. Focused behavior and complete IntelliSense examples pass; combined regression and selected comparisons are sealed.

## Behavior

`substr(start?: int, length?: int): string` returns a selection without changing the receiver. Bounds use native UTF-8 byte offsets. Start defaults to zero; negative starts count from the end, then clamp to the string bounds. Length defaults to the remaining bytes. Nonpositive length gives empty text; positive length caps at the remaining bytes without overflow. Both arguments accept signed 64-bit ints without coercion.

Nonempty selections require both endpoints to be Unicode scalar boundaries. Invalid boundaries fail with the call's source location. Empty selections skip boundary checks. JavaScript's legacy substr counts UTF-16 code units instead, so Unicode comparisons need an explicit coordinate conversion.

The receiver and supplied arguments are evaluated once in source order. The receiver remains borrowed through argument evaluation and stays reusable. Empty results use static empty text. Full nonempty results clone the existing Cow; partial static text borrows a static slice, while partial owned text is independently copied. This uses standard Rust String/Cow storage.

## Documentation and focused acceptance

The new built-in entry must explain behavior, bounds and defaults, errors, mutation, examples, UTF-8 differences and ownership. Hover, completion and optional parameter help must agree. Verify examples with the current compiler and generated O0/O3 execution. Focused fixtures must cover integer extremes, UTF-8 boundaries, empty selections, ordered evaluation, borrow restrictions, result independence and disposal.

Comparison sources must retain complete operation results and independently verify ASCII and Unicode bounds. New timings remain pending until checkpoint 81; earlier benchmark rows and raw records remain unchanged.

## Focused evidence

Four compiler groups pass with generated O0/O3 execution. One editor group verifies hover, completion, optional parameter help, chained String inference and five documentation examples with 13 exact output lines per optimization level. A separate source-order witness matches 20 complete native output lines across O0/O3.

Four comparison sources pass 548 cases, 64 independent literal hands, 101 negative guards and nine executed semantic mutants. The original observer failure is retained: a deliberately wrong Unicode receiver produces invalid text, which must be recorded as rejection rather than requiring a valid observation. Initial editor compilation encountered incomplete parallel nullish-type edits; its affected replay passes after those matches were repaired. Preserve the [implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T06-58-56-000Z-string-substr-implementation.json). Full regression and all four new timing rows remain pending at checkpoint 81. No performance or allocation-rate result is claimed here.

## Checkpoint 81 acceptance

The combined checkpoint accepts 700 compiler integration groups and 735 workspace groups with retained passes and no full restart. [Runtime and resource evidence](../../benchmarks/runtime/diagnostics/2026-10-03T08-44-00-000Z-checkpoint81-runtime-proof.json) binds 29 selected workloads across eight runtimes and 87 fully disposed allocation samples. [BENCHMARKS.md](../../BENCHMARKS.md) contains actual min/avg/max ops/s. Earlier pending statements describe the implementation-stage queue; this checkpoint closes that queue.
