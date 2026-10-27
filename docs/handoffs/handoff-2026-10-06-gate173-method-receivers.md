# Handoff: method-specific native receivers

Read AGENTS.md, docs/development-workflow.md and plans/generic-function-values-plan.md.
Previous checkpoint: 80b6d9c. Gate173, captured polymorphic functions and the full
language objective remain open. This checkpoint prepares native representation;
it does not claim the five captured witnesses now pass.

CheckedClosureMethod has invocation: Mode. Native closure_method signatures,
captured-field preparation and materialized_method_call receivers use that method
mode. Existing closure_methods replay initializes it from the original uniform
closure invocation. Capture-free deferred methods initialize Read. The global
environment summary remains available for ordinary callable paths.

A backend test starts from a checked capture-free generic origin, then assigns
three distinct method receiver modes. Generated Rust borrows for read, borrows
exclusively for mutate and moves for consume; native output is exactly
1, true and text on separate lines. This proves emission and invocation agree.
It does not prove captured field ownership or source-level rejection after a
consuming specialization. Six existing higher-order tests pass, including O0/O3
factory, access, disposal and recursion controls. Whole editor crate: 284 pass.
No parser/analyzer dispatch changes occurred; no new depth-canary run is claimed.

Five untimed native/Node-after-tsc cases pass for generic_function_deferred.
Canonical verify record: benchmarks/runtime/results/2026-10-06T22-34-57-044Z-verify.json.
All 117 compiler and 258 oracle input hashes match final source. execution is
empty and verifiedOnly is true. No new workload or measurement is claimed for
this backend boundary. Existing workload implementationAccepted remains false.

Nine original diagnostic/build/test logs and the verify record are stored and
read back byte-for-byte in benchmarks/records.sqlite. Log record paths are
tsrust-method-access-{red,green,repaired,higher-order,three-modes,
three-modes-repaired,build,editor,matched}.log. Original failures include missing
method metadata, a borrow-check failure in its initializer and a test Type path
repair. Loose evidence is removed after verified readback.

Five executable consumers are atomically refreshed and SHA-verified. All tool
handles are terminal at handoff. Obsolete test executables and the benchmark
native tree are pruned; library caches and five consumers remain. Four owner
fixture edits remain untouched and uncommitted. Hourly checks remain paused.

## Next implementation

Integrate named_captures.source_context into deferred captured origin construction.
Retain lexical identity and one environment without selecting a dummy generic type.
Then replay solved concrete signatures against that environment. Source-flow
checking must select the actual method access before applying callable ownership,
retain exact loans, reject reuse after consuming calls and keep read/mutate calls
reusable. Audit method-specific capture_reference and binding modes against the
new emitter boundary. Do not infer captured access from the capture-free test.

Keep all five original red captured witnesses. Continue inferred factory lifetime,
argument/result contexts, callback/storage adaptation, generic alias IntelliSense
and the wider gate173 audit from the earlier handoffs. This checkpoint is not a
substitute for that scope or its meaningful family regression/performance review.

## Continuation prompt

Continue Tsrust from docs/handoffs/handoff-2026-10-06-gate173-method-receivers.md.
Read AGENTS.md, docs/development-workflow.md and plans/generic-function-values-plan.md.
Implement captured deferred generic origins and concrete method replay with one
environment and sound method-specific ownership. Preserve five red witnesses and
four owner fixture edits. Use explicit Git paths, bounded checks, the warm target,
CARGO_INCREMENTAL=0 and exact SQLite evidence.
