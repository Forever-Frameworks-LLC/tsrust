Superseded by [the captured-default value handoff](handoff-2026-10-06-gate173-default-values.md).

# Handoff: named result inference checkpoint

Continue Tsrust from this checkpoint. Read AGENTS.md, docs/development-workflow.md,
docs/plans/nested-function-declarations-plan.md and
milestones/173-nested-function-signatures.md first. Inspect Git for the current
checkpoint hash. Gate172 is accepted at d74f34d. Gate173 and the broader goal remain
incomplete. This checkpoint implements focused nonrecursive named result inference.

Preserve four uncommitted owner fixtures under crates/tsrust-compiler/tests/fixtures:
comma-contexts/guards.tsr, interfaces/shapes.tsr, map-for-each/values.tsr and
map-group-by/documented.tsr. Explicit Git paths only. One agent in this tree.

## Implemented behavior and module seams

The original nested_declarations_infer_nonrecursive_result_when_omitted red witness
now passes unchanged. Nonrecursive scalar, generic, captured and array returns infer
from checked expressions. Context.inferred_body_result replaces the former arrow-only
flag. Signature.inferred_result retains omitted result intent through ordinary and
named generic instantiation. Main stays void. A body without value returns stays
void and remains usable recursively. Value-returning recursive inference reports
result-inference with explicit result annotation advice. Explicit void, inconsistent
return types and missing returning paths retain checks.

return_type_inference.rs shares checked return collection with arrows, detects value
returns and finishes named result hints without executing calls. Generic argument
hinting treats unchecked omitted results as unknown rather than void. It checks
referenced inferred bodies during argument preparation before outer generic
constraints use their result. This matters for strictEqual(twice(3),6), captured
length calls and inferred generic identity calls. The original nested-call witness
was retained throughout the repair; replacing it with an intermediate variable
would have hidden the defect.

The named-result-inference/basic.tsr fixture uses std:testing and has a passing
independent Node reference. The new nested_function_results workload is appended
at END of manifest. Its six observations cover doubling, generic identity, a
per-iteration captured function, inferred array return and input length. Matched
sources, independent oracle, source mutation guards, registration, fallible-capable
ABI and oracle/run/profile/report hooks are complete. Its implementationAccepted
flag stays false until full gate173 acceptance. No timing is claimed.

## Remaining gate 173 work

Implement generic function values and capturing declaration values with optional
or default parameters. local_function_values::header currently rejects these
signatures instead of constructing a wrapper. Preserve defaults and rest arity,
argument access, invocation, ownership and source symbol identity; do not merely
suppress that diagnostic or make callers write an arrow. Evaluate each default
exactly once at the correct invocation boundary. Specializations must keep their
lexical captured contexts and original source identity.

Audit inferred callable factory results, parameter patterns, abstract callbacks,
imported nominal types, catch/finally scopes, deep nesting and precise fixed-array
caller storage. Verify each requirement directly; existing scalar inference does
not prove complete callable or type-alternative inference. Shared inference currently
requires a uniform return type, matching the arrow checker. Broad type-alternative
inference needs its own requirement review and native witnesses before a parity
claim. Named function expressions remain separate from declaration support.

Add genuine red tests before changes. Keep parser/analyzer dispatch thin; use
isolated helpers. Preserve all gate172 owned escape, exact disposal and dead-inner
capture behavior. Runtime call/value dependencies are distinct from completed
lexical context snapshots used to check unused bodies. Never reintroduce runtime
borrows solely to typecheck an unused declaration. No shared JavaScript cells or
implicit clones. Complete full gate173 and the plan audit before the meaningful
family regression/performance milestone; gate counts alone do not trigger it.

## Current checks and environment

Focused result/declaration/capture/owned tests pass 37. The final result tests plus
all eight depth canaries pass 65 (63 canaries). The whole editor passes 279; its
inferred hover/signature witness also passes stronger assertions in a focused
supplement. Exact basic fixture runs natively and on Node with no stdout. Source
registration passes. Five matched native and Node-after-tsc inputs pass untimed.

Use CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target, CARGO_INCREMENTAL=0,
offline locked Cargo. Node: /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node.
Current borrowing syntax is ReadOnly<T>. Five consumers are refreshed atomically
via .refresh-tmp then rename. Preserve libraries and consumers, prune obsolete
executable test/native scratch at handoff. Pruned tests will relink. No archives,
snapshots, binary backups, routine full compiler matrices or timing. SQLite is
canonical: import exact original logs/results, read back byte-for-byte and remove
loose copies. Hourly checks remain paused.

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

Pruned 52 obsolete executable test outputs and native/example scratch.
Five consumers and library caches remain; tests will relink.
