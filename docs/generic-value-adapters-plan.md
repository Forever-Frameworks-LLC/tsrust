# Generic value adapters

This implements the remaining callable adaptation portion of gate173 and
docs/generic-function-values-plan.md. It does not replace the wider context audit
or the named-declaration feature-family milestone.

## Required behavior

An already constructed generic value can acquire a concrete callable contract.
The adapter retains that value's existing environment. It does not reconstruct
the original declaration, copy mutable scalar state or clone owned captures.
Mutable and read-only adapters borrow the original environment under existing
loan rules. An owned adapter transfers that environment once. A consuming call
retains the existing Once rules. Separately constructed factory values remain
independent.

The same mechanism must support explicit annotations, callback arguments and
typed record/array storage. Context solves the original generic argument/result
relation. Original parameter access, omission, defaults and rest contracts remain
checked. Constructing or solving an adapter must not evaluate invocation arguments
or default initializers. Incompatible relations and arity remain errors.

## Current red boundary

generic_value_adapters.rs contains three native witnesses and matched strict
TypeScript fixtures under generic-value-adapters. contracts.tsr checks a mutable
borrow followed by a generic call and an owned string adapter of the same factory
value. Its captured counter must print 11, 12, 13. callbacks.tsr checks two map
invocations followed by a generic string call, with the same counter output.
storage.tsr checks a record and an array using separate factory instances, with
counter output 11, 21, 22. Return values use std:testing assertions.

The first and third fail at deferred_generic_requirement. The callback witness
fails at materialize_closure_method because a deferred origin has no ordinary
checked source replay body. All three TypeScript references pass after strict
compilation. These are requirements, not implementation acceptance.

## Implementation seams

Add a dedicated generic_value_adapters module. Invoke it when a checked name or
value has a deferred generic origin and context supplies a real concrete callable
contract, before ordinary require rejects the incompatible empty prototype.
Do not remove that guard globally, treat void as a wildcard, or select a cached
method merely because its argument types happen to match.

Use the original value_signature solver to establish source type relations and
the contextual callable contract to build an adapter wrapper. The wrapper calls
the existing generic value binding, not the original declaration name. Its capture
is the existing runtime environment. Reuse normal closure checking, concrete
deferred invocation and native closure emission where they preserve that identity.
Keep construction and solving outside shared recursive expression frames.

Audit escape classification explicitly: synthetic wrappers are created after
the initial source escape walk. Owned adapters must transfer the original value;
borrowed adapters must retain its exact loans. Establish the synthetic wrapper's
escape behavior from the contextual callable ownership instead of assuming its
source span is already in the escape set. Do not permanently mark an unrelated
source origin escaping.

Forward optional/default/rest inputs using their existing checked representation.
The source declaration chooses its defaults at invocation, exactly once. Retain
source declaration and alias navigation, expose the concrete adapter signature
and omit unresolved empty prototype facts. Contextual array callbacks need a
real checked wrapper replay body, while generic calls through the original value
must continue to select methods on the same retained environment.

## Gates and evidence

First make explicit borrowed/owned contracts pass and add ownership rejection
controls: reuse after transfer, simultaneous mutable access and consuming reuse.
Check capture-free/result-only/default/rest/parameter-pattern cases and failures.
Then integrate callback and aggregate contexts without weakening their contracts.
Add exact disposal controls at O0/O3 for successful and caught-failure paths.

Add editor controls for concrete adapter hover, completion, parameter help,
source identity and diagnostics. Update the guide. Complete a matched workload
family appended at manifest end, including independent result oracle, source
fingerprints, mutation guards, registration, ABI/driver and oracle/run/profile/report
hooks. Run representative untimed native/Node-after-tsc checks. Run all eight depth
canaries after dispatch changes and the whole editor crate before acceptance.
Preserve original red and repaired evidence in SQLite with byte-exact readback.
Keep full gate173 open until its complete plan and context audit are fulfilled.
