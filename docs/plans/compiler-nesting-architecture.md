# Compiler nesting architecture

Status: active, required before S12. Full nesting acceptance remains pending;
ordinary acyclic dependency measurements are recorded below. Baseline source is
`6fe878c2`; `f26712cf` tracks the
owner's later UI plans without changing compiler behavior.

## Acceptance boundary

Replace call-stack-dependent function and dependency traversal with explicit
worklists. Audit parser, type analysis, emission, cloning and disposal. Preserve
ownership inference, generic lexical scopes, point-sensitive effects, evaluation
order and diagnostic locations. Raising worker stacks or existing depth constants
is not acceptance. Runtime recursion and JSON depth budgets remain separate.

Require deep finite source structures, long ordinary call chains, recursive
ownership/type cases, native execution, normal worker stacks and compilation
scaling evidence. Finish affected sections before validation. Full compiler/LSP
regressions belong at the completed architecture milestone. Keep original failures
and measurements in SQLite and resume affected/unrun checks.

## Required optimization phase after nesting

The owner requires optimization of existing code before S12–S21 continuation.
Compare matched workloads against handwritten Rust timings and allocations, and
require at least comparable Node-after-tsc performance. Preserve semantics,
ownership and safety checks, independent result oracles, original measurement
protocols and baseline records. Report unmatched contracts and unresolved gaps
explicitly; implementation acceptance does not imply measured parity. Begin with
the handoff’s collection, recursive, pipeline and export hotspots after nesting
acceptance.

## Initial audit and section boundaries

- `analysis/mod.rs` shares one depth counter among ordinary functions, blocks,
  expressions and callable result discovery. Direct calls, inferred generic
  results and callable values can recursively request another function body.
  The 128-level guards remain until their recursive paths are replaced.
- Function completion also recursively checks uncalled capturing declarations.
  Lexical owner discovery recursively enters ancestor dependencies. Separate
  scheduling from body checking first, retaining depth-first completion order.
- `source_walk.rs` already walks expression and statement trees iteratively,
  but callback parameter/result annotation traversal was recursive. Give that
  traversal explicit continuations and retain first-error visitation order.
- Generic inter-call effect inference already has an explicit finite worklist
  under `generics/call_validation/branch_flow/invocation_worklist`. Preserve its
  context, environment identity and point-sensitive invalidation semantics.
- Parser/type syntax, type substitution, expression hints, block checking,
  error-effect annotation, emission and recursive syntax cloning/disposal still
  require conversion/audit. No successful annotation-only witness establishes
  end-to-end deep-source acceptance.

## First section in progress

Function scheduling now has its own analysis module. Completed owners enqueue
uncalled bodies in reverse order for a depth-first worklist. Lexical ancestor
resolution stores its continuations explicitly. Direct call dependencies inside
body checking remain recursive and are the next major section.

Callable annotation traversal uses an explicit worklist. Its isolated 16,384-level
witness uses the normal test worker stack and deliberately disposes its synthetic
tree iteratively; production tree disposal remains pending. Focused checks must
also preserve nested capture diagnostics, generic scopes, recursive inference,
callback contracts and generated execution. These are architecture changes with
no new built-in APIs; existing IntelliSense signatures remain unchanged.

S12–S21 remain pending. Preserve the S11 lifetime issue and original schema
performance records described in the acceptance handoff. Profile collection,
recursive, pipeline and export costs after this nesting priority.

## Initial section verification (2026-10-13)

Focused verification passes: two annotation worklist unit tests, 75 compiler
integration tests and 17 nested editor intelligence tests. The six compiler suites
are `control_flow`, `generic_default_scope`, `nested_function_captures`,
`nested_function_declarations`, `recursion` and `recursive_callable_results`.
They retain native ownership/evaluation-order checks and existing depth guards;
passing those guards does not satisfy the requested removal of nesting limits.
The new capture fixtures verify formatting/idempotence before checking.

Whole-project formatting changes zero files and zero benchmark fingerprints.
SQLite records `tsrust-nesting-annotation-worklist-final.log`,
`tsrust-nesting-function-scheduling.log`, `tsrust-nesting-editor.log`,
`tsrust-nesting-section-format.log` and `tsrust-nesting-section-evidence.json`
preserve outputs, source hashes and toolchain identity. Each record was restored
byte-exactly in memory before removing the loose copy; SQLite quick_check passes.
The original cold-build unit run and pre-edit formatting record are also retained.
No compilation scaling or runtime timing samples were collected. No public API
changed, so installed consumers remain on the accepted S11 build until the next
accepted public checkpoint. The shared Cargo target remains active for this goal.

