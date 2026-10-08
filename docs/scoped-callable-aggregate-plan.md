# Scoped callable aggregate plan

This continues Gate173 after scoped traversal and nullable transport. The goal is
local record and array storage of callable families that borrow their environments,
with ordinary lifetime checking and no implicit clone. Existing explicitly owning
callback storage contracts must remain owning. No new public ownership wrapper is
planned. Start from failing source/native witnesses before choosing internal layout.

## Record gate

First establish record admission with separate failing witnesses. Ordinary local
record literals currently require a named type; anonymous record inference is a
separate dependency. Named ReadOnly/Mutable callable fields currently fail with
callback-escape at the declaration. Decide and document a scoped field contract
before changing admission, preserving existing explicitly owning field contracts.
Then establish a local record containing a scoped conditional callable and invoke
the field while its captured owner is live. Cover readonly and mutable
captures, a moved record/field handle, a nullable field and a consuming field that
cannot be called twice. Reject return, throw and opaque retaining transfers of the
record when a field borrows local storage. Verify that conflicting owner access is
rejected while the record is live and permitted after its scope ends.

Inspect record inference and callable field registration first. Distinguish an
actual scoped field from an explicitly owning declaration. A borrowed family must
not be relabeled StoredCallable to reuse existing layout. Native record layout must
carry the concrete environment type and its real lifetimes through field projection.
Keep source lifetime/loan facts separate from native layout metadata. Original owned
record field behavior, nominal identities and clone restrictions must remain intact.

Acceptance requires std:testing project witnesses, an independent Node counterpart,
native disposal checks at optimization levels 0 and 3, editor field hover/call help,
matched workload observations and guards, the affected suites, eight depth canaries
and the editor crate. Unknown lifetime facts remain rejection conditions.

## Array gate

Extend local dense arrays to scoped callable elements after record field facts are
reliable. Cover construction, moving an element through pop/shift, nullable element
results, indexed invocation, for-of and repeated readonly/mutable invocation. Include
consuming elements, branch selection and abandoning arrays before invocation. Arrays
own their element handles; they do not own the values those handles borrow.

Add a distinct checked scoped element representation rather than weakening owning
ArrayElement::StoredCallable admission. Preserve element identity and capture loans
through indexing, transfer and container scope exit. Reject whole-array escape,
retaining transfers and mutations that invalidate a live borrowed element. Do not
claim homogeneous storage for incompatible callable contracts. Verify original-owner
disposal, failure payloads and public editor types before accepting the array gate.

## Composition gate

Join scoped records/arrays with nullable fields and nested aggregates. Resolve
callable coalescing across distinct families using a checked common carrier that
preserves all concrete alternatives, invocation access and lifetimes. No fabricated
monomorphic signature, dynamic dispatch or clone may hide incompatible origins.

Cover defaults/rest, loop exits, exceptions/finally, partial field transfer and
recursive type layouts with finite traversal and convergence. Benchmark support must
land with each accepted behavior, including matched sources, independent oracles,
source hashes, registrations and existing run/profile/report hooks. Timing remains
pending until a meaningful completed family milestone. Full Gate173 still requires
polymorphic replacement and point-sensitive mutable environment restoration.

## Record implementation boundaries

The first persistent reds are scoped_callable_records.rs: a named readonly field
with repeated invocation and original-array reuse, and a named mutable field that
updates the original capture and releases its loan at scope exit. Both initially
failed at declaration admission. Their independent TypeScript assertions pass in
scoped-callable-records-reference.test.mjs. Anonymous inference is not needed for
these witnesses and remains a separate dependency.

Keep declaration contracts distinct from constructed environment layouts. Today
record_layout resolves every field through storage_declaration, which rejects
borrowed protocols; record_literal binds owning fields through stored_callable_value.
Introduce a scoped field resolution path without weakening storage_declaration's
ownership guarantee or manufacturing a borrowed family before concrete alternatives
are known. Construction must specialize the record layout against checked concrete
field environments while retaining its nominal source identity. Shared declaration
metadata cannot be mutated according to whichever instance is constructed first.

