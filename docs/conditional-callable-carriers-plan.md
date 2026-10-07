# Common callable carriers

## Scope and invariants

Complete conditional selection and replacement across distinct function origins
without changing ownership. Preserve parameter access, omission/rest cardinality,
result types, capture identity, callback failures and disposal. A common contract
is not permission to merge captured owners. Keep native carrier construction
separate from source evaluation and transfer checking. No implicit cloning or
shared capture cells are introduced.

## Current owned conditional implementation

The retained conditional-arrow red now executes. Concrete, nonborrowed,
nonpolymorphic checked alternatives use the existing callable storage registry.
Their environments retain separate origins. The inferred common contract becomes
consuming if either branch is consuming; a reusable alternative is adapted once
without rebuilding its captures. Incompatible argument or result contracts remain
errors. Borrowed alternatives now use a separate scoped family described below.
Polymorphic alternatives remain open, not erased into this owned carrier. Template-only bodies do not certify a concrete layout.

checked_callable_storage.rs converts already checked values without rerunning the
source expression. conditional_callable_carriers.rs constructs the common contract.
The compiler exports checked conditional expression facts. The editor includes
source grouping delimiters inside a checked expression without swallowing caller
delimiters, preserving signature help and result chains.

Native witnesses cover fresh arrows, saved aliases, parameters, copied scalar
captures, consuming factory results, incompatible signatures, callback failures
and original-owner disposal after success and abandonment. Four selected original
record owners dispose exactly once at optimization levels 0 and 3; unselected
branches create no owners.

The matched generic adapter workload adds two independently observed conditional
results and now retains 129 observations. Its oracle, token guards, hash registration
and existing run/profile/report hooks are ready. Timing stays pending.

## Remaining gates

1. Preserve borrowed alternatives in a native carrier with their actual lifetimes
   and read/mutate/consume access. Do not make a local borrow own its source to fit
   the current stored-family constructor. Include native borrow conflict and escape
   rejections, selected receiver mutation and disposal of owning alternatives.
2. Transport polymorphic declarations without choosing a fabricated monomorphic
   signature or flattening per-witness invocation contracts. Preserve arity and
   capture identity after selection, replacement, aggregates and factory transport.
   The literal_guard_native_callable_replacement_has_common_carrier red remains.
3. Connect checked capture-mode facts to the source environment solver through an
   explicit early/late validation boundary. Track unique and merged instances in
   flow-local state. Repair the sequential restoration red with native-compatible
   witnesses before removing read-only/conflict guards.
4. Cover recursive groups, loops, defaults, rest arguments, exceptions and final
   completion overrides. Keep convergence finite without an iteration cap or
   recursive environment graphs embedded in request keys.

Run focused tests, eight depth canaries after dispatch changes and the complete
editor crate at each acceptance boundary. Keep matched benchmark support ready
and correctness evidence in SQLite. Accept the complete feature only when borrowed
and polymorphic cases and replacement are also verified. Gate 173 stays open.

## Borrowed carrier gate started

The retained witness
`borrowed_conditional_carriers_preserve_selected_local_capture_mutation` currently
fails at the two readonly array-capturing alternatives: their closure origins
are distinct, and the conditional join still requires identical origins. The
same witness also defines two local mutable scalar captures. Its intended native
output is `1 2` followed by `15 40`, including repeated invocation and observation
of the original locals after the selected handle leaves its scope. No production
implementation for this gate has been added.

The current storage admission explicitly rejects borrowed environments. Merely
removing that guard is invalid: callable_dispatch.rs emits concrete origin fields
without a lifetime-bearing family contract, and readonly stored dispatch means
shared ownership rather than a local borrow. First introduce a separate checked
borrowed-family contract and native representation that carries each alternative's
actual capture lifetime and invocation access. Keep source capture loans attached
to the selected expression through scope exit. Verify readonly/mutable alternatives,
conflicting access, escapes and selected-branch construction independently before
extending mixed owning/borrowed alternatives. Preserve the existing owned-family
admission controls. A generic native sum of concrete environment alternatives is
a candidate to investigate; do not choose dynamic dispatch or cloning solely to
make the witness pass.

The red and formatting evidence are byte-verified in SQLite under
`runtime/diagnostics/borrowed-conditional-carriers/`. Whole-project formatting
changes zero files and fingerprints. The accepted owned prerequisite is commit
30c92f1. All 15 owner-edited fixtures remain unstaged.

## Borrowed native dispatch prerequisite

borrowed_callable_dispatch.rs now emits a static two-alternative generic sum.
Each alternative retains its concrete environment type, including Rust lifetime
parameters. Readonly and mutable invocation implement the existing ReadCallback
and Callback protocols, respectively. Count-aware dispatch forwards the original
argument value and supplied count to exactly one alternative. Results, including
errors, pass through unchanged. The sum introduces no heap allocation, unsafe
code, shared cells or cloning. Consuming contracts remain unsupported by this
reusable dispatch prerequisite; their source semantics must be addressed before
the full carrier feature is accepted.

