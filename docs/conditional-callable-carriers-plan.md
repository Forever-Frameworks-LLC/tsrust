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
errors. Borrowed and polymorphic alternatives remain open, not erased into this
carrier. Template-only bodies do not certify a concrete layout.

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
