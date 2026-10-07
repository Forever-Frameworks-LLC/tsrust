# Unused generic call validation

This closes source-validation requirements in the Gate173 generic value audit.
It does not replace concrete type/ownership checking or complete the wider audit.

## Verified defect

At accepted compiler c18b211, an unused generic function accepts all three invalid
calls through immutable aliases: identity<T,T>(value), exact<T>(1) where exact is
monomorphic, and identity(value,value). The retained compiler test collects all
incorrectly accepted cases rather than stopping after the first. Strict TypeScript
rejects each with TS2558 or TS2554. Unknown explicit annotation validation already
works, but does not establish call-contract validation.

Borrowed-result escape controls pass for both capture-free and captured generic
adapters. Their precise rejection is parameter-access: a ReadOnly parameter cannot
be consumed to produce an owned result. Retain this diagnostic rather than weakening
ownership or expecting an unrelated borrowed-place code.

## Implementation seams

Add a dedicated source call-contract validator alongside generic_validation. Run it
before specialization drains source functions, with a complete declaration table
including lifted nested functions. Contracts retain declaration identity, generic
parameter count, required/optional/default fixed slots, rest admission and source
spans. Do not execute bodies, evaluate defaults or allocate runtime specializations.

Resolve immutable alias chains using lexical binding identity and ordered scopes.
Respect function parameters, block shadowing, closure parameters, loop bindings and
catch bindings. Annotated concrete callback aliases are monomorphic even when their
initializer originated from a generic function. Arbitrary callable expressions or
unknown flow must never inherit a fabricated generic declaration identity.

Recognize lowered generic forwarding wrappers using their trusted source provenance
and existing wrapper structure; do not classify every arrow returning a generic call
as a generic declaration value. Preserve the separation between original generic
values, fixed adapters and stored monomorphic callable fields.

Check explicit type-argument counts against the resolved declaration and reject
explicit application on known monomorphic aliases. Check static value cardinality
against required/default/optional/rest contracts. Spread arguments need the existing
static cardinality rules; dynamic spreads must retain conservative checks rather
than being counted as one value or rejected indiscriminately. Diagnostic spans name
the call/type arguments and related declaration, not a synthetic wrapper.

Implement an iterative lexical traversal outside recursive analyzer dispatch.
Branches and reassignments must not leak alias facts across scopes or assume one
path's target. A mutable alias requires a declared stable callable contract or a
sound join of all reachable target contracts. Keep type-dependent constraints for
concrete instantiation; source validation cannot assume a type variable is void or
an unconstrained wildcard. Existing generic annotation validation remains separate.

## Acceptance gates

First preserve the three red witnesses and repair declaration/immutable alias
validation. Add positive aliases in unused templates, alias chains, self/forward
references and independent block shadowing. Then cover callbacks, concrete adapters,
closures, reassignment/branches, lifted capturing values, defaults/rest and static/
dynamic spread boundaries. Add precise diagnostic and source-navigation editor
checks; unused source errors must appear without invoking the function.

Retain paired strict TypeScript witnesses for supported successful contracts and
independent negative diagnostic evidence. Update guide, plan/milestone, coverage and
roadmap. Extend matched support only where executable accepted behavior changes;
compile-time-only rejection controls belong in source/registration checks rather
than artificial timing scenarios. Run focused generic suites, all eight depth
canaries if dispatch changes, and whole editor before acceptance. Store original
failed/repaired logs in SQLite with exact readback, then remove loose copies.

Do not claim full Gate173 completion from these arity repairs. Alias/editor context,
borrowed-result lifetime and generic-field contracts still require the full audit.


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

### Next structured-flow checkpoint

Replace source-position reconstruction with a dedicated iterative flow walker.
Use lexical binding identity for assignments, not spelling alone. Initializers and
assignment right sides snapshot contracts before their destination changes.
Sequence evaluation must preserve receiver, argument and assignment order.

For if/conditional expressions, fork the incoming environment and join contracts
on completing paths. Return, throw, break and continue contribute only to their
respective exits. Loop analysis needs a finite fixed point over contract sets,
including updates and subsequent iterations; a later textual write can affect an
earlier textual call on a back edge. Catch and finally must retain exceptional
and normal exits. Captured writes require separate conservative effects until the
invocation relationship is known; never treat a hoisted or escaped function body
as an ordinary later statement.

Start with red witnesses for definite assignments inside blocks, branch-local
calls, both-arm replacement, snapshots before branch mutation, loop-carried changes
and shadowed writes. Include valid controls so precision does not introduce false
rejections. Preserve source declaration provenance across joins. Unknown contracts
still require concrete checking and must not acquire a fabricated generic origin.

After flow, audit stored callable fields separately. Typed stored callbacks remain
monomorphic even when initialized from a generic value. Also repair concrete
replacement across compatible callable environments using actual environment
transport, rather than signature-only identities or implicit cloning. Keep native
ownership/disposal and Node reference witnesses for that runtime work. These are
required remaining steps, not exclusions from the full goal.

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

## Exception source-flow red checkpoint

Eight focused witnesses currently accept invalid known-target calls in try bodies,
catch bodies, normal cleanup, cleanup after return, catch after explicit throw,
cleanup during break/continue and nested cleanup. Six valid controls preserve saved
snapshots, definite cleanup/catch replacement, non-completing branches, cleanup
return overriding throw and lexical shadowing. These are source-validation tests
in generic_call_exceptions.rs; no implementation acceptance is claimed.

