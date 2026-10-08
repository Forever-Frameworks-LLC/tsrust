# Gate 173: named declaration signatures and inferred results

Status: accepted for its stated scope: omitted result types, and function values of
generic functions and of functions with optional, default and rest parameters,
including functions that use variables of the enclosing function. The sections below
are the chronological record; the acceptance section at the end is current. Work on
callable replacement that grew out of this gate continues separately and is listed
there.

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

## Local generic alias editor checkpoint

Deferred origin metadata now links local value bindings to their original generic
declaration. Hover and completion retain the source type relation and documentation;
parameter help uses each concrete checked invocation, including default omission
and rest parameters. Four editor controls and the full editor suite pass; an
additional control verifies unused-alias documentation. This
does not accept gate173. Cross-module alias contracts, access presentation and
callback/storage adaptation remain pending.

## Project generic alias editor checkpoint

Imported generic aliases and captured factory results retain source signatures
and documentation across module boundaries. Concrete help preserves nominal
aliases, unsaved parameter-name edits, default omission and checked ownership
access. Deferred prototypes are excluded from project callable types. Mixed
source signatures and conditional/optional alias contexts remain under audit;
actual callback/storage adaptation and full gate173 acceptance remain open.

## Fixed nominal generic alias headers

Original template annotation uses now retain nominal declaration identities.
Project hover and completion display visible imported aliases while preserving
labels, literal defaults and generic placeholders. Applied and private nominal
controls pass; conditional/optional contexts and actual callable adapters remain
open. No gate173 or feature-family acceptance is claimed.

## Existing generic environment adapters

Explicit concrete contracts, array transform callbacks and typed record/array
storage now adapt an existing generic value through a checked wrapper. Ownership,
exclusive borrowing, consuming reuse and exact O0/O3 disposal controls pass. The
original environment and state remain intact. Other callback and nullable contexts,
default/rest/pattern audits and adapter-specific benchmark support remain pending.
This source checkpoint does not accept gate173.

## Nominal adapter results and matched benchmark support

Adapter construction now materializes substituted nominal parameter and result
applications before checking its wrapper signature. The retained Receipt<T>
witness preserves one counter across a borrowed integer adapter and a direct
string call. Seven adapter tests, 17 existing generic value tests, all eight depth
canaries (63 tests), and the whole editor crate (298 tests) pass.

The generic_value_adapters pair observes seventeen values per scenario across
borrowed/owned views, map callbacks and separate record/array environments.
Independent result oracle, fingerprint and semantic guards, registration, ABI,
run/profile/report hooks are complete. Five native/Node-after-tsc correctness cases
pass untimed. Exact evidence is stored in SQLite under
benchmarks/runtime/results/2026-10-06T23-58-21-955Z-verify.json. Original failures
and repaired checks are retained; loose copies were removed after exact readback.
Five consumers were atomically refreshed and hash verified. Full adapter and
Gate173 acceptance remain open. No performance measurement is claimed.

## Callback adapter integration checkpoint

Reduction and sort now construct real adapters for existing generic environments.
Reduction no longer treats an empty deferred prototype result as an accumulator
constraint. Search uses the existing shared transform path. Nine adapter tests,
63 depth canaries, 13 existing reduction/sort/predicate tests and 298 editor tests
pass. Strict TypeScript references reproduce reduction and search outputs. Full
context audit and matched benchmark expansion remain pending. This is a source
checkpoint, not full adapter acceptance or performance evidence.

## Matched callback adapter coverage

The adapter workload now observes twenty-six values. Added reduction/reduceRight,
stateful search and sort/toSorted scenarios invoke the original generic environment
again after traversal. The independent oracle detects reconstructed state. Sort
uses stable equal comparisons and a retained state marker, avoiding assumptions
about engine-specific comparator invocation counts. Semantic mutation controls
reject missing capture updates and replacing retained invocations with new factories.
Five native/Node-after-tsc cases pass untimed. Exact verification is retained in
SQLite at benchmarks/runtime/results/2026-10-07T00-02-45-912Z-verify.json; all
oracle input fingerprints match current files. Registration passes. Loose records
are removed after exact readback. Wider adapter contexts and Gate173 remain open.

## Default and rest adapter forwarding checkpoint

A retained default/rest fixture first failed because rest spread planning used the
empty deferred prototype arity. Forwarding its formed pack directly then exposed
an internal RestPack array-literal assumption; both failures are retained. The
adapter now forwards its checked pack, and RestPack checks nonliteral array
expressions with the original access and context. The native witness passes,
matching strict TypeScript output. It verifies default evaluation once on omission
and explicit undefined, no default evaluation on overrides, empty/nonempty rest
packs and a subsequent different instantiation on the original environment.
Owned rest/disposal, parameter patterns, nullable contexts and matched benchmark
expansion remain open. This checkpoint does not accept the full adapter family.