## Ordinary dependency section in progress

Source function references are collected with explicit lexical scope frames.
Parameters, block bindings, closure parameters, loop variables and catch bindings
mask global function names. A worklist computes dependency finishing order and
recursive components without recursion. Components retain their first traversal
entry so existing recursive result seeds are not reordered. Capturing functions
wait for their defining typed context. Dynamic specializations continue through
ordinary checking; recursive component bodies still use the prior inference path.

The new normal-stack source witness checks, emits and disposes a 2,048-function
acyclic chain. Separate native witnesses cover 512-function inferred-return,
function-value, owned-string and imported chains, plus a 256-function throw chain
with argument evaluation order and typed catches. The previous 50-function/four-
branch combined-depth rejection now checks successfully. Deep syntax and large
recursive components remain pending; their guards are not raised or removed.

The first test run exposed two incorrect test expectations: the shared native
helper compares the entire contract map, not a selected subset. Explicit contract
assertions now check the intended entry after native execution. The original
failed run is preserved as `tsrust-nesting-dependencies-tests.log` in SQLite.

A live sample of the confirmed long-chain test identified repeated whole-stream
cloning in formatting annotation probes. The formatter now reuses one scratch
stream and journals packed-closer edits, restoring them even after failed probes.
A differential unit witness compares the original parser result and exact token
restoration. Sorted annotation span membership uses binary search. This repair
retains formatting's token-preservation check; the whole fixture corpus verifies
239 lexable fixtures, 84 standalone contracts and idempotence.
The original sample is `tsrust-nesting-dependencies-sample.log` in SQLite.

`benchmarks/compiler/function-dependencies` registers exact generated native and
TypeScript chains at 32, 128, 512 and 2,048 functions. The fixed protocol is two
warmups and eleven fresh-process samples for native checking, native emission and
Node/TypeScript checking. It retains exact source bytes/hashes, compiler identity,
commands and durations. This scope excludes recursive components, deep syntax,
backend compilation and runtime throughput. Registration is not measurement.

## Ordinary dependency focused evidence (2026-10-13)

Focused verification covers 142 compiler checks: the retained 11-test control-flow
run, 126 successful integration checks after the test-expectation repair, three
scope/probe unit tests, one imported-chain native check and one formatter corpus
check. The latter covers 239 fixtures and 84 standalone contracts. Fifteen editor
intelligence tests pass, including the new 512-function inferred hover/signature/
definition witness. These are affected checks, not complete compiler/LSP regressions.

The release build passes. All 12 scaling groups pass premeasurement validation
and complete two warmups plus eleven samples, totaling 132 measured fresh-process
durations. SQLite stores the original record and proof as
`tsrust-nesting-dependency-scaling.json` and
`tsrust-nesting-dependency-scaling-proof.json`; normalized rows use analytics kind
`compilation` and epoch `function-dependencies-v1`, not runtime throughput.
From 512 to 2,048 functions, median native check time grows about 3.10x and emission
about 3.16x for four times as many functions. Small counts are dominated by startup.
No prior comparable scheduler baseline exists. Recursive components, deep syntax
and backend compilation remain unmeasured.

Whole-project formatting changes zero files/fingerprints. Remaining recursive
checking inside components, capture-dependent/dynamically selected calls, parser,
block/expression analysis, types and syntax disposal are not accepted by this
section. Keep the full architecture priority active before S12.

## Consumer validation and record-context repair

Refreshing consumers from `3f7a38e9` exposed a regression in the existing
`fallible-record-callbacks/values.tsr` fixture. Both the installed CLI's project
check and the live editor reported `Expected int, found #RuntimeError.` at
21:89. The original failed refresh, installed hashes and CLI output are retained
in SQLite as `tsrust-nesting-consumer-refresh.json`,
`tsrust-nesting-consumer-build.log` and `tsrust-nesting-installed-check.log`.

Eager scheduling of record-parameter functions ran before their callers had
constructed the callback environments that determine field error contracts.
Such dependencies now retain their original checking point. This is a semantic
ordering constraint, not stackless acceptance for record-parameter call chains;
explicit suspension with the caller's context remains required. All four
existing fallible-record-callback tests pass with this repair. The scalar-chain
measurements retain their original source identity and are not relabeled as
measurements of the repair.

