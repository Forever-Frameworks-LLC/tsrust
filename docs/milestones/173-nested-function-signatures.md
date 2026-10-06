# Gate 173: named declaration signatures and inferred results

Status: in progress, not accepted. Gates170–172 are accepted. The full nested
function plan requires this stage and the eventual feature-family audit.

## Result inference checkpoint

The original nested_declarations_infer_nonrecursive_result_when_omitted witness
now passes. Named nonrecursive functions can infer a uniform checked result type,
including scalar, generic, captured and array values. Named generic instances keep
the inference flag. Unknown results are not treated as void constraints when an
outer generic call gathers argument hints; the referenced body is checked before
those constraints use its result. No source call is evaluated during this preparation.

return_type_inference.rs isolates checked return collection, detection of value
returns and preparation of inferred named hints. Arrows and declarations share
return collection. Bodies without value returns retain void, including recursive
void procedures. Value-returning recursive inference reports result-inference with
explicit annotation advice. Explicit void annotations keep their existing checks;
missing returning paths and disagreeing return types remain errors under the shared
inference contract. Main remains a void entry.

The basic fixture asserts inferred scalar, generic and captured returns and array
ownership without changing the original red witness. The independent Node fixture
passes. Compiler controls verify void, recursive boundaries, mixed-return errors,
missing-return errors and explicit void. The editor preserves inferred call-value
types and source signatures. Full function values and inferred callable factories
still require their own direct acceptance evidence; this checkpoint does not claim
the entire gate.

The nested_function_results workload is appended at manifest end. It retains six
observations from scalar doubling, a generic identity, a per-iteration captured
function, an inferred array return and input length. Matched TypeScript, independent
oracle, source mutation guards, registration, native ABI and run/profile/report
hooks are complete. implementationAccepted remains false for the full gate;
measurement remains deferred. No measurement protocols or sample counts change.

## Remaining scope

Implement function values of generic functions and capturing declarations with
optional/default parameters. Preserve defaults/rest arity, invocation contracts,
ownership and original symbols across specialization. Verify parameter patterns,
abstract callbacks, imported nominal types, catch/finally scopes, deep nesting and
precise fixed-array caller storage against direct evidence. Audit inferred callable
factory results and type alternatives rather than claiming broad inference from
scalar witnesses. Named function expressions remain a separate feature.

After this scope is complete, audit the full plan and execute the meaningful named
family regressions and performance milestone. Preserve completed evidence and
resume only failed/unrun groups; gate numbers alone do not trigger heavy work.

## Durable checkpoint evidence

Canonical untimed result: benchmarks/runtime/results/2026-10-06T20-40-50-601Z-verify.json
in benchmarks/records.sqlite. All 113 compiler source fingerprints, oracle inputs
and matched source hashes are current. The record contains five correctness cases
and no startup, compilation or execution timing samples.

All 18 original checkpoint logs/results were imported and read back
byte-for-byte before removing loose copies. This includes the original red, build
errors and stale generic-hint failures, followed by verified repairs. Final checks
are focused37, result2 plus canaries63, editor279 plus stronger signature1, exact
native/Node fixture, registration1 and five matched native/Node cases.