## Owned rest and editor controls

Native O0/O3 drop instrumentation observes six owners exactly once across normal
return and caught failure: retained environments, fallback arguments and rest
arguments all dispose normally. Transfer controls reject reuse of the original
generic environment and of a rest element. Concrete adapter signature help keeps
optional/rest labels and three parameter slots; definition resolves to the original
alias. An exploratory record-literal reuse call first failed inference before the
ownership check, so the retained transfer control supplies an inferable scalar.
The original diagnostic remains evidence; unconstrained nominal literal inference
is not claimed. Matched default/rest benchmark expansion remains pending.

## Matched default/rest coverage

The retained adapter pair now observes thirty-seven values, including default
invocation effects, explicit undefined and override behavior, empty/nonempty rest
packs and later calls through the original generic environments. Independent
oracle and semantic mutation guards retain existing callback/storage coverage.
Five untimed native/Node-after-tsc cases pass; no timing samples exist. Canonical
SQLite verification is benchmarks/runtime/results/2026-10-07T00-08-56-395Z-verify.json.
All oracle hashes match current files. Registration and exact readback pass; loose
records are removed. Full adapter context and Gate173 acceptance remain open.

## Fresh direct factory staging

Fresh direct declaration calls now stage their generic environment once in an
ordinary checked lexical block, then construct the existing ownership-preserving
adapter. Traversal uses source parameter hints to solve a concrete contract before
staging. The retained factory-expression witness prints 10,11,12,20,21,22, matching
its original TypeScript reference. Native O0/O3 observes three retained record
owners disposed once across normal return, callback failure and factory failure.
Borrowed temporary escapes report closure-escape. No factory reconstruction,
implicit clone, empty-prototype wildcard or source argument replay was introduced.
Sixteen adapter controls pass after tightening the borrowed-escape diagnostic;
17 generic value tests and all eight depth canaries pass. Whole editor checks
pass 299 tests, followed by the added fresh-expression protocol witness. Original
failures and repairs are preserved. Wider expression and full Gate173 acceptance
remain open; matched verification is recorded in the current handoff.

## Fresh factory aliases and same-origin selection

Checked call-result hints now admit fresh environments returned through local
factory aliases and selected callees with the same known factory origin. Original
alias witness prints 10,11,20,21,22; selected-callee witness matches that output.
Eighteen adapter tests, 17 generic value tests and 63 depth canaries pass. Whole
editor passes 300 tests, followed by the added alias definition/result-help control.
Matched workload expands to fifty-one observations; five native/Node cases and
semantic mutation registration pass untimed. Canonical SQLite record:
benchmarks/runtime/results/2026-10-07T00-20-42-729Z-verify.json.
Generic factories, differing-origin selections and remaining contexts remain open.

## Generic factory partial parameter facts

Generic factory parameter help now consumes additive partial call facts when the
returned generic environment has no concrete display contract. Checked seed types,
parameter access and original declaration locations remain available. An unknown
result adds no result annotation; it never becomes void or an empty callable.
Conflicting specializations at one source span remain distinct and suppress
ambiguous editor overrides. Source and project projection tests cover string/int
variants and imported aliases with Unicode source offsets. Full Gate173 acceptance
and the remaining adapter contexts are still open.

Focused acceptance passes the retained factory argument-help witness, nine source
and project projection tests, one existing project nominal-identity control and
all 302 editor tests. Five consumers were refreshed atomically and SHA-256 checked.
The unchanged fifty-eight-observation workload passes five matched native and
Node-after-tsc cases untimed. Canonical SQLite record:
benchmarks/runtime/results/2026-10-07T00-34-37-057Z-verify.json.
Exact build, original test compile failure and repaired checks were imported and
read back before loose copies were removed. No runtime semantics, timing or full
Gate173 acceptance is claimed by this metadata repair.

## Selected factory environments

Selected fresh factory calls now adapt their branches before joining callable
origins. The predicate runs first; only the selected factory and its arguments run.
Each branch must satisfy the concrete signature, and an owned storage family keeps
the selected environment. Traversal accepts this storage through the existing
callback tuple and family error contract, including map, search, reduction,
flatMap and sorting. Immediate staging borrows end after factory evaluation;
returned owned environments still cannot retain borrowed factory parameters.
Nested selected callees and heterogeneous inferred result joins remain unaudited.
Full adapter and Gate173 acceptance remain open.

