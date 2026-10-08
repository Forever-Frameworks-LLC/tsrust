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

Thirty-three focused composition tests cover native execution after outer-scope
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
confirms that source arrays remain transferred. Push and scalar record replacement
through indexed nested arrays now widen the root container's leaf environment.
Carrier planning rewrites its initial nested literals and incoming records to that
layout. Insertions attach new capture owners to the root storage conservatively;
the rejection witness uses an owner absent from the initial literal. The native
execution witness checks retained and replaced origins, and the Node reference
checks the same results. Existing splice, unshift, ordinary and owned array suites
passed 32 tests before the capture-owner attachment repair.

Nested array-valued push and indexed replacement now join their leaf callback
environments. Indexed pop and splice results participate in storage planning through
the root array while retaining their own nesting depth. Result expressions are
retyped before carrier conversion, avoiding a reproduced borrowed-origin emitter
panic. A rejection witness verifies that an indexed nullable removal retains its
capture after the source container's scope ends. The affected composition, pop/shift,
splice, unshift and owned-array batch passes 49 tests. The expanded Node result
reference passes after correcting an expected value affected by its preceding removals.
All eight depth canaries pass on this batch; their original log is verified in SQLite
as `tsrust-b5-nested-array-writes-depth.log`. The focused editor protocol witness
`scoped_record_array_composition_keeps_declared_callback_help` passes for nested
indexing and a narrowed nullable removal, preserving the declared parameter label
and hiding internal carrier names from hover.

Record construction now materializes concrete record-array field types rather than
discarding their callback environments behind a declaration-only field. Field-array
mutation widens that field in the root record, and record carrier joins include
array leaf environments. Array carrier planning resolves field paths and keeps
removed nullable payloads consistent. Native construction, push and pop pass; a
paired rejection confirms capture protection after the root record's scope ends.
The composition, scoped-record, field-move and array-field batch passes 99 tests.
Both focused array composition editor witnesses pass, including parameter help and
hover through `box.items[0].run`. The expanded independent Node reference passes.

Mixed storage paths with a record-field prefix followed by array indices now retain
that field prefix when widening callback layouts and attaching insertion captures.
The native witness combines push, nullable removal and indexed replacement through
`box.groups[0]`. A paired rejection protects an owner introduced only by insertion.
The composition, array-field and field-move batch passes 59 tests, and the expanded
Node reference passes. This does not establish a record field accessed after an
array index, such as `boxes[0].items`, or arbitrary alternating storage paths.

Indexed records containing callback-bearing array fields now support construction,
push, nullable removal and indexed record replacement through `boxes[0].items`.
Storage paths preserve field names across intervening indices. Iterative type
rebuilding propagates a changed field layout through its enclosing records and
arrays. A focused red ownership witness exposed capture loss on removal through
such a path; capture transport now retains its owners without consuming sibling
payloads. All 103 affected composition, scoped-record, field-move and array-field
tests pass, and the independent Node reference passes. Broader combinations of
alternating paths still require operation-specific evidence before acceptance.

Named local Maps now specialize record-value layouts when `set` receives borrowing
callback records. A separate Map carrier module retargets the empty constructor and
stored values to the final concrete environment layout. Native Map aliases carry
the payload type as a generic argument. Two distinct record origins can coexist and
be invoked after `get`; source capture conflicts and local Map escape are rejected.
The affected composition, Map and Map-forEach batch passes 41 tests, and the expanded
independent Node reference passes. Map support remains incomplete: earlier get
results followed by later widening, moved Maps, nested storage, borrowed iteration,
snapshot restrictions, precise removal/replacement release and disposal still need
their own evidence and implementation.

The owner prefers Rust-style collection semantics that reduce side effects and
unexpected behavior. Plan record Set keys around field equality and hashing:
separately constructed records with equal supported fields represent the same key.
Admit only records whose fields have consistent equality and hashing, and prevent
mutation of stored keys from invalidating those contracts. Define eligible field
types and floating-point behavior explicitly before implementation acceptance.
Callback fields have no automatic value equality; records containing callbacks
remain rejected as ordinary Set keys. Their B5 collection use needs an explicit
stable-key design, which remains pending rather than accepted through rejection.
Map values can contain callbacks because values are not hashed. Map value work can
continue independently. JavaScript object identity is not the Set reference model.