Three focused tests pass. Generated native execution at optimization levels 0 and
3 verifies both alternatives, repeated readonly/mutable invocation, count forwarding,
callback failure without mutation, and two original owners dropping exactly once
after their scoped borrowers disappear. Generated Rust rejects a reference escaping
its local owner and replacement of a mutably borrowed local before invocation.
The independent Node reference verifies the retained source witness's intended
`1 2` and `15 40` results. Whole-project formatting changes no files or hashes.
Exact red and green logs are stored under
`runtime/diagnostics/borrowed-dispatch-kernel/` in SQLite and restored byte for byte.

This module is not wired into source analysis or emission. The Tsrust witness is
still red. Next introduce a checked borrowed-family identity separate from owned
StorageContract admission; retain environment-origin alternatives and invocation
access. Carry actual concrete generic environment types into native use sites,
add the call adapters expected by existing expression emission, and preserve source
capture loans across conditional joins and scope exit. Audit return/aggregate
escape checks before treating a family as a generic stored value. In particular,
Type::StoredCallable currently denotes owning storage, so reusing it without
checking every ownership and escape consumer would conceal a borrowed lifetime.
Do not remove the existing owned environment rejection to reuse the owned emitter.
After integration, run the retained source/native/editor witness, borrow conflict
and escape controls, eight depth canaries, the full editor crate, and matched
workload correctness before acceptance.

## Checked borrowed-family bridge prerequisite

borrowed_callable_contracts.rs validates actual CheckedClosure alternatives. It
retains both origin IDs, parameter input types/access/omission/rest cardinality,
result type and the strongest reusable invocation access. Different default
expressions remain in their original bodies; the common contract retains omission
cardinality without replacing either default. At least one environment must borrow
its source. Missing origins, template/polymorphic bodies, consuming invocations and
incompatible signatures are rejected. Owned-only alternatives continue to belong
to the owned storage path. Origin compaction updates both alternatives atomically
and rejects missing origins or accidental merging of distinct environments.

Five new focused tests exercise checked readonly and mutable environments, default
parameters, incompatible arity/result/argument access, unchecked bodies, consuming
invocation and transactional origin remapping. The bounded `borrowed_callable_`
unit selection passes nine tests, including the native dispatch tests and an
existing exception-state escape control. The original focused red is retained in
SQLite with the green batch and whole-project formatting result, under
`runtime/diagnostics/borrowed-family-contracts/`. Formatting changes zero files.

This checked metadata is not a source capability yet. Connect it by introducing
a distinct scoped family identity in syntax.rs and Analyzer/CheckedProgram; do
not silently mark it as owned Type::StoredCallable. conditional_join must register
the checked alternatives and attach source capture loans once. Native emission
must construct the selected concrete generic environment and provide call/count
adapters. Add both origin dependencies to error_effects and origin compaction.
Return, throw, aggregate and factory paths must reject invalid escapes using the
actual capture lifetimes. Export the common callable contract through projections
and editor signature help. Only then rerun the source witness and acceptance
checks. The source red, mixed lifetime/ownership cases, consuming borrowed cases,
polymorphic carriers and environment restoration remain open.

## Borrowed source integration

The retained borrowed conditional source witness now runs with its intended
`1 2` and `15 40` results. Type::BorrowedCallable identifies scoped families
separately from owning Type::StoredCallable. The existing registry stores their
structural callback contract and the distinct concrete origin pair. Its borrowed
identity lookup is constant time, and origin compaction updates the identity keys
as well as the origin lists. Borrowed contracts cannot certify owned storage.

borrowed_conditional_carriers.rs checks each alternative once and registers the
common contract. Conditional flow retains both branches' capture loans. A direct
call constructs a family only for distinct alternatives; repeated invocation of
the same origin keeps its previous receiver borrowing behavior. Return and throw
paths reject scoped borrowed escapes. Owning callback transfers are rejected;
matching scoped readonly/mutable callback parameters are supported.

borrowed_callable_emit.rs emits the concrete lifetime-bearing native sum and
call/count methods. Each arm forwards to its original environment. The common
error contract joins both origins. callback_family_result_adapters.rs converts
that checked error payload when the family crosses a callback parameter boundary.
The source/native witness caught the missing callback error dependency and the
initial native Result mismatch; both are repaired. Projections export the common
signature for editor hover, parameter help and result chains.

Native checks cover readonly/mutable selection, repeated calls, defaults, direct
conditional invocation, callback parameters, failures through those parameters,
borrow conflicts and scoped return rejection. Original record owners dispose
exactly once at optimization 0 and 3 after their scoped handles disappear. The
matched workload adds two observations of readonly selection and mutation of
original locals, retaining 131 values. Its oracle and mutation guards are updated.
Full feature acceptance still requires consuming borrowed cases, polymorphic
carriers and replacement, recursive environments and the remaining captured-effects
plan. General nullable/aggregate and higher-order builtin transport of scoped sums
also needs explicit coverage before claiming parity for those paths.

## Scoped consuming parameter acceptance

Consuming borrowed carriers and concrete scoped Once parameter inference now have
focused acceptance. See milestones/173-scoped-once-parameters.md. Native family
identity accounting, owned failure adapters and higher-order input compatibility
are verified. Public syntax remains Once. Scoped nullable/aggregate and builtin
transport, polymorphic replacement and mutable environment restoration remain open.