Twenty-four adapter controls pass across the retained bounded run and affected
follow-up checks. All eight depth canaries pass (63 tests); whole editor passes
303 tests on the final compiler source. The original selected-call failure and
subsequent capture-loan, traversal-effect and fixed-storage defects are retained
in SQLite. Selected native disposal observes three owners exactly once at O0/O3
across success, callback failure and factory failure. Borrowed factory data remains
rejected. Both strict TypeScript fixtures pass under Node. The matched workload
now has sixty-seven observations; registration mutations and five native/Node
after-tsc cases pass untimed. Canonical SQLite record:
benchmarks/runtime/results/2026-10-07T00-45-46-040Z-verify.json.
Five consumers were refreshed atomically and SHA-256 checked. No timing or full
Gate173 acceptance is claimed.

## Nested selected factory callees

Nested selected factory callees now lower through an iterative branch walk. Each
leaf must resolve to a deferred generic factory; the original expression remains
unchanged if that requirement fails. Predicate order and short-circuit selection
remain intact, and each chosen invocation evaluates its arguments once. Traversal
solves its concrete context from a factory leaf and checks every branch against
that contract. No recursive lowering frame or unchecked origin join is introduced.
The nested-selected-factories fixture observes four predicate evaluations, two
argument evaluations and independent selected environments. Heterogeneous result
joins and the remaining storage, inference and ownership context audit stay open.

Twenty-five adapter tests and all eight depth canaries (63 tests) pass on this
source. Whole editor passes 303 tests; its selected factory protocol control now
checks both the original and nested fixtures. Strict TypeScript and Node preserve
the original output and nested evaluation assertions. Matched workload expansion
has seventy-six observations; registration mutations and five native/Node-after-tsc
cases pass untimed. Canonical SQLite record:
benchmarks/runtime/results/2026-10-07T00-50-57-473Z-verify.json.
Original red and intermediate compile failures remain exact SQLite records. Five
consumers were refreshed atomically and SHA-256 verified. No timing or full Gate173
acceptance is claimed.

## Fresh aggregate storage and omitted factory results

Fresh record fields and typed callable arrays pass native acceptance directly,
including selected factory expressions and independent retained counters. No
compiler repair was needed for those aggregate destinations. Fresh traversal of a
generic function with an omitted result annotation did fail at its unresolved
prototype. Result discovery now checks the original generic body using supplied
type hints and its retained capture context before constructing the concrete
adapter. It does not execute arguments, defaults or factory environments. Explicit
result mismatches remain type errors. Nested callable results, parameter patterns,
nullable/access contexts and heterogeneous result joins still require their audit.

Twenty-seven adapter tests and all eight depth canaries (63 tests) pass, followed
by the added explicit-result rejection control. Whole editor passes 304 tests on
the final compiler source. Both strict TypeScript fixtures match native output.
The matched workload now has eighty-four observations; registration mutations and
five native/Node-after-tsc cases pass untimed. Original eighty-observation evidence
is retained separately. Canonical final SQLite record:
benchmarks/runtime/results/2026-10-07T00-58-44-644Z-verify.json.
Original red and repaired checks were imported with exact readback before loose
cleanup. Five consumers were refreshed atomically and SHA-256 verified. No timing
or full Gate173 acceptance is claimed.

## Nested callable reconstruction checkpoint

Fresh generic factory values now preserve nested callable input and result
contracts. Checked reconstruction retains invocation ownership, names, access,
omission and rest metadata; SharedRead remains shared through internal source
annotations and project facts. Exact round trips pass for all six invocation kinds.
Void callback results and optional/default/rest inputs execute correctly. Borrowed
callbacks remain usable by their owners. Single-use results cannot satisfy reusable
contracts. Concrete nested signature hints constrain generic inference without
executing arguments. Deferred invocation checks any further body specialization
selected by callable arguments before retaining its method.

All 34 adapter tests, four shared runtime tests and eight depth canary suites pass;
whole editor passes 305 tests. Matched native/Node-after-tsc verification passes five
cases with 88 observations. Canonical SQLite record:
benchmarks/runtime/results/2026-10-07T01-20-05-313Z-verify.json.
Original failed witnesses, repaired checks and metadata round-trip evidence are
retained exactly in SQLite. Five executable consumers were refreshed atomically
and hash verified. No timing was run. This accepts the nested callable contract
chunk; nullable contexts, heterogeneous joins and the wider Gate173 audit remain
open. No full Gate173 acceptance is claimed.

## Nullable contexts and heterogeneous factory results