The affected repair batch passes 28 integration checks: four fallible record
callbacks, six function dependencies and eighteen schema pipelines. Whole-project
formatting changes zero files/fingerprints. SQLite retains the diagnostic probe
and repair outputs as `tsrust-nesting-callback-order-probe.log`,
`tsrust-nesting-callback-order-repair.log` and
`tsrust-nesting-callback-order-format.log`.

## Recursive dependency groups in progress

Annotated recursive groups with concrete data results now use explicit group
entry, body and release work items. Calls within an active group read the finite
access summaries already maintained by recursive inference. Bodies are checked
in reverse dependency discovery order, so a forwarding cycle does not need a
whole-program inference round for each edge. Recursive callback membership and
the borrowed-wrapper rejection remain explicit while group members are active.
The scheduler cleans group membership on errors as well as normal completion.

This removes body-call recursion for those groups without increasing any depth
guard or worker stack. A 2,048-function cycle witnesses normal-stack checking,
emission and disposal. A 256-function owned-string cycle witnesses native
execution, consuming access propagation and the original transferred-value
diagnostic span. Editor coverage includes a 512-function cycle. Existing recursive
scalar and borrowed-callback depth-rejection witnesses now expect success.

Inferred results, concrete callable result environments, context-dependent record
parameters and dynamically selected specializations still need further work.
Deep syntax, type operations and production tree disposal remain separate
pending sections. The new `recursive-manifest.json` registers matched generated
native/TypeScript compilation workloads with the existing two-warmup/eleven-sample
protocol; registration and untimed validation are not measurements.

Focused verification passes 94 compiler checks across the retained batches
(`fallible_record_callbacks`, `function_dependencies`, `exception_handling`,
`recursion`, `recursive_callable_results`, `recursive_callbacks` and
`recursive_owned_callbacks`) and 16 editor intelligence checks. Two original
runs stopped at obsolete scalar/callback depth-rejection assertions; those
original outputs are preserved and the affected/unrun suites subsequently pass.
The 32- and 2,048-function recursive sources pass all three untimed benchmark
phases. SQLite records use the `tsrust-nesting-recursive-groups` prefix, including
the exact generated sources and six validated groups in `-untimed.json`.

## Inferred recursive results in progress

Recursive groups now retain actual DFS completion order as well as discovery
order. Reversing discovery alone is insufficient when a later sibling depends
on an earlier sibling's inferred result. Eligible inferred functions publish
only their existing independent-return hint, using a context of required scalar
parameters. A call to an unfinished inferred member without that hint reports
the existing `result-inference` diagnostic at the declaration's original span.
No seed is invented from another function or from an unresolved local.

The normal-stack witness includes a 2,048-function inferred cycle; a shorter
256-function cycle and a shared-callee graph execute natively. Negative witnesses
preserve the entry-seed and local-variable boundaries. Editor coverage adds a
512-function inferred cycle with scalar result hover. The matched compilation
manifest uses explicit result annotations in TypeScript, whose recursive
inference differs; this distinction is recorded with the workload.

Defaulted/destructured parameter contexts, captured environments and generic
templates retain their previous path until full context suspension is available.
Callable results, record-dependent calls and deep syntax remain pending. This
section does not claim complete nesting acceptance or new timing measurements.

The first batch exposed stack exhaustion in the still-recursive callable-result
path: each nested request also carried a worklist interpreter frame. The dispatch
method now routes an already scheduled request directly to body checking; the
separate interpreter owns its queue only where scheduling is needed. Existing
callable-result and callback suites pass after that repair, with unchanged stack
sizes and depth guards. The aborted original run is retained in SQLite rather
than replaced by the repaired output.

The remaining audit includes parser blocks/expressions and 32-level destructuring
in `syntax/mod.rs`; recursive annotation parsing in `types/syntax.rs`; the
128-level substitution checks in `types/substitution.rs`; block/expression and
callable analysis; and recursive statement/expression error effects in
`error_effects/mod.rs`. The latter currently widens to unknown effects at depth
128, so removing parser limits alone would change typed catches. Syntax clone
and drop paths must be converted with these consumers. The emitter's worklist
depth cutoff in `small_record` is an optimization decision, while generated
console truncation is runtime behavior; neither is a compiler-stack workaround.

