# Callable replacement family milestone

Family acceptance is pending. Implementation checkpoints are `c354b84` (bounded array provenance and independent extraction), `cd52a03` (nested owned capture graph transfer) and `530015c` (implementation and milestone checks), building on the earlier replacement carrier and checked invocation work.

Consumer smoke verification found that imported `std:testing.strictEqual` calls invalidate point-sensitive factory contracts. A focused project witness reproduces the arity failure; the same source with logging passes. The stronger witness also exposes lost identity through a stored native `join` alias. Native symbols now retain their provenance through aliases, and explicitly listed observer calls preserve capture state on normal and possible throwing exits. Callback runners and callback-sharing intrinsics remain opaque. The combined project witness passes at native optimization levels 0 and 3. All 535 compiler unit tests and 88 project tests pass. Final repair checks pass all eight depth canaries and all 358 editor tests. Combined consumer publication remains pending.

## Scope and acceptance evidence

The scope is finite generic callable replacement with point-specific invocation contracts and independently owned mutable factory state. It includes saved values, different declarations and arities, defaults and rest arguments, callback and selected-body failures, borrowing and consuming access, disposal, recursive requests, loop-created factories, array removal/reinsertion and nested owned captures. It does not claim general scoped aggregate composition, arbitrary borrowed callback storage or JavaScript shared capture cells.

| Requirement | Current evidence | Status |
| --- | --- | --- |
| Literal and runtime replacement, saved original values | generic_call_capture_effects source/native witnesses; matched replacement workload | Focused checks pass |
| Reachable arity checking and related origin diagnostics | generic-call source proof controls and capture-effect rejection witnesses | Focused checks pass |
| Callee borrowed during argument evaluation | generic_callable_replacement_rejects_mutation_of_the_borrowed_callee | Pass |
| Independent checked bodies, defaults, rest, nullable arguments and callback errors | Adjacent native replacement witnesses and nineteen-observation matched workload | Focused checks pass |
| Capture borrowing, consumption and exactly-once disposal | Capture-effect ownership rejection and counted-drop native witnesses | Focused checks pass |
| Point-sensitive factory state, independent loop instances and restored captures | Factory, restoration, reinsertion, repeated extraction and fresh allocation witnesses | Focused checks pass |
| Nested owned capture graph transfer | Nested factory native witness; cycle, borrowed-root and failed-transfer graph controls | Focused checks pass |
| Finite array provenance and transfer identities | Concrete-word sequence controls, loop convergence and allocation-key controls | Pass |
| Editor diagnostics and selected signatures | Direct, loop and nested factory intelligence tests | Focused checks pass |
| Formatting and executable guide | Native helpers format before execution; whole-project formatting is idempotent; nested-functions guide | Pass |
| Matched sources, oracle, ABI, registrations and runner hooks | generic_callable_replacement and mutation guards; five native/Node-after-tsc cases | Untimed checks pass |
| Complete compiler regressions | 232 integration targets covered; 2,325 unique passes after affected repair checks, zero failures, two intentional worker ignores; interrupted and repaired evidence retained | Pass |
| Complete LSP regressions | 358 passing checks, zero failures | Pass |
| Performance review under original protocol | 93 samples per engine; median native batch 0.157334 ms versus Node-after-tsc 1.575708 ms; historical profiles are not comparable | Pass |
| Acceptance metadata and five consumer refreshes | Five consumers refreshed from 530015c; combined smoke exposed imported assertion analysis failure; repair publication and smoke remain pending | Pending |

All eight default-stack depth canaries pass on the graph-transfer implementation. Exact focused evidence and untimed verification records are retained in benchmarks/records.sqlite. The latest workload verification is runtime/results/2026-10-08T07-20-19-446Z-verify.json. That record contains zero execution timing samples.

## Review boundaries

Allocation keys exclude recursive environment edges. Array removals read current live capture facts and assign independent analysis identities; unknown and consumed states do not authorize stale reconstruction. Reachable owned child environments are remapped iteratively with cycles and shared edges preserved. Borrowed lexical roots retain their identity. Failed graph validation leaves the value and outgoing flow state unchanged. Native owners are moved under the existing source ownership checks and are not implicitly cloned.

Run complete regressions once for this family, retain passed groups, and repair/resume only affected checks. Record actual failures without classifying them as historical baselines unless the existing accepted evidence proves that classification. Measure only accumulated affected workload profiles, keep original sample protocols, and distinguish correctness acceptance from performance results. Preserve all fifteen protected owner fixture edits.


## Measurement protocol for this family

After compiler and LSP milestone regressions finish, measure generic_callable_replacement with native and Node-after-tsc using the normal execution-only command. Retain the manifest protocol: three processes per engine, eight warmup samples per process and thirty-one measured samples per process. The workload performs 2,048 complete scenarios per batch and observes nineteen scalar results per scenario. Keep the denominator as one complete scenario, not nineteen operations.

Do not compare the new nineteen-observation cost profile directly with the earlier nine-, thirteen- or sixteen-observation profiles. Compare native and Node within the same current run. Any historical regression comparison must use matching source semantics and measurement scope. Avoid timing while full compiler/native regression workers compete for machine resources. Preserve all original samples and the exact record in SQLite before any optimization. A slower ratio warrants review, not weakened correctness checks or an altered sample protocol.

The original compiler milestone preserves 85 completed passing groups (1,188 tests), then records explicit termination of an unbounded recursive mutable-array test. The sampled worklist hashed indefinitely growing exact sequence keys. Recursive request widening repairs the defect; the original native witness and all 91 adjacent generic-call controls pass. The resumed milestone retains the original completed groups rather than restarting them. Full compiler acceptance still requires the remaining terminal results and affected depth/editor checks.

Compiler milestone reconciliation verifies all 232 integration targets and 234 accepted groups. The affected library has 534 passes; closure and for-of suites have 14 and 10 passes. Compiler documentation tests contain zero cases. All eight depth canary names are verified in repaired milestone logs. Receipt: tsrust-callable-family-compiler-coverage.json in SQLite. Full editor evidence: tsrust-callable-family-editor-milestone.log. Closed evidence was read back exactly before removing loose copies. Performance and consumer acceptance remain pending.

Execution-only measurement runtime/results/2026-10-08T07-43-42-877Z.json retains 186 original timing samples across six processes. Native median speedup is approximately 10.02 for this workload and machine. This is not a general language performance claim. The performance review is tsrust-callable-family-performance-review.json in SQLite. Normalized analytics preserve 93 samples and three process identities per engine. Proof runtime/diagnostics/callable-replacement-family-2026-10-08/measurement-acceptance.json explicitly submits no allocation profile. No historical nineteen-observation baseline exists, and no additional optimization is warranted by this faster native result alone.
