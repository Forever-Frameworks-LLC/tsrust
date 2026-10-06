# Handoff: gate 172 accepted, gate 173 next

Read AGENTS.md, docs/development-workflow.md, docs/nested-function-declarations-plan.md
and docs/milestones/172-nested-function-owned-escape.md first. Inspect Git for the
accepted checkpoint hash. Gate171 is accepted at 1c6dd58. Gate172 is now accepted;
the full nested declaration plan and the active broader goal remain incomplete.

Preserve these four unrelated owner fixtures, uncommitted and untouched:
comma-contexts/guards.tsr, interfaces/shapes.tsr, map-for-each/values.tsr and
map-group-by/documented.tsr under crates/tsrust-compiler/tests/fixtures. Explicit
Git paths only. One agent works in this tree.

## Current architecture and accepted behavior

Named capture calls use hidden native arguments; named capture values use the arrow
wrapper in local_function_values. Existing ownership transfer and loans remain the
contract. Captured owners can be returned through once-callable factories. Retained
records, inferred or explicit callable arrays, independent instances, callback
failures and recursive retained array/record owners work. Instrumented debug and
optimized native probes verify five distinct owners drop exactly once.

The new inferred_callable_arrays module creates a shared owned/owned-once storage
family from the checked first closure element. Subsequent elements use existing
stored callable conversion. The emitter prepares exception contracts for every
emitted non-template definition, including unused throwing factories.

Dead inner functions no longer impose runtime captures on their enclosing function.
local_functions no longer adds an enclosing-to-declared-child runtime dependency;
actual calls and value uses still do. named_capture_context retains completed
contexts only for relevant lexical ancestors and prepares unused bodies from those
types after their owners finish. This resolves the early outer-call readiness error
without filtering hidden slots or changing the contiguous native ABI. Deferred
empty placeholders are cleared before real checking. Invalid unused bodies still
fail, while live call/value uses still require capture initialization. Existing
seventeen capture tests pass along with eleven owned-escape tests.

The matched owned-escape workload sits at the END of manifest. It records eight
observations per iteration and independently asserts dead-inner initialization
behavior. Matched sources, source mutation guards, independent oracle, fallible ABI,
registration, run/profile/report hooks and coverage are current. implementationAccepted
is true; measured is false. Do not call untimed verification a performance measurement.

## Gate 173 and the next failing witness

Preserve and implement nested_declarations_infer_nonrecursive_result_when_omitted
in crates/tsrust-compiler/tests/nested_function_declarations.rs. Its fixture is
nested-function-declarations/inferred-result.tsr. An omitted result currently means
void. Infer nonrecursive results and handle recursive inference consistently or
produce a precise inference-boundary diagnostic; do not weaken the original test.

Full signature/context coverage also requires function values of generic functions
and capturing functions with optional/default parameters, rest and parameter
patterns, abstract callbacks, imported nominal types, catch/finally scopes, deep
nesting, source identity and precise fixed-array storage for callers of capturing
functions. Audit existing coverage before treating any item as complete. Add real
red witnesses for missing behavior. Current arrow-wrapper lowering explicitly lacks
optional/default capturing values and generic function values; saying to use an
arrow is a temporary limitation, not acceptance of the full plan.

Keep new implementation seams small and independently testable. Capture context
completion must remain distinct from runtime capture readiness and borrowing.
Specializations must preserve typed declaration identity and lexical context; do
not introduce shared JavaScript cells, implicit clones or uninitialized native
views. Once gate173 scope is complete, perform the named-declaration family audit
and meaningful full regressions/performance milestone, preserving completed evidence.
Gate counts alone do not trigger heavy work.

## Evidence and environment

Focused capture/owned tests pass 28 total; local compiler library selection passes
19 tests; all eight depth canaries pass 63 tests; whole editor passes 278 tests.
The expanded Node reference and source registration pass. Five matched native and
Node-after-tsc cases pass untimed. Original failed attempts remain canonical evidence,
including the stale hover spacing assertion and stale benchmark source fingerprints.
Both were repaired; source hover was correct.

Use CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target and CARGO_INCREMENTAL=0,
offline locked Cargo. Node is /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node.
ReadOnly<T> is current borrowing syntax. Refresh five consumers atomically with a
.refresh-tmp copy then rename. No archives, binary backups, routine whole compiler
matrix or timing. Import exact original logs/results into benchmarks/records.sqlite,
read back and verify, then delete loose copies. Retain library caches and five
consumers while pruning obsolete executable test/native outputs. Hourly checks stay
paused. A future test invocation may relink pruned tests.

Canonical untimed record: benchmarks/runtime/results/2026-10-06T20-29-06-606Z-verify.json
in benchmarks/records.sqlite. All 112 compiler inputs, all oracle inputs and matched
source hashes match the accepted source. The manifest acceptance flag was promoted
after this verification; the final registration test verifies that metadata.
All 20 original logs/results from the final repair were imported, read back
byte-for-byte and removed from loose storage. Earlier red evidence remains intact.

Pruned 51 obsolete executable test outputs and native/example scratch trees.
Five refreshed consumers and library caches remain.
