# Polymorphic callable transport through generic factories

The source effect engine now distinguishes returned factory environments, but the
native compiler cannot infer `A` in `factory(one)` when `one` is a generic function.
The retained native witness is `callable-environments/retained.tsr`, exercised by
`returned_factory_contracts_execute_from_distinct_native_environments`. It must
execute its assertions, not merely pass source cardinality validation.

## Checked inference and identity

The current `generic_argument_hint` uses `expression_type_hint`, which does not
provide a checked type for an unresolved generic declaration used as an argument.
`checked_generic_callable_shape` deliberately excludes deferred generic closures.
The inference annotation path supports concrete callable contracts, not a
polymorphic value with its own type parameters. Do not infer an arbitrary concrete
function signature to make this witness pass.

Add a finite checked identity for a deferred polymorphic callable argument,
including its original declaration and actual captured environment type. Preserve
its type parameters, required/default/rest parameter cardinality, parameter access
modes and result inference obligations. Make this identity transportable through
outer generic substitution without changing the source signature or exposing an
invented ordinary record type. Isolate this representation and its conversions
from recursive expression dispatch. Check how existing deferred callable carrier
emission, source contexts and specialization requests retain nominal identity
before selecting the internal annotation form.

Inference must create metadata without evaluating an argument or transferring an
owner twice. The ordinary argument checker owns evaluation and access effects.
Specialization keys include the finite callable identity and capture type layout,
not recursively copied source contexts or mutable runtime state. Repeated requests
reuse specializations. Distinct callable origins remain correlated with their
captured instance; an identical signature does not authorize combining owners.

## Transport and native checks

Repair the retained named factory witness, then add the arrow equivalent. Verify
saved aliases, direct and selected calls, repeated calls with different concrete
types, inferred result returns, aggregate storage, default/rest arguments and
nested factory transport. Keep arity diagnostics at the eventual invocation,
including the related original declaration. Add rejection witnesses for consumed
captures, borrowed escape and incompatible destination contracts.

Use std:testing assertions and formatted-source round trips in executable fixtures.
Keep the independent Node reference for ordinary factory identity and arity.
Native ownership-specific checks must count original owners at optimization levels
0 and 3 on normal completion, thrown callbacks, producer failure, abandoned
factories and aggregate disposal. Do not substitute an extra cloned owner or
reconstruct the factory to obtain a callable adaptation. Existing disposal checks
are useful controls, not proof of this newly supported argument transport.

## Mutable source environments

Before lifting the read-only environment guards, introduce point-sensitive flow
state for captured instances. A global weak environment union alone cannot model
sequential restoration or two calls to one owned mutable callback. Keep bindings
and a separate finite handle-to-capture-state map in each flow state. Store graphs
retain stable allocation identity; flow states retain state at a program point.
Branch and summary joins are monotone. Strong updates require one proven owned
instance; merged instances use weak updates. Do not recursively embed capture
vectors in callable Values or request identities.

Derive capture storage from the native checker: local noncopy captures borrow;
escaping noncopy captures transfer; copied scalar captures follow their checked
access and escape mode. Local mutable scalar captures can borrow, while escaping
scalar captures copy at closure creation. Named function value wrappers use the
same closure rules. Do not infer this policy from the source `mutable` flag alone:
formal slots currently have that flag false even when assignments can appear in a
body. Keep contextual proof conservative until checked capture mode facts are
available. Test each policy with source effects and native output together.

Track ownership transfer, consumption and disposal without introducing shared
JavaScript cells. Preserve pending return snapshots before finally, overriding
returns and exceptions. Prove recursive environment requests converge without an
iteration cap and without removing unknown alternatives. Unknown or disposed
handles must never certify an effect-free call.

## Acceptance

Keep each red witness until its implementation passes. Run focused source/native
checks, the eight depth canaries after analyzer dispatch changes and the whole
editor crate before acceptance. Add hover, completion and diagnostic witnesses
for transported polymorphic contracts and document observable ownership behavior.
Prepare matched TypeScript/native workloads, independent oracles, source contract
checks, registrations and driver hooks during implementation. Extend the existing
generic-value workload family with unmeasured cases rather than claiming existing
measurements cover new transport.

Format the whole project before checkpoints. Store exact verification records in
SQLite, read them back and remove closed loose logs. Preserve owner fixture edits.
This prerequisite and mutable environment work remain part of the larger captured
call effects plan; neither is accepted by the source-only paired factory test.

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


## Checked polymorphic argument transport

