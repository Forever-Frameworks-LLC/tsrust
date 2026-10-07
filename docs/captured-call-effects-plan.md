# Captured callable writes

This continues Gate173 after e2f5289. It does not change the ownership model or accept the complete gate.

## Reproduced defects

The focused generic_call_capture_effects suite preserves eight incorrectly accepted calls after a captured callable changes from a one-argument declaration to a two-argument declaration. The cases cover named and arrow writers, transitive and recursive calls, conditional invocation, writes before throw, synchronous collection callbacks and argument evaluation. All eight precision controls pass: unused declarations, unused arrows, saved snapshots, explicit restoration, lexical shadowing, restoration inside a writer, callee snapshots before argument effects and declarations after a call.

A separate witness exposes a dependency: a nested function cannot explicitly name its enclosing generic type parameter. The lexical resolver lifts that function into an independent declaration, while generic specialization only knows the lifted declaration's own parameters. Do not hide this defect by changing the witness to an inferred result.

## Implementation gates

1. Preserve lexical generic environments across lifting and specialization. Keep inherited parameters separate from explicit call type-argument counts. Concrete instances must substitute the enclosing instance's arguments in nested annotations and capture types. Cover generic shadowing, multiple factory instances, nested recursion and native owned results. Simply appending inherited parameters to the public generic signature would produce incorrect arity and is not a repair.
2. Add an isolated callable-effect index keyed by declaration identity and lexical captured binding identity. A declaration contributes a body to the index, never an executed write. Preserve trusted arrow provenance and distinguish unknown callback contracts from known bodies. Reuse source graph evaluation rather than scanning textual writes into caller state.
3. Apply known invocation effects after arguments, using the saved callee snapshot. Summaries need normal and exceptional exits separately, ordered overwrites and finite joins. Recursive groups require a finite fixed point; transitive calls must not depend on declaration order. Unknown invocations conservatively affect reachable captured roots without invalidating unrelated local bindings.
4. Model synchronous built-in callback invocation through compiler contracts. Begin with forEach, including empty, singleton and unknown-length inputs. Complete other relevant callback methods with their short-circuit and cardinality rules before claiming collection-wide effects. Never execute a closure merely because it appears as an argument or declaration.
5. Finish editor diagnostics, related declarations and navigation. Preserve source spans through summaries. Run formatted successful sources, native ownership/failure/disposal controls and independent TypeScript checks. Extend the existing matched family only for newly executable behavior; rejection-only controls do not need artificial timing scenarios.

Each gate retains red witnesses, runs bounded affected compiler checks and the complete editor before acceptance. Dispatch changes require all eight depth canaries. Format the entire project before checkpoints. Keep failed and repaired evidence in SQLite with exact readback. Full Gate173 also retains optional expression regions, stored callable fields, concrete environment replacement and lifetime/context work.

## Current checkpoint

No compiler source is changed by this checkpoint. The isolated red run has three tests: one precision test passes; invocation effects and enclosing generic annotations fail. Both original and isolated failed logs are retained in SQLite. This is a test and planning checkpoint, not feature acceptance or measured performance.

## Lexical scope foundation

lexical_generic_scopes.rs derives inherited type parameters from the lifted declaration catalog, independently of value capture plans. It walks lexical owners iteratively, keeps original declaration spans, applies nearest-scope shadowing and returns outer-to-inner ordering for stable instance keys. A nested declaration's own type parameters remain unchanged. Two unit controls pass after formatting their source: mixed nesting/shadowing/siblings and capture-free inheritance.

This module is a prerequisite, not yet wired into specialization. The concrete integer/string echo witness remains red. Do not accept generic transport merely because scope discovery works. Next, contextualize each lifted declaration using its enclosing instance's lexical bindings, substitute inherited annotations, and preserve the nested declaration's own generic signature. Cache contextual identities separately from explicit argument identities. Retain the original declaration identity for named capture requests and editor spans. Rebase lexical identities for linked modules; do not use spelling or enclosing value capture types as a substitute for type identity.

Materialize capture-free and captured nested declarations through the same lexical path, including uncalled declaration validation. Calls and value uses must resolve the same contextual declaration. Recursive peers need their contextual mapping installed before bodies are walked. A nested function's own generic variables shadow the enclosing variables and must not be replaced by the enclosing substitution. Native tests must prove independent integer/string instances and shadowed explicit generics before this prerequisite becomes accepted behavior.

## Contextual specialization implementation (not accepted)