The retained batches cover 91 passing compiler integration checks and two editor
witnesses. After the interpreter-frame repair, dependency and lexical-capture
suites were rerun along with callable results and callbacks. Whole-project
formatting changes zero files/fingerprints. Untimed native check/emission and
TypeScript checking pass at 32 and 2,048 functions; the native phases are repeated
after the dispatch repair. SQLite retains the original failure and all subsequent
records under `tsrust-nesting-inferred-groups`, including exact generated sources,
source hashes and separate untimed evidence. No timing samples were collected.

## Callable result environments in progress

Callable result discovery now has a dependency queue in
`analysis/result_discovery.rs`. An unresolved named result records the function
currently discovering its body as a dependent. Publishing a concrete result
wakes only waiting bodies, in stable order; newly instantiated functions join
through a monotonic admission frontier. This replaces repeated scans of all
unresolved functions. Discovery state is restored on both success and error.

Recursive groups whose callable result environments have been resolved now use
the body worklist. The initial pass requests the existing recursive inference
phase rather than descending through the cycle to find its first back edge.
Unknown or infinitely growing environments retain their existing diagnostic.
The former 200-function callable-result rejection now executes successfully.
A 2,048-function cycle checks, emits and disposes on the normal worker stack,
with native execution at 256 functions and editor coverage at 512 functions.

`callable-recursive-manifest.json` registers native and TypeScript cycles with
declared function results and the independent result oracle `7`. This is separate
from scalar result inference and remains unmeasured. Context-dependent record
calls, dynamic specializations and deep syntax/type operations are still pending;
the full nesting architecture is not yet accepted.

Focused verification passes 68 compiler checks across function dependencies,
named result inference, generic defaults, ordinary callable results and the
three recursive callback/result suites, plus three editor witnesses. All six
untimed phases at 32 and 2,048 functions pass. Whole-project formatting changes
zero files/fingerprints. SQLite retains outputs, exact generated sources and
source identity under the `tsrust-nesting-callable-groups` prefix, with ordinary
callable checks in `tsrust-nesting-callable-results.log`.

## Ordinary record dependencies in progress

The scheduler distinguishes closed data graphs from record contracts that need
caller-created environments. A heap worklist follows record fields, union
members, arrays and collection element types, with cycle detection and a cache
limited to an unchanged graph inspection. Callback-bearing and opaque native
records retain their contextual path. Ordinary data-record parameters no longer
force recursive dependency checking, and closed data results can use recursive
group summaries.

Normal-stack witnesses check, emit and dispose both a 2,048-function record chain
and a cycle. Native 256-function witnesses preserve transfer without cloning,
inferred mutation, post-call field values and transferred-value source spans.
Editor coverage includes a 512-function record cycle. The matched record
compilation manifest distinguishes native moves from TypeScript references and
uses the independent field-value oracle `7`; timing remains pending.

Caller-dependent callback records, opaque native records, inferred results that
need full record/default/capture contexts and dynamic specializations still need
conversion. Preserve the handoff's known lifetime boundaries; this change does
not erase borrows or add clones to make rejected programs pass.

Focused verification passes 274 compiler integration checks, including all 202
scoped-record composition cases, the borrowed-store and replacement suites,
recursive storage, recursion, fallible record callbacks and function dependencies.
Four editor witnesses and all six untimed compilation phases at 32 and 2,048
functions pass. Whole-project formatting changes zero files/fingerprints. SQLite
preserves exact outputs, generated sources and source identity under the
`tsrust-nesting-record-groups` prefix. These checks do not establish acceptance
of the remaining contextual dependency or deep-syntax paths.

## Regional effects and completion worklists in progress

Expression and statement error annotation now retain continuations on heap stacks.
Catch contracts are published after their body is annotated; unreachable handlers
are still annotated but do not contribute escaping failures. Finally effects
replace pending failures only when the completion proof establishes an abrupt
exit. The completion proof also uses a heap worklist. The old depth-dependent
unknown-effect fallback and conservative completion cutoff are removed from these
walkers; parser and checker guards remain until their own conversion.

Payload ordering retains the existing list-fold behavior. Optional absent branches
and uninvoked closure bodies remain unvisited. New isolated 16,384-level witnesses
cover expression point annotations, nested regions and abrupt cleanup. They build
syntax directly and dispose it iteratively, so they do not claim production parser,
checker, emission or disposal acceptance. Existing native exception/optional suites
cover observable behavior; no public signature or built-in documentation changes.
Compilation measurements remain pending for the completed architecture milestone.