The retained native factory witness now passes for named selectors and arrow
selectors. A checked internal annotation retains deferred source identity across
generic substitution, signature resolution and definitive closure compaction.
Source syntax does not expose this annotation. Generic factories preserve the
original callable contract instead of inventing a concrete signature or rebuilding
captured owners. One retained callable can specialize for both integer and string
arguments when its checked capture access permits reuse. Independent one-argument
and two-argument factories retain their distinct contracts.

Native disposal witnesses count the four original owners exactly once after
success, callback failure and abandoned selectors, at optimization levels 0 and 3.
Borrowed escaping captures, transferred callable reuse and consuming capture reuse
remain rejected. Editor tests verify concrete signature help and integer/string
results without displaying internal identities. Existing nullable generic inference
now applies result context before retrying deferred null constraints; its existing
regression witness passes.

The bounded compiler batch passes 191 checks, including all eight depth canaries;
78 generic unit checks and the whole editor crate pass. The existing matched generic
value adapter family now observes 127 values, with independent oracle arithmetic
and source mutation guards. Timing remains pending. This is a prerequisite within
gate 173, not acceptance of the complete captured-effects or environment plans.
Point-sensitive mutable environments, recursive convergence and common native
callable carriers remain open.


## Point-sensitive mutable environment red

The new unit witness
owned_factory_updates_are_point_sensitive_and_instances_stay_separate creates two
instances of a selector factory. Its first instance returns the original one-argument
function, switches its captured slot to the two-argument function, restores the
one-argument function, and returns each prior value. The second instance must
retain its independent two-argument slot. Expected invocation cardinalities are
1, 2, 1 and 2. Repeating the root request must reuse requests and environment
identities. The current solver returns no outcome, so the witness remains red.
The independent Node reference verifies cardinalities and results 7, 9, 7 and 9.

This witness targets source-flow infrastructure. It is not native acceptance of
assigning differently typed polymorphic functions to an unconstrained A parameter.
Native type compatibility and capture modes must still be verified separately.
Do not infer ownership from the witness name or a false mutable flag.

### Implementation sequence

1. Isolate a flow-state module containing local bindings and a finite map from
   stable environment handle to capture state. Store allocation identity and graph
   edges separately. Capture state includes binding facts, known availability and
   whether a handle denotes one proven instance or merged instances. Test state
   joins, exact restoration, distinct handles, unknown handles and consumed handles.
2. Replace binding-only state transport in summary_flow::Outcome, expression
   Invocation, node transfer and graph edges with the explicit flow state. Preserve
   separate normal, returning and throwing exits, pending return snapshots and
   finally overrides. Existing dispatch implementations must preserve incoming
   environment state instead of silently discarding it.
3. Include canonical flow-local environment state in contextual request keys.
   Read capture facts from that request state; the global environment store supplies
   identity and dependencies, never a strong sequential update. Join summaries
   monotonically. Preserve Unknown alternatives and reader scheduling.
4. Project callee capture updates back onto the matching handle on each exit.
   Permit replacement only for a single instance with checked access facts; merged
   instances join weakly. Resolve local borrowed captures through the caller binding
   identity, and retain transferred escaping captures in their own instance state.
5. Feed checked capture facts into contextual analysis before allowing mutable
   snapshots to certify calls. Test local borrow, escaping scalar copy, escaping
   owned transfer, callback failure and disposal with actual generated native code.
   Keep the existing mutation/conflict guards until this evidence is available.
6. Make the new red pass, add recursive and branch-join witnesses, then editor
   diagnostics/help, guide examples and matched workload support. Run eight depth
   canaries and the complete editor crate at acceptance. Keep timing separate.

The previous factory transport prerequisite remains committed at 1c22457.


## Flow-local state kernel WIP

generic_call_environment_state.rs now isolates local bindings and a canonical
handle-to-capture map from allocation records. Capture entries are live, consumed
or unknown. A caller-provided single owned instance permits exact replacement;
merged instances retain prior and incoming alternatives. Single-instance
consumption records consumed state; merged-instance consumption records unknown
availability instead of claiming all represented instances were disposed.
Missing, consumed and unknown entries cannot be updated or reinstalled.

Capture slot identities are canonicalized and validated before updates. Branch
joins preserve missing and consumed possibilities, reject mismatched slots without
partial mutation, and discard conflicting scalar contract certainty. Tests cover
restoration, separate handles, weak updates, non-revival, availability joins and
commutative, associative and idempotent joins. The caller still must prove checked
ownership and unique instance identity; the enum is not itself that proof.

The module is compiled but not yet wired into expression dispatch or summary
states. The mutable factory sequence witness from dc4a184 remains red. Next thread
this state through summary exits, expression Invocation, node transfer and graph
edges, then request keys and captured updates. Keep guards until capture-mode
facts and native controls prove their removal. No new language behavior is accepted
by this internal kernel, and no new timing or editor acceptance is claimed.