Fresh generic factory values now acquire nullable owned callable contracts.
Present optional calls retain factory state; absent calls skip argument evaluation.
Selected factories returning T and T[] acquire an int | int[] result contract
without changing the inferred input payload. Argument constraints establish the
payload before contextual result compatibility is checked. Adapter result views
validate the materialized source result first, including invocation ownership.
A single-use callable result still cannot satisfy a reusable result contract.
Generic nullable constraints accept an existing payload member without inventing
a union for incompatible inputs.

All 37 adapter tests, the repaired bounded generic/default controls, two result-only
signature controls, eight depth suites and 307 editor tests pass. Both strict
TypeScript reference fixtures match native execution. Matched coverage appends five
observations for nullable presence/absence and heterogeneous results, preserving
previous observations. All five native/Node-after-tsc cases pass with 93 observations.
Fallible workload ABI registration accepts the checked Result signature and rejects
unknown error signatures. Canonical SQLite record:
benchmarks/runtime/results/2026-10-07T01-30-18-635Z-verify.json.
Original failures and final checks are stored with byte-exact readback. Five
consumers were refreshed atomically and hash verified. No timing was run. This
accepts the nullable/heterogeneous context chunk; full Gate173 and its remaining
ownership/context audit stay open.

## Generic callable payloads

Focused callable payload support is accepted separately from the remaining Gate173
context audit. Generic T can retain a complete callable contract across argument
inference, source specialization and returned values. Keys preserve invocation
ownership, argument access, omission and rest metadata. Equivalent bare/owned and
optional/defaulted contracts normalize without including parameter spelling or
source spans. Distinct invocation and required-position contracts remain distinct.
Checked borrowed closures retain their borrowed invocation kind.

Native witnesses retain one factory environment across int and string payload
signatures. Readonly and mutable borrowed payloads leave the owner's callable
available. Once payloads remain single-use. Argument access mismatches and repeated
once calls remain errors. Success and factory failure dispose exactly four original
captured records, with identity mask 30, at O0 and O3. No implicit cloning is added.
Editor hover, signature help and original factory navigation retain concrete
returned callable contracts.

Verification passes 45 adapter tests, 72 bounded generic/default/value controls in
total, all 63 depth canaries and all 308 editor tests. Matched support adds three
observations, for 96 per scenario, with guards for int/string payload results and
retained factory state across a third specialization. All five native/Node-after-tsc
cases pass untimed. Exact record
`benchmarks/runtime/results/2026-10-07T01-48-46-795Z-verify.json` is stored in SQLite.
Registration and metadata controls pass. Mutable, readonly and once reference
fixtures retain strict TypeScript counterparts; callable invocation references use
a TypeScript generic constraint where required by that checker.

No timing or complete compiler regression is claimed. The wider generic context
and alias/editor audit remains open, and full Gate173 acceptance is still pending.

## Explicit applications on retained generic values

Retained named generic values now accept explicit type applications, including
callable payloads. Explicit arguments are kept by enclosing function identity and
call span, so distinct enclosing specializations substitute their own type bindings.
Checked preparation resolves the original generic declaration and captured context,
materializes the explicit signature, and then uses ordinary argument, result and
ownership checking. It does not replace supplied arguments with inferred types or
reconstruct a factory environment. Nongeneric values, wrong type-argument counts,
conflicting arguments and reused owners remain errors.

Explicit and inferred invocations retain one captured environment across scalar,
string and callable payload types. Enclosing generic substitutions, result-only
arguments, defaults and rest packs have focused controls. Defaults execute once at
invocation; rest packs retain their normal ownership. Checked method identity also
includes generic application arguments, keeping unused explicit type arguments
separate while reusing equivalent applications. Existing cache controls pass.
Editor signature help and result hover remain concrete, and invocation navigation
points to the local alias declaration.

The original committed red witness now passes. Fifty adapter tests plus the added
consuming-capture control cover 51 unique adapter tests. Seventeen generic value
controls and all 63 final depth canaries pass. The whole editor crate is verified by
preserving passed suites, repairing an accidental existing-test assertion edit,
and resuming the unrun suites. The expanded matched workload contains 101
observations per scenario; all five native/Node-after-tsc cases pass untimed.
Exact record benchmarks/runtime/results/2026-10-07T01-56-31-516Z-verify.json and original
failed/repaired logs are retained byte-for-byte in SQLite. No timing or complete
compiler regression is claimed. Full Gate173 and the broader context audit remain
open, including field/optional generic applications and remaining parameter-pattern
and borrowed-result contexts.

## Optional and selected generic invocation