Focused compiler verification passes 46 effect unit checks and 72 native
integration checks across exception handling, return moves, native completion,
native exception regions, optional chaining, unreachable handlers and array
callback failures. Four catch hover/ownership editor witnesses and the project catch/finally
navigation/signature witness pass. Whole-project
formatting changes zero files and fingerprints. Benchmark source/oracle checks
retain the existing matched exception and callback workloads. The original
exception registration test incorrectly required its already measured S07 row
to remain unmeasured; the corrected assertion names the existing S07 evidence.
Original failure and affected repair are retained separately. No new timings or
allocation measurements are claimed. SQLite evidence uses the
`tsrust-nesting-effects` prefix. Installed consumers remain on `ee3dd790` for this
internal traversal checkpoint; no public API changes were introduced.

## Annotation storage conversion in progress

`types/annotation_storage.rs` gives owned annotation trees iterative cloning,
equality, nesting inspection and destruction. Clone retains callable ownership,
parameter names/access/omission/rest metadata, polymorphic identities, exact
literal spellings, child ordering and every source span. Leaf clone/equality
avoids allocating traversal storage. Owned extraction uses `into_kind`; compiler
and editor consumers move their children through that boundary.

A 16,384-level mixed tree covers arrays, groups, unions, tuples, applications,
callable results and callable parameters. Its clone and differing copy use the
ordinary production destructor on the normal test worker stack. This closes the
`ValueAnnotation` clone/drop prerequisite; source parameter/result annotation
wrappers, expression/statement tree disposal, annotation parsing/substitution/
resolution and recursive debug rendering remain pending.
Existing substitution and parser guards remain until those traversals are safe.
No guard was raised and no larger worker stack was introduced.

All-target checking is used during the ownership migration to locate required
move-site changes. Original migration diagnostics are retained as evidence.
Public Tsrust type contracts and benchmark sources remain unchanged; focused
generic, native-declaration and editor witnesses validate existing consumers.
Compilation timing remains a separate architecture-milestone requirement.

Verification passes 28 type unit checks, including the production deep-storage
witness, plus 86 compiler integration checks across type syntax, aliases, generic
default scope, generic value signatures and native declarations. The latter
executes the existing schema catalog examples with formatting and output checks.
Editor verification passes 66 generic checks and 39 schema checks, with no overlap.
All-target migration checking passes; whole-project formatting changes zero files
and fingerprints. Evidence, including original ownership-migration diagnostics,
is byte-verified in SQLite under `tsrust-nesting-annotation-storage`. No new timing
or allocation samples are claimed. This internal storage checkpoint leaves
installed consumers on the last public behavior checkpoint, `ee3dd790`.

## Bidirectional callable annotation conversion in progress

Source callback/result wrappers now lower to structured annotations through an
explicit construction worklist. The reverse adapter also uses construction
frames, separating parameter callback adaptation from result invocation modes.
Nullable callable detection traverses nested unions/groups iteratively while
retaining array and direct-callable boundaries.

The conversion preserves owned/once/shared modes, inferred result and parameter
markers, omission/rest/access metadata and grouped source spans. Borrowed result
wrappers retain the existing named-form resolver path. Source parameter ranges
still end at the annotation payload, excluding access-wrapper closing delimiters;
the initial new round-trip assertion incorrectly expected the wider parsed range
and was corrected against the unchanged pre-refactor rule.

A 16,384-level mixed parameter/result tree converts both ways on the normal worker
stack and compares through the production structured-annotation equality walker.
A deep union/group witness retains nullable detection boundaries. Structured
outputs use production disposal; source callback/result wrappers are deliberately
disposed iteratively by the isolated witness because their storage conversion is
still pending. Parser, source wrapper clone/drop, substitution and resolution
remain required before lifting the associated guards or claiming full acceptance.
No public Tsrust contract or benchmark protocol changes in this section.

Verification covers eight adapter unit checks (seven initial passes plus the
corrected span assertion), 96 compiler integration checks and 35 callable editor
checks. Native coverage includes the existing optional/default benchmark source-
order witness. Both optional/default benchmark registration/oracle checks pass
after repairing a stale historical prefix assertion: the new fingerprint was
verified byte-for-byte against the accepted `6fe878c2` protocol prefix. No workload,
manifest, measurement status or sampling protocol changed. Original unit/benchmark
failures and affected repairs are preserved in SQLite under
`tsrust-nesting-annotation-conversion`. Whole-project formatting changes zero files
and fingerprints. Installed consumers remain on the last public behavior
checkpoint; no timing or allocation measurements are claimed for this section.

## Substitution continuations in progress