lexical_generic_instances.rs now contextualizes lifted declarations before each enclosing instance is drained. It substitutes inherited annotations while leaving the declaration's own explicit type parameters unchanged. Recursive peer names are installed as a group before body traversal. Contextual generic templates retain inherited bindings for later explicit or inferred application. Concrete applications retain their original source declaration and a separate concrete lexical owner; named capture preparation uses that owner rather than searching for a drained source template.

The original unused nested annotation witness and concrete integer/string echo now pass. Formatted native controls also cover capturing reads, forwarded peers, explicit and inferred nested generic calls, saved function values, recursion, uncalled captures and deeper nesting. Editor checks retain source parameter help and definition locations while outer calls expose concrete result types. The original invocation-write rejection test is still red and deliberately remains separate from this successful lexical batch.

This is implementation progress, not feature acceptance. Complete linked-module identity, capturing generic value/escape and ownership/disposal controls, independent TypeScript evidence and matched workload support before acceptance. Source rejections must still validate every unused lifted declaration. Preserve bounded checks already completed and supplement affected checks after repairs instead of restarting all suites.

## Focused lexical transport acceptance

The lexical dependency now has focused acceptance; see handoff-2026-10-07-lexical-generic-transport.md. Linking preserves inherited type names and validates nested explicit applications in the innermost function. Escaped ordinary and generic values retain independent owned captures, and O0/O3 failure disposal observes each original owner once. Public arity and repeated consumption rejection controls pass. Current bounded evidence is 177 compiler tests and the whole 327-test editor, plus five matched cases with 124 observations, all untimed. The original eight invocation-effect reds remain required implementation work.

## Callable-body index WIP

The isolated generic_call_effect_bodies module indexes named and arrow bodies by
source declaration span without executing them. It records binding declarations
and resolves uses to lexical BindingId values. Existing capture-plan provenance
handles hoisted named uses that occur textually before initialization. Lexical
ranges supply identities for ordinary locals, closure parameters, loop and catch
bindings; declaration identities agree with planned uses when available.

Two formatted-source tests pass after a retained failing stub. They distinguish
hoisted captured writes from local shadowing, and arrow captures from parameter
shadowing. The module is registered but not invoked by production validation yet.
No invocation-effect witness is repaired by indexing alone. This is a prerequisite
checkpoint, not source-flow or Gate173 acceptance.

Next, attach the index to source validation and map existing flow bindings to its
declaration identities. Known callee contracts must select indexed bodies through
original declaration spans, including alternative provenance. Use ordered normal
and exceptional summaries, preserving caller snapshots and explicit restoration.
Local callee bindings must never overwrite caller bindings with the same spelling.
Calls resolve after arguments using the saved callee; closure declarations do not
execute indexed bodies. Unknown callable origins must not inherit a fabricated
body. Recursive groups require a finite worklist rather than recursive evaluation.
The eight invocation reds and all precision controls remain mandatory.

## Source-flow identity wiring WIP

Source validation now constructs the effect index once and assigns declaration
identities to flow parameters, locals and closure parameters. Branch/expression
lookup uses an indexed identity before textual ranges. A known declaration missing
from the current flow frame does not alias a same-named caller slot. Snapshots and
assignments retain these identities through ordinary joins and state copies.

The retained red demonstrated both requirements: a hoisted capture was missed by
textual lookup, while a callee's local shadow could alias the caller binding. Its
repaired unit test passes. Three index tests pass as well. A new red found that
synthetic named-value forwarding wrappers give generated parameters one shared
source span. The index now recognizes trusted forwarding provenance and indexes
the original named body, not an independent synthetic body. Do not synthesize
capture identities from those forwarding argument spans.

The current bounded integration batch passes 186 tests, including all eight depth
canaries and successful lexical/precision controls. The invocation rejection test
is deliberately excluded from this green batch; eight invocation defects remain.
No summary body is executed by this wiring. It is WIP, not Gate173 acceptance.

Before invocation summaries are complete, preserve callable body origin separately
from an annotated signature. Typed local callback annotations and type assertions
can override source cardinality without erasing a known initializer's body. An
unknown callback parameter has no fabricated body; a supplied known argument may
provide one for a contextual invocation. Summaries must preserve this distinction
through saved aliases, parameters and selected values. Then integrate ordered normal
and exceptional effects after arguments, with finite recursive contexts and real
collection callback invocation rules. Keep the full eight-red and precision scope.

Whole-editor continuation passes all 327 tests. Candidate alias traversal and the
legacy conservative paths still need identity-aware resolution alongside summary
frames; binding selection alone is not complete alias provenance. Preserve these
remaining requirements in the next implementation chunk.