The focused `map_get_before_later_insertion_keeps_callback_environment` witness
exposed a compiler panic in optional-element emission. It reads and calls the first
entry inside a completed block, then inserts a different callback origin and reads
the second entry. The repaired Map planner propagates the final value carrier into
earlier `get` results, nullable bindings, projections and callback targets. Borrowed
result types retain their view representation. The native witness passes, and a
paired rejection still reports `view-conflict` when mutation overlaps a live view.
The affected compiler batch passes 42 tests. The focused editor witness preserves
the declared parameter name in signature help and hides internal carrier names in
hover. The independent Node reference passes. These are untimed checks, not B5
acceptance or performance evidence.

Map moves now propagate a common value carrier backward through source bindings
and forward through destination bindings until the move chain stabilizes. This
keeps construction and insertion layouts compatible without copying Map storage
or captured owners. A two-move native witness verifies both callback origins after
later insertion. Paired diagnostics verify capture conflicts after moving and
rejection of source use after transfer. The affected compiler batch passes 44 tests;
the updated editor witness and independent Node reference also pass. Nested Map
storage and precise removal/replacement release remain pending.

Record fields now retain concrete Map value layouts at construction and widen them
on insertion. The Map planner follows root bindings and record field paths, including
move edges inside record literals and existing carrier wrappers. It propagates the
final layout backward into source Maps and forward into field reads, nullable
projections and callback targets. Maps and captured owners are moved without copying.
The previously failing `record_fields_hold_maps_with_borrowing_callback_values`
witness passes natively. Paired rejection witnesses preserve newly inserted capture
conflicts and reject record escape with local borrowed captures. The affected compiler
batch passes 46 tests, focused editor help and the independent Node reference pass,
and all eight normal-worker depth canaries pass. Project formatting changes no files
or benchmark fingerprints. Depth evidence is preserved under
`tsrust-b5-record-map-fields-depth.log` in SQLite. This remains B5 work in progress;
Map storage behind array indices, field replacement, precise removal/replacement
release, disposal and matched benchmark support still need evidence.

Remaining B5 work includes broader nested method composition,
nullable replacement across multiple origins,
precise element capture release, Map/Set
composition and nullable replacement. Complete editor, guide, disposal and matched
benchmark support before accepting the feature. Keep native execution and ownership
rejection witnesses paired; do not permit a container to outlive borrowed captures.

The native disposal witness
`scoped_aggregate_callback_environments_drop_once_on_each_storage_boundary`
attaches Drop observers to seven generated borrowed closure environments and
verifies their exact output order at Rust optimization levels 0 and 3. It covers
Map replacement, deletion, clear and scope exit; array splice removal and pop result
scope exit; and a Map moved into a record that is then moved again. Every environment
drops exactly once at the expected boundary. The source passes the shared formatter
round trip. This proves these normal disposal paths, not failure-path disposal or
precise compile-time capture release after individual removal. Those requirements
remain pending. Optimization-level checks are correctness evidence, not timing.

The companion failure disposal witness
`scoped_aggregate_failure_disposes_environments_before_catch_and_finally` invokes
throwing borrowed callbacks from an array and a Map. Drop observers verify that
each environment is disposed exactly once before its catch body and finally body
execute, at optimization levels 0 and 3. Both normal and failure witnesses share
the same native observer helper and exercise formatter round trips. This covers
these checked error propagation paths; it does not prove release of capture loans
while a surviving container remains in scope. That analysis requirement remains
open, as do broader composition, Set key design and matched benchmark support.

Matched workload `scoped_record_composition` now supplies B5 array and Map coverage.
Each iteration retains eight scalar observations from array pop followed by a new
callback origin, a Map moved into a record field, field insertion, nullable reads, whole-field replacement,
clear and capture release at scope exit. Its TypeScript counterpart uses the same
scenario; an independent arithmetic oracle derives the expected values without
using collections or callbacks. Source contracts enforce both fingerprints,
scenario operations and the unchanged operation denominator. Registration covers
ABI, oracle/checksum routing, runner/profiler provenance hooks, report placement
and rejection of semantic source mutations even with refreshed hashes.
Three affected registration tests pass. Native and Node-after-tsc verification
passes all five harness cases; exact original evidence is preserved in SQLite as
`runtime/results/2026-10-08T15-41-01-965Z-verify.json`. No timing samples were taken.
The manifest row is appended at the end and remains explicitly unmeasured, with
whole B5 implementation acceptance pending. Broader benchmark coverage must follow
the remaining composition and Set decisions as those features land.

