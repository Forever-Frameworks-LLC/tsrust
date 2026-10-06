# Handoff: captured deferred generic invocation

Read AGENTS.md, docs/development-workflow.md, generic-function-values-plan.md and
nested-function-declarations-plan.md. Previous checkpoint: c495ed2. This checkpoint
implements captured deferred generic values; full generic values, gate173 and the
named-declaration family milestone remain open.

## Implementation

Deferred Source retains a typed lexical Context and BindingIds. Construction uses
named_captures.source_context without selecting a sample generic type. Original
captures share one native environment across concrete methods. Owned factory calls
construct independent instances. The capture-access lattice now also converges by
original declaration name and BindingId across concrete generic instances.
Single-file check requests the same finite inference before exposing captured
layouts; otherwise its provisional scalar copy could miss a live borrow conflict.

Concrete call preparation merges retained lexical identities into a private
checking context, attaches them before checking the selected generic instance and
retains original parameter, omission, rest and type constraints. Capture-free calls
avoid this Context clone. Existing resolved_call specialization remains intact.
Methods retain their own receiver access and capture binding slots, distinct from
stable environment field slots. Read methods reborrow exclusive stored references;
copied scalar fields follow the selected method's capture_reference representation.
No unresolved data binding, ordinary void wildcard, shared cell or implicit clone
is introduced. The prototype remains excluded from result hints and editor facts.

The five original captured witnesses now pass: shared local state, independent
inferred factories, mixed scalar/array values, array patterns and default/rest
prefixes. A result-only generic declaration can be stored without invoking it;
propagating expected result types into its alias calls is still pending.

## Evidence and repairs

Final affected native checks pass: 15 generic_function_values tests, two
signature tests and six higher-order controls. Four layout/backend tests pass.
Exact disposal observes five owners once at O0/O3 across data types, failure,
consuming captures and recursive groups. Invalid controls reject owner reuse and
mutable capture conflicts. Concrete captured call hovers and original declaration
navigation pass. Whole editor crate: 285 tests.

All eight depth-canary targets pass (63 tests). Retain both broad runs: the first
passed all its 79 cases; the later run retained all canary passes but exposed a
contextual array callback regression. Its repaired affected group passes 23 tests.
The wrapper dispatch must distinguish already solved explicit parameters from
inferred generic ones; do not replace that guard with every generic-looking call.
The shared reborrow backend witness was genuinely red with E0507 before repair.

The generic_function_captures workload is appended at manifest end with a native/
TypeScript pair, independent nine-value oracle, source hashes, mutation guards,
registration, existing array ABI/driver and oracle/run/profile/report hooks.
Two registration tests pass. Five native/Node-after-tsc cases pass untimed.
Canonical record: benchmarks/runtime/results/2026-10-06T22-54-57-153Z-verify.json.
All 117 compiler and 260 oracle input hashes match final source; execution is empty
and verifiedOnly is true. implementationAccepted and measured remain false.
The first matched attempt omitted the required empty main in the new witness;
that source and its guard were repaired. The failed attempt and completed earlier
workload preparation remain preserved; the repaired run selects the affected family.

Twenty-five original logs, the summary target/editor/captured-deferred-evidence.json
and the verify record are imported and read back byte-for-byte in
benchmarks/records.sqlite. Log paths are tsrust-captured-deferred-*.log and tsrust-captured-guide-example.log.
The complete guide factory example executes with exact output 11, 21 and 12. Loose copies
are removed only after verification. Original failures and repairs remain intact.
Five executable consumers are atomically refreshed and hash-verified. All tool
handles are terminal. Obsolete test/native executables and scratch trees are
pruned; library caches and five current consumers remain.

## Remaining work

Complete generic alias hover, completion and parameter help from real source
contracts. Propagate expected result contexts through unannotated alias calls.
Implement actual polymorphic callback and record/array storage adaptation rather
than weakening the prototype guard. Audit callback-dependent access, generic
outer factories, imported nominal types, patterns, result alternatives, catch/
finally and deeper context combinations against the full gate173 plan. Complete
the meaningful named-declaration family regression/performance review only when
its scoped implementation and editor audit are ready. No acceptance of the full
gate or objective is claimed here.

Four owner fixture edits remain untouched and uncommitted: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr and map-group-by/documented.tsr.
Use explicit Git paths, offline locked Cargo, the single warm target and
CARGO_INCREMENTAL=0. Hourly checks remain paused.

## Continuation prompt

Continue Tsrust from docs/handoff-2026-10-06-gate173-captured-deferred.md.
Read AGENTS.md, docs/development-workflow.md and both generic/nested function plans.
Captured deferred values now pass their original five witnesses. Continue full
alias IntelliSense, result-context propagation, real callback/storage adaptation
and the wider gate173 audit. Preserve four owner fixture edits, original SQLite
evidence, existing ownership rules and explicit clone policy. Use bounded TDD
checks and explicit Git paths. Do not mark gate173 or the full goal complete yet.