Focused optional/selected generic invocation now preserves the original deferred
environment instead of reading its empty prototype. Named receiver access and
selected-expression access share specialization and argument checking, while
selected receivers retain real ownership roots and selected_access checks. A fresh
receiver already checked during construction is not checked again; an owned factory
argument is transferred once. Conditional invocation forwards expected result context.

Nullable factory alternatives prepare original omitted results before conditional
hint joining, without executing runtime arguments or factories. Optional result-only
context remains complete until the optional checker gives payload context to its
present branch. Real void-union rejection remains intact. Dynamic absent calls skip
argument effects, and present calls retain captured state. Failures propagate without
rebuilding that state. Fresh consuming environments work; named optional selections
still cannot consume a borrowed owner. Type, arity and receiver-loan failures remain
checked even for absent invocation branches.

Editor initializer inference now prefers the complete checked expression type over
selected-method return metadata at the same span. Optional results retain int |
undefined, while signature help reports the concrete invoked method. The original
red witnesses and dynamic strict TypeScript counterpart pass. All 58 adapters pass
in the batch; one additional consuming-capture control passes separately, covering
59 unique tests. Seventeen generic-value controls, two signature controls and all
63 depth canaries pass. Whole editor passes 310 tests. Native O0/O3 disposal observes
three original records exactly once, with identity mask 14, across success/failure.

The matched pair observes 106 values per scenario, retaining the prior 101. Five
native/Node-after-tsc cases pass untimed with independent oracle, fingerprints and
semantic guards. Exact record
benchmarks/runtime/results/2026-10-07T02-10-49-964Z-verify.json is stored in SQLite.
No timing or complete compiler regression is claimed. Full Gate173 remains open;
field applications, parameter patterns, nested callable defaults/rest, alias contexts
and unused generic source-validation controls still require the wider audit.


## Owned parameter-pattern and callable-payload audit

Focused pattern adapters now retain their original function body and concrete
parameter/result contracts. Forwarding wrapper names no longer masquerade as
pattern initialization bodies. Explicit consuming array parameters extract elements
without implicit cloning, with independent offsets for nested holders. Record/array
mixtures, skipped positions, lazy missing/undefined defaults and explicit rest are
covered. Readonly patterns retain borrows, transferred sources remain unavailable,
and missing required positions raise range errors. Successful and failed calls
dispose three original records exactly once at O0/O3, with identity mask 14.

Call statements can discard a returned callable without requiring an artificial
binding. A separate O0/O3 control verifies the discarded environment disposes its
original captured record once. Callable payload optional/default/rest contracts
retain default effects and spread forwarding. Dedicated editor checks preserve
concrete pattern-adapter result hover and argument help. Strict TypeScript/Node
references match the ordinary successful pattern contracts, including undefined
default selection. Native range failures are separate ownership/error controls.

The matched pair retains the prior 106 observations and adds pattern values/counters
and callable payload results for 116 observations per scenario. Independent oracle,
source fingerprints and mutation guards pass. All five native/Node-after-tsc cases
pass untimed in benchmarks/runtime/results/2026-10-07T02-33-38-582Z-verify.json.
Sequential array extraction uses remove and can shift remaining elements; no measured
performance claim is made. Full Gate173 remains open for wider alias/editor,
borrowed-result escape and unused generic source-validation coverage.

Final focused acceptance passes 159 bounded compiler tests and all 311 editor tests.
Exact failed/repaired evidence and both matched records are imported and verified
byte-for-byte in SQLite. See handoff-2026-10-06-owned-pattern-adapters-accepted.md.


## Unused immutable alias call validation checkpoint

A dedicated generic_call_validation module checks known source callable cardinality
before generic specialization drains declarations. It retains generic parameter
count, required/default/optional/rest slots and original declaration spans without
executing bodies or defaults. Immutable alias chains use ordered lexical scope
ranges; function and closure parameters, block locals, catch and loop bindings
shadow declaration names. Known concrete callback annotations remain monomorphic.
Synthetic forwarding wrappers retain their original declaration contracts.

Wrong explicit type counts, explicit application on known monomorphic aliases and
wrong static value counts are rejected even when the surrounding generic function
is unused. Ordinary callback parameters and arrow aliases have the same checks.
Dynamic spread cardinality remains deferred rather than being treated as one value.
Unknown mutable targets and broader selected/field callable contracts remain pending;
this checkpoint does not complete unused source validation or Full Gate173.