Map record-field replacement now checks against the nominal declared field contract
and updates the concrete local field layout. The existing carrier planner joins
old and replacement origins through both source Maps and the field destination.
The native witness verifies calls before and after replacement. Paired diagnostics
preserve replacement capture owners and reject replacement while an entry view is
live. The affected compiler batch passes 50 tests; focused editor help and all eight
normal-worker depth canaries pass. The matched composition workload adds replacement
as its eighth retained observation, with independently updated oracle expectations
and explicit semantic source hashes. Source acceptance is still separate from B5
acceptance and timing. Precise old-owner capture release is not established here.

The expanded workload exposed a missing earlier insertion origin after whole-field
replacement. The Map planner now includes every checked `set` payload before
joining move edges, preserving origins that the final point-sensitive field type
no longer contains. The corrected workload passes all five native/Node-after-tsc
cases; exact evidence is in SQLite as
`runtime/results/2026-10-08T15-47-01-119Z-verify.json`. Current depth evidence is
`tsrust-b5-map-field-insertion-replacement-depth.log`; the earlier passing depth
run remains separately preserved as `tsrust-b5-map-field-replacement-depth.log`.
No timing was run. All 50 affected compiler checks and focused editor help pass
after the repair. Whole B5 acceptance remains pending.

Borrowed Map `forEach` composition now has focused evidence. Native witnesses
invoke record callbacks before and after a later insertion widens the carrier,
verify insertion-order accumulation, propagate an inner callback failure without
visiting later entries, and reject receiver mutation during iteration. Focused
editor help preserves the declared record callback parameter name and hides
internal carrier names. The matched workload adds iteration as its ninth retained
observation; its arithmetic oracle expects the sum of the two original entries.
Registration and all five native/Node-after-tsc cases pass. Exact verification
evidence is preserved in SQLite as
`runtime/results/2026-10-08T15-50-08-110Z-verify.json`. This establishes these
iteration paths without a new compiler change or timing run. Precise capture
release, broader nested storage, Set design and whole B5 acceptance remain open.

Map storage paths now pass through array indices as well as record fields. The
planner shares the existing array path lookup and reconstruction helpers: indices
are ignored conservatively for layout planning, while field names remain explicit.
Native witnesses cover insertion and invocation through `Box[]` and `Box[][]`, and
an indexed insertion rejection preserves the new capture owner. Local nominal
field contracts and concrete field updates now traverse array layers too, supporting
whole-Map replacement through an indexed record field. The affected compiler batch
passes 54 tests, focused editor help passes, and all eight normal-worker depth
canaries pass. The matched workload exercises the indexed path for insertion,
iteration, nullable reads, replacement and clear while retaining nine observations.
This does not establish independently precise capture release for separate runtime
indices or every aggregate construction shape. Those broader B5 requirements remain
open. Current depth evidence is `tsrust-b5-indexed-record-map-fields-depth.log` in
SQLite; matched verification remains untimed.

The combined indexed workload exposed two additional replacement defects. Indexed
field writes were missing from immutable write collection, and replacement capture
loans remained attached to their consumed source Map instead of the destination
container. Write collection now uses the shared indexed storage path. Indexed
replacement stages and consumes its RHS owners, then merges them into the
conservative destination path without discarding captures owned by sibling indices.
A focused native witness verifies release when the container scope ends, while
capture-conflict rejection remains covered. The final affected batch passes 55
compiler tests; editor help and all eight depth canaries pass on the repaired source.
All five native/Node-after-tsc cases pass. Exact matched evidence is preserved as
`runtime/results/2026-10-08T15-58-44-726Z-verify.json` in SQLite. The final depth log
is `tsrust-b5-indexed-map-replacement-loans-depth.log`; the earlier depth run is
preserved separately. Precise release of an individual replaced element while
siblings remain live is still pending. No timing or whole B5 acceptance is claimed.

