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

## Initial implementation checkpoint

generic_value_adapters.rs now solves concrete source contracts and builds normal
checked wrappers around an existing generic value binding. A private lexical
alias prevents source parameter names from shadowing that binding. Closure capture
projection preserves lexical aliases as well as binding declaration names.
Synthetic owned wrappers receive temporary escape classification during checking;
borrowed wrappers retain existing loans. The source compatibility guard remains
for contexts that have not received a real adapter.

Array transform callbacks obtain source parameter hints from the supplied element,
position and receiver tuple. Their checked adapter is consumed by callback analysis
once, without rebuilding its environment or rechecking after the private lexical
alias is removed. Reduction, search and other callback paths still need integration.

The three original native witnesses now pass. Transfer, exclusive borrow, consuming
reuse and incompatible type/access controls pass. Three retained owners drop
exactly once at O0/O3 across successful and caught-failure paths. All eight depth
canaries and whole editor checks pass, including concrete parameter help and alias
navigation. Adapter-specific matched benchmark support and the broader default,
rest, pattern, nullable and other callback audit remain pending. This is a source
implementation checkpoint, not gate173 or full adapter acceptance.

The matched adapter workload now also exercises nominal generic results. Its first
native check exposed unresolved Receipt<T> applications in solved wrapper
signatures. Adapter construction now materializes substituted result and parameter
annotations through the same record cache used by ordinary specialization, then
registers concrete checked fields. A focused native witness confirms borrowed
integer and direct string calls share one retained counter. Broader verification
is tracked in the current benchmark handoff; this does not close the full plan.

Reduction and sorting now reuse the traversal adapter helper. Reduction excludes
unresolved deferred prototypes from parameter/result hints and solves accumulator
context from the initial value or element type. Sort supplies two element types.
Search methods already route through array_transform and retain that adapter.
Focused witnesses verify reduction followed by a different generic call, stable
sort/toSorted behavior and stateful findIndex followed by a direct call. Matched
benchmark expansion for these paths remains pending; the existing workload is
not evidence of their coverage. An exploratory inferred record literal containing
two callable fields failed for lack of a named record context; that distinct
boundary remains in the audit and its original diagnostic is retained.

Matched benchmark expansion now covers the implemented reduction/search/sort paths.
Twenty-six observations retain independent expected values and captured-state
checks, without requiring equal comparator counts across native and Node engines.
Five untimed cases and semantic mutation registrations pass. Remaining context
and ownership audits still prevent full adapter acceptance.

Default/rest forwarding audit exposed spread planning against an unresolved empty
prototype. The adapter now forwards its already formed rest array using the checked
RestPack representation. RestPack checking accepts a checked array expression as
well as a literal, retaining the supplied access mode and expected type. It does
not clone the pack or evaluate source elements again. The retained fixture checks
omission, explicit undefined, an explicit override, empty/nonempty packs and a
subsequent different generic instantiation of the original environment. Native and
strict TypeScript output agree. Owned rest/disposal, patterns, nullable contexts
and benchmark expansion still require their separate audit before full acceptance.

Owned-rest audit now observes six distinct record owners disposed exactly once at
native O0/O3 across successful and caught-failure calls. Additional controls reject
reuse of the transferred generic environment and of an owned rest element. A new
editor protocol witness preserves concrete optional/rest parameter labels, three
parameter slots and navigation to the original alias declaration. Matched default/
rest workload expansion and the remaining context audit are still owed.

The matched workload now includes default and rest effects. A default factory
checks omission, explicit override and explicit undefined through a concrete
borrowed adapter, then invokes the original generic value with a string. A rest
factory checks empty and nonempty packs, then invokes its original environment
with a different element type. Thirty-seven independent observations retain prior
callback/storage coverage. Semantic guards reject default replacement, changed
undefined forwarding and reconstructed rest environments. Verification evidence
is recorded in the current handoff; full context acceptance remains separate.

## Fresh expression staging

The result-only and nullable named-adapter witness passes without compiler changes.
An owned Function<int> annotation applied directly to make(10) still fails at the
polymorphic compatibility guard. The retained factory-expression fixture requires
both direct annotation and map(make(20)). Its strict TypeScript reference prints
10,11,12,20,21,22, proving each factory runs once and one environment serves both
subsequent invocations. This is a red requirement, not accepted support.

