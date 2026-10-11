# Handoff: generic alias result contexts

Read AGENTS.md, docs/development-workflow.md, plans/generic-function-values-plan.md and
plans/nested-function-declarations-plan.md. Red checkpoint: a3bf380. Captured implementation:
522986e. Gate173, full generic values and the broad language objective remain open.

Expected result types now pass from expression_call through named_call,
bound_named_call and closure_method_call into deferred_generic_call. Original
prepare_generic_call solves the combined argument/result constraints. The deferred
method key includes concrete result as well as checked argument types. Zero-argument
int[] and string[] methods remain distinct; repeated int[] calls reuse one method.
Existing environments, argument evaluation and bounded specialization limits remain
intact. Ordinary named-call unit callers pass None explicitly.

The original red native witness now passes: capture-free and captured aliases infer
result-only arrays, retain independent int/string native methods and print exactly
11, 12 and 3 x. Its strict TypeScript/Node reference is retained in SQLite from the
red checkpoint. Invalid result/argument relations and missing arguments are rejected.
The structural cache control proves two methods for three result-context calls.

Evidence: 17 generic value and two signature tests, all eight depth canaries
(63 tests), one cache unit, whole editor crate 286 tests and one registration test.
The focused/canary aggregate is 82. Editor controls retain int[]/string[] facts and
original source navigation. Full generic alias signature help remains pending.

The generic_function_results workload is appended at manifest end: native/TS pair,
independent six-value oracle, fingerprints, mutation guards, registration, existing
array ABI/driver and oracle/run/profile/report hooks. Five native/Node-after-tsc
cases pass untimed. Canonical record:
benchmarks/runtime/results/2026-10-06T23-10-42-773Z-verify.json.
All 117 compiler and 262 oracle input hashes match final source. execution is empty;
verifiedOnly is true. implementationAccepted and measured remain false.

Eight current diagnostic/build/check logs and the verify record are imported and
read back byte-for-byte in benchmarks/records.sqlite; loose copies are removed.
Log paths are tsrust-result-context-{implementation,canaries,cache,coverage,
registration,build,editor,matched}.log. Original red/reference evidence remains at
tsrust-result-context-red.log and tsrust-result-context-node.log in SQLite.
Five consumers are atomically refreshed and hash-verified. All handles terminal.
Obsolete test/native artifacts are pruned; libraries and five consumers retained.
Four owner fixture edits remain untouched and uncommitted. Hourly checks paused.

Next complete real generic alias hover/completion/signature contracts and actual
polymorphic callback/record/array storage adapters. Retain the prototype guard
until adaptation is implemented. Continue the full context audit, including
conditional/optional call paths, generic outer factories, imported nominal types,
patterns, callback-dependent capture access and catch/finally. Complete family
regression and performance review only when the scoped family is ready.

Continuation prompt: Continue Tsrust from this handoff. Read the workflow and both
function plans. Implement full generic alias IntelliSense and real storage/callback
adaptation, then the wider gate173 audit. Preserve owner fixtures, explicit clone
policy, SQLite evidence and the full original objective. Do not mark gate173 complete.
