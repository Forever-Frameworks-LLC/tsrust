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
rest parameters. Optional/default captured values now have a verified checkpoint. Preserve defaults/rest arity, invocation contracts,
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

## Captured optional/default value checkpoint

local_function_values now retains parameter omission in generated wrappers.
Defaults become optional supplied slots in the wrapper and are evaluated only by
the original function. Forwarding undefined preserves omission and avoids both
duplicated side effects and transfer of the same owned capture twice. Supplied
arguments bypass defaults. Optional values, dependent defaults using an earlier
parameter, reusable borrowed defaults, consuming defaults, array callback index
arguments, failure propagation, arity and type errors have focused native checks.

A stale capture test that rejected defaulted values is replaced by positive native
array assertions. Generic function values remain a genuine missing feature. No
ownership restriction is relaxed; a borrowed closure releases its loan at its
existing scope boundary, and consuming defaults retain once invocation.

The editor witness exposed a projection collision: hidden wrapper calls have the
same source span as their function value. Their scalar result could overwrite
the callable type and remove signature help. The projection filter now identifies
actual wrapper closure spans and excludes only their hidden call result. Numeric
coercion, object spread and in-operator projections remain covered by the complete
editor run. Optional/default signature help is verified.

The new nested_function_defaults workload is appended at manifest end. Eight
observations verify omitted/undefined/supplied calls, exact default side effects,
optional argument fallback and consuming capture defaults. Independent oracle,
matched sources, mutation guards, registration, ABI and all execution/profile/report
hooks are present. Full gate173 implementation acceptance and measurement remain
pending. Existing result registration now finds its own row rather than requiring
it to stay the final manifest row after future appends.

A returned-value factory witness also passes: two independently owned defaulted
callbacks preserve separate scalar state, accepting omission, supplied values and
undefined through the explicit optional callable result contract. Inferred callable
factory results still require the remaining gate173 audit.

## Durable checkpoint evidence

Canonical final untimed record: benchmarks/runtime/results/2026-10-06T21-02-40-677Z-verify.json
in benchmarks/records.sqlite. All 113 compiler source fingerprints, all oracle inputs
and matched source hashes match this checkpoint. It records five correctness inputs
and no timing samples. The earlier passed record is retained separately; its source
fingerprint predates the narrowed projection repair.

All 26 original logs/results and 3 actual scratch test inputs
were imported and verified byte-for-byte before removing loose copies. This includes
original unsupported/duplicate-transfer witnesses, loan and assertion drafts, the
broad projection regressions and their repaired checks. The original scratch inputs
are gate-source records captured-default-value.tsr, captured-owned-default.tsr and
captured-default-reference.ts. They are inputs used by checks, not repository snapshots.

## Captured rest value checkpoint

Captured rest declarations now lower to rest arrow wrappers that spread the final
pack into the original function. Fixed defaults still forward omission to the
original declaration. Spread signature lookup now prepares named capture context
before checking the body, matching direct-call checking. A direct spread control
and stored-value witnesses cover this boundary. Hidden spread-wrapper call results
are excluded from source callable type facts using the same precise wrapper-span
recognition as default wrappers.

Focused native witnesses retain empty, ordered and spread packs, dependent fixed
defaults, compatible fixed callback adaptation, independent returned factory state,
owned strings, once captures and invalid element types. A heterogeneous array
callback tuple remains a type error for int[] rest. Matched nested_function_rest
support is appended at manifest end, with seven independent observations, source
mutation guards, registration, native ABI and all oracle/run/profile/report hooks.
Gate 173 is still open for generic values and the final inference/context audit.

Rest checkpoint evidence and the retained accidental timing record are documented in
[the rest handoff](../handoff-2026-10-06-gate173-rest-values.md).

Generic function value requirements now have four red native witnesses and passing
strict TypeScript/Node references. See [the generic-value implementation plan](../generic-function-values-plan.md). No generic-value implementation is accepted yet.

## Contextual construction checkpoint

Concrete generic values now retain the original template call in a checked wrapper.
Generic-dependent parameter annotations receive concrete callable context, while
non-generic annotations and stated access remain intact. The original solver checks
parameter/result relations, including result-only inference. Capturing declarations
are wrapped during lexical resolution before escape analysis; capture-free values
are constructed when their context is checked. A dedicated generic_function_values
module owns parameter dependency detection and contextual value dispatch.

The original fixed-context witness now passes, along with owning string/array,
mutable array argument, empty generic array result, explicit returned factory,
array callback and invalid type-relation controls. Two draft access annotations
were invalid under existing rules: string return requires Owned<string>, while
scalar removal borrows its array with Mutable<int[]>. Their corrected controls
retain transfer/mutation observations. Unannotated polymorphism and inferred generic
factory environments remain genuine red witnesses; this is an interim checkpoint,
not full generic-value or gate173 acceptance.