The original three reds and fresh callback reds pass. Positive chains, block scopes,
default/rest and static/dynamic spread controls pass. A paired strict TypeScript/Node
fixture preserves unused template acceptance and a native std:testing assertion.
Editor diagnostics name invalid type arguments and related source declarations.
Borrowed-result escape controls reject consumption of ReadOnly inputs using the
precise parameter-access diagnostic. Existing runtime workload/oracle support is
retained: compile-time rejections do not receive artificial timing scenarios.

Initial known-contract checkpoint passes 152 bounded compiler tests, a paired fixture
separately and all 312 editor tests. Existing five matched native/Node cases pass
untimed in 2026-10-07T02-44-10-712Z-verify.json, stored exactly in SQLite. Full validation
remains open; see handoff-2026-10-06-unused-immutable-alias-validation.md.


## Closure callback and dynamic spread cardinality checkpoint

Source validation retains declared callback contracts for closure parameters,
including required/optional/default/rest slots. An iterative cardinality helper
counts known positions through nested literal spreads and separately records unknown
lengths. Unknown lengths cannot prove too few arguments, but cannot erase an already
provable excess. Exact nested empty spreads retain zero cardinality.

Fresh red controls reject excess closure callback arguments, known excess before a
dynamic spread and known excess around a nested dynamic spread. Positive controls
preserve unknown lengths that may fit, optional/rest closure callbacks and existing
paired TypeScript/native source contracts. Arity diagnostics state expected counts
and distinguish exact supplied counts from a known lower bound; editor ranges retain
source call/declaration locations. Mutable and selected/field target flow remain open.

This focused extension passes 155 bounded compiler tests and all 313 editor tests.
Existing five native/Node cases pass untimed in 2026-10-07T02-49-56-230Z-verify.json,
stored exactly in SQLite. Full validation remains open; see
handoff-2026-10-06-source-call-cardinality.md.


## Mutable source contracts and straight-line snapshots

The source validator checks common cardinality across mutable aliases and selected
expressions, retaining all alternative declaration locations. A separate linear
flow module snapshots initializers and replaces targets after definite direct
assignments. Saved aliases retain the callable contract at initialization; later
writes cannot hide earlier errors. The linear path requires a straight-line body
without closures, nested writes or expression assignments. Complex flow retains
a conservative whole-function candidate set and still needs structured analysis.

The original mutable/selected reds, direct selection, loop/closure/expression-write
controls, definite overwrite and saved snapshot reds pass. Bounded checks pass
82 adapter tests, 17 generic function value tests and all 63 depth canaries. All
315 editor tests pass, including alternative and current declaration diagnostics.
A formatted owner fixture exposed a brittle documentation substring check; the
check now compares canonical formatting while retaining catalog output checks.

The paired strict TypeScript/native fixture validates self-assignment followed by
owned transfer into a saved callable. Distinct generic callable environments still
cannot replace each other in concrete execution; source cardinality proofs do not
change that runtime limitation or imply assignment compatibility. Full branch/loop
flow, stored fields, lifetime/context auditing and Gate173 acceptance remain open.

Existing five matched native/Node cases pass untimed in
2026-10-07T03-11-48-332Z-verify.json, retained exactly in SQLite. See
handoff-2026-10-06-mutable-source-contracts-in-progress.md for continuation.


## Block and branch source flow checkpoint

A dedicated iterative source-flow module builds a graph for blocks, if statements
and normal or abrupt completion. Initializers snapshot the current contract; direct
assignments replace it. Branch-local calls use their own environment, and completing
paths join common cardinality with source declaration provenance. Returned or thrown
paths do not contaminate subsequent calls. Call facts are computed once for the graph
rather than reconstructing assignments separately for each supported call.

Three original reds now reject definite block replacement, a call after replacement
inside one branch, and replacement in both completing arms. Valid self-assignment,
shadowing and early-return controls pass. A separate native/strict TypeScript pair
asserts both branch outcomes with integers and owned strings. Editor diagnostics
retain the invalid branch call and its current declaration.

This is not complete flow acceptance. Differing joined cardinalities currently
defer rather than retaining a full alternative set. Loops, exceptional regions,
expression assignments and captured effects still use the conservative path. The
next checkpoint must retain distinct candidate contracts, implement loop fixed
points and explicit exits, then cover exceptional and expression flow. Stored fields,
concrete environment replacement and the broader lifetime/context audit remain open.

This extension passes 165 bounded compiler tests and aggregate whole-editor
coverage of 316 tests after the focused Set documentation repair. Existing five
matched cases pass untimed in 2026-10-07T03-19-54-872Z-verify.json, retained exactly
in SQLite. See handoff-2026-10-06-branch-call-contracts.md.