Array construction now joins record fields containing independent scoped Maps.
The join requires equal key types and compatible record callback contracts, then
materializes a common value carrier. Existing move-edge planning propagates that
carrier back into each source Map; no Map or captured owner is copied. A native
witness constructs two Box elements with distinct callback origins, inserts a
third origin into one element and verifies all three results. Paired rejection
checks protect each independent capture owner and preserve incompatible-key errors.
The affected compiler batch passes 56 tests, with the additional key/capture guard
test checked separately. Focused editor help and all eight normal-worker depth
canaries pass. The matched workload retains the peer Map result as its tenth scalar
observation. All five native/Node-after-tsc cases pass; exact evidence is in SQLite
as `runtime/results/2026-10-08T16-03-04-914Z-verify.json`, with depth evidence under
`tsrust-b5-independent-record-map-elements-depth.log`. Timing and whole B5 acceptance
remain pending, including precise per-element capture release and Set key design.

Focused growth witnesses now verify independent scoped Map environments inserted
into record arrays through `push`, `unshift` and `splice`. Each case checks both
callback results through the project/native path after formatting and verifies
formatter idempotence. Existing carrier planning handles these paths without a
compiler change. This evidence does not establish precise per-element capture
release, Set key support or whole B5 acceptance.

A removal witness verifies that popping a record containing a scoped Map retains
its concrete callback environment while the remaining array element stays usable.
A second form transfers the removed record into an outer nullable binding and
invokes its Map callback after the array scope ends. Both formatted project/native
forms pass without a compiler change. This proves those ownership transfers, not
precise capture release while unrelated sibling elements remain live.

Local Map `clear` now discards the cleared payload capture owners and restores
only loans still held by other live owners. A red native witness previously
reported capture-conflict when mutating the former capture after clear; it now
passes. A shared-capture rejection ensures another live Map still protects its
capture. Parameter and captured-reference receivers, and indexed paths sharing
conservative ownership storage, are excluded from this release change. All eight
normal-worker depth canaries pass. The affected batch passed 59 checks; its new
rejection helper initially used unsupported direct-check imports and was repaired
and checked separately. Broader precise release and B5 acceptance remain pending.

Clear-release coverage now includes a named record with two independent Map
fields. Clearing one field permits mutation of its former capture while the other
field remains callable. A conditional-clear rejection verifies that branch joins
retain captures from the uncleared path. Focused editor diagnostics agree: direct
clear is accepted and conditional clear reports capture-conflict. Both compiler
and editor witnesses pass; native record-field execution passes after formatting.

### Clear/refill carrier lifetime defect

The focused `cleared_map_refill_tracks_only_current_capture_owners` witness was
initially red. Clearing a local Map releases its checked capture owners, but
refilling that same Map with a different callback origin retains the first
capture lifetime in the shared native carrier type. Mutating the cleared first
capture before reading the new entry passes analysis and fails generated Rust
with E0502. The second entry must remain protected until it is cleared too.

The required fix must separate concrete carrier lifetimes across clear/refill
cycles, rather than suppressing the Rust diagnostic or restoring conservative
source rejection. Plan point-sensitive Map carrier epochs together with storage
emission: a completed clear ends the old payload lifetime, later insertions start
a fresh concrete environment, and moves, record fields and control-flow joins
must preserve the current environment. Native assertions must verify old-capture
mutation, new-entry invocation and subsequent new-capture release. Retain the
shared/new capture rejection and existing conditional-clear protections. The
current witness includes the native success requirements and a paired new-capture
rejection; its native half fails before the rejection runs.

This is a B5 defect, not an accepted limit. Earlier clear-only evidence remains
valid for its covered paths, but does not prove clear/refill support. No timing or
whole B5 acceptance is claimed.

The straight-line local-declaration case is now repaired in Map emission. A clear
statement for a scoped Map declared in that same block drops the old storage and
shadows it with fresh empty storage of the planned carrier type. Fresh lifetime
inference allows the new callback origin to borrow independently of discarded
captures. The helper stays in `map_emit.rs`; shared statement dispatch only calls
it. Outer-block storage, parameters, record fields and conditional expressions do
not use this transformation. The refill witness and its new-capture rejection now
pass. The affected compiler/native batch passes 61 tests, including normal and
failure disposal checks. Cross-block and field refill remain open B5 requirements.
