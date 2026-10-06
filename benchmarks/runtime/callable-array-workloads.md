# Callable array workloads

These workloads are registered for checkpoint 101. Correctness has been checked; timing remains pending. Run focused admission with:

```sh
node --test benchmarks/runtime/callable-array-pipeline.test.mjs benchmarks/runtime/callable-foundation-oracle.test.mjs
```

The admission test verifies emitter ABI, strict TypeScript compilation, Node execution, and unoptimized/optimized native execution. Each workload checks empty input, small input, zero seed, and 20,000 iterations. The independent oracle additionally checks 21 iteration/seed combinations against per-iteration arithmetic.

| Workload | Retained operations | Expected checksum |
| --- | --- | --- |
| callable_array_mutable_pipeline | Two independent mutable captured records; map, flatMap with indices, then indexed calls | seed + 6*n*seed + 3*n*(n-1) + 19*n |
| callable_array_transfer_pipeline | Push, unshift, replacement, remove, splice; invoke removed and remaining owners; final length | seed + 4*n*seed + 2*n*(n-1) + 12*n |
| callable_array_reduction_pipeline | Forward sum, reverse order-sensitive accumulation, then final state calls on two independently captured records | seed + 15*n*seed + 15*n*(n-1)/2 + 54*n |
| callable_array_once_transfer | Same mutations; explicitly remove each once owner before its sole invocation; final empty length | seed + 4*n*seed + 2*n*(n-1) + 10*n |

The TypeScript reference uses `splice(0, 1)[0]` for `remove(0)`. That creates additional temporary arrays; it is a conventional observable equivalent, not an allocation-equivalent implementation. The native compiler enforces ownership and single-use contracts; the reference follows those invocation conventions. These workloads measure whole pipelines at the scheduled checkpoint, not isolated method latency or dispatch overhead. Optimizers may inline environments.

Short-circuit predicate coverage, borrowed data argument coverage, and remaining callable-container features still need matched workloads. No complete container gate acceptance is implied by these four workloads.