## Distinct branch contract sets

Branch joins and initializer snapshots retain distinct generic and value argument
cardinalities. Calls in unused generic declarations are checked against every known
reachable contract instead of becoming unknown when shapes differ. Required,
optional/default and rest parameters keep separate contracts. Definite assignments
replace the candidate set, while saved aliases preserve their initializer set.
Unknown targets still defer to concrete validation; this does not manufacture an
origin or relax callable environment assignment rules.

The isolated generic_call_contract_sets module resolves finite candidate sets and
canonicalizes equal signatures with all original declaration locations. Canonical
ordering prevents propagation from depending on traversal order. Diagnostics name
the failing signature; equal-signature alternatives retain all related locations.

Three rejection witnesses cover too few values, too many values and explicit generic
argument counts across differing branches. Positive witnesses retain valid optional
and rest calls, saved snapshots and narrowing after a definite overwrite. The editor
witness identifies the invalid call and the reachable failing declaration. Native
and strict TypeScript controls retain the existing branch fixture; the registered
generic_value_adapters family is verified untimed. No artificial runtime workload
is introduced for a source-only rejection repair. Full Gate173 remains open for
loop/exception flow, expression ordering, captured writes, stored fields, concrete
environment replacement and the remaining lifetime/context audit.


## Exception and cleanup source flow

Structured source flow now retains callable contracts through try, catch and
finally. A failed checked expression enters its handler with the state before its
destination assignment commits. Return, throw, break and continue execute each
intervening cleanup, then resume their own destination. Abrupt completion inside
finally replaces the suspended completion. Cleanup entries are cached by their
continuation context and call facts join by source span. Unknown targets and
expression/capture writes retain the existing conservative validation path.

Eleven rejection cases cover explicit and implicit failure, normal and abrupt
cleanup, loop continuation and nested cleanup. Six precision controls retain
snapshots, definite replacements, shadowing and completion overrides. A 24-level
cleanup witness checks iterative handling. Formatted native and strict TypeScript
controls assert integer and owned-string results plus exact cleanup order. The
editor diagnostic identifies the invalid call and its reachable declaration.

Bounded acceptance totals 201 unique compiler tests, including all eight depth
canaries, formatter tests and the fixture corpus. Whole editor passes 324 tests.
The existing five generic adapter scenarios pass native/Node-after-tsc untimed in
2026-10-07T04-28-17-908Z-verify.json, stored exactly in SQLite. Source-only rejection
checks reuse this registered family; no artificial runtime workload or timing is
claimed. Wider checks also exposed and repaired nested generic closer token
preservation in the formatter, retaining exact comparison and shift operators.

Full Gate173 remains open for expression evaluation order, capture writes, stored
fields, concrete environment replacement and the remaining lifetime/context audit.
Supported statement dispatch remains explicit; unsupported regions still use the
conservative path and are not claimed as complete source-flow parity.


## Expression order and named assignment invocation checkpoint

The iterative expression evaluator now passes all thirteen rejection cases and
nine precision controls. Calls retain their callee snapshot before evaluating
arguments; selected branches, assignment results and per-step failure environments
retain source contracts. Explicit generic arguments on grouped callable expression
forms are parsed as calls. Unsupported optional expression regions and captured
writes remain on the conservative path for the continuing audit.

The previously failing native witness now passes. A named callable assignment used
as callee stages an ordinary destination replacement followed by invocation of
that destination. This reuses existing ownership, loan, consumption and disposal
checks. It does not clone an environment or introduce shared aliases. Captured
state survives self-assignment and call failure. Fresh factory replacement retains
its own state, and O0/O3 disposal observes each original record exactly once. Once
consumption and argument-side receiver replacement violations are rejected. The
helper is isolated in assignment_callee.rs. Owned assignment-result escape and
record/array assignment callees still need their storage/lifetime audit.

Checked compiler call facts now supply concrete signature help for expression
callees. The original missing-help witness is preserved, then repaired; all 184
intelligence tests pass. Aggregate whole-editor acceptance is 326 tests, retaining
the previously completed unaffected groups. Bounded compiler acceptance is 207
unique tests, including eight depth canaries, formatter checks and fixture corpus.
The earlier filter-only canary command is retained but is not counted as depth
coverage; the unfiltered eight suites pass 63 tests.

The existing unmeasured generic adapter workload appends four assignment-result
observations for 120 per scenario. Matched Tsrust/TypeScript sources, independent
oracle, source guards, mutation rejection and exact fingerprints are updated.
Runtime/oracle/profile/report hooks already route this registered family. Operation
and sample counts are unchanged. Five native/Node-after-tsc cases pass untimed in
2026-10-07T04-54-09-224Z-verify.json, retained in SQLite. Earlier 116-observation
records remain immutable. No timing or complete compiler regression is claimed.

