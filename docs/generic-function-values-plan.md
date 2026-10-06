# Generic named function values

This is the remaining generic-value part of gate173, not a replacement for the
full nested-function plan. The rest-value checkpoint is 2f4ad5d. Generic values
must preserve existing ownership, source identity and explicit clone requirements.

## Current evidence

The four generic_function_values integration tests are genuinely red at the
unsupported-value boundary. Their strict TypeScript counterparts pass native Node
execution after tsc. They establish these observable requirements:

1. A capture-free generic declaration can be assigned to a value and called with
   unrelated scalar and owning array types.
2. A captured declaration can be assigned to different fixed callable contracts.
   Context supplies a specialization; its parameter/result relation still holds.
3. An inferred factory can return a generic value. Separate factory calls retain
   separate state, while each returned value shares state across specializations.
4. A local captured value shares its one mutable environment across int and string
   calls. Its borrow ends at the existing scope boundary, allowing the owner to
   observe the exact call count afterward.

The factory output must be exactly 11, 21, 12 on separate lines. The other three
fixtures use borrowing assertions and have no output. Do not replace these tests
with arrow forms or weaken their ownership assertions.

## Existing architecture and required extension

local_function_values currently wraps only capturing non-generic declarations.
Header rejects type parameters. expression_name rejects generic template names,
including a contextual fixed callable use. function_value separately guards named
capture contexts. These are distinct dispatch boundaries; removing one guard does
not implement values.

CheckedClosure already has an origin, a captured environment and specialized
methods. Its current polymorphism substitutes callable parameters, not arbitrary
generic data slots. An unannotated arrow data parameter requires context and is
not an unconstrained generic placeholder. Simply replacing T with an inferred
arrow parameter therefore does not establish a polymorphic value.

Use a dedicated generic_function_values module for source metadata, construction
and specialization. Retain the original generic declaration symbol, type-parameter
annotations, parameter access, omission/rest contracts, source spans and capture
binding identities. Reuse generic_functions and generic_inference for solving and
materializing the original body. Keep generic preparation outside ordinary
recursive operator frames. Never evaluate source arguments during inference.

A generic value has one runtime environment, not one environment per selected
argument type. Specialized native entry points refer to that environment. Keep
method identity separate from environment identity. Do not clone owned captures
when a second method is requested. Borrowed values retain their existing loans;
escaping values transfer their original owners exactly once.

## Iterative implementation

First implement contextual value construction. A concrete callable expectation
supplies a checked argument/result relation. Solve the declaration's type variables
using that relation, materialize its concrete signature, then form the existing
capture wrapper and check it against the requested callable contract. Preserve
required/optional/default/rest arity, stated access and result compatibility. Retain
a real red witness before each extension. Contextual construction is an interim
checkpoint; it does not complete generic values or permit marking gate173 accepted.

Then implement an origin with deferred data specialization for unannotated values.
Each call solves the original template using actual checked argument hints and
result context. Reuse the method cache and its existing finite bounds. Store explicit
source metadata for unresolved data positions; do not let Type::Void become a
wildcard in ordinary type checking. Inspect template emission and type facts so
unresolved placeholders can never reach native emitted signatures.

Discover captured binding types at construction using lexical identity. Keep the
access inference fixed point separate from type solving. Audit methods whose
specializations read, mutate or consume captures differently: the same owner must
become unavailable after a consuming invocation, while reusable invocations must
not invent new owners. Use direct native witnesses rather than assuming the current
callable-parameter specialization already handles these cases. Never introduce
shared cells or an implicit clone to sidestep a conflict.

Finally support escape and contexts throughout storage, calls and editor facts.
Returned generic factory values must retain one environment across int/string
methods and independent environments across factory calls. Check aliases, record
and array storage, callback arguments, explicit type application if supported by
callable syntax, generic defaults/rest, callback-valued parameters, parameter
patterns and inferred result boundaries. Preserve SourceSymbol provenance across
instances so navigation and diagnostics name source declarations rather than
synthetic wrapper identifiers. Keep exact source loans and readiness checks.

## Required acceptance witnesses

Add invalid same-T argument/result relations, wrong explicit contexts, arity errors,
required extras and incompatible rest element types. Add borrowed and consuming
owned arguments, once captures, rejection after transfer, exact native disposal at
O0/O3, callback failure propagation and recursive generic boundaries. Verify
specialization counts and ordinary test-worker depth without increasing the stack.

Editor checks must cover hover, completion, parameter help, inferred call result
facts and source navigation for both contextual and polymorphic values. Preserve
existing hidden-wrapper span repairs; do not exclude every same-span expression.

Add a matched workload at manifest end as text after implementation lands. Its
independent oracle must observe multiple types, exact shared call counts, separate
factory state, owning arrays and contextual callback results. Complete native/TS
sources, fingerprints, mutation guards, registration, native ABI and all
oracle/run/profile/report hooks. Use --verify for routine native/Node-after-tsc
checks. Keep full gate173 implementationAccepted false until its complete scope
and audit pass; no routine timing.

Run the eight dispatch canaries after analyzer/parser changes and the whole editor
crate before acceptance. Persist original failed and repaired evidence in SQLite,
verify readback, remove loose copies and prune obsolete executable scratch at
handoff. After generic values, continue the wider gate173 inference/context audit
and the full named-declaration family milestone; these requirements remain intact.

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

## Deferred data specialization boundary

The next implementation must extend data-slot substitution rather than reuse the
current callable substitution unchanged. materialize_closure_method_types replaces
Type::Callback parameter slots and optional callback payloads. Its source Context
has already checked data bindings; an unresolved generic data parameter cannot be
passed through that code as an ordinary void binding. Introduce explicit deferred
source metadata for the original declaration and parameter positions, then solve
its template before constructing the concrete method context. Keep argument
preparation separate from argument evaluation and preserve supplied-count metadata.

The native emitter's closure_method currently selects its receiver from the global
closure.invocation. Audit that boundary when methods require different access to
the same retained owners. Do not silently make every invocation consuming because
one materialized method consumes. A mode-dependent extension needs source flow and
native receiver evidence; the existing callback methods' uniform contract is not
proof that arbitrary data specialization is covered.

Two further native witnesses are red, with committed strict TypeScript counterparts
passing Node-after-tsc. array-patterns checks T[] inference, mutation of both int[]
and string[] callers and exactly two writes to one borrowed captured counter.
rest-prefix checks a T fallback, defaulted int offset, explicit undefined and T[]
rest across scalar/string calls, with exactly three writes to one environment.
Keep these alongside the three original polymorphic red tests. Their passing Node
references do not establish Tsrust implementation acceptance.

## Invocation replay module checkpoint

Invocation argument-type collection and method materialization now live in
`crates/tsrust-compiler/src/closure_methods.rs`. The extraction preserves the
original bodies exactly, except for sibling-module visibility. It isolates the
method cache, substitution, source replay and captured-layout checks before
introducing deferred generic data metadata. It changes no supported behavior and
does not close the five pending native witnesses or gate173.

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

A focused invalid-annotation witness exposed false structural compatibility with
the empty prototype: an unannotated identity value could be assigned to () => void.
Fixed-contract adaptation is now rejected explicitly until a real adapter exists.
Assigning the original generic declaration to a concrete contract still uses the
verified contextual path. The guard is isolated outside shared recursive checking
frames. It does not reject an unchanged origin through nullable admission.