Native record emission currently writes a nongeneric Record struct using declaration
field types. Scoped construction needs concrete generic environment parameters and
lifetimes, propagated through field projection, moves and nullable payloads. Put
layout specialization in a dedicated module, separate from declaration resolution
and capture-loan analysis. Two instances with different families must coexist.

Current return/throw guards inspect a direct nullable callable payload. Add bounded
aggregate traversal over checked field identities so wrapping a borrowed handle in
a record cannot bypass those guards. Audit parameter retention and whole-record
assignment too. Source loan provenance must follow field moves and be released only
when all live handles end. Test escape and conflicting owner mutation before accepting
field admission, including nested and nullable fields. Do not rely on a later Rust
compilation error to enforce a source lifetime contract.

The aggregate escape traversal prerequisite is implemented in aggregate_callable_facts.
It supplies retention, scoped-storage and unknown-layout facts independently. Return,
throw and fresh-callee loan release now share this traversal. Concrete record layouts
and field-level provenance remain the next implementation work; source construction
is not accepted by this prerequisite.

Concrete direct scoped record construction is implemented in the current work-in-progress
checkpoint. Read docs/handoff-2026-10-07-scoped-record-layout-in-progress.md for verified
behavior and outstanding lifetime, nested-layout, nullable-absence and acceptance work.
The complete record gate remains open.

The record gate is accepted: see milestones/173-scoped-callable-records.md. The
array and composition gates remain open. Their shapes are rejected at source level
with an `unsupported` diagnostic that names arrays, Maps, Sets and unions.

## B5 continuation after gate 181

`tests/scoped_record_composition.rs` now contains a failing array witness with two
distinct readonly closure origins. It exercises indexed invocation, traversal,
nullable `pop` and `shift` results, and original-owner mutation after the container
scope ends. The independent TypeScript reference in
`tests/fixtures/scoped-record-composition/node-reference.ts` passes on the pinned
Node runtime. The compiler currently rejects the array at the explicit guard in
`scoped_record_containers.rs`. No compiler admission rule has been changed.

Before this continuation, the array analyzer retained the declared element type while allowing
nominally compatible concrete record instances through `require`. Each record
literal separately specializes its concrete callback field environment in
`scoped_record_layout::Registry`. Admitting the array therefore needs a common
checked element layout that preserves those distinct environments; removing the
final container guard alone does not establish correct native layout or source loans.
Reuse existing checked callable and record carriers where their guarantees apply.
Keep container layout specialization separate from ownership provenance.

The current uncommitted prerequisite now joins concrete record array elements with
`join_record_types`, wraps differing element environments with existing record
carriers, and preserves nominal annotated array compatibility. The container guard
still rejects the positive witness. Existing arrays (12), scoped records (42) and
record boundary tests (7) pass. This establishes no source acceptance yet: capture
provenance, transferred element layout and nullable results still need verification
before admission can change. The red-witness checkpoint is `b5ee6ff`.

Capture-loan attachment now traverses array construction and propagates possible
owners through indexed aliases and pop/shift results. Removing one element does
not release owners of elements remaining in the array. The extracted-record witness
checks that a popped record keeps its capture conflict after the source container's
scope ends. Dynamic element identity is currently conservative: aliases retain all
possible container captures. Admission is still guarded, so this is an unaccepted
prerequisite; refine identity/release precision and verify generated native lifetimes
before accepting the composition gate.

Native admission now accepts checked concrete record instances inside containers;
declaration-only layouts remain rejected. The array witness passes native project
execution, including its nullable pop/shift results. Emission preserves generic
nominal record-array aliases and keeps scoped union payloads explicit instead of
writing illegal anonymous lifetime defaults. Existing arrays, data unions, optional
fields, scoped records and record boundary suites pass after this change.

The prerequisite checkpoint is `9576720`. Admission and emission repairs remain
uncommitted and unaccepted. Before the B5 gate closes, add mutable/consuming and
escape witnesses, disposal checks, insertion/replacement behavior, precise element
identity/release, Map/Set and nullable composition, editor and benchmark evidence.
Run the dispatch depth canaries and editor crate for the completed batch. The prior
notes describing guarded admission document the earlier prerequisite state.

Mutable indexed record callbacks now have a passing native witness: they update
the original capture repeatedly and release its loan at container scope exit.
Returning an array that hides a local capture borrow correctly reports
`callback-escape`. Array push now attaches inserted payload capture owners without
discarding existing element loans.

