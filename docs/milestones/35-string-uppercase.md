# Milestone 35: standard Rust uppercase

[Roadmap](../roadmap.md) · [String contract](../string-semantics.md)

## Outcome and scope

Support TypeScript-style `text.toUpperCase()` using Rust's standard locale-independent `str::to_uppercase()`. The result owns a standard UTF-8 `String` through the existing Cow representation. The receiver is borrowed and evaluated once; source values remain reusable. The method takes no arguments and performs no implicit source clone.

Unicode case expansion follows the pinned Rust toolchain. This milestone adds no locale API, encoding cache, literal dispatch, or custom string representation.

## Acceptance evidence required

- Native development and release execution covers ASCII, Unicode expansion, combining marks, NUL, astral scalars, and empty strings.
- Results remain valid after owned or borrowed sources are disposed; repeated conversion leaves sources intact.
- Named parameters, captured values, record fields, array elements, and chained string methods retain borrowing and evaluation rules.
- Receiver types, argument counts, transfers, and overlapping accesses receive source diagnostics.
- Complete compiler regression and required Rust checks pass.
- Matched literal and dynamic Node/TypeScript counterparts validate actual returned contents independently, preserve raw records, and update `BENCHMARKS.md`.

## Current status

The reviewed implementation is integrated after milestone 34's complete acceptance. All three uppercase groups and nine existing string groups pass, including native execution at optimization levels 0 and 3. Conversion delegates directly to the Rust standard library. The single complete regression run passes all 484 groups. Formatting, workspace Clippy with warnings denied, release build, and doctests also pass. The [compiler acceptance record](../../benchmarks/runtime/diagnostics/2026-10-01T23-52-06-751Z-native-uppercase-compiler-acceptance.json) preserves commands, logs, toolchain identity, and unchanged source hashes.

Benchmark acceptance is complete. The [targeted two-workload run](../../benchmarks/runtime/results/2026-10-01T23-55-50-833Z.json) records 80 independent runtime checks and 1,488 timing batches across eight runtimes. Earlier measurements remain unchanged. The [native allocation profiles](../../benchmarks/runtime/diagnostics/2026-10-01T23-56-12-477Z-native-allocations.json) contain six samples with zero live owned bytes after disposal. The [runtime proof](../../benchmarks/runtime/diagnostics/2026-10-01T23-57-07-000Z-native-uppercase-runtime-proof.json) preserves actual source identities, prior raw records, and two failed preparation attempts that emitted no timing records.

Literal ASCII uppercase averages 70,368,107 ops/s native versus 45,604,963 for Node after tsc. Dynamic Unicode uppercase averages 13,425,138 versus 301,223,135. These are measured standard-library and engine costs, not a universal native performance claim. The dynamic input is reused, and immutable engine strings can have different optimization and allocation behavior. Native conversion retains real owned results and delegates directly to Rust's standard library.

Both native workloads allocate 20,001 times and reallocate 12 times. Peak and retained owned storage is 1,019,767 bytes for literal ASCII and 1,226,432 bytes for dynamic Unicode; all storage is released. Literal correctness includes NUL. External benchmark input excludes NUL because process arguments cannot transport it; the limitation is explicit in the workload metadata.

This gate is complete. Results appear in the string section of [BENCHMARKS.md](../../BENCHMARKS.md).