Extend the iterative graph with explicit normal and exceptional successors.
Evaluation failure must propagate the incoming environment before a destination
assignment commits. Explicit throw enters the closest catch; failures inside that
catch enter outer handlers after cleanup. Return, break and continue must execute
each intervening finally in order, then resume their own completion destination.
An abrupt completion inside finally replaces the suspended completion.

Represent cleanup with continuation-specific graph entries, preserving lexical
control destinations and outer handler context. Repeated cleanup copies must join
call facts by source span, as the existing analyzer does, rather than overwrite
facts from another completion. Audit implicit failures from call/index/member and
operator expressions alongside explicit throws. Do not route all try paths into
catch indiscriminately or join returned states into the normal successor. Retain
iterative traversal and bounded growth for nested cleanup graphs.

Add editor diagnostics and formatted native/strict TypeScript controls for cleanup
ordering and target snapshots. Reuse the existing registered generic adapter family
for untimed matched verification; this source-only rejection repair does not need
an artificial runtime benchmark. Run affected generic suites and the eight depth
canaries after dispatch changes. Full Gate173 and performance measurements remain
open.


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

## Expression evaluation-order red checkpoint

Thirteen invalid known-target calls are currently accepted when expression writes
force the conservative fallback. They cover receiver effects before arguments,
nullish selection, logical assignment, ordered sequences, assignment-result callees,
argument-local mutation, callee snapshots, conditional joins, both logical operators,
condition mutation, for-update sequences and partial evaluation before failure.
Nine formatted precision controls pass, including lazy dead branches, definite
replacement and saved initializer/callee snapshots. The focused test is
`generic_call_expression_flow`; its rejection test is intentionally red.

Implement a separate iterative expression-flow evaluator. Each expression result
must carry a resolved callable snapshot rather than a name that is looked up again
after later arguments mutate the binding. Evaluate the receiver/callee once, then
arguments from left to right, then record the call against that saved contract.
Evaluate assignment references before right sides, commit only after successful
evaluation and return the right-side snapshot. Initializers and statement-level
assignments must use the evaluator result, not reconstruct it from final bindings.

Conditional, logical and nullish branches fork and join environments only on paths
that can execute. Retain literal truthiness/nullish facts so false-and, true-or and
non-nullish coalescing do not execute writes in skipped operands. Logical assignment
reads the target once and commits only on its selected branch. Scalar writes may
invalidate their own callable metadata but must not invalidate unrelated bindings.
Do not invent contracts for unknown values or stored fields.

Track exceptional environments at each potentially failing evaluation step. A
sequence that writes a callable, then indexes an array, must enter its catch with
the written target if indexing fails; the expression's entry state is insufficient.
A call failure occurs after argument evaluation and its side effects. Route these
states into the accepted exception graph while keeping normal continuation distinct.
Join call facts across iterations and branches by source span as before.

Use an explicit task/value stack rather than recursive descent so the depth canaries
remain valid. Do not execute nested closure bodies as expression effects. Keep
captured writes on the separate effects path until their audit; source contracts do
not relax concrete callable ownership or environment compatibility. Add focused
editor diagnostics and formatted native/strict TypeScript evaluation-order controls,
reuse the registered generic adapter family for untimed verification, then run the
affected generic suites and all eight depth canaries. Full Gate173 remains open.

## Expression evaluator implementation checkpoint (not accepted)

The isolated generic_call_expression_flow module now uses explicit task and value
stacks. Calls snapshot the callee before arguments. Assignments return the evaluated
right-side snapshot and commit after evaluation; conditional/logical/nullish paths
join only selected environments. Checked operations retain per-step failure states,
which feed the accepted exception graph. Failure states are not cloned when there
is no active handler. Typed callable metadata alone does not prove a nullable or
optional parameter truthy. Unsupported expression regions retain the existing
conservative path, including optional chains and closure/captured effects.

All thirteen source rejections and nine precision controls pass. Existing generic,
loop and exception suites, eight depth canaries and the whole editor pass. The
parser now recognizes explicit type arguments on grouped assignment, sequence,
conditional, call and index expressions rather than treating those forms as
comparisons. Numeric comparison forms retain their ordinary parsing. An editor
witness retains the source call and failing declaration.

A required native witness remains red: invoking `(action = action)<T>(value)` is
rejected with “Unsupported call or method.” The strict TypeScript/Node counterpart
passes. Do not drop this test, call this checkpoint accepted or silently narrow
assignment expressions to scalar-only behavior. The existing concrete call dispatch
does not select assignment-expression callees; scalar_assignment also restricts
owned assignment values. Finish this ownership-aware invocation path before
accepting the expression chunk.

Investigate the existing discarded-assignment and sequence staging paths first.
For an assignment used as callee, replace the destination once, then invoke the
stored callable using its existing read/mutate/once contract. Preserve destination
loans during argument evaluation, consuming invocation and disposal. Do not copy
an owned environment or use a signature-only replacement identity. Add failure,
consuming and capture lifetime witnesses before broader assignment-result escape
support. Keep the native witness and its owned-string assertion, and preserve the
passing source checks while implementing this runtime path. Full Gate173 remains
open beyond this chunk for capture effects, optional expression regions, stored
fields, concrete environment replacement and lifetime/context coverage.


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