## Alias candidate identity WIP

Candidate traversal now accepts the effect index for every source-name resolution
path. Expression snapshots, straight-line assignment snapshots and conservative
call reconstruction use lexical identity. A known captured/local declaration that
is absent from the supplied frame cannot fall back to a same-named global function.
Conservative write accumulation excludes a different known declaration with the
same spelling. Pure contract-set joins retain their index-free helper because they
do not resolve source names.

Two retained reds are repaired: hoisted captured aliases were unresolved by textual
ranges, and an empty candidate set was reported as a known callable. Empty sets now
remain unknown. The identity witness also covers local shadows and absent frames
with a real same-named global declaration. Six unit tests, 186 bounded integration
tests including all depth canaries, and the whole 327-test editor pass.

No invocation summary executes yet. The eight invoked-write reds remain required.
Next preserve known body provenance independently of annotated signatures and type
assertions, then apply ordered normal/exception summaries after arguments using the
saved callee. Reuse the identity index across summary frames, retain unknown origins,
and use a finite worklist for recursive calls. This remains WIP, not Gate173 acceptance.

## Separate body provenance WIP

The isolated generic_call_body_origins module now extracts actual named or arrow
bodies independently of callback signature assertions. It follows value-preserving
assertions, assignment results, final sequence values and selected alternatives.
Unknown values remain unknown. Trusted generated forwarding arrows retain their
original named declaration rather than becoming independent synthetic bodies.
Three focused unit tests pass. This extraction is not yet carried through binding
states and expression snapshots; invocation summaries remain unimplemented.

Its red witness exposed a parser defect in grouped arrows inside conditional arms.
Closure lookahead now parses exactly one result annotation rather than scanning
across a conditional colon for an unrelated later arrow. Grouped zero-argument
arrows execute correctly when selected into an explicitly typed record callback.
The retained unannotated_conditional_arrows_infer_common_callable_carrier test is
still red: ordinary conditional locals reject distinct arrow origins even when
both signatures match. A Function annotation on the local also does not supply
the common storage carrier. Do not hide this dependency by claiming general
conditional-arrow acceptance. Retain it for the callable carrier implementation.

## Body provenance state wiring WIP

Bindings and expression values now carry origins separately from signature
contracts. Annotated initializers retain actual bodies; typed reassignment changes
body provenance without rewriting the declared call signature. Straight-line
initializer snapshots preserve saved aliases, and expression assertions keep the
already evaluated operand's body rather than re-reading a mutable name.

Branch state joins canonicalize finite body sets and retain unknown alternatives.
Alias resolution uses lexical binding identities; an absent captured frame cannot
resolve to a same-named global. Cycle detection is path-local, so repeated paths to
one known alias do not become unknown, while a genuine cycle retains an unknown
alternative. Parameters remain unknown until supplied contextual arguments are
installed by invocation summaries.

Fifteen focused metadata tests, 186 bounded compiler tests and all 328 editor tests
pass. The compiler pass explicitly skips invoked_capture_writes_reject_invalid_reachable_calls.
Neither that red nor the conditional common-carrier red has been repaired. Body
origins are ready for summary evaluation but calls still discard their body effects.
No new workload observations, timings or executable-consumer refresh is claimed.

## Separate completion graphs and ordered summary flow WIP

Summary graphs now have distinct fallthrough, return and uncaught-throw terminals.
Their cleanup entries preserve pending completion routes and permit finally to
replace a return with a throw or a throw with a return. Existing diagnostic graphs
retain their original terminal convention.

The shared generic_call_node_flow module evaluates operands before committing a
destination. Both ordinary source flow and the standalone summary runner use it.
The runner maintains finite binding-state joins, separate completion states and
saved call-signature facts. Tests cover ordered replacement/restoration, writes
before throw, catch and cleanup overrides, local shadows, argument-side writes and
callee snapshots before arguments. It is not yet called from expression invocation
dispatch, and does not yet execute nested or recursive summaries.

A retained red exposed literal guards including dead writes in source flow. Shared
normal-edge selection now honors known Boolean predicates; uninstantiated generic
source checks and editor diagnostics pass both guard controls. Native specialization
exposed a separate unresolved carrier constraint: changing between generic named
functions still fails callable-origin even when the assignment is unreachable.
The literal_guard_native_callable_replacement_has_common_carrier test remains red.
Do not claim end-to-end guard or generic replacement acceptance from source-flow
checks alone. This joins the existing conditional-arrow common-carrier dependency.

