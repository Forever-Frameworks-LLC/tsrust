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