Generic substitution now keeps parent annotations and child addresses in explicit
frames. Children move into and out of these frames without cloning the input tree.
Only supplied replacement arguments are cloned, as before. Simultaneous bindings
do not re-enter replacement syntax. Parameter access adjustments occur only after
the corresponding child completes. On an error, frames restore the exact partially
visited tree: earlier substitutions remain, later siblings remain untouched, and
the first diagnostic retains its original span.

Private containment projection and copy-mode classification also use heap worklists.
The production 128-level admission boundary remains because type resolution still
has recursive consumers. Tests invoke the same substitution engine without that
boundary to prove 16,384-level traversal, projection, classification and production
structured-tree disposal. This is evidence for removing the substitution call-stack
dependency, not public deep-source acceptance. Remove the admission boundary only
with the resolver conversion; do not increase it or count it as a final contract.

Verification passes ten substitution unit checks, 116 compiler integration checks
and 66 generic editor checks. Integration coverage includes generic value adapters,
default built-ins/scope, value signatures, literal types and aliases; three adapter
cases execute matched TypeScript references. Whole-project formatting changes zero
files and fingerprints. SQLite records under `tsrust-nesting-substitution` preserve
outputs and source identity with exact restoration verification. No timings or
allocation measurements were collected, and no public admission boundary changed.
Installed consumers remain on the last accepted public behavior checkpoint.

## Shared type-resolution continuations in progress

Data annotations resolve through postorder frames for groups, unions and array
leaves. Callable resolution has one worklist for value annotations, source and
structured callbacks/results, parameter contracts, unions and storage declarations.
Structured signatures resolve directly without repeatedly constructing recursive
source wrappers. Parameter names are checked in source order; contract identities
are published only after their children, preserving distinct source ownership paths.

Per-run caches retain successful value/storage/array resolutions for immutable
annotation nodes. They distinguish node identity and the allow-void context rather
than conflating equal source spans. This prevents repeated data-union fallback
traversal; errors are not cached. Active grouped-storage frames remap error spans
in the same inner-to-outer order as the former recursion. Omitted structured
parameters retain the previous Optional adaptation, while executable source
defaults retain their own omission metadata.

Normal-stack witnesses cover 16,384-level mixed callable parameters/results,
nested data unions through array-storage fallback, grouped storage diagnostics,
stable repeated contract identities and source result wrappers. Structured trees
use production disposal; the source-wrapper witness still disposes its input
iteratively. The shared resolvers are now iterative, but generic materialization
and source wrapper storage still need conversion before removing substitution’s
production admission boundary. Array rank representation limits are unchanged.
No public Tsrust contract or benchmark protocol changes in this section.

Verification passes 12 resolver unit checks across the repaired initial batch and
the additional deep source-wrapper witness, 265 compiler integration checks and
133 distinct editor checks. Compiler coverage includes callable fields/results,
nullable and recursive callables, generic value adapters, literals, aliases, type
syntax and native declarations, including executable schema catalog examples.
Editor coverage includes callable navigation/help and schema types. Original
compile diagnostics (module documentation placement and borrowed name-set types)
and successful repairs are retained in SQLite under `tsrust-nesting-resolution`.
Whole-project formatting changes zero files and fingerprints. This section has
no new timing/allocation samples or public admission-boundary changes; installed
consumers remain on the last accepted public behavior checkpoint.

## Generic declaration and source-wrapper traversal

Generic record template validation now uses an explicit annotation worklist.
Constructor arity checks still precede children, children retain source order,
and callable results retain their grouped-void exception. Result type arguments
continue to permit only directly named void at this stage. Source parameter and
result wrappers likewise traverse with mutable-reference worklists, preserving
parameter-before-result materialization without moving or cloning source trees.

A normal-stack 16,384-level witness validates nested tuple annotations and walks
source result wrappers. Focused diagnostics verify first-error order, constructor
arity precedence and the distinct void rules. Structured annotation disposal uses
the production destructor; source wrapper disposal remains isolated in the test.
Generic annotation materialization and instance expansion still recurse, and
existing admission limits remain pending their conversion. This section adds no
public API, benchmark workload or timing claim.

Verification passes four generic record unit checks and 108 integration checks
across generic value adapters, generic value signatures and Result matching.
The native suites retain matched TypeScript references, nominal identities,
ownership diagnostics and payload cleanup checks. Generic editor verification
passes 66 checks. Whole-project formatting changes zero files and fingerprints.
SQLite records under `tsrust-nesting-template` retain outputs and source hashes
with byte-exact restoration. No timing samples or public admission changes are
claimed; installed consumers remain on the last public behavior checkpoint.

