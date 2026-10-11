# Resumable callable summary scheduling

## Scope

The compiler retains contextual invocation frames and intraprocedural body state rather than replaying a caller's completed statements when a nested callee summary changes. Dependency coordinates identify the affected parent request and graph node. Resumption starts from saved pre-node partitions and preserves normal, return and throwing exits, pending return values through finally, source call facts and guard correlation.

Parameter defaults retain their completed predecessor state when a later default suspends. Changes to completed parameter-entry dependencies rebuild the frame conservatively. Capture reader changes invalidate cached sessions, and an epoch prevents stale in-flight state from being cached again. Existing recursive widening, nominal/provenance canonicalization, ownership and opaque-summary rules remain enforced.

An independent bounded eager path completes invocation-free return summaries and single-return wrappers whose fixed named target is such a leaf. It permits only one wrapper layer. Shadowed or dynamic targets, recursive bodies, deeper wrapper chains, default/rest signatures, argument invocations and mutation remain on the worklist. General non-leaf bodies use retained sessions; their correctness does not depend on eager eligibility.

No language syntax, public API, ownership keyword or built-in signature changes in this family. The editor continues to consume checked compiler facts.

## Focused evidence

Source checkpoints are 3a263b5 for persistent body extraction, bfbe317 for node resumption primitives, 366e0b0 for invocation sessions and parameter resumption, and 974fa8e for bounded wrapper completion. Body and invocation controls match independent one-shot reference paths. Delayed normal and failure cases preserve suffix and finally behavior; idle drives perform no additional calls.

All 102 generic-call controls pass with no scheduling budget excluded. At 8/32/64 nested independent calls, the unchanged budget records 17/65/129 evaluations and 16/64/128 invocation visits. Separate non-leaf controls verify that the outer prefix and suffix run once and cleanup executes on every retained normal and throwing path. Eight depth canaries, 23 affected native contracts and 57 editor projects pass.

Exact emitted hashes for generic_value_adapters, nested_function_defaults and scoped_callable_records match historical records on current debug source. Comparisons apply the benchmark harness's trimmed-output convention. The incorrect initial raw-versus-trimmed comparison and its resolution are retained as separate immutable SQLite records.

## Measurement and publication status

The earlier invocation-session checkpoint passes all five registered generic adapter native and Node-after-tsc correctness cases and preserves emitted/instrumented hashes. Its bounded release source check completes in about 2.47 seconds; this is a single compiler observation, not a repeated throughput or runtime benchmark. Existing runtime measurements remain historical evidence. Do not retime unchanged generated programs solely for presentation.

Source acceptance at 974fa8e passes the complete compiler crate: 2,342 checks and zero failures. Three intentional ignores are the opt-in worklist cost diagnostic, module frontend measurement and subprocess contention worker. The complete editor crate passes 360 checks with zero failures or ignores. Current release verification passes all five native and Node-after-tsc cases for each of generic_value_adapters, nested_function_defaults and scoped_callable_records. The record is runtime/results/2026-10-08T10-22-46-855Z-verify.json. No runtime timing is repeated for unchanged generated programs.

All five executable consumers are refreshed atomically from accepted source and pass CLI checking, native execution and editor protocol smoke checks. Smoke coverage includes bounded nested wrappers, retained generic factories, array transfer, rest invocation and the integrated path/OS exports. Cleanup removes 339 closed test/native/generated files totaling 6,505,287,564 bytes while preserving installed consumers and warm target root binaries. No cache-warming rebuild follows cleanup; future test builds may need to relink removed executables.

Canonical evidence remains benchmarks/records.sqlite. Read docs/plans/resumable-callable-summaries-plan.md and the current invocation-resumption handoff for continuation requirements. Preserve all fifteen owner fixture edits and serialize any parallel stdlib integration through exclusive main ownership.