Contextual checkpoint evidence: [current handoff](../handoff-2026-10-06-gate173-generic-contexts.md). Full generic values and gate173 are still incomplete.

## Solved source contract checkpoint

The dedicated generic_value_signatures module solves the original declaration
without allocating instances or environments. It returns substituted parameter
and result annotations with source omission/rest metadata. Capture-free contextual
values use this contract before forming their checked wrapper. Required parameters
and same-type/result constraints retain normal checking; ordinary void is not
a wildcard. Unannotated values and generic factory environments remain open.

A default-forwarding defect was found and repaired: generic argument hints remove
only undefined from omission-admitting slots. A forwarded int | undefined selects
the declared default when absent; null remains a supplied type constraint. Solver
controls cover conflicting types, missing arguments, result-only inference, array
patterns, defaults/rest and no instance allocation. The paired executable
context-default-rest fixture checks omitted/undefined defaults and a generic rest
pack under a concrete callable context.

## Capture-free deferred invocation checkpoint

Capture-free generic declarations can now be stored without a concrete callable
context. Explicit source metadata distinguishes these origins from ordinary
closure contracts. Each call prepares the original generic template, checks its
concrete parameter access and ownership, and reuses a bounded method keyed by
checked argument types. The empty origin retains no captures or unresolved data
bindings. Its prototype is excluded from result hints and editor type projections;
ordinary void checking remains strict.

Native witnesses cover scalar/string/array types, repeated method reuse, borrowed
arrays, transferred owners, defaults evaluated once, rest packs and caught failures.
Invalid same-type constraints, arity, result types and reused owners are rejected.
A structural check retains one empty origin with three methods for three argument
types; repeated int calls reuse their method. The paired strict TypeScript/Node
fixture checks the same output and values. Actual call results retain concrete
editor types and source navigation. Generic alias parameter help, result-only
context propagation and polymorphic callback/storage adaptation remain pending.

The generic_function_deferred workload is appended at manifest end with an
independent seven-value oracle, matched native/TypeScript sources, fingerprints,
mutation guards, registration, existing array ABI/driver support and all
oracle/run/profile/report hooks. Verification is untimed. Captured polymorphic
values and inferred generic factory environments remain red; full generic values
and gate173 are not accepted. Continue that scope rather than treating this
capture-free subset as the final deliverable.

## Captured deferred invocation checkpoint

Captured generic values now retain typed lexical binding identities before solving
any invocation types. Capture access converges by original declaration and binding
identity across concrete instances. The public single-file checker requests this
same finite inference instead of exposing a provisional capture layout.

Concrete method bindings remain distinct from stable environment fields. Each
method retains its own receiver access and capture-slot mapping. Native witnesses
verify borrowed shared state across data types, independent inferred factories,
array patterns, defaults and rest packs. Ownership controls reject owner reuse and
live mutable-capture conflicts. Five owners drop exactly once at O0/O3 across
multiple types, caught failures, consuming captures and recursive groups.
Concrete call results and original declaration navigation have editor witnesses.

The five original captured witnesses now pass. Full generic values and gate173
remain open: result-only context through aliases, polymorphic callback/storage
adaptation, complete alias IntelliSense and the wider context/inference audit still
require implementation and evidence. The generic_function_captures workload is
appended at manifest end with a matched TypeScript source, independent nine-value
oracle, fingerprints, guards, registration and all runtime hooks. Verification is
untimed; implementationAccepted and measured remain false.

## Generic alias result-context checkpoint

Expected result types now pass through named/bound callable dispatch into deferred
generic preparation. The original solver checks both argument and result relations.
Method cache identity includes the concrete result as well as argument types, so
zero-argument int[] and string[] methods remain distinct while repeated int[] calls
reuse their method. Captured environments remain shared across those methods.

The native and strict TypeScript fixture checks distinct owning array results and
exact captured counter output. Invalid result/argument constraints and missing
arguments remain rejected. A cache witness checks two methods after three calls;
editor controls retain result array types and source navigation. The matched
generic_function_results workload is appended at manifest end with an independent
six-value oracle, complete pair/guards/registration/ABI hooks and untimed verification
support. Full generic alias help, callback/storage adaptation and the wider gate173
audit remain open. No completed gate or family measurement is claimed.
# Local generic alias editor checkpoint

Deferred origin metadata now links local value bindings to their original generic
declaration. Hover and completion retain the source type relation and documentation;
parameter help uses each concrete checked invocation, including default omission
and rest parameters. Four editor controls and the full editor suite pass. This
does not accept gate173. Cross-module alias contracts, access presentation and
callback/storage adaptation remain pending.
