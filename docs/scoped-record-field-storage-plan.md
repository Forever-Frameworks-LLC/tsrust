# Independent storage for scoped record fields

## Problem and verified evidence

The independent-field disposal witness now passes Tsrust analysis but fails native
Rust E0502. Emission already moves the left Closure0 out of Record0; it does not
clone it or create a reference for that extraction. Later use of the remaining
aggregate keeps the left capture lifetime relevant to native borrow checking.

A diagnostic probe used the current compiler library to emit a minimal equivalent
program. Its declaration is Record0<Closure0, Closure1> and its extraction is a
move from field0. Destructuring that declaration into two independent native
bindings, and selecting those bindings directly, compiles and runs with output
`3 11` followed by `11`. This experiment verifies the representation direction;
it is not an implementation or feature acceptance.

## Representation and invariants

Keep nominal record identity and public ownership rules. Introduce a dedicated
emitter module for field places. Use binding IDs and numeric field paths for
native names; source names must not determine storage identity. Compute layouts
from checked record instances and nominal declaration field order. Keep shared
recursive dispatch small.

Evaluate each initializer once. Independent native bindings own the original field
values. Consuming extraction moves those values. Borrowing references the selected
place. Never clone implicitly, retain a second aggregate owner, use unsafe lifetime
casts, or rely on a runtime empty placeholder to release a native lifetime.

Static nested record paths need compatible layouts. Sum types, nullable records,
indexed containers and dynamic selection require explicit representation support;
do not flatten them by guessing a variant. Track these composition boundaries
separately and preserve their current diagnostics until their own support is proven.

## Implementation milestones

1. **Diagnosis complete.** Preserve original generated Rust and successful native
   experiment in SQLite. Retain the original failing project witness unchanged.
2. **Field-place layout foundation implemented.** The isolated layout module
   `scoped_record_places.rs` has three focused tests. Emitter integration remains
   pending. The module identifies eligible owned
   scoped record storage and stable native names. Preserve field order, nested
   paths, nominal identity and cycle/depth limits. Borrowed parameters and borrowed
   aliases remain references to their original record storage.
3. **Declaration rendering foundation implemented.** The layout module renders
   nested declaration patterns and whole owning reconstruction. A native component
   test proves evaluation and disposal without Clone. Main emitter wiring remains
   pending. Destructure eligible local declarations
   after evaluating their initializer once. Owned parameters may destructure at
   function entry while retaining their native record ABI. Establish mutability
   from checked writes. Deferred declarations need the same definite-assignment
   behavior as ordinary bindings.
4. **Selected value rendering foundation implemented.** Numeric paths select
   independent leaves or reconstruct a selected static subtree. Seven component
   tests pass, including native capture release after a selected callback move.
   Main emitter routing and stores remain pending. **Selected access and stores.** Route record field reads, consuming moves,
   mutation, restoration, nested destructuring and self-assignment through the
   layout module. Preserve assignment evaluation order and failure behavior. Turn
   the existing independent-field native witness green with both live-field
   rejection controls unchanged.
5. **Whole-record view rendering foundation implemented.** Shared and mutable
   renderers borrow independent leaves. A native component test proves string
   storage identity, in-place mutation and subsequent owning reconstruction. View
   declarations also propagate generic field slots through nested records. Six
   focused component tests pass, including mutable callback environment borrowing.
   The nominal borrowing bridge generator now supports owned, shared-view and
   mutable-view carriers through associated leaf types and read/mutate accessors.
   Native component checks cover callback bounds and shared-view mutation rejection.
   Field-position metadata supplies distinct generic slots even for equal callback
   contracts. Owned carriers may use generic child records with separate read/mutate
   child constraints, associated leaf types and accessor expressions. Repeated child occurrences receive separate parent generic arguments while
   sharing one nominal child declaration. Ten component tests pass.
   An isolated emitter adapter now derives view/bridge support from checked nominal
   records and actual native ABI slots. Native execution verifies distinct nested
   callback environments against compiler-generated owned records. Ordinary type emission now registers support with separate read-view, mutable-view
   and bridge caches shared across nominal roots. All 64 depth controls pass; the
   record project batch retains 19 passes and the known independent-field E0502
   failure. Parameter/field routing and independent storage integration remain pending.
   **Whole-record transport.** Reconstruct owning values from field places only on
   a checked whole move. Borrowed whole-record consumers need a compatible view
   or bridge that references original fields and supports existing callable bounds.
   Whole mutation and replacement must commit fields after RHS checking and
   evaluation. Do not introduce a temporary owned copy to simulate borrowing.
6. **Captures and completion.** Cover record capture environments, return/throw
   transport where permitted, finally replacement and disposal. Retain independent
   lifetimes when an owned record becomes a closure environment. Account for
   pending payload loans without keeping discarded environments alive forever.
7. **Control-flow safety.** Verify branch-only moves, field restoration, loop
   continuation/break, exception cleanup and outer assignments. Native execution
   must demonstrate both release after last ownership and continued conflict while
   a callback remains live. Include nontrivial payload disposal assertions.
8. **Record acceptance.** Complete editor signature/hover/diagnostic coverage,
   executable guide examples and a matched benchmark family with Node reference,
   independent oracle, source contracts, registration/driver hooks and manifest
   fingerprints. Run the whole editor crate before acceptance. Preserve bounded
   evidence in SQLite and update milestone, coverage, roadmap and handoff records.

## Verification and checkpoint rules

Use the existing project/native witness first, with formatting round-trip and
std:testing assertions. Add focused red witnesses for each new transport boundary.
Keep accepted positive record, parameter, factory, assignment and destructuring
cases. Run all eight depth canaries after analyzer/emitter dispatch changes, and
focused exception/loan controls when ownership changes. Avoid routine full compiler
or benchmark matrices and do not claim measurements for unmeasured workloads.

The declaration/owning renderer checkpoint is a7efe63. The subsequent view
renderer component has five passing focused tests; main emitter integration remains
pending. Installed consumers remain f1dd284.
Record gate and gate 173 remain open. Preserve all fifteen owner fixtures and use
explicit Git paths. A partial representation milestone does not close the record
feature family.
