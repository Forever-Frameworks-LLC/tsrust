# Milestone 38: blocks and additional loops

[Roadmap](../roadmap.md) · [Feature coverage](../guides/language/feature-coverage.md)

## Behavior

The compiler supports standalone statement blocks, `do ... while`, classic `for` loops, and `else if` chains. Blocks create lexical scopes and release local captures and owned values at scope exit. Existing binding rules, including the restriction on shadowing active names, remain explicit limitations.

Classic `for` evaluates its initializer once, then its condition before each body execution. A missing condition means true. Normal body completion and `continue` execute the update before the next condition. `break` and `return` skip the update. The initial syntax uses existing declarations, assignments, and expressions; update operators such as `++` remain separately tracked.

`do ... while` executes its body at least once. Normal completion and `continue` evaluate the condition; `break` and `return` skip it. Conditions remain strictly boolean under the current native contract.

Nearest-loop control applies across nested loops. Generated labeled Rust loops and iteration blocks preserve tail execution and cleanup. These internal labels do not add source-language labels.

Ownership analysis must join continuing paths before checking updates and conditions, retain the existing back-edge safety invariant, and distinguish definite first execution from a possible zero-iteration exit. An update that restores transferred storage must be checked before the next iteration, while an unrepaired transfer remains an error. Callback discovery, captures, dependency analysis, and storage proofs must traverse every loop component.

`else if` lowers into the existing nested branch representation, preserving branch ownership joins. Long chains receive an explicit nesting guard.

## Acceptance

Verify empty and bounded loops, initializer/condition/body/update order, first execution, nested nearest-loop behavior, and cleanup under break, continue, and return. Include accepted and rejected storage transfers, record-field restoration, mutable captures, callback flow, and strict condition diagnostics. Execute native examples at optimization levels 0 and 3, followed by one settled full regression.

Matched benchmarks must compare the same loop work against TypeScript/Node and pinned runners, observe real results, and retain raw operation records. Reuse unaffected startup and compilation baselines.

## Status

Compiler acceptance is complete: all 502 unique integration groups across 48 binaries pass, together with doctests, formatting, Clippy, and the release build. Seven focused control-flow groups execute native examples at optimization levels 0 and 3. They cover tail order, first execution, nested targets, transfers and restoration, captures, cleanup, branch joins, and diagnostics.

The single full regression stopped after 77 passes at an existing deep-index stack-overflow case. Narrow parser repairs remove an extra statement wrapper, extract call-argument temporaries from recursively retained index frames, and separate classic-for and recursive for-of parser frames. The 128-level nesting guard is unchanged: both 100-deep accepted cases and both 200-deep rejected cases pass. The later for-of continuation likewise exposed its existing deep-loop stack limit; both its 100-deep acceptance and 200-deep rejection pass after the frame separation. Both original depth fixtures remain unchanged apart from the already-reviewed classic-for rejection update. Only the failed depth groups, unpassed array/for-of cases, affected control-flow cases, and previously unrun binaries were executed afterward; there was no full restart. The [compiler acceptance record](../../benchmarks/runtime/diagnostics/2026-10-02T01-00-12-000Z-native-control-flow-compiler-acceptance.json) preserves commands, logs, reviewed integration identities, the repair, and the continuation inventory.

The gate is complete. Twelve source pairs cover three new control operations (`for`, `do...while`, and `else if`) and nine existing primitive comparison gaps. All 75 light tooling groups, 480 independent runtime checks, and 8,928 timed batches pass. The [execution record](../../benchmarks/runtime/results/2026-10-02T01-08-44-019Z.json) measures only these twelve rows; earlier startup, compilation, and 65 dated execution rows remain preserved. One fixture required declaration reordering to satisfy the benchmark entry ABI before timing; both function bodies and calls remained unchanged.

All twelve native rows exceed Node after `tsc` in this run, by 5.641–10.124 times. Units are complete workload iterations with setup and observation costs. Eight primitive rows each target one operator; short-circuiting includes OR, AND, and observable RHS calls and remains explicitly compound. These rates do not claim isolated instruction latency.

The [native allocation record](../../benchmarks/runtime/diagnostics/2026-10-02T01-09-31-341Z-native-allocations.json) contains 36 samples. Every sample reports zero operation allocations, reallocations, retained requested bytes, and peak requested bytes, with complete disposal. Input storage is accounted for separately. The [runtime acceptance proof](../../benchmarks/runtime/diagnostics/2026-10-02T01-09-41-000Z-native-control-flow-primitive-runtime-proof.json) verifies 188 frozen inputs, 296 unchanged historical records, all 130 prior source files, and exact timed/profile emission identities.

Standalone blocks provide lexical scope and cleanup guarantees, with correctness checked at optimization levels 0 and 3 and within compound control workloads. They have no meaningful isolated operations-per-second counterpart; no no-op block timing is claimed.