A new persistent red exposes insertion layout widening: pushing a second concrete
callback origin into an initially single-origin array reports `Expected Holder,
found Holder`. `array_mutation` currently checks its argument against the concrete
current element rather than the nominal declaration. The repair must join incoming
record environments, preserve both origin-specific loan facts, and plan one widened
carrier for the initial literal and all earlier/later uses. Updating only the push
argument type would leave earlier field calls dispatching through obsolete origins.
Reuse the record sequence planner's monotone storage propagation in a dedicated
array carrier module; include push, unshift, splice and indexed replacement.

The shared write collector now records concrete record-array initializers and
push payloads, distinguishing element insertion from whole-storage assignment.
Existing record planning filters insertion facts so it cannot treat an array
element write as replacement of the containing record. All 42 scoped record
regressions pass. The array carrier planner still needs to consume these facts;
the second-origin insertion witness remains intentionally red.

The insertion red now passes. `record_array_carriers.rs` plans named local array
storage from checked writes, widens initial elements and pushed records into one
carrier, and updates indexed field dispatch. The witness invokes an element both
before and after the new origin is inserted. All five composition witnesses, 12
array tests and 42 scoped record tests pass. Capture ownership remains separately
tracked; the planner performs no clone. This implementation currently covers named
local push storage; other insertion methods, nested storage paths, branch/loop
propagation and precise dynamic element identities remain open. The preceding
work-in-progress checkpoint is `f88e92d`. B5 is not accepted yet.

The dispatch canaries caught stack growth from adding a second planner call directly
to the recursively retained block analyzer. Routing array planning through the
existing record storage planner, with a small no-inline eligibility check, restores
the `for_of` depth canary. Keep new heavy planning frames outside recursive block
dispatch; do not raise thread stack limits to hide this regression.

Current array composition supports named local storage with distinct callback
origins, indexing, traversal, push/unshift, indexed replacement and splice.
Removed arrays retain capture protection after the source container ends. Whole
array reassignment preserves the concrete environment layout, including deferred
outer storage. Moves between named arrays propagate a common element carrier
back through their storage dependencies; subsequent insertion can widen that
carrier without generating incompatible native types.

Twenty focused composition tests cover native execution after outer-scope
assignment, insertion after a move, both branch insertion paths, and invocation of
removed records after their source array widens. Splice results participate in the
same storage dependency plan as moved arrays. Pop and shift results now update their
nullable payload and narrowed field dispatch when the source array widens; this
repairs an emitter panic reproduced by the focused witness. Repeated insertion from
a loop preserves both the initial and loop-created callback origins. The expanded
independent Node reference passes. The latest affected pop/shift, data-union and
composition batch passed 33 tests, followed by the new loop execution witness.
Nullable removal values also move into another local and into deferred outer
storage while retaining their concrete payload layouts. Contextual union injections
are retargeted alongside the underlying removal result. A paired rejection witness
checks that reassigned nullable storage still blocks mutation of its borrowed owner.
The latest combined composition, boundary, union and removal run passes 43 tests.
Capture-conflict and local-escape witnesses
remain enforced. These checks establish this array storage batch, not B5 acceptance.
Prior completed diagnostic and depth records remain verified in SQLite.

Nested array literals now join callback environments across their leaf records.
The transformation uses an iterative traversal, and native nested array types retain
the concrete callback type arguments at every array level. Distinct-origin indexing
and capture release at scope exit pass native execution; paired rejection witnesses
check capture conflicts and local escape. The existing 24 ordinary and owned array
tests also pass. Independently stored arrays can now move into nested storage with
distinct callback origins. Carrier conversion consumes the source arrays and moves
their records into the common layout, without cloning records or captured values.
The native witness and independent Node result check pass; a rejection witness
confirms that source arrays remain transferred. This does not yet establish insertion
or replacement through nested storage.

Remaining B5 work includes nested mutation, nullable replacement across multiple origins,
precise element capture release, Map/Set
composition and nullable replacement. Complete editor, guide, disposal and matched
benchmark support before accepting the feature. Keep native execution and ownership
rejection witnesses paired; do not permit a container to outlive borrowed captures.
