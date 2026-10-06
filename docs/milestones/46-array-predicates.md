# Milestone 46: array callback predicates and index search

[Roadmap](../roadmap.md) · [Primitive coverage](../feature-coverage.md)

## Scope

Add `some`, `every`, `findIndex`, and `findLastIndex` across all current array element types and ranks. Reuse the typed reusable callback contract from milestone 44: zero through three parameters for value, int index, and readonly source. Require a boolean result. Support typed arrows, named callbacks, reusable callback parameters, and current named, field, indexed, and temporary receivers.

Evaluate the receiver and callback once, including empty arrays. Invoke callbacks in ascending index order for `some`, `every`, and `findIndex`, and descending order for `findLastIndex`. Stop immediately when the method determines its result. Empty arrays return false, true, -1, and -1 respectively. Index results use int, with -1 for a miss.

Use standard Rust iterator operations and existing borrowed storage. Preserve readonly source and owned-element loans, reusable mutable captures of other storage, and rejection of consuming callbacks. No new output container or clone is needed. Keep the existing restrictions on `thisArg`, truthiness, and sparse arrays explicit.

## Acceptance

Focused programs run at optimization levels 0 and 3. Cover each supported leaf and rank, callback arity and origin, empty inputs, matches at both ends, misses, exact callback order and count, receiver/callback factory effects, mutation and ownership diagnostics, and short-circuit disposal. Preserve depth guards and existing callback contracts. Run one settled full regression after integration.

Matched TypeScript comparisons use the actual four array methods across int, number, boolean, owned string, record, and nested-int domains, with independent result and callback-trace oracles. Record min/avg/max throughput and native allocation/disposal evidence. Keep complete results observable and preserve unrelated historical measurements.

## Dependencies and status

The exact reviewed compiler candidate is integrated and compiler acceptance is complete. Changed compiler timestamps are refreshed to prevent stale Cargo artifacts. All four focused groups pass at optimization levels 0 and 3, and affected depth, generic callback, and fixed-number receiver checks pass. The single full regression passes all 535 groups across 56 test binaries and the doctest stage without repairs or restarts. Formatting, clippy, and release checks pass. Actual fresh debug/release emission agrees; the prior release rejects the new methods, while the fresh release executes their exact short-circuit result and count. The [immutable compiler proof](../../benchmarks/runtime/diagnostics/2026-10-02T04-29-08-000Z-array-predicates-compiler-acceptance.json) records source identities, commands, release provenance, and archived artifacts.

Runtime acceptance is complete for twenty-four comparison pairs across six element domains. Independent validation passes 1,128 source result/trace cases, 160 guards, 48 prepared TypeScript driver checks, and all 106 light benchmark groups. All 24 actual native wrappers pass ABI and complete-output checks. The [execution-only record](../../benchmarks/runtime/results/2026-10-02T04-35-52-687Z.json) validates 960 engine cases and 17,856 measured batches; all 72 [native profile samples](../../benchmarks/runtime/diagnostics/2026-10-02T04-36-18-227Z-native-allocations.json) fully dispose storage. The [immutable runtime proof](../../benchmarks/runtime/diagnostics/2026-10-02T04-36-28-000Z-native-array-predicates-runtime-proof.json) preserves 120 previous pairs and 340 preceding records. BENCHMARKS.md contains 144 separately dated runtime rows and nine retained memory probes. Only the new rows were measured. Native rates exceed Node by 1.70–6.45 times and the fastest measured scripting runtime by 1.25–4.65 times for these complete workloads; these are not isolated method latencies.

Preparation initially failed because ES2022 TypeScript library declarations omit `findLastIndex`. Upgrading declarations to ES2023 fixes the six affected preparations while retaining the ES2022 output target and byte-identical JavaScript/native wrappers. The failed attempt produced no timing samples; its log and source snapshot remain archived. Accepted compiler and native preflight checks were retained rather than repeated.

Forward methods use standard iterator short-circuit operations. Reverse search borrows elements at original descending indices through ordinary `get`; runtime storage and iterator traits remain unchanged. Index-plus-one observation is reversible, and native string clone costs versus JavaScript immutable string reuse are explicit. `find` and `findLast` await a real optional-result type and ownership contract; this gate does not substitute a panic or invented missing-value sentinel.
