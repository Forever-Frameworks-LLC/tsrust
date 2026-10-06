# Milestone 41: standard Rust Math transforms

[Roadmap](../roadmap.md) · [Primitive coverage](../feature-coverage.md)

## Scope

Add static `Math.abs(value)`, `Math.floor(value)`, `Math.ceil(value)`, `Math.trunc(value)`, and `Math.sqrt(value)`. Each accepts exactly one number argument and returns a number. Numeric literals use contextual number typing; int bindings require explicit `Number(...)` conversion. Evaluate the argument once and call the corresponding standard Rust `f64` method. Preserve existing ownership, capture, and borrow rules.

Signed zero, NaN, infinities, subnormals, fractions, and finite boundaries follow the native methods. Negative square roots produce NaN; square root preserves negative zero. Math rounding with halfway tie rules, other Math methods, constants, and coercion from other types remain separate work items.

## Acceptance

Focused native programs run at optimization levels 0 and 3. Verify exact finite and signed-zero results, semantic NaN classification, infinity signs, argument evaluation order, captures, and type/arity diagnostics. Preserve accepted Number predicates and existing fixed-array proof behavior. Run one settled full regression after integration.

Five matched comparisons use one transform per complete iteration and disclose input selection, loop, and accumulation costs. Include ordinary non-perfect-square square-root inputs, with independent IEEE result validation. Keep special-value correctness cases separate when they would poison accumulated outputs. Record execution-only timing and native allocation/disposal profiles in immutable files, then update top-level BENCHMARKS.md.

## Status

The compiler candidate is integrated onto accepted milestone 40, preserving its reduced recursive analyzer frames. All three focused native groups pass at optimization levels 0 and 3 without repairs. The existing operator depth group and three Number predicate groups also pass. Compiler acceptance passes all 515 groups across 51 integration binaries, doctests, formatting, Clippy, and release build without repairs or restarts. The [compiler proof](../../benchmarks/runtime/diagnostics/2026-10-02T02-16-44-000Z-math-unary-compiler-acceptance.json) preserves exact inputs, source identities, and command logs. Five comparison pairs and independent oracles are integrated into the benchmark harness; all 88 light tool groups pass. The [source proof](../../benchmarks/runtime/diagnostics/2026-10-02T02-02-22-000Z-native-math-benchmark-source.json) preserves prior sources and raw records. Native measurements pass 200 independent engine checks and 3,720 timed batches across eight runtimes. Fifteen allocation/disposal samples pass with one input-table allocation, no reallocations, and zero bytes after final disposal. Light Node checks pass 120 matched workload cases and 110 direct IEEE cases. The square-root oracle uses integer arithmetic to determine nearest-even results, then validates each accumulated floating-point addition in execution order. Timed square-root inputs include ordinary non-perfect squares as well as perfect squares. Runtime acceptance is complete. The [runtime proof](../../benchmarks/runtime/diagnostics/2026-10-02T02-18-53-000Z-native-math-unary-runtime-proof.json) retains actual timing, allocations, exact IEEE validation, and unchanged prior records. BENCHMARKS.md now contains 102 dated runtime rows and nine retained memory probes. These complete iterations lead Node and the fastest scripting runtime in this run; table setup, selection, loop, and accumulation are included. No new measurements are claimed.
