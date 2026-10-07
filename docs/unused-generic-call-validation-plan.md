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