## Generic materialization continuations

Concrete annotation materialization now uses owned parent continuations, visiting
children in source order before resolving their enclosing tuple or application.
Record expansion shares that worklist: a nominal identity is published before
its fields, each field receives simultaneous lexical substitution, and the complete
field list is published only after all fields succeed. Errors restore the caller's
partially processed syntax, preserving earlier changes and leaving later siblings
untouched. Failed record expansion retains its published identity and empty fields.

Result arguments use the same worklist, and specialization label rendering uses
an explicit output stack. The obsolete 32-level Result and record specialization
guards are removed. Existing per-declaration and program instance-count budgets
remain independent limits; parser, substitution and checked-type reconstruction
still have pending nesting boundaries. This does not establish full architecture
acceptance. Matched generic-declaration compilation sources are registered under
`benchmarks/compiler/generic-materialization` with the existing fresh-process
measurement protocol; registration and untimed validation are not measurements.

Verification passes six unit checks, 43 generic project checks, the new nested
Result native witness, and 170 native integration checks across generic adapters,
Result matching and native declarations (including executable schema examples).
The normal-stack witnesses cover 16,384 mixed annotation levels with production
clone/drop, 512 generated record layers through checking/emission, native execution
at 64 layers, and native compilation/execution with 40 nested Results. Error tests
preserve partially materialized parents and unpublished record fields. All 67
generic editor checks pass, including field help beyond the former depth boundary.

Six untimed compilation phases pass at 32 and 512 declarations. At 32 declarations,
native and Node-after-tsc execution match the independent empty-array oracle `0`.
Existing function workload generators retain exact source bytes across twenty
representative native/TypeScript pairs. No timing or allocation samples were added.
Whole-project formatting changes zero files and fingerprints. SQLite evidence uses
`tsrust-nesting-materialization`, including the original consumer build invocation
with an incorrect package name and its corrected invocation. Consumer refresh is
required for this newly admitted public behavior; full nesting acceptance remains
pending the remaining parser, analysis, reconstruction and syntax storage paths.

## Checked annotation reconstruction

Checked callable and generic value annotations now share a postorder construction
worklist in `types/reconstruction.rs`. Callable parameter/result metadata remains
owned by the analyzer; generic nominal and named recursive union resolution remain
owned by inference. The constructor preserves child order, shared acyclic subgraphs
and first-error locations. A structural cycle reports an unsupported recursive
contract at the requested annotation span. This replaces the analyzer's 128-level
reconstruction cutoff; finite depth no longer controls this helper's acceptance.

Normal-stack witnesses reconstruct 16,384 mixed callable parameter/result levels
and 16,384 alternating union/array levels, then use production annotation disposal.
Callable ownership, parameter access/omission and requested spans survive; an
injected contract cycle terminates with the expected diagnostic. Named recursive
unions retain nominal references instead of expanding their recursive payloads.
Source parser, generic constraint unification, source wrapper storage and other
consumers remain pending; these isolated witnesses do not establish deep-source
acceptance. Existing benchmark workloads and protocols remain unchanged, and no
new timing or allocation claim accompanies this internal construction conversion.

Verification passes ten distinct unit checks, 163 callable/generic integration
checks, four native schema-object checks and 67 generic editor checks. Existing
native fixtures retain ownership, recursive environments, nullable storage,
matched TypeScript references and executable object examples. Whole-project
formatting changes zero files and fingerprints. SQLite evidence uses the
`tsrust-nesting-reconstruction` prefix, with exact restoration verification.

A comparative source probe passes a returned callable through generic identity.
Both the prior checkpoint and current compiler accept 64 inferred callable levels.
At 160 levels, the prior compiler rejects checked reconstruction; the worklist
reaches the still-retained substitution guard without stack failure. A preliminary
bare-function probe failed inference in both builds and is preserved separately;
it is not counted as deep-source acceptance. Parser, constraint unification and
source wrapper storage must be completed before removing the remaining admission
boundary. These probes collect no timing samples.

## Generic contract key storage

The inference audit found another recursive dependency: generic function keys
used nested enums whose derived clone, hash, equality and drop recursed through
source structure. Keys now retain a flat prefix-token sequence. A temporary arena
interns canonical nodes using local child IDs; serialization removes those IDs so
construction order cannot affect key equality across requests. Token variants and
container arities distinguish every structural boundary.

