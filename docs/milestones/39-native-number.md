# Milestone 39: standard Rust numbers

[Roadmap](../roadmap.md) · [Primitive coverage](../guides/language/feature-coverage.md)

## Contract

TypeScript-style `number` uses standard Rust `f64` and copies as a scalar. Existing `int` retains checked signed 64-bit operations. Decimal and scientific literals infer `number`; integer spellings retain inferred `int` for compatibility and become `number` in a number annotation, argument, return, field, array, or arithmetic context. Nonliteral `int` bindings require explicit conversion when a number is needed.

Number arithmetic uses ordinary floating-point operations. NaN, infinities, division by zero, overflow, underflow, and signed zero follow Rust's IEEE behavior; checked integer helpers do not apply. Equality and ordering use standard `PartialEq` and `PartialOrd`. Unary plus is number identity rather than JavaScript coercion. Native display uses Rust formatting.

Number arrays use ordinary Rust vectors, including nested ranks. Fields, indexing, mutation, push, remove, reversal, cloning, iteration, parameters, callbacks, and captures retain existing ownership contracts. Search uses native `PartialEq`: NaN does not match in `includes` or `indexOf`. JavaScript `includes` matches NaN; that difference is deliberate and must be disclosed in comparisons.

`Number()` returns positive zero. `Number(value)` accepts the implemented primitive types: int casts through standard Rust conversion, number copies unchanged, boolean becomes zero or one, and string is borrowed once and parsed with Rust `trim` and `parse::<f64>()`. A trimmed empty string becomes zero; failed parsing produces NaN. Rust whitespace and accepted numeric spellings can differ from JavaScript, including BOM, U+0085, and hexadecimal strings. Object and array coercion remain unsupported.

## Acceptance

Verify literal contexts and ranges, int separation, operator evaluation order, IEEE boundaries, precision, signed zero, every existing number-bearing storage path, explicit conversion, and source reuse. Cover type, arity, ownership, capture, and borrow diagnostics. Execute native tests at optimization levels 0 and 3, then one settled full regression.

Matched runtime comparisons must preserve operation order and precision domains, validate actual results independently, and state deliberate semantic differences. Primitive number and number-array memory probes use normalized units and retain raw records. The remaining Number and Math member catalog stays separately tracked.

## Status

Number support is integrated onto the accepted control-flow implementation, preserving string search, array reversal, temporary length, loop traversal, and reduced parser stack frames. All seven focused groups and eleven affected checks pass at optimization levels 0 and 3. Compiler acceptance passes 509 unique groups across 49 integration binaries, doctests, formatting, Clippy, and release build. The single full regression required two old diagnostic inputs to use `(1).method()` after decimal lexing was added; the failed group alone was rerun and previously unrun suites resumed. No compiler repair or full restart was needed. The [compiler acceptance proof](../../benchmarks/runtime/diagnostics/2026-10-02T01-29-22-000Z-native-number-compiler-acceptance.json) retains the commands, logs, fixture diff, and source identities. Runtime acceptance also passes, as recorded below.

Sixteen comparison pairs cover six arithmetic operators, six comparisons, three primitive conversions, and one explicitly compound number-array workload. They pass 340 matched Node cases and 384 independent IEEE comparison cases. Scalar rows accumulate ordinary numeric results without mandatory output arrays. Exact result observers and a normalized number-value memory probe are integrated into the benchmark harness; all 82 light tool checks pass. Native adapter checks pass at optimization levels 0 and 3. Sixteen workloads pass 640 independent engine checks and record 11,904 timed batches across eight runtimes. Forty-eight allocation profile samples and six normalized scalar memory samples pass, with zero native bytes remaining after final disposal. The [immutable source proof](../../benchmarks/runtime/diagnostics/2026-10-02T01-16-58-000Z-native-number-benchmark-source.json) preserves the previous 77 measured pairs and 300 raw records. New source entries do not claim measurements. Zero-argument construction and number identity conversion remain correctness guarantees rather than fabricated isolated throughput rows.

The [runtime acceptance proof](../../benchmarks/runtime/diagnostics/2026-10-02T01-34-50-000Z-native-number-runtime-proof.json) links immutable timing, allocation, and memory records. Top-level [BENCHMARKS.md](../../BENCHMARKS.md) now contains 93 dated runtime rows and nine primitive probes, retaining previous measurements. These complete iterations lead Node in all sixteen new cases; Bun leads boolean conversion and the compound array case. Scalar number storage is eight bytes with zero operation allocations. Number arrays retain ordinary vector allocation behavior; this gate does not introduce a custom representation or claim every runtime comparison is a native win.