Full Gate173 remains open for capture effects, optional expression regions, stored
callable fields, concrete environment replacement and remaining lifetime/context
coverage. This checkpoint does not claim complete callable or assignment parity.

## Lexical generic transport continuation

Nested declarations now retain enclosing generic type bindings through explicit
and inferred specialization. The isolated lexical scope and instance modules keep
explicit arity separate from inherited variables, preserve nearest shadowing and
install recursive peer identities before body traversal. Concrete applications
retain the original declaration and a separate concrete lexical owner for capture
preparation. Linked modules preserve inherited type names and validate explicit
applications in their innermost lexical function rather than every enclosing span.

Standalone and imported mixed integer/string instances execute. Escaped ordinary
and generic nested function values retain independent captures. O0/O3 checks observe
three original record owners exactly once, including a failing consuming call.
Affected generic suites and eight unfiltered canaries pass 168 tests; editor and
matched acceptance evidence is recorded in the current handoff. The existing matched
family now has 124 observations, adding lexical echo and escaped capture controls.
Operation and sample counts remain unchanged; this is untimed support.

Captured invocation writes still have eight red witnesses. Optional expression
regions, stored callable fields, concrete environment replacement and the broader
lifetime/context audit remain required. This does not complete Gate173.

## Acceptance (2026-10-07)

### Scope accepted

- **Omitted result types.** A nonrecursive named function, nested or top level, can
  omit its result annotation. Scalar, string, array, captured and generic results are
  inferred from the checked return expressions; all return paths must agree. A
  recursive value-returning function reports `result-inference` and needs an explicit
  result type. A body without value returns is void.
- **Optional, default and rest function values.** A nested function with such
  parameters that uses variables of its enclosing function can be assigned, passed
  and returned. Omission forwards to the declaration, a default is evaluated once per
  call, and an array callback supplies the index as the second argument.
- **Generic function values.** A generic function, also one that uses variables of
  the enclosing function, is a value. One environment serves every type it is called
  with; a fixed callable annotation selects one specialization; factories return
  generic values with independent state.

The fixture `nested-function-captures/function-value-signatures.tsr` exercises the
three parts together, natively and against the pinned Node reference.

### Repaired at acceptance

- The unused `workload` function of `generic_value_adapters` was no longer emitted
  since commit 4cdf1e0, so its matched verification could not run. Callback error
  contracts were grounded only by calls reachable from `main`; an unreferenced
  function that calls a function with a scoped `Once` parameter therefore had an
  unresolved contract and was deferred. Functions without callback parameters have
  one native signature and now count as callers.
- `nested_function_captures::capture_borrows_follow_argument_and_view_rules` still
  expected an `unsupported` diagnostic for a stored generic function value.
- The guide no longer says that generic and rest functions cannot be values.

### Limits

- Recursive value-returning functions need an explicit result type.
- An array callback lends its elements for reading; a generic function that returns
  its argument cannot be passed to `map` over strings or other owned values. This is
  the ordinary callback access rule, not a limit of generic values.
- Selection and replacement across distinct polymorphic origins remain open, with two
  witnesses kept red:
  `generic_call_capture_effects::literal_guard_native_callable_replacement_has_common_carrier`
  and the unit test `owned_factory_updates_are_point_sensitive_and_instances_stay_separate`.
  They belong to the callable replacement work that follows this gate.

### Evidence

- Gate suites (`nested_function_captures`, `nested_function_declarations`,
  `nested_function_owned_escape`, `named_result_inference`,
  `captured_function_defaults`, `captured_function_rest`, `generic_function_values`,
  `generic_value_signatures`, `generic_value_adapters`, `declared_function_values`,
  `higher_order_function_values`): 167 pass, 0 fail.
- Whole compiler crate: 2178 pass; the two witnesses above are the only failures.
  Whole editor crate: 339 of 339.
- Matched workloads `nested_function_results`, `nested_function_defaults`,
  `nested_function_rest`, `generic_function_contexts`, `generic_function_deferred`,
  `generic_function_captures`, `generic_function_results` and
  `generic_value_adapters`: native and Node-after-tsc agree with their independent
  oracles on five untimed cases each. Their acceptance flags are promoted. No timing.
- Editor: 15 focused witnesses for inferred results, default and rest signature help
  and generic values, inside the whole editor crate.
- Logs and results are in SQLite, read back byte for byte.
