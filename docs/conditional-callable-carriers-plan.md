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