Twenty-one focused tests, 187 bounded compiler tests and all 329 editor tests pass.
The original invocation-effect red was explicitly skipped in the compiler batch;
the new native carrier red was added afterward and verified separately. The guard
source check was rerun after separating that red. Node independently verifies the
guard results. All eight depth canaries ran unfiltered. No new runtime observation,
measurement or consumer refresh is claimed. Completion values, contextual parameter
frames, caller-frame projection and finite inter-call recursion remain next work.

## Contextual invocation frames WIP

The body index now retains formal parameter slots, default initializer metadata and
rest flags for named functions and real arrows. Trusted forwarding wrappers still
use their original declaration. Parameters preserve their declared signatures while
receiving actual supplied callback body provenance.

The isolated invocation frame masks callee-owned lexical slots, initializes fresh
locals and projects only caller bindings back out of an outcome. Recursive re-entry
can therefore install a different parameter body without rewriting the suspended
caller's parameter. Projection indexes outgoing identities once instead of scanning
all outgoing slots for every caller binding. Contextual summary invocation connects
these frames to the standalone runner and preserves separate completion states.
It requires already evaluated default arguments; initializer execution and rest
packing are not implemented by this layer. It remains unwired from Task::Call.

Twenty-four focused tests pass, covering named and arrow parameters, default/rest
metadata, absent-body uncertainty, recursive slot isolation and a supplied callback
body transferred into a captured binding. The 187-test compiler batch passes all
eight unfiltered depth canaries, explicitly excluding the invocation-effect and
native-carrier reds. The affected editor intelligence and stdio suites pass 197
tests. This is metadata WIP, not acceptance; the preceding whole-editor 329-test
record remains preserved, and full editor verification is still required before
accepting the eventual invocation implementation. Formatting changes zero files
and fingerprints. No measurements, new observations or consumer refresh is claimed.

## Finite inter-call worklist WIP

Expression dispatch now saves the callee before arguments, supplies evaluated
argument Values, merges nested signature facts and distinguishes an absent normal
exit from an ordinary unknown result. A definite throwing invocation stops the
remaining expression and destination commit while preserving its exceptional state.
Node and summary evaluators carry the dispatch interface without recursively
executing another Rust evaluator.

The standalone invocation engine caches contextual requests and schedules reverse
dependencies on an explicit worklist. New known requests start at bottom: no exit
is fabricated while their bodies are pending. Normal, return, throw and signature
facts grow through finite joins. Recursive and transitive tests converge, and a
repeated request reuses the completed cache. Seven original effect scenarios now
have correct facts through this standalone engine: named, arrow, transitive,
recursive, conditional, caught throw and argument-side writes. Compiler validation
is not connected to it yet; the original integration red remains unchanged.

A supplied-false guard witness exposed missing scalar context. Bindings now carry
finite truth, nullish and undefined facts independently of bodies/signatures; frame
installation and projection preserve them, and joins retain only agreed facts.
Opaque dispatch invalidates mutable selection assumptions rather than guessing
that an unknown call preserves them. No concrete numeric domain was added.

Twenty-eight focused tests, 187 bounded compiler tests and 197 affected editor tests
pass. Both existing integration reds are explicitly excluded. Full editor remains
required before accepting actual invocation integration. Formatting changes zero
files/hashes. No observation, timing or consumer refresh is claimed. Default/rest
materialization, unsupported/unknown-region eligibility, return-value snapshots,
collection rules and root-validator integration remain required before acceptance.

## Opaque dependency propagation

The contextual worklist now propagates missing-body and unknown-callee uncertainty
through reverse dependencies. Opaque status is monotone, including repeated cache
lookups. A failing transitive callback witness demonstrates the previous unsafe
parent summary; it now passes for missing known bodies and unknown callbacks.
All 29 focused tests and eight unfiltered depth canaries (63 tests) pass. The root
compiler validator remains unchanged. Default/rest execution and integration remain
open. Exact red and passing evidence is stored in SQLite.

## Ordered default entry WIP

A separate parameter-entry module installs arguments and evaluates defaults from
left to right before body execution. It retains captured writes, earlier parameter
selections, exceptional states and source call facts. Undefined uncertainty joins
supplied and default branches; missing required arguments retain diagnostic facts
for the ordinary arity validator. Rest entry remains explicitly opaque until
aggregate element provenance exists. Focused tests exercise omitted and explicit
undefined arguments, supplied values and throwing defaults. Compiler integration
and native/editor acceptance remain open.