Direct union/group boundaries are flattened before constructing their children,
avoiding repeated intermediate union prefixes. Stable null/undefined ordering and
adjacent-only deduplication preserve the existing contract. Callable names and
spans remain excluded; rest, access mode, omission and normalized ownership retain
their prior identity rules. A bounded reference implementation checks equivalence
across 6,400 ordered annotation pairs. Normal-stack 16,384-level witnesses cover
construction, clone, hash, equality, cache lookup and disposal, plus skewed union
normalization with production source-tree disposal.

No public admission boundary or benchmark protocol changes in this section.
Generic constraint unification and source wrapper storage still require conversion
before lifting substitution admission. These storage witnesses do not establish
end-to-end deep-source acceptance; no timing or allocation samples are claimed.

Verification passes 11 generic function unit checks, 108 native integration checks
across generic defaults, function values and adapters, 43 generic project checks,
and 67 generic editor checks. Existing coverage retains cache reuse, exact instance
budgets, private nominal identities, callable ownership and matched TypeScript
fixtures; the registered generic pipeline executes at both native optimization
levels. Whole-project formatting changes zero files and fingerprints. SQLite
records under `tsrust-nesting-contract-keys` preserve exact outputs and source
identity with restoration verification. Installed consumers remain on `36093d5b`,
the last accepted public behavior checkpoint; this section changes private key
storage without changing admitted programs or public signatures.

## Generic constraint name materialization

Constraint name materialization now suspends parent annotations in explicit frames,
visits children in source order and resolves each applied nominal identity after
its arguments. Failed resolution restores the partially visited tree: completed
siblings and child materializations remain, the failing constructor remains
unresolved, and later siblings remain untouched. Callable contracts retain their
existing opaque treatment in this visitor. The nullable deferral classifier also
uses a worklist and preserves its callable boundary.

Normal-stack witnesses exercise 16,384 enclosing groups for successful nominal
materialization, nullable classification and first-error restoration. They use the
production annotation clone, equality and disposal paths. Existing generic adapter
and function-value fixtures cover contextual inference, ownership and native
execution. This internal conversion does not lift admission guards, add public
methods or change benchmark protocols. Generic unification itself, source wrapper
storage and the broader parser/checker/emitter conversion remain pending. No timing
or allocation measurements are claimed for this section.

Verification passes eight inference unit checks and 105 native integration checks
(17 generic function values and 88 generic adapters). Whole-project formatting
changes zero files and fingerprints. SQLite records under
`tsrust-nesting-constraint-names` retain both unit runs, final native outputs,
formatting and source hashes with byte-exact restoration verification. The first
unit run predates the additional deep failure-unwind wrapper; the final unit run
covers that wrapper. Installed consumers remain on `36093d5b`, since admitted
programs and public signatures are unchanged.

## Generic constraint unification

Generic constraints now share an explicit worklist for structural unification,
callable signatures, deferred nominal field resolution and nullable callable
members. Source constraints are borrowed. Substituted payloads own their trees,
and child extraction moves those trees instead of cloning each nested remainder.
Union-member continuations retain one template and prepare only the current
member, so pending members do not each retain a deep annotation copy.

Parameter constraints precede result constraints; tuple fields and generic
arguments retain source order. Field annotation resolution runs immediately
before its constraint, including the previous eager fallback for already checked
fields. First-error spans and partial bindings remain observable in regression
witnesses. Optional callback inputs still strip only the sole non-undefined
payload; callback ownership and access checking remain with their existing owner.

Normal-stack tests cover 16,384 alternating callable parameter/result levels,
borrowed constraints, owned nullable payloads and production disposal. Focused
ordering tests preserve earlier bindings, untouched later variables and deferred
field diagnostics. This section does not remove substitution admission: source
wrapper storage and downstream parser/checker/emitter paths remain unfinished.
Full architecture acceptance and compilation scaling remain pending; no new
performance claim or public method is introduced here.

Verification passes 11 inference unit checks, 108 native generic checks and 67
generic editor checks. Existing native fixtures retain matched TypeScript sources,
ownership/default/nullable cases and the registered generic pipeline. SQLite
records under `tsrust-nesting-unification` preserve initial checks, the final
compiler rerun after the union storage refinement, editor results and source
hashes. Whole-project formatting changes zero files and fingerprints. These are
focused internal acceptance checks, not full compiler architecture acceptance or
new performance measurements. Installed consumers remain on `36093d5b`, the last
accepted public behavior checkpoint.
