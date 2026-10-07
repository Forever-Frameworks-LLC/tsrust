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