## Flat rest/array element provenance WIP

Rest entry packs trailing argument snapshots into flat element facts. Arrays,
aliases, frames and joins retain individual positions; constant indexing supplies
the selected callback body to invocation dispatch. Shapes do not recursively nest.
Unknown indices and writes invalidate the proof rather than retaining stale alias
contents. Storage identity, precise mutation, spread, iteration and returned
aggregate snapshots remain required before acceptance. The root compiler path is
still unchanged. Focused witnesses cover packing, aliasing and separate positions.

## Root invocation integration WIP

Compiler source cardinality validation now tries converged contextual summaries
before existing fallback. All eight retained invocation-effect reds now reject
with related arity diagnostics. Fixed-array forEach dispatch captures its receiver
before arguments, sequences callback requests and preserves empty and exceptional
regions. Formatted compiler witnesses cover element guards and restoration.
Opaque/unknown regions still require stronger fallback effects, and shared storage
mutation is not represented. Those bodies refuse contextual proof. Full acceptance
still requires return snapshots, collection coverage, spread, mutation, carrier
repairs, matched workload support and final editor/native verification. The whole
editor crate passes 329 tests; no timing or consumer refresh is claimed.

## Cached aggregate-write eligibility

Immutable aggregate-write eligibility now has a dedicated module and is computed
once while indexing each body and its parameter defaults. Contextual invocation
reads a cached flag; it no longer clones and scans source or builds a separate
eligibility graph per request. Focused witnesses retain statement, expression,
default and nested-update coverage. No timing improvement is claimed.

## Console leaf effects

Supported console methods on the unshadowed namespace now preserve captured-slot
state instead of invalidating the entire invocation summary. A separate classifier
keeps this rule distinct from user callbacks and collection dispatch. Known writes
and restoration survive output before or after the write, including collection
callbacks. Unknown methods and shadowed names remain ineligible. Other opaque
regions, aggregate mutation, return snapshots and carrier failures remain open.

## Pending return snapshots

Contextual summaries now preserve evaluated return Values separately from binding
state. Normal finally cleanup retains the snapshot; a new return replaces it and
exceptional completion clears it. The worklist joins returned Values and supplies
them to caller expression evaluation, including flat returned-array elements.
Compiler witnesses cover cardinality and cleanup, and direct summary tests cover
throw/catch overrides. All 34 focused, 191 bounded compiler and 329 editor tests
pass. Escaped captured environments still require context/lifetime identity; this
checkpoint does not accept general factory effects or native carrier changes.

## Capture availability guard

Named and arrow contextual frames now require indexed captured binding identities
to exist in the supplied context. Named recursive-group plans and direct arrow
uses/defaults provide the requirements. Missing producer slots and unrelated
same-spelled slots make the invocation opaque instead of fabricating a complete
frame. Direct frame tests preserve live producer eligibility. This guards absent
environments; contextual instance identity and lifetime projection still remain
required for full escaped factory effects. Source IDs alone cannot distinguish
multiple producer instances. No ownership-model change is introduced.

## Paired factory environment red

A retained focused witness now exercises separate named and arrow factory
instances carrying different callable contracts. Both lose producer slots in the
current engine. The independent TypeScript/Node reference passes. The dedicated
docs/callable-environments-plan.md specifies finite correlated environment handles,
capture semantics and native lifetime gates before implementation. All other 36
focused tests pass with the new red explicitly excluded. No implementation or
factory acceptance is claimed by this test checkpoint.

## Environment store kernel WIP

A separate source environment store allocates stable handles from creation sites,
producer call sites and leaf capture facts. Correlated cyclic edges remain in
records, outside allocation keys. Weak joins preserve explicit unknowns and report
record changes. Controls verify distinct contexts, stable reuse, pair correlation,
canonical slots and atomic rejection of malformed inputs. All 40 focused checks
excluding the paired factory red pass. The store is not connected to invocation
and does not implement ownership, strong updates or disposal. Those requirements
and the paired red remain required before factory acceptance.

## Environment reader scheduling WIP

The environment store now records request-reader subscriptions separately from
allocation identity. A changed state or edge record queues each subscribed reader
once until the queue is drained. Unchanged and malformed merges queue nothing;
subscriptions to invalid handles are rejected. Reader notification does not add
versions or reader IDs to allocation keys. A failing stub witness preceded the
implementation, and the focused baseline passes 41 tests with the retained paired
factory red explicitly excluded. This kernel remains unused by Engine. Next connect
record reads to request subscriptions and drain changed readers into Engine's work
queue while wiring correlated value transport. No native or editor acceptance is
claimed for this isolated store change.