Implement expression staging in a dedicated helper and checked representation.
Check the source expression once without imposing the concrete callable contract
on its unresolved prototype. Retain its deferred origin and original argument/
result relation. Give the resulting environment a temporary binding visible only
while constructing the normal checked adapter. Owned contexts transfer that
binding into the wrapper; synchronous traversal contexts retain it through the
callback and release it afterward. Reject borrowing an ephemeral environment into
an escaping borrowed result. Restore temporary lexical names on both success and
failure. Keep effect checking and runtime evaluation separate: never replay the
source expression to obtain an origin, never reconstruct a factory and never clone
captures. Emission must evaluate the environment before adapter construction in a
single scoped expression, then preserve the existing wrapper capture layout.

Audit all checked expression consumers for the new staging representation:
source traversal/remapping, ownership roots, escape classification, checked-type
facts, native emission and cleanup. Tests must cover evaluation once, multiple
calls, factory failure before adapter creation, callback failure after creation,
owned disposal, borrowed escape rejection and editor navigation/signatures.
Run canaries and whole editor after dispatch changes. Extend the matched workload
and independent oracle before accepting expression adaptation.

## Initial fresh-expression implementation

A dedicated generic_expression_adapters module stages fresh direct declaration
calls through ordinary checked closure/block nodes. Declaration result preparation
obtains the deferred origin without evaluating invocation arguments. The staged
block binds the source call once, constructs the existing concrete adapter from
that temporary, and returns it using a structured callable result annotation.
Traversal contexts solve the original source contract from supplied parameter
hints, then use the same staging path. Normal checking and emission own capture
transfer, loans, failures and cleanup; no additional opaque emitter representation
or source replay is required. The compatibility guard remains elsewhere.

The original direct annotation and map(make(...)) witness passes with one factory
execution per expression. O0/O3 disposal observes three record owners exactly once
across success, callback failure and factory failure. Borrowed temporary escapes
report closure-escape. Editor navigation and concrete signature/result help pass.
Fresh local callable calls, generic factories, selected/conditional expressions,
stored callable contexts, omitted generic result annotations and nested callable
contracts still require their separate audit. This increment does not accept all
arbitrary expressions or the full adapter family.

Fresh origin discovery now uses checked call-result hints after preparing direct
named declaration results. Local factory aliases and selected callee expressions
with the same known origin use ordinary staging, preserving one evaluation and
capture identity. Eighteen adapter tests and all depth canaries pass; whole editor
checks and an added alias navigation/result-help control pass. The matched workload
has fifty-one observations and verifies local alias and same-origin selection
paths. Generic factories and selections with differing deferred origins still need
separate source solving and joining; do not describe them as accepted.

## Generic factory specialization and partial editor facts

Fresh generic factories now prepare specialization from type hints without
executing invocation arguments. The retained string/integer factory witness exposed
a second defect: nested generic method instances were cached by declaration and
invocation type arguments alone, so different captured types reused one checked
body. Captured specializations now include projected capture types in both checked
and source instance identities. Capture-free cache identities and existing limits
remain unchanged. Original native witness and matched workload pass after repair.

A new editor witness remains red: make('owner') still displays seed:T even though
the compiler has checked seed:string. Generic call projection drops the entire
fact when its returned deferred callable has no concrete result spelling. Fix this
with additive partial parameter-call facts, not a fabricated void result or a
fictional concrete callable shape. Collect source/callee/declaration spans and
checked parameter contracts independently of result display. Project those spans
with original module identities. Merge partial facts into editor help only when
no full unambiguous fact exists, and omit the result suffix when unknown. Preserve
ambiguity at shared specialization spans and nominal/access/default/rest metadata.
Keep legacy/full call facts intact; add focused source/project/editor tests before
accepting the generic factory increment. The existing failing editor witness stays.


Generic factory parameter help now consumes additive partial call facts when the
returned generic environment has no concrete display contract. Checked seed types,
parameter access and original declaration locations remain available. An unknown
result adds no result annotation; it never becomes void or an empty callable.
Conflicting specializations at one source span remain distinct and suppress
ambiguous editor overrides. Source and project projection tests cover string/int
variants and imported aliases with Unicode source offsets. Full Gate173 acceptance
and the remaining adapter contexts are still open.
