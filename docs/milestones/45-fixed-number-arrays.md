# Milestone 45: fixed scalar array storage

[Roadmap](../roadmap.md) · [Primitive coverage](../feature-coverage.md)

## Scope

Extend the existing fixed-array proof from int and boolean literals to number literals. Stable, bounded local arrays use standard Rust `[f64; N]` storage when the compiler proves that their length does not change and their storage does not escape. Preserve ordinary storage for growing, returned, captured, or otherwise unproved arrays.

Directly indexed temporary scalar literals may also use standard Rust arrays when they fit the same function storage budget. Evaluate every literal element and the index once in source order, preserve checked bounds failure, and transfer the selected Copy scalar. Arrays returned from a constructor keep their return ABI even if the caller immediately indexes them; this gate does not specialize escaping return values.

Keep the existing conservative proof limits: 4 KiB of fixed storage, 64 variants, 10,000 traversal steps, and bounded depth. Temporary plans share the storage budget with fixed locals. Exceeding a proof limit falls back to ordinary storage. Use standard Rust structures without a new runtime representation.

## Acceptance

Focused native programs run at optimization levels 0 and 3. Cover number literals and indexed writes, signed zero and IEEE values, current array methods and callbacks, evaluation order, bounds failure, empty arrays, and fallback after mutation, capture, escape, or proof exhaustion. Check emitted Rust storage and allocation behavior for representative proved and unproved programs. Preserve existing depth guards and borrowing diagnostics.

Run one settled full compiler regression after integration. Benchmark actual supported operations against matching TypeScript programs, including direct literal indexing and stable local number arrays. Retime only newly measurable or affected workloads; keep unrelated historical records and dates. Record allocation and disposal evidence separately from throughput.

## Status

Complete. All four new focused groups pass at optimization levels 0 and 3. The single settled regression passes 531 groups and doctests; formatting, Clippy with warnings denied, and the corrected release artifact pass. The [compiler proof](../../benchmarks/runtime/diagnostics/2026-10-02T03-54-19.029Z-fixed-number-array-compiler-proof.json) records the one fixture assertion correction. An actual CLI audit caught stale release caching caused by copied source timestamps; the [release correction](../../benchmarks/runtime/diagnostics/2026-10-02T03-56-46.843Z-fixed-number-array-release-correction.json) preserves that diagnosis and verifies rebuilt fixed-array emission. Source bytes stayed exact and the full regression did not repeat.

The [source proof](../../benchmarks/runtime/diagnostics/2026-10-02T03-39-47-000Z-fixed-number-array-benchmark-source.json) records 102 light checks and genuine parent emissions. Actual emission changes justify retiming 22 existing workloads alongside the two new pairs. The [runtime proof](../../benchmarks/runtime/diagnostics/2026-10-02T04-03-15-000Z-native-fixed-number-array-runtime-proof.json) seals 960 independent engine checks, 17,856 execution batches, and 72 allocation/disposal samples, all disposing to zero. BENCHMARKS.md contains 120 individually dated workloads and nine retained memory probes. The other 96 workloads and historical auxiliary measurements did not rerun.

## Runtime results

All 24 targeted workloads lead the measured runtimes in this run. Direct literal selection reaches 32.676 million whole iterations per second versus Node after tsc at 5.686 million; the stable local compound sequence reaches 98.416 million versus 7.345 million. Input evaluation, checked selection or writes, cloning, reversal, search, retained results, and loop work remain part of their stated measurement units. These rates do not isolate individual instructions.

The 21 scalar Number/Math workloads now allocate no heap storage. Each new workload allocates only its retained output vector: direct selection retains 16,384 bytes with nine reallocations; the compound sequence retains 524,288 bytes with fourteen. The number slice workload retains its owning result and existing return ABI. All 72 samples free completely. Escaping constructor results remain ordinary vectors; this gate does not remove their allocations.