## Correlated value transport WIP

Values, bindings and flat aggregate elements now carry optional correlated callable
alternatives. An absent metadata field retains the existing body-origin path; when
joined with explicit alternatives, that path is converted to body-without-environment
or explicit unknown alternatives. Joins keep each body/environment pair together
and canonicalize the union, including two environments for the same body. No
independent environment/body cross product is constructed.

Transport covers source snapshots, assignments, declaration commits, formal entry,
rest elements, frame projection, assertions and selected array elements. Pending
return snapshots already carry Values and therefore retain these pairs. Opaque
fallback resets do not retain stale environment metadata. A failing alias witness
preceded the implementation; focused controls cover assertions, array storage,
indexing, unknown joins and formal entry. The focused baseline passes 42 tests with
the retained paired factory red excluded. Whole-project formatting changes no files
or benchmark fingerprints.

Engine still dispatches by origins and does not create or consume these environment
pairs. The new metadata is preparation, not factory behavior acceptance. Next add
producer call-site context, creation hooks, capture ownership-aware environment
initialization and frame entry, and connect reader subscriptions to scheduling.
Keep the missing-capture guard and the paired factory red intact. Native ownership,
disposal, mutable instance updates, recursion convergence and feature acceptance
remain pending.

## Read-only factory environment entry WIP

Engine now owns the environment store and includes producer call-site context and
an optional correlated environment handle in contextual requests. Expression
snapshots allocate environments for available nonmutable capture slots. Entry
subscribes the request to record changes, restores the exact captured identities
and retains the existing missing-capture guard. Changed records queue dependent
readers. Invocation and forEach consume correlated body/environment alternatives;
projected results remove producer slots from the original caller state.

The retained paired named/arrow factory test now passes without exclusion. Repeated
requests reuse the same request population. A compiler integration witness verifies
that the second instance rejects wrong<T>(value) with an arity diagnostic and
related declaration, while wrong<T>(value, value) passes. No source signature or
runtime ownership rule was changed.

This is read-only capture support, not complete factory acceptance. Mutable capture
slots still follow the previous live-caller path. Environment-backed requests
refuse contextual proof if captured slots change on any exit, or if live caller
slots conflict with the retained snapshot. Returned environment snapshots are also
checked against producer exit state; a new failing control exposed parameter
reassignment after callback creation and is now guarded. These conservative guards
must be replaced by verified ownership-aware local borrow and owned-instance
updates, not removed merely to make mutable examples pass.

The focused baseline passes 44 tests with no factory exclusion. All eight depth
canaries and affected compiler flow suites pass, excluding the independently
retained native common-carrier red. The whole editor crate passes. Exact logs are
stored in SQLite. Whole-project formatting changes no files or fingerprints.
Native factory ownership/disposal fixtures, mutable updates, recursive environment
solver convergence, transitive environments and matched workload acceptance remain
pending. Gate173 and the larger captured-effects plan remain open.

## Native polymorphic argument red

A new executable std:testing fixture, callable-environments/retained.tsr, exposes
an additional native prerequisite: factory(one) fails generic-inference for A.
The passing source effect test does not establish native transport of polymorphic
arguments. The integration test is retained failing and unskipped. Read
polymorphic-callable-transport-plan.md before lifting mutable environment guards;
it specifies checked argument identity, capture timing, point-sensitive instance
state, ownership/disposal controls and acceptance requirements. No compiler source
changed for this red checkpoint.

## Polymorphic argument hint WIP

polymorphic_argument_hints.rs now prepares a deferred callable source identity for
unshadowed generic declarations used as unconstrained generic arguments. It uses a
separate compiler preparation context and leaves the original expression as a name;
ordinary argument checking still owns evaluation and transfers. Repeated requests
reuse source identity, and lexical or pending bindings prevent global fallback.
The prototype retains the original template rather than inventing a concrete
parameter/result signature. A focused failing stub witness preceded implementation.

Generic call inference now consumes this hint for bare type-variable argument
patterns when no result context supplies a concrete contract. Existing contextual
adaptation stays on its current path. The native factory red advances from missing
A inference to the unsupported checked-type annotation path. It remains failing:
this checkpoint does not implement polymorphic type substitution or native carrier
transport. Next provide a checked internal annotation identity that survives
substitution and signature resolution without erasing polymorphism or rebuilding
captures. Do not turn the prototype's empty parameter list and void result into a
source callable contract.
