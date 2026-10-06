# Milestone 33: native UTF-8 strings

[Roadmap](../roadmap.md) · [String contract](../string-semantics.md) · [Benchmarks](../../BENCHMARKS.md)

The earlier JavaScript UTF-16 contract is superseded by the user's native Rust direction. Its implementation, experiments, tests, and documentation are archived outside the active crates.

## Contract

Use standard Rust UTF-8 storage and methods. Length counts bytes; slice uses checked byte boundaries; charAt uses scalar iteration. Rust trim and lowercase rules apply. Valid escaped surrogate pairs decode to scalars, while isolated surrogates are syntax errors. Array ownership, operand order, source diagnostics, and independent mutable clones remain unchanged.

## Acceptance

- Remove obsolete encoding variants, cache/intern metadata, slice-view owners, unreachable syntax variants, and unused support from the crates.
- Exercise ASCII, astral scalars, combining sequences, NUL, signed offsets, valid/invalid UTF-8 slice boundaries, literal/runtime values, clones, calls, captures, and final disposal.
- Run the complete compiler suite, formatting, and Clippy after migration.
- Update benchmark workloads, counterparts, independent oracles, and memory instrumentation to the same native contract. Preserve historical records and identify the new semantic epoch.

This gate is complete. Compiler migration and cleanup pass all 475 integration tests, formatting, Clippy, release build, and documentation checks. See the [compiler proof](../../benchmarks/runtime/diagnostics/2026-10-01T23-05-37-467603Z-native-utf8-compiler-migration.json). The [fresh timing run](../../benchmarks/runtime/results/2026-10-01T23-20-28-895Z.json) completes all 50 workloads across eight runtimes and three processes, with 37,200 measured batches. All 150 [native allocation samples](../../benchmarks/runtime/diagnostics/2026-10-01T23-21-05-187Z-native-allocations.json) and 48 native samples across eight [primitive memory probes](../../benchmarks/runtime/results/2026-10-01T23-21-47-473Z-allocations.json) release owned bytes after disposal. The [final comparison proof](../../benchmarks/runtime/diagnostics/2026-10-01T23-22-32-000Z-native-utf8-current50-proof.json) preserves current source identities and unchanged historical records. Performance differences are recorded honestly; semantic parity machinery is not restored merely to win a historical comparison.
