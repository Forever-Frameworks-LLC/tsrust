# Scoped callable aggregate plan

## Indexed replacement checkpoint, 2026-10-09

Known constant-index replacement in straight-line dense local record arrays now
releases the replaced callback capture through source ownership and a moving
native carrier epoch. Twenty affected array tests, exact disposal at optimization
levels 0 and 3, repeated replacement, later element borrowing, the focused editor
check and all eight depth canaries pass. The matched composition workload has
110 observations and passes all five cases untimed on native and Node-after-tsc.
Evidence and conversion costs are documented in indexed-capture-release-plan.md.

B5 remains incomplete. Next extend native epochs to record-field arrays and moves,
then precise removal and reaching slot states across control flow. Nullable and
same-expression compositions, structural record keys, family regressions,
performance review and installed-consumer refresh remain pending.


This continues Gate173 after scoped traversal and nullable transport. The goal is
local record and array storage of callable families that borrow their environments,
with ordinary lifetime checking and no implicit clone. Existing explicitly owning
callback storage contracts must remain owning. No new public ownership wrapper is
planned. Start from failing source/native witnesses before choosing internal layout.

## Current B5 scope and remaining work

B5 remains one unfinished feature-family task. Recent checkpoints establish dense
record arrays, nested array fields, scoped Map values and fields, nullable reads
and removal, moves, insertion-origin joins, borrowing iteration, exact disposal,
and local/record-field clear/refill lifetimes across blocks, branches and leading
infallible clears in exception regions. These checkpoints are
focused implementation evidence, not separate accepted gates.

Known Map entry release now has focused source, native and editor coverage through
branches, loops, exception phases and typed normal/error/return/control transport,
including controls created by finally. The latest affected batch passes 101 Map
cases and 27 completion-layout proofs. This does not establish indexed element
release or every aggregate composition.

Next implement precise capture release for individual indexed elements, then
reconcile remaining nullable/union, expression and outer-storage compositions
against the named requirements below. Implement record Set keys using the owner's
value equality/hashing direction; callback records require an explicit stable-key
design. Complete editor and guide coverage and run feature-family regression and
performance acceptance. The matched workload is current and untimed; executable
consumers still require acceptance refresh. Older implementation notes below are
retained history, including superseded gate status and next-step instructions.

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

The matched workload now includes local Map clear/refill, mutation of the old
capture before calling the new entry, and mutation of the new capture after the
second clear. Its eleventh scalar observation is independently derived as
`3 * value + 15`. Both source counterparts, exact fingerprints, oracle and
semantic mutation guards are current. Registration and all five native/Node-after-tsc
cases pass. Untimed evidence is preserved byte-exact in SQLite under
`runtime/results/2026-10-08T16-17-38-835Z-verify.json`. No timing or whole B5
acceptance is claimed.

Local record-field clear/refill now uses the existing independent owning leaves
to renew only the cleared Map storage. Static field paths preserve sibling fields
and their storage; indexed paths and outer-block declarations remain excluded.
The formatted native witness verifies old-capture mutation, refill invocation,
new-capture release and an unchanged sibling scalar. Its initially empty Map move
also exposed a planner join defect: a declaration payload and its matching concrete
record instance now join to that instance instead of requiring two concrete
environments. This preserves the nominal record contract. The affected 61-test
batch, eight depth canaries and focused editor diagnostics pass. Exact checks are
stored in SQLite as `tsrust-b5-map-field-refill-checks.log`. Broader cross-block
and indexed lifetime renewal remain pending.

The matched refill scenario now moves an initially empty Map into a named record
field and retains its sibling marker in the result. Clear/refill and capture
mutation proceed through that field. The independent eleventh observation is
`3 * value + 16`. Source guards, mutation controls and hashes are updated; all
five native/Node-after-tsc cases and registration pass. Untimed evidence is
byte-exact in SQLite under `runtime/results/2026-10-08T16-22-05-993Z-verify.json`.

Clear-only unconditional blocks now preserve lifetime renewal in the declaring
scope. Emission recognizes blocks containing only checked local scoped Map clear
statements, including nested clear-only blocks, and emits their resets in order
without a new binding scope. There are no declarations or control effects to
escape. Labels, mixed statements, conditional paths and indexed storage do not
use this lowering. The native witness covers direct Map and record-field refill
through these blocks. Broader cross-block lifetime transport remains required;
this repair does not substitute for that work.
The affected 61-test batch and eight normal-worker depth canaries pass. Exact
checks are preserved in SQLite as `tsrust-b5-clear-only-block-checks.log`.

Mixed unconditional blocks now transport renewed local Map storage back into the
declaring scope. Block planning gathers statically named clear targets without
traversing conditional or looping control flow. Emission retains the block as an
expression, renews the Map at its original clear statement, executes remaining
statements in order, and returns renewed storage in a tuple to the parent scope.
Nested blocks and independent record leaves use the same transport. Block-local
bindings keep their original scope; siblings are not reconstructed or copied.
Labels and control-flow statements prevent this lowering. Exception regions,
conditional joins and runtime-indexed storage still need their own lifetime work.

Native witnesses cover local declarations and assertions before clear, a size
observation after clear, and nested record-field clearing with sibling retention.
The focused witness was red with generated-Rust E0502 before this repair. The
61-test affected batch, all eight normal-worker depth canaries and focused editor
diagnostics pass. Exact batch/depth evidence is preserved in SQLite under
`tsrust-b5-mixed-block-refill-checks.log`. B5 acceptance remains pending.

Both branches of an `if` can now renew the same local scoped Map storage. The Map
emitter compares guaranteed clear targets from straight-line branch bodies. When
the target sets match, each branch returns its renewed owning leaves and the
parent binds their joined fresh lifetimes. Conditions evaluate once before either
branch; branch statements retain their order. The native refill witness chooses
the first branch on its first clear and the second branch on its second clear.
The one-branch capture-conflict rejection remains required and unchanged. Different
target sets, early exits, loops, exception regions and runtime indices remain
outside this transport and need further B5 work. The helper owns branch planning
state outside shared recursive emission.
The affected 61-test batch, eight depth canaries and focused editor diagnostics
pass. Exact checks are verified in SQLite as
`tsrust-b5-both-branch-refill-checks.log`. Formatting changes no files.

A multi-field native witness now clears two independently capturing Map fields in
opposite orders across the two branches. Both branch choices execute. Old capture
mutation, refilled callback invocation, sibling marker retention, an empty sibling
Map and final new-capture release are asserted. This checks that transport orders
its storage targets consistently without changing source disposal order. Existing
emission handles the witness without a source change.

The matched workload now chooses a clear branch from value parity before refill
and before final new-capture mutation. The arithmetic oracle remains unchanged;
source guards require the complete two-branch clear statements, so removing a
clear remains a semantic rejection even when the source hash is repinned.
Registration, focused multi-field native execution and all five matched
native/Node-after-tsc cases pass. Exact untimed evidence is verified in SQLite as
`runtime/results/2026-10-08T16-31-37-088Z-verify.json`. B5 remains incomplete.

Map storage lifetime renewal is isolated in `map_storage_epochs.rs`. Ordinary
Map operations and runtime type emission remain in `map_emit.rs`. The extracted
93-line helper block is byte-identical to its checkpointed source; only module
registration changes. All 21 scoped Map compiler/native tests and focused editor
clear diagnostics pass. This structural checkpoint adds no supported behavior,
benchmark epoch or B5 acceptance claim.

Guaranteed clear targets now propagate through nested conditionals within mixed
unconditional blocks. Both branch target sets must match at each checked join;
the containing block returns renewed storage as before. A focused native witness
previously failed with generated-Rust E0502, then passed after this planner repair.
It retains a local marker and observes empty Map size before refill. Both inner
branch choices execute across the two clears. The affected 62-test batch, eight
normal-worker depth canaries and focused editor diagnostics pass. Exact evidence
is verified in SQLite as `tsrust-b5-nested-branch-refill-checks.log`. Different
branch target sets, early exits, exceptions and indexed storage remain pending.

Sibling safety now has paired source and native evidence around field renewal.
Clearing one field on both branches permits mutation of its former capture and
refill from that same owner; the uncleared sibling callback still returns its
original value. Three rejection forms preserve the sibling capture through direct
branch renewal, nested-block renewal and a one-branch clear. Editor diagnostics
accept the released first capture and report capture-conflict for sibling mutation.
The focused native/source and editor tests pass without another compiler change.

Branch storage transport now supports different clear target sets. Planning joins
the touched owning leaves from both branches; each branch returns either renewed
storage or its original uncleared storage. Source capture-owner joins remain the
lifetime authority, so conditional clearing does not release captures on paths
where entries remain. The previous identical-target restriction caused native
E0502 even when one field was cleared on both branches and an independent sibling
was cleared on only one. The native witness now executes both choices, refills the
common field, mutates its released owner and checks the sibling's branch-specific
size. A rejection protects the sibling capture across the uncleared path.

The affected 63-test batch, eight depth canaries and focused sibling editor
diagnostics pass. Exact evidence is verified in SQLite as
`tsrust-b5-different-branch-target-checks.log`. Transport still needs consumption
and early-exit coverage in addition to exception and indexed storage work; this
checkpoint does not establish all control-flow compositions or B5 acceptance.

### Next repair: moved storage at block exits

`renewed_map_can_be_consumed_inside_its_clear_block` was initially red. Source
analysis accepts clearing a Map, moving it into another binding inside that block,
observing the moved Map's empty size, and mutating the former capture after the
block. Native transport incorrectly appends the original Map to its return tuple
after the move, producing E0382. The source witness uses std:testing and formatted
project execution. This is an emission defect, not an ownership restriction.

Block transport currently collects clear targets without an exit-availability
snapshot. Retain the checked availability of each owning storage path at block
and branch exits. Return only leaves that remain live at that exit; consumed leaves
still need their clear reset at its original statement, but must not be returned.
Retain sibling leaves independently. Do not use the function's final `transferred`
span alone: later assignment can restore a binding and erase its earlier moved
state. Cover local and field moves, a move followed by restoration, branches with
consumption, and exact disposal before accepting this repair. Keep existing
clear/refill and uncleared-sibling rejection evidence. No new compiler source was
changed for this checkpoint; B5 remains incomplete.

Block-exit availability is now retained in a dedicated analysis module,
`storage_exit_availability.rs`, and carried with checked functions and closure
methods. Each body exit records unavailable outer roots and moved field paths
before scope cleanup. Map transport consults that snapshot and returns only live
storage; clear resets still occur at their original source statements. An empty
transport tuple is valid when all renewed storage was consumed. Siblings retain
independent paths. This uses point state rather than final function transfer spans.

Native witnesses pass for consuming a renewed root, consuming a record field with
an untouched sibling, restoring a consumed binding after the block, and restoring
it before block exit. Restoration exposed a separate empty-constructor mismatch:
a typed empty Map now accepts a matching concrete scoped payload context while
preserving nominal key/value contracts. Nonempty constructor admission is unchanged.
The affected 64-test batch, eight depth canaries and focused sibling editor checks
pass; exact evidence is verified as `tsrust-b5-block-exit-availability-checks.log`
in SQLite. Cloned generic methods retain their checked exit snapshots. Compiler
unit-test compilation and the focused move witness are checked after that metadata
propagation adjustment. Branch consumption and exact moved-storage disposal still
need evidence; B5 remains incomplete.

Both-branch consumption now consults the checked body-exit snapshots. Renewed
Map storage consumed on both paths is omitted from the branch return tuple while
clear resets remain at their original statements. The native move witness executes
both branch choices and checks empty moved storage and subsequent former-capture
mutation. It was red with E0382 before emission used the snapshots. Storage live
on either path remains retained; one-path consumption still needs explicit
transport/disposal work rather than dropping the survivor early.
The affected 64-test batch, eight depth canaries and focused sibling editor
diagnostics pass. Exact checks are verified in SQLite as
`tsrust-b5-both-branch-consumption-checks.log`. Whole B5 acceptance remains pending.

One-path consumption now balances native branch storage with an empty internal
Map on the consumed path. Checked source availability remains consumed, so this
storage cannot revive the source binding. The other path returns its actual live
owner and preserves its scope disposal boundary. The empty Map contains no callback
payload and copies no capture; the runtime constructor uses empty Vec/HashMap
storage without reserving entries. Later checked assignment can replace either
path's storage normally. Both-consumed paths continue to omit storage entirely.

The move witness previously failed with E0382 and now executes both one-path branch
choices. An independent disposal witness executes a cleared/consumed path and an
uncleared survivor path at Rust optimization levels 0 and 3. Exact output proves
the cleared payload drops before the moved marker, the survivor drops after the
inside marker at its original enclosing scope, and internal empty storage adds no
callback drop. The affected 65-test batch, eight depth canaries and focused sibling
editor diagnostics pass. Exact checks are verified in SQLite under
`tsrust-b5-one-branch-consumption-checks.log`. Exception and indexed storage remain
open; this is focused implementation evidence, not whole B5 acceptance or timing.

The matched workload now consumes the renewed record-field Map on one branch
while retaining its empty storage on the other. Its twelfth scalar observation is
that selected Map size, independently expected to be zero. The sibling marker
remains part of the refill observation after the partial field move. Both source
counterparts, exact hashes and semantic mutation guards are current. Guards require
the move and its observer, as well as clear before subsequent capture mutation.
Registration passes. This adds matched support for checked one-path ownership
transport; measurement remains pending the completed family milestone.

The twelve-observation workload passes untimed native and Node-after-tsc verification.
Exact evidence is imported and read back byte for byte in SQLite as
`runtime/results/2026-10-08T16-52-55-903Z-verify.json`; its loose copy is removed.
No timing samples were collected. One-path consumption now has matched runtime
support in addition to the focused compiler, disposal and editor evidence.

Caught-refill witness: `map_clear_refill_survives_caught_failure` clears an
outer Map, refills it with a different captured array, and throws into a catch
that observes the surviving entry. The optional get view ends in its own block
before the final clear. Source analysis accepts mutation of the old array after
catch; generated Rust rejects it with E0502 because the original Map storage
lifetime was not renewed across the exception region. Exact failing evidence
is verified in SQLite as `tsrust-b5-caught-refill-red.log`. This is an open
compiler defect, not an accepted restriction. Next work must transport renewed
storage together with completion outcomes, preserve throw/catch/finally order,
and test failures before and after clear without losing live capture owners.

Leading exception-region clears now renew their native storage in the enclosing
scope before entering the remaining completion region. Only a contiguous prefix
of unlabeled direct clears is eligible. Clear is infallible; no statement or
observable effect is crossed. Catch/finally retain their checked contracts and
execution order. Refilled storage remains visible to catch and to later source
uses without retaining the old captured array borrow. Arbitrary effects before
clear and conditional exception paths still require general storage transport;
this prefix implementation does not accept the whole exception requirement.

Focused native witnesses cover thrown and normal completion, a record-field Map
with a surviving sibling, and exact old/new environment disposal before catch,
finally and subsequent mutation at Rust optimization levels 0 and 3. Editor
verification accepts mutation of the released old capture and rejects mutation
of the current capture. The affected 59-test composition suite, eight depth
canaries and focused editor test pass. Evidence is preserved in SQLite as
`tsrust-b5-leading-try-clear-checks.log`. Matched workload extension and broader
exception paths remain pending. B5 is not accepted and no timing was run.

Caught clear/refill now contributes the thirteenth matched workload observation.
Both sources clear an old captured environment, refill from a new one, throw,
observe size in catch, invoke the surviving entry, and mutate each former capture
after release. The independent oracle expects `3 * value + 28`. Source contracts
and rehashed mutation controls require the throw, catch observer and callback
result. The native workload now has a checked Result ABI; a focused adapter
witness was red before registering this family for fallible transport. Both
plain and fallible signatures are tested, with unrelated families still rejected.
Untimed native and Node-after-tsc verification passes. Exact evidence is verified
in SQLite as `runtime/results/2026-10-08T17-03-09-859Z-verify.json` and the loose
copy is removed. The manifest observation description now matches all thirteen
scalars. Whole B5 acceptance and timing remain pending.

General exception transport has a focused red witness:
`map_refill_after_fallible_prefix_preserves_both_completion_paths` calls a
fallible function before clear, renews/refills the Map on the normal path, and
clears/refills the original Map on the caught path. Both paths release the old
capture and preserve the new callback; the witness exercises both choices.
Source analysis accepts the program, but generated Rust fails with E0502.
Exact evidence is verified in SQLite as `tsrust-b5-fallible-prefix-red.log`.
Do not hoist this clear across the call: the caught path must retain the original
owner until its own clear executes. Next implementation must carry storage with
completion outcomes and preserve point-sensitive availability on each exit.

Straight-line try/catch storage transport now repairs the fallible-prefix witness.
The dedicated `map_exception_epochs.rs` helper carries normal storage separately
from thrown storage. Explicit throws, runtime failures and fallible Result calls
attach the current Map owners to the thrown outcome. Catch receives those owners
before executing its own statements. Each clear renews storage at its original
point; normal and caught paths return their surviving owners to the enclosing
scope. The initial owners move into the exception scope unconditionally so Rust
cannot keep an obsolete conditional implicit drop in the outer scope. There is
no clone or early clear.

Admission currently covers straight-line bodies with catch, live Map storage at
both exits, and no finally. Nested control, consumed storage and finally still
need their checked exit-specific transport. The existing leading-clear path
continues to cover its accepted finally cases. This is intermediate B5 progress,
not acceptance of all exception composition.

The first disposal instrumentation pass exposed E0502 from the residual old
owner; its original 60-pass/1-fail evidence remains in SQLite as
`tsrust-b5-exception-storage-checks.log`. After the unconditional scope move, all
61 composition tests pass, including both failure choices, callback invocation,
post-release capture mutation and exact disposal at optimization levels 0 and 3.
Eight normal-worker depth canaries and the focused editor test pass. The editor
accepts the old capture mutation and rejects mutation of the current capture.
Final evidence is stored as `tsrust-b5-exception-storage-repaired-checks.log`.
Matched coverage of the fallible-prefix case remains next; no timing was run.

The matched workload now retains fifteen scalar observations. Two separate
scenarios invoke a fallible prefix with false and true, then clear/refill on
normal or caught completion. Catch observes the original Map size before its
clear, so early disposal cannot preserve the oracle result. Independent expected
observations are `3 * value + 40` and `3 * value + 42`. Each scenario invokes the
new callback and mutates the old/new captures after release. Token-based guards
require both calls, both catch and callback observers, both mutation boundaries,
and both returned results; rehashed deletion controls pass. Native and
Node-after-tsc verification passes untimed. Exact evidence is verified in SQLite
as `runtime/results/2026-10-08T17-12-49-977Z-verify.json`; its loose copy is
removed. B5 family acceptance and measurement remain pending.

Finally transport has a confirmed red witness:
`fallible_prefix_map_refill_preserves_finally_storage` extends the normal/caught
fallible-prefix scenarios with cleanup that observes the surviving Map size.
Both outcomes must retain the new callback through cleanup, release the old
capture, and dispose the new capture at the later explicit clear. Analysis
accepts the source; generated Rust still fails with E0502 because the existing
storage helper excludes finally. Exact red evidence is verified in SQLite as
`tsrust-b5-finally-storage-red.log`. Next transport must carry pending completion
and storage through cleanup, including handler failures and cleanup overrides;
do not skip finally for abrupt outcomes or move its observations earlier.

Straight-line try/catch/finally transport now carries the normal or thrown
Map owner into cleanup. Catch failures retain their current storage until cleanup
executes. Cleanup normal completion resumes the saved error; cleanup failure
replaces it after disposing the saved payload. Incoming, handler and cleanup
contracts retain distinct native layouts. Error-storage tuples have explicit
native types even where a checked effect contract has no emitted throw.
Definitely abrupt cleanup does not require an injection for a superseded pending
error. The same helper continues to exclude nested control, consumed storage
and try/finally without catch; those requirements remain open.

The original finally witness now executes both choices with the new callback
visible after cleanup. Disposal instrumentation additionally checks handler
failure before and after its clear, cleanup clear, resumed failure, and cleanup
throw overriding the pending error. Exact output passes at optimization levels
0 and 3. All 63 composition tests, eight depth canaries, 15 exception-contract
tests and the focused editor diagnostic test pass. The editor preserves the
current capture while releasing the old one. Exact evidence is verified in
SQLite as `tsrust-b5-finally-storage-checks.log`; the loose copy is removed.
Matched finally coverage remains next. Whole B5 acceptance and timing are pending.

The matched workload now retains seventeen scalar observations. Normal and
caught fallible-prefix cases each execute finally, observe the refilled Map size
as one, and retain that separate cleanup result in the returned array. Source
contracts require each finally observer and the output fields; deletion controls
pass after rehashing. All preceding observations retain their independent oracle
values. Untimed native and Node-after-tsc verification passes. Exact evidence is
verified in SQLite as `runtime/results/2026-10-08T17-23-56-576Z-verify.json`; its
loose copy is removed. No timing was performed.

Nested exception branches have a confirmed red witness:
`nested_exception_branches_preserve_map_renewal` throws on one try branch,
clears/refills on the other, clears/refills inside a nested catch block, and
observes surviving storage in finally. Both choices execute and must release the
old capture while preserving the current callback. Source analysis accepts the
program; generated Rust fails with E0502. Exact evidence is verified in SQLite
as `tsrust-b5-nested-exception-red.log`. Next work must admit checked nested
blocks and branches while attaching storage to exceptions at their actual exit
points. Loops, consuming exits and try/finally without catch remain open.

Nested unlabeled blocks and if branches now participate in exception storage
transport. Admission and clear-target discovery use an explicit statement stack
while retaining the existing checked nesting limit. Block/branch Map renewal
recognizes throws as abrupt exits; the active exception region attaches the
current owner at the throw instead of treating it as a normal block result.
Loops, return/break/continue, nested try regions and consumed storage remain
excluded from this helper pending their exit-specific transport.

The committed red witness now executes both paths. Additional native variants
throw after clear/refill, observe storage through a cleanup branch, and preserve
a record sibling after nested Map field renewal. Disposal instrumentation also
executes a nested throw branch at optimization levels 0 and 3, retaining the
exact catch/finally drop sequence. All 64 composition tests, eight depth canaries,
15 exception-contract tests and the focused nested editor diagnostic test pass.
The editor releases the old capture and rejects mutation of the current capture.
Exact checks are verified in SQLite as `tsrust-b5-nested-exception-checks.log`;
its loose copy is removed. Matched nested-branch coverage remains next; B5
feature-family acceptance and measurement remain pending.

Matched nested composition now retains eighteen scalar observations. Both
fallible-prefix scenarios branch on value parity, renewing Map storage in either
branch; catch renews it inside a nested block and finally observes the survivor.
The added branch trace independently expects two for even values and four for
odd values, proving both branch choices execute across the verified inputs.
Prior seventeen results remain unchanged. Token guards count both call sites
and branch updates, so deleting one duplicated operation cannot pass merely
because the other remains. Rehashed mutation controls pass. Untimed native and
Node-after-tsc verification passes. Exact evidence is verified in SQLite as
`runtime/results/2026-10-08T17-30-36-281Z-verify.json`; its loose copy is removed.
B5 acceptance and timing remain pending.

Consumed storage has a confirmed exception-region red witness:
`consumed_map_on_normal_and_caught_paths_releases_capture` calls a fallible
function before clear, consumes the empty Map on both normal and caught paths,
and mutates the old capture after the region. Size observations are infallible
assignments; their assertions execute after catch, avoiding any fallible operation
after the normal-path transfer that could enter catch with unavailable storage.
Source analysis accepts the corrected program, but generated Rust fails with
E0382. Exact evidence is verified in SQLite as
`tsrust-b5-consumed-exception-red.log`. The earlier draft was correctly rejected
for a potentially throwing assertion after transfer; its diagnostic is retained
as `tsrust-b5-consumed-exception-current.log`. Next work must represent unavailable
owners at completion exits without reviving source availability or introducing
implicit clones. Both failure choices and disposal need native verification.

Consumed normal exits of exception bodies now consult the checked body-exit
availability snapshots. Each live Map returns its owner; each consumed Map
returns an empty internal native Map with no callback payload. This balances
completion storage without cloning captures or reviving the source binding.
Source diagnostics still reject later reads of that consumed binding. The same
normal-exit normalization applies to catch and cleanup bodies. Error exits after
transfer still require point-specific availability rather than body-exit snapshots;
this change does not claim those paths are complete.

The committed red witness executes both normal/caught choices and mutates the
released capture. A record-field variant preserves an independent sibling.
Disposal instrumentation proves exactly one old callback drop at each clear,
with no callback drops added by internal empty storage, at optimization levels
0 and 3. Observations inside the region are infallible size assignments; logging
and assertions occur after the region when storage has been consumed. All 66
composition tests, eight depth canaries, 15 exception-contract tests and the
focused editor test pass. Exact evidence is verified in SQLite as
`tsrust-b5-consumed-exception-checks.log`; its loose copy is removed. Matched
consumption coverage and point-sensitive error exits remain next. B5 remains
incomplete and no timing was performed.

Matched consumption coverage now retains twenty scalar observations. Two
additional scenarios consume cleared Map storage on normal and caught paths,
observe the moved size through infallible assignments, and mutate the old capture
after release. Independent results expect `value + 21` and `value + 22`. Token
guards require all four transfer/size observations, both final results and both
capture mutations. Rehashed deletion controls pass. Native and Node-after-tsc
verification passes untimed; exact evidence is verified in SQLite as
`runtime/results/2026-10-08T17-39-22-541Z-verify.json`, with its loose copy removed.
No timing samples were collected.

Error-after-transfer transport has a confirmed red witness:
`error_after_map_transfer_restores_storage_in_catch` fails either before transfer
or after clearing and consuming the Map. Catch explicitly restores an empty Map
in both cases; source analysis accepts the subsequent capture mutation and reads
of restored storage. Generated Rust fails with E0382 because the thrown payload
tries to carry the already moved owner. Exact evidence is verified in SQLite as
`tsrust-b5-error-after-transfer-red.log`. Next work must track availability at
each error-producing expression, including changes within a statement, while
keeping the body-exit snapshots for normal completion. Empty replacement may
also require renewing its native lifetime rather than retaining the old carrier
lifetime. B5 acceptance and measurements remain pending.


Error transport now uses checked statement-entry availability in addition to
normal body-exit availability. A thrown payload carries each live owner, or an
empty internal Map after transfer. Catch must explicitly restore source storage
before reading it; the editor witness confirms that reads before restoration
still report a transfer diagnostic. Empty Map assignment now renews native
storage so the previous callback lifetime ends at replacement.

Disposal instrumentation covers failure before transfer and after transfer at
optimization levels 0 and 3. It first exposed E0502 from retaining the old Map
lifetime; the repair releases exactly one callback in each scenario. The first
affected suite then found an E0382 regression when restoring already consumed
storage outside completion transport. That path now omits the drop of the moved
binding, while completion storage still drops its internally initialized owner.
The repaired suite passes all 68 composition tests. Eight depth canaries and
15 exception-contract tests pass on the normal worker stack. The focused editor
test passes. Original failures and repaired checks are preserved byte for byte
in SQLite as `tsrust-b5-statement-error-disposal.log`,
`tsrust-b5-statement-error-disposal-repaired.log`,
`tsrust-b5-statement-error-checks.log`,
`tsrust-b5-statement-error-repaired-checks.log` and
`tsrust-b5-statement-error-depth-checks.log`. Loose copies are removed.

This checkpoint covers transfer in a prior statement. Moves within the same
expression before a fallible operation still need expression-level snapshots.
Conditional transfers need exact disposal witnesses for both the surviving and
consumed paths; source unavailability after a join does not itself prove every
native owner was consumed. Loop/control exits and nested exception regions also
remain pending. B5 is not accepted and no timing was performed.


Matched error-restoration coverage now retains twenty-two scalar observations.
The added early-error path preserves the old Map until explicit replacement;
the late-error path clears and transfers it before throwing. Both catch bodies
restore empty storage and then mutate the released capture. Independent arithmetic
expects `value + 29` and `value + 30`. Token guards require both failure choices,
transfers, observations, restorations and capture mutations. Rehashed mutation
controls pass. Native and Node-after-tsc verification passes untimed; exact
original evidence is verified in SQLite as
`runtime/results/2026-10-08T17-57-52-860Z-verify.json` and
`tsrust-b5-restored-benchmark.log`, with loose copies removed. Project formatting
is token-preserving and idempotent. B5 acceptance and timing remain pending.


The within-expression transfer witness now fails in native generation with E0382:
`expression_transfer_before_failure_restores_map_storage` assigns the cleared
Map into another owner and then calls a throwing function in the same comma
expression. Catch explicitly restores the original Map. A following explicit
throw rules out a normal exit with unrestored source storage. The initial draft
omitted that throw and correctly received a transfer diagnostic at the later
read; both original records are verified in SQLite as
`tsrust-b5-expression-transfer-red.log` and
`tsrust-b5-expression-transfer-native-red.log`, with loose copies removed.
Expression evaluation availability is the next implementation step.


Expression failure transport now checks exact expression spans before the existing
statement-entry fallback. Snapshots record checked availability after call
arguments or operator operands have been analyzed; call/callee and operator spans
match the native error sites. A nested expression can start at the same byte as
its parent, so an exact `(start, end)` key is required. Statement-entry snapshots
remain separate and are not overwritten by expression completion.

Native witnesses cover transfer before a failing call, failure before a later
transfer, a nested arithmetic expression, a bounds failure after transfer and a
throwing array factory before an index-expression transfer. The arithmetic
variant first exposed E0382 because overflow errors use the operator span;
operator snapshots repair that path. Disposal instrumentation at optimization
levels 0 and 3 proves one callback drop and correct restored storage. All 70
composition tests, eight depth canaries, 15 exception-contract tests and the
expanded editor witness pass. Original attempts and repaired checks are verified
byte for byte in SQLite as `tsrust-b5-expression-transfer-first-repair.log`,
`tsrust-b5-expression-transfer-exact-check.log`,
`tsrust-b5-expression-transfer-variants.log`,
`tsrust-b5-expression-transfer-variants-repaired.log` and
`tsrust-b5-expression-transfer-batch.log`. Loose copies are removed.

These witnesses do not prove conditional partial-transfer disposal, loop/control
exits or all rewritten/synthetic expression spans. Those remain pending. B5 is
not accepted and no timing was performed.


Matched expression-failure coverage now retains twenty-four observations.
Two scenarios fail before or after transfer inside a comma expression, explicitly
restore the original Map in catch, and mutate the released capture. Independent
results expect `value + 39` and `value + 40`. Guards and rehashed deletion controls
pass. Untimed native and Node-after-tsc verification passes, preserved exactly in
SQLite as `runtime/results/2026-10-08T18-05-25-609Z-verify.json` and
`tsrust-b5-expression-benchmark.log`; loose copies are removed.


Conditional transfer has a confirmed disposal-order defect. The new red witness
`conditional_transfer_error_preserves_survivor_until_restoration` executes both
choices. One branch clears and transfers the Map; the other retains its callback.
After the join, an explicit throw enters catch, which first prints `caught` and
then replaces the Map. The consumed branch has the expected single drop at clear.
The surviving branch currently drops before `caught`, rather than at the explicit
replacement. Exact evidence is verified in SQLite as
`tsrust-b5-conditional-survivor-red.log`; the loose copy is removed.

Source unavailability and native initialization must be modeled separately.
`map_clear_if_epoch` returns initialized storage on both paths when one source
owner survives: an empty internal owner on the consumed path and the live owner
on the other. The subsequent error payload consults joined source unavailability
and substitutes another empty Map, discarding the survivor at exception exit.
Normal body exits can lose the same survivor. A correct repair must preserve the
normalized native owner through later errors and body exits while keeping source
reads rejected, and must invalidate native availability after a later transfer.
Nested branches must isolate their native state during emission. Do not treat
all source-unavailable Maps as live or globally retain a branch's initialization.
Add a dedicated native availability plan or scoped emitter state with explicit
branch joins, replacement and transfer points. Verify surviving and consumed
paths, nested branch ordering, subsequent restoration/transfer, record-field
siblings and exact native disposal before acceptance. B5 remains incomplete.


The conditional survivor repair separates source usability from native storage
initialization. Checked snapshots now retain the transfer span for roots and
moved fields. A small `native_storage_availability.rs` module tracks when native
owners are initialized by completion entry or balanced branch/block returns.
Error and normal completion payloads carry those owners even when source reads
remain rejected. A later source transfer invalidates that native state. Sibling
branch emission saves and restores its initialization state before publishing
the join. Both consumed branches return empty internal owners without cloning.

The first attempt used the `if` keyword span as the join boundary and retained
the original disposal failure. Checked transfer endpoints provide the actual
boundary. Native witnesses now cover both branch choices, nested blocks,
record-field storage and a sibling marker, replacement followed by another
transfer, subsequent restoration, and normal completion with scope-end disposal.
All disposal assertions run at optimization levels 0 and 3. The field witness
also exposed E0502 from a named partially moved generic record temporary. Direct
annotated field binding removes that residual lifetime without changing source
ownership. Two variant drafts had malformed generated test source because their
string substitutions assumed double quotes despite preserved single quotes;
those failed attempts are retained rather than attributed to compiler behavior.

All 72 composition tests, 42 scoped callable-record tests, four fallible
record-callback tests, eight normal-stack depth canaries, 15 exception-contract
tests and the expanded focused editor witness pass. The editor still rejects
reads after possible transfer and before restoration. Exact original failures,
traces and repaired checks are verified in SQLite under the
`tsrust-b5-native-availability-*` keys, plus
`tsrust-b5-conditional-survivor-repair.log`,
`tsrust-b5-conditional-survivor-repaired-boundary.log` and
`tsrust-b5-direct-record-regressions.log`. Loose copies are removed.

This native initialization state currently belongs to completion storage regions.
Replacement after a conditional transfer outside such a region still needs a
focused disposal witness. Loops, control exits, nested try and finally without
catch remain pending. Broader per-entry release, callable compositions and record
Set keys remain part of B5; this checkpoint is not family acceptance or timing.


Matched survivor coverage now retains twenty-six observations. A record-field
Map consumes its owner on even values and retains its callback on odd values,
then throws and explicitly restores storage. A second scenario completes normally
and releases the survivor at scope exit. Independent results expect
`value + 50 + (even ? 0 : 1)` and `2 * value + 51 + (even ? 0 : 1)`.
Guards and rehashed mutation controls pass. Native and Node-after-tsc verification
passes untimed, preserved exactly in SQLite as
`runtime/results/2026-10-08T18-22-16-811Z-verify.json` and
`tsrust-b5-native-survivor-benchmark.log`; loose copies are removed. Earlier
observations remain unchanged. No timing samples were collected.


Replacement outside exception handling has a confirmed red native witness:
`conditional_transfer_replacement_preserves_disposal_without_try` clears and
transfers on one branch, retains the callback on the other, then explicitly
replaces the Map before mutating the capture. Generated Rust fails with E0502:
the surviving native owner is shadowed without dropping, retaining its callback
borrow until scope exit. Exact original evidence is verified in SQLite as
`tsrust-b5-outside-replacement-red.log`; the loose copy is removed. Native
initialization tracking must extend beyond completion storage while isolating
callables and sibling branches. Source reads after possible transfer must remain
rejected.


Native availability now has its own scoped stack within the dedicated module.
It remains active when no completion-storage payload exists. Callable emission
starts fresh state and restores its caller's emission state on exit; sibling
branches and completion regions save/restore their own boundaries. Completion
storage attaches its initialized entry owners to this stack. Returning balanced
storage from a try statement publishes a new native initialization boundary.
Source usability and transfer diagnostics are unchanged.

The outside-replacement red witness now passes, including nested blocks,
record-field storage with an independent marker, another transfer and subsequent
restoration. Exact disposal is verified at optimization levels 0 and 3 on both
branch choices. All 73 composition tests, 42 scoped callable-record tests, four
fallible record-callback tests, eight depth canaries, 15 exception-contract tests
and the focused editor witness pass. The editor accepts replacement and still
rejects reads between a possible transfer and replacement. Original repaired
checks are verified byte for byte in SQLite as
`tsrust-b5-outside-replacement-repair.log`,
`tsrust-b5-outside-replacement-variants.log` and
`tsrust-b5-outside-replacement-batch.log`; loose copies are removed.

This closes the verified outside-replacement case, not B5. Completion storage
still needs loops, return/break/continue, nested try and finally without catch.
Precise per-entry capture release, remaining callable compositions and record
Set key support remain pending. No timing or whole family acceptance occurred.


Matched outside-replacement coverage now retains twenty-seven observations. The
new scenario transfers on even values, retains on odd values, explicitly replaces
outside exception handling, then mutates the released capture. Independent
arithmetic expects `value + 60 + (even ? 0 : 1)`. Guards and rehashed deletion
controls pass. Native and Node-after-tsc verification passes untimed, preserved
exactly in SQLite as `runtime/results/2026-10-08T18-30-50-740Z-verify.json` and
`tsrust-b5-outside-replacement-benchmark.log`; loose copies are removed. Earlier
observations remain unchanged. No timing samples were collected.


Finally without catch has a confirmed non-leading-renewal red witness:
`finally_without_catch_renews_nonleading_map_storage` calls a fallible prefix,
clears/refills the Map on normal completion, and clears the current owner in
finally on both paths. Mutation after cleanup is source-valid but generated Rust
fails with E0502. Its first draft also logged the caught error, exposing a separate
console emission panic. A scalar-only isolation passed; a Map with two borrowed
callback origins plus runtime-union error logging reproduces the panic as
`caught_error_logging_ignores_uninspectable_callback_records`.

The emitter's record inspection admission omitted `BorrowedCallable`, so it tries
to emit an inspection helper for an unused callback-bearing record when error
logging requires record inspection support. Align that helper admission with
source inspection rules, including nested Maps/Sets, before continuing the
finally storage repair. Exact original evidence is verified in SQLite as
`tsrust-b5-no-catch-finally-red.log`,
`tsrust-b5-no-catch-finally-storage-red.log`,
`tsrust-b5-finally-console-isolation.log` and
`tsrust-b5-console-callback-record-red.log`. Loose copies are removed.

### Straight-line no-catch finally storage repaired

The two red witnesses committed in `0a2a5e5` now pass. A separate completion
helper preserves live root and record-field Map storage across normal and
throwing execution, runs cleanup with the current owner, resumes pending errors
and allows cleanup errors to override them. Disposal is verified at optimization
levels 0 and 3. Record inspection admission also excludes borrowed callbacks and
checks nested Map/Set contents, fixing caught-error logging with unrelated
callback-bearing records.

The affected batch passes 75 composition, 42 scoped-record, four fallible-record,
eight depth-canary and 15 exception-contract tests, plus the focused editor test.
The 28-observation matched workload passes native and Node-after-tsc verification
untimed. Exact evidence is stored in SQLite under
`runtime/results/2026-10-08T18-47-39-945Z-verify.json` and the
`tsrust-b5-no-catch-finally-*.log` repair/batch/registration records. Failed drafts
remain preserved separately. Project formatting changed no files.

This is a B5 implementation checkpoint. Control exits, nested try regions,
precise element/entry capture release and record Set equality/hashing remain
open; no family regression or timing acceptance is claimed.

The next control-exit witness is
`returning_through_finally_preserves_renewed_map_until_cleanup`. Early return alone
passes; a branch that returns and a branch that falls through cleanup before
mutating released captures produce native E0502. Both cases remain in the test.
SQLite preserves the initial passing observation and the combined failing
observation under `tsrust-b5-return-finally-red.log` and
`tsrust-b5-return-finally-fallthrough-red.log`. No implementation repair claimed.

### Return/fallthrough completion storage repaired

The committed return-through-finally witness now passes. Pending results carry
the current Map storage into cleanup. Successful cleanup resumes the evaluated
result; abrupt cleanup replaces it. Post-return checked snapshots keep moved
storage consumed. Root and record-field cases, fallthrough mutation and cleanup
return override pass exact disposal checks at O0/O3.

The affected suites retain 75 previously passing composition checks plus the
repaired expanded witness, 42 scoped-record and four fallible-record checks.
Eight depth canaries, 15 exception-contract tests and focused editor diagnostics
pass. The 29-observation matched workload verifies untimed on native and
Node-after-tsc. SQLite record:
`runtime/results/2026-10-08T18-55-03-597Z-verify.json`. Exact failed attempts and
passing supplements remain under `tsrust-b5-return-finally-*.log`.
No family regression/timing acceptance is claimed. Loops, break/continue, nested
regions and the remaining B5 capture/key work are open.

The next loop witness is
`loop_exits_through_finally_release_renewed_map_captures`. Break and continue
through cleanup pass independently. Combining them with normal fallthrough and
capture mutation before the next iteration fails native E0502. Exact disposal
assertions remain in both cases. SQLite preserves the passing initial case as
`tsrust-b5-loop-finally-red.log` and the failing combined case as
`tsrust-b5-loop-finally-fallthrough-red.log`.

### Loop-local control exits through finally repaired

The witness from `a95e5af` now passes. An internal storage-exit carrier distinguishes
return, break and continue while carrying live Map owners into finally. Cleanup
runs before the pending destination is dispatched and can override it. Normal
fallthrough returns fresh storage, permitting capture mutation before another
iteration. Root and record-field witnesses include exact disposal at O0/O3 and
cleanup break overriding a pending continue. No implicit clone is introduced.

The affected batch passes 77 composition, 42 scoped-record and four fallible-record
tests. Eight normal-stack depth canaries, 15 exception-contract tests and the
focused editor check pass. The matched workload has 30 observations; registration
includes rehashed mutation rejection for both clears, loop destinations and
cleanup observation. Full family regressions and timing remain pending.

This checkpoint covers Maps declared within the loop. Storage declared before
the loop needs its own live-owner boundary witness. Loops inside try, nested try
regions, precise entry/element capture release and record equality/hash keys
remain in B5 scope.

The 30-observation workload verifies untimed on native and Node-after-tsc,
record `runtime/results/2026-10-08T19-01-37-909Z-verify.json`. That record and
`tsrust-b5-loop-finally-*.log` evidence were imported/read back byte-exact into
SQLite before deleting loose copies. Formatting changed only workload layout
and its token-preserving fingerprint.

## Outer Map loop boundary red witness

The loop-local repair is committed as `2e63d02`. Its outer-storage follow-up,
`outer_map_survives_loop_exit_cleanup_and_capture_renewal`, is source-valid but
fails native E0502. A Map declared before the loop is refilled each iteration,
cleared in finally on continue, normal and break paths, then observed after
the loop. Captures are mutated after normal cleanup and after the loop. Exact
disposal remains asserted at O0/O3. SQLite retains the original failure as
`tsrust-b5-outer-map-loop-red.log`; its loose copy was removed after read-back.

Next continuation: repair the loop storage boundary before nested regions.
Carry renewed owners across iterations and out of the loop without extending
callback loans or dropping owners before cleanup. Preserve the complete witness
and the accepted loop-local cases. Continue remaining B5 work and the full
compiler/editor plan after this repair.

### Proven-empty outer Map iteration epochs

The `a8cf844` witness is repaired by a separate
`map_loop_storage_epochs.rs` helper. It retains the original empty owner and
creates an independent native owner for each iteration. Admission requires an
empty constructor immediately before the while loop, an unconditional direct
clear in finally, no earlier cleanup control exit and no later Map refill or
reference other than reading size. Prefix and trailing loop statements cannot bypass the cleanup.
Nested declaration context reaches the existing completion-storage helper.
Native witnesses verify continue, fallthrough capture mutation, break, post-loop
observation and zero iterations with exact disposal at O0/O3.

The affected batch passes 78 composition, 42 scoped-record and four fallible-record
tests. Eight normal-stack depth canaries and focused editor diagnostics pass.
The matched workload has 31 independent observations. Registration now validates
each loop helper separately so duplicate operations cannot hide a removed clear
or control destination. Full family regression and timing remain pending.

This is an empty-epoch implementation slice. Retained/populated owners crossing
iterations, record-field outer storage, broader loop forms, loops inside try and
nested exception regions remain B5 implementation work; they are not accepted
source-language rejection boundaries.

The 31-observation workload verifies native and Node-after-tsc untimed, record
`runtime/results/2026-10-08T19-14-53-091Z-verify.json`. The initial workload
failed on a post-clear size read omitted by the helper's first eligibility
check. That case now has a focused disposal witness and passes. A trial requiring
all prefixes to have empty error contracts was overly restrictive for checked
scalar arithmetic and was removed; the original empty owner remains intact
and exceptions dispose the current iteration owner normally. Original failed
batches and registration attempts remain beside passing supplements in SQLite
under `tsrust-b5-outer-map-loop-*.log`. All records were read back byte-exact
before deleting loose copies. Final formatting changed no files.

## Populated and record-field loop witnesses

Retained entries across iterations already pass, including replacement disposal
and explicit post-loop clear. A populated initial owner requires that explicit
clear before capture mutation: zero iterations can retain its callback. The
first test draft omitted clear and correctly received `capture-conflict`; the
corrected populated witness passes exact native disposal.

The outer record-field loop witness instead exposes an emitter panic before
loop lowering: `unchecked record carrier conversion Map(8) to Map(2)`.
`nested_empty_map_initializer_adapts_record_callback_carriers` isolates that
panic without a loop. Repair contextual empty Map initialization/Map carrier
conversion, then resume outer record-field iteration support. Exact originals
are in SQLite as `tsrust-b5-populated-retained-loop-red.log`,
`tsrust-b5-outer-record-loop-red.log` and
`tsrust-b5-map-carrier-conversion-red.log`; loose copies were removed after
byte-exact read-back. Existing owner fixture changes remain untouched.

## Contextual nested Map initializer repaired

The constructor panic isolated in `8014586` is repaired. Checked record carrier
chains now contextualize empty Map initializers in the destination carrier.
Record literal emission retains source initializer order, once-only evaluation
and omitted optional fields. Values and existing owners still use ordinary
carrier conversion; the repair introduces no runtime clone. Native examples
verify marker initialization exactly once, reordered fields and absent labels.

The affected batch passes 81 composition tests, 42 scoped-record and four
fallible-record tests; the pending outer record-field loop witness is excluded
from that green batch and rerun independently. It now fails native E0502 rather
than panicking, confirming the separate lifetime boundary work. Eight normal-
stack depth canaries and focused editor diagnostics pass. All 32 matched
observations verify untimed on native and Node-after-tsc, record
`runtime/results/2026-10-08T19-28-27-873Z-verify.json`. Exact failed attempts and
passing supplements are read-back verified in SQLite under
`tsrust-b5-map-carrier-*.log`; the remaining red native witness is
`tsrust-b5-outer-record-loop-native-red.log`. Loose copies are removed.
The workload's first draft used Box rather than RefillBox for its marker field;
that draft failure is preserved separately. Project formatting changed only the
new workload layout and its token-preserving fingerprint.

Next: extend `map_loop_storage_epochs.rs` to empty owning record leaves. Resolve
the Map leaf through the record layout, prove its initializer is empty through
checked carrier wrappers, and retain sibling fields and key-order metadata.
Permit independent sibling access after cleanup while preventing refills or
whole-record transfers from bypassing the epoch proof. Preserve exact root and
record-field disposal, zero-iteration behavior and source ownership rules.
Retained/populated root Maps with explicit post-loop clear already pass; do not
reimplement them or weaken the correct zero-iteration capture-conflict rule.

Continuation prompt: Continue B5 from this constructor repair checkpoint. Read
AGENTS.md, development-workflow.md and scoped-callable-aggregate-plan.md. Repair
`outer_record_map_cleanup_releases_captures_across_iterations` (native E0502) by
extending empty loop epochs to record leaves, with sibling and zero-iteration
witnesses. Keep compiler/editor parity, matched workloads, independent oracles
and mutation controls current. Preserve exact evidence in SQLite, unrelated
fixture edits and explicit Git staging. Format the whole project before commits.
B5, full family regressions/timing and the full compiler/editor goal remain open.

## Empty record-leaf loop epochs repaired

The remaining record-field witness now passes. `map_loop_storage_epochs.rs`
resolves owning Map leaves through record layouts and proves their literal
initializers empty through checked carrier wrappers. Each iteration receives
a fresh native Map leaf; the original empty owner and sibling fields remain
intact. Post-cleanup path checks allow independent siblings, size reads and
Object.keys metadata reads, while excluding transfers/refills of the selected
leaf and whole-record operations that could retain it.

Native disposal witnesses at O0/O3 cover continue, normal fallthrough, break,
sibling mutation, record key order and zero iterations. The affected batch
passes all 82 composition tests, 42 scoped-record and four fallible-record
tests. Eight normal-stack depth canaries and focused editor diagnostics pass.
All 33 independent matched observations verify untimed on native and
Node-after-tsc, record `runtime/results/2026-10-08T19-37-33-183Z-verify.json`.
Exact `tsrust-b5-record-leaf-loop-*.log` evidence is read-back verified in
SQLite; loose copies are removed. Formatting changed only the new workload
layout and its token-preserving fingerprint.

B5 remains open. The epoch helper still targets a directly initialized empty
owner, one proven cleanup boundary and while loops. Multiple renewed leaves,
broader loop forms, nested exception regions, precise entry/element capture
release and remaining callable/nullable compositions require witnesses and
implementation. Retained/populated root Maps with explicit post-loop clear
already pass; preserve the correct zero-iteration capture-conflict rule.
Rust-style record Set equality/hash work and family regressions/timing remain
pending. Five user consumers still represent gate181.

Continuation prompt: Continue B5 from this record-leaf loop checkpoint. Read
AGENTS.md, development-workflow.md and scoped-callable-aggregate-plan.md. Root
and record-leaf empty loop epochs now pass disposal, sibling/key-order and
zero-iteration checks. Add nested exception-region and broader loop witnesses,
then extend completion storage without premature disposal or implicit cloning.
Keep compiler/editor parity, matched workloads, independent oracles, mutation
controls and exact SQLite evidence current. Preserve unrelated fixtures and
explicit Git staging; format the whole project before checkpoints. Complete
remaining B5 scope before family regressions/timing or acceptance. The full
compiler/editor completion goal remains active.

## Nested finally red witness

`nested_finally_preserves_map_owners_and_cleanup_order` is source-valid but
fails native E0502 after both cleanup blocks. It checks normal/failing body paths,
inner-before-outer cleanup order, exact callback disposal and capture mutation
after successful cleanup. SQLite preserves `tsrust-b5-nested-finally-red.log`
byte-exact; its loose copy was removed. Extend nested completion dispatch to
carry initialized storage and evaluated control payloads into enclosing regions,
without restoring consumed source names or dropping owners before cleanup.

## Nested finally storage repair checkpoint

Nested no-catch finally regions now carry Map storage through normal completion,
errors and evaluated returns. Cleanup errors replace pending errors while outer
cleanup still runs. Root Maps and owning record fields preserve exact callback
disposal and inner-before-outer cleanup order. Consumed source names are not
restored, and the transport introduces no implicit cloning.

The affected batch passes 83 composition tests, 42 scoped-record tests and four
fallible-record tests. All eight normal-stack depth canaries, 15 exception-contract
unit tests and focused editor diagnostics pass. Exact native disposal witnesses
pass at O0 and O3, including cleanup-error override and a pending return. The
matched workload verifies all 34 independent observations untimed on native and
Node-after-tsc. Registration and rehashed mutation rejection checks pass after
project formatting. SQLite preserves the failed initial repair and all successful
`tsrust-b5-nested-finally-*.log` evidence byte-exact, together with
`runtime/results/2026-10-08T19-49-50-065Z-verify.json`; loose copies are removed.

B5 is not accepted. Nested catch regions, broader loop forms, multiple renewed
owners, precise entry/element capture release and remaining callable compositions
still need implementation witnesses. Record Set keys will use Rust-style value
equality and hashing, immutable stored keys and explicit ownership transfer or
clone. Floating-point key policy remains unresolved. Family regressions and
timing remain pending. Five user executable consumers still represent gate181.

Continuation prompt: Continue B5 from this nested-finally repair checkpoint. Read
AGENTS.md, docs/development-workflow.md and docs/scoped-callable-aggregate-plan.md.
Add a source-valid nested-catch storage witness next; repair completion transport
without premature disposal, implicit clone or restoring consumed source names.
Keep compiler/editor behavior, matched workload support and SQLite evidence
current. Preserve unrelated fixture semantics and use explicit Git staging.
Format the whole project before each source checkpoint. Complete remaining B5
scope before its family regressions, timing and acceptance. The full compiler
and editor goal remains active.

## Nested catch discovery repair checkpoint

Outer finally storage discovery now traverses nested catch bodies as well as
inner try and finally bodies. Previously source-valid nested handlers left
callback capture loans alive in generated Rust after outer cleanup. The focused
witness now passes exact disposal and cleanup-order assertions at O0 and O3
on normal, caught and rethrown paths. The affected composition batch passes
84 tests; focused editor diagnostics pass. The matched workload adds an
independent caught-refill observation, with source guards and rehashed mutation
controls. All 35 observations verify untimed on native and Node-after-tsc.

SQLite preserves `tsrust-b5-nested-catch-*.log` byte-exact, including initial
witness-count and command errors, and the successful verification record
`runtime/results/2026-10-08T19-56-56-300Z-verify.json`. Loose copies are removed.
Project formatting preserves tokens and refreshes only the changed fingerprint.

This checkpoint does not accept B5. Nested catch regions with their own abrupt
cleanup or return/control exits still need witnesses. Broader loops, multiple
renewed owners, precise entry/element release, remaining callable compositions
and Rust-style record Set keys remain open, followed by family regressions and
timing. Five user executable consumers remain at gate181.

Continuation prompt: Continue B5 after nested catch storage discovery. Read
AGENTS.md, docs/development-workflow.md and docs/scoped-callable-aggregate-plan.md.
Add a nested catch-plus-finally witness whose handler throws or returns and whose
inner cleanup must preserve owners until outer cleanup. Use focused red tests
before extending carried completion transport. Preserve explicit ownership,
exact disposal, editor behavior, matched workload oracles and SQLite evidence.
Format the entire project before checkpoints; stage explicit paths and preserve
unrelated fixture semantics. B5 and the full compiler/editor goal remain active.

## Nested handler cleanup storage checkpoint

Catch-region cleanup now carries initialized owners on abrupt completion and
rebinds them before resuming a pending error. It shares the dedicated carried
cleanup dispatch helper with no-catch finally regions. This repairs native E0382
when a nested handler rethrows through its own finally into outer cleanup.

Native witnesses pass exact callback disposal and inner-before-outer cleanup
order at O0 and O3. A cleanup error replaces the handler error, and outer cleanup
still observes the explicitly cleared owner. The affected batch passes all
84 composition, 42 scoped-record and four fallible-record tests. Focused editor
diagnostics pass. Its first fixture included unconditional abrupt cleanup and
unreachable statements; the reported diagnostic was valid, and the fixture was
corrected to the conditional native witness.

All 36 matched observations verify untimed on native and Node-after-tsc. Source
guards and rehashed mutation rejection pass; the repeated throw-9 snippet now
has an occurrence-count guard so a different scenario cannot mask its removal.
Project formatting preserves tokens and updates the changed source fingerprint.
SQLite preserves every `tsrust-b5-nested-handler-cleanup-*.log` byte-exact, including
the original native failure and failed check attempts, plus verification record
`runtime/results/2026-10-08T20-00-54-839Z-verify.json`. Loose copies are removed.

B5 remains open. Return/break/continue through nested handlers, broader loops,
multiple renewed owners, precise element/entry release, remaining callable
compositions and record Set equality/hash remain pending. Family regressions and
timing are not run. Five user executable consumers remain at gate181.

Continuation prompt: Continue B5 from the nested handler cleanup checkpoint.
Read AGENTS.md, docs/development-workflow.md and the scoped aggregate plan. Add
a source-valid return through a nested catch-plus-finally region next, preserving
the evaluated return payload, callback disposal and outer cleanup. Extend handler
completion storage for control exits where the native witness requires it. Keep
editor behavior, matched workload support and exact SQLite evidence current.
Format the whole project before checkpoints and stage explicit paths. Preserve
unrelated fixture semantics. B5 and the full compiler/editor goal remain active.

## Nested handler return storage checkpoint

Catch-region completion now carries evaluated return payloads with initialized
Map owners, runs a dedicated cleanup path for pending control and rebinds storage
before resuming into the enclosing region. Normal return and handler return both
preserve exact callback disposal and inner-before-outer cleanup order.

The initial return witness already passed. Its stronger mutation witness first
reported a valid source capture conflict: fallible console observation before
inner clear can bypass that clear. The retained rejection asserts that rule.
Explicit outer clear makes the source valid and exposed native E0502; the storage
transport repair resolves that defect without changing source ownership rules.
Native witnesses at O0/O3 verify return evaluation exactly once, unchanged return
values and capture mutation after explicit cleanup. The affected batch passes
85 composition, 42 scoped-record and four fallible-record tests; focused editor
diagnostics pass. All 37 matched observations verify untimed on native and
Node-after-tsc, including return values evaluated before cleanup changes captures.
Registration, mutation controls and token-preserving project formatting pass.

SQLite preserves `tsrust-b5-handler-return-*.log` byte-exact, including isolation
and the original rejection/native failure, plus verification record
`runtime/results/2026-10-08T20-07-26-037Z-verify.json`. Loose copies are removed.
B5 remains open; loop exits through handlers, broader loops, multiple owners,
precise element/entry release, remaining callable compositions and record Set
equality/hash still need work. Family regressions and timing remain pending.
Five user executable consumers remain at gate181.

Continuation prompt: Continue B5 from the nested handler return checkpoint.
Read AGENTS.md, docs/development-workflow.md and the scoped aggregate plan. Add
loop-exit witnesses through catch-plus-finally, including continue, break and
cleanup overrides, then repair any failure. Do not reimplement passing returns
or weaken the correct capture-conflict when an inner observation can fail before
clear. Keep exact native disposal, editor behavior, matched workloads and SQLite
evidence current. Format the whole project before checkpoints; stage explicit
paths and preserve unrelated fixture semantics. B5 and the full compiler/editor
goal remain active.

## Catch loop storage checkpoint and retained cleanup-mutation witness

Nested handler continue/break and an outer cleanup overriding continue already
pass through the existing carried control transport. New exact-disposal witnesses
cover these paths at O0/O3. A direct catch-plus-finally loop with an initially empty
outer Map exposed native E0502 after cleanup. Empty while epochs now admit catch
regions under the existing unconditional-clear and no-refill proof. Root and
record-field owners, zero iterations, normal fallthrough, continue and break pass.

One distinct defect remains red: `catch_cleanup_clear_releases_captures_before_cleanup_mutation`
is source-valid but fails generated Rust E0502 when capture mutation occurs after
clear within that same finally block. Moving the mutation after the cleanup passes;
the original behavior remains a real executable witness, not an accepted source
restriction. Investigate generic owner lifetimes in typed cleanup completion and
end the emptied owner's native lifetime before mutation without weakening source
checks or moving observable cleanup operations. The first attempt merely admitting
catch loops did not repair this defect; original evidence is retained.

The affected batch passes 87 composition, 42 scoped-record and four fallible-record
tests, explicitly excluding that one red witness. Focused editor diagnostics pass.
All 38 matched observations verify untimed on native and Node-after-tsc, with source
guards, independent arithmetic oracle, registration and rehashed mutation rejection.
Project formatting preserves tokens and refreshes the changed source fingerprint.
SQLite preserves `tsrust-b5-handler-loop-*.log`, `tsrust-b5-direct-catch-loop-*.log`
and `tsrust-b5-catch-cleanup-mutation-*.log` byte-exact, including failed attempts,
plus `runtime/results/2026-10-08T20-14-14-148Z-verify.json`. Loose copies are removed.

B5 is not accepted. Fix the retained cleanup-mutation defect next, then broader
loops, multiple owners, precise entry/element release, remaining callable
compositions and Rust-style record Set equality/hash. Family regressions and
timing remain pending. Five user executable consumers remain at gate181.

Continuation prompt: Continue B5 from this catch-loop checkpoint. Read AGENTS.md,
docs/development-workflow.md and the scoped aggregate plan. Repair the retained
`catch_cleanup_clear_releases_captures_before_cleanup_mutation` native E0502.
It is source-valid; do not reject it, move its mutations outside cleanup or remove
the witness. Direct catch-loop root/record empty epochs and nested control exits
already pass. Preserve exact O0/O3 disposal, cleanup order and source ownership.
Complete editor and matched workload coverage for the repair, then proceed with
remaining B5 scope. Keep SQLite evidence, whole-project formatting and explicit
Git staging. Preserve unrelated fixtures. The full goal remains active.

## Cleanup-mutation investigation checkpoint

The source-valid cleanup-mutation witness remains red at the loop checkpoint.
Focused native experiments show that naming completion temporaries alone,
separating control from storage, splitting cleanup at the clear, and reconstructing
empty storage after an owner-free suffix do not independently resolve E0502.
Combined separation and scoped-suffix experiments also fail. All unsuccessful
compiler changes and temporary instrumentation have been removed; source remains
at the verified catch-loop implementation. The retained executable witness is
unchanged. This is an investigation checkpoint, not an implementation acceptance.

Diagnostics identify both completion shells and scoped Map bindings whose drop
glue retains borrowed callback lifetimes across cleanup. The exact cause still
needs reduction; do not treat the temporary-shell explanation as proven sufficient.
SQLite contains each `tsrust-b5-cleanup-*.log` experiment, original reproduction
and the complete formatted generated Rust as `tsrust-cleanup-native-repro.rs`.
Each record is read back byte-exact and its loose copy removed. Project formatting
changes no files. No family regression, timing or consumer refresh was performed.

Next work should reduce the generated Rust ownership/control shape before further
compiler changes. One untested direction is dispatching abrupt prefix completion
directly into its enclosing region instead of joining its borrowed owner type with
fresh empty suffix storage. Preserve nested outer cleanup: any escaping error must
carry surviving owners where required, and normal cleanup must drop them before
mutation. Keep the current source witness and exact disposal assertions. Avoid
unsafe lifetime erasure, implicit clones, moving source mutations, or rejecting
source-valid cleanup. B5 and the full goal remain active.

Continuation prompt: Continue the retained B5 cleanup-mutation repair from this
investigation checkpoint. Read AGENTS.md, development-workflow.md and the scoped
aggregate plan. Restore `tsrust-cleanup-native-repro.rs` from SQLite into the shared
temporary build area for focused reduction. Do not repeat naming-only or scoped
suffix experiments as if untested; their native failures are preserved. Repair
`catch_cleanup_clear_releases_captures_before_cleanup_mutation`, then complete
compiler/native/editor and matched workload coverage. Preserve checked ownership,
cleanup order, exact disposal, current passed evidence and unrelated fixtures.
Format the whole project and stage explicit paths before source checkpoints.

## Cleanup capture release checkpoint

The retained `catch_cleanup_clear_releases_captures_before_cleanup_mutation`
witness now passes native execution at O0 and O3, including exact disposal.
The test also checks a record-field Map owner. Pending try/catch outcomes finish
inside an owner scope, and the emitted storage tuple is destructured directly
before cleanup begins. This ends the completion shell and input-owner lifetimes
before a clear releases callbacks and capture mutation follows.

This lowering applies when cleanup directly clears every carried Map. Paths
retaining callbacks keep their existing branch-specific transport: using the
joined owner type for those paths caused six native regressions. The failed run
is retained, followed by a corrected affected batch with 88 composition, 42
scoped-record and four fallible-record tests passing. The additional record-field
witness and focused editor diagnostic check pass separately. No source restriction,
implicit clone or unsafe lifetime erasure was introduced.

The matched caught-loop scenario now mutates captures inside finally after clear.
All 38 observations verify untimed on native and Node-after-tsc. Independent
arithmetic expectations remain unchanged; source guards, mutation controls and
registration pass with refreshed hashes. SQLite retains the original failed and
successful `tsrust-b5-cleanup-joined-*` and `tsrust-b5-cleanup-selected-*` logs plus
`runtime/results/2026-10-08T21-24-07-353Z-verify.json`, verified by exact read-back.
Whole-project formatting changes only the benchmark source layout and fingerprint.

B5 remains incomplete. Continue with broader loop/nested-region discovery,
multiple owners, precise Map-entry/array-element release and remaining callable
compositions, then Rust-style record Set equality/hash. Keep full family checks,
timing and consumer refresh separate from this focused source checkpoint. Five
user executable consumers remain at gate181; the scratch CLI includes this repair.

Continuation prompt: Continue B5 after the cleanup capture release repair.
Read AGENTS.md, development-workflow.md and this handoff. The former red cleanup
witness now passes for root and record-field Maps at O0/O3. Preserve branch-specific
transport for retained callbacks, and direct destructuring for release cleanup.
Proceed with broader loops, multiple owners and precise entry/element release.
Keep editor, matched workloads and SQLite evidence current, format the whole
project before checkpoints, and stage explicit paths. Preserve unrelated fixture
semantics. The family and full goal remain active.

## Multiple empty Map loop owners checkpoint

A new paired-owner witness exposed native E0382 and E0502 when two record-field
Maps were cleared in catch/finally. Empty while epoch discovery previously
renewed only one leaf. It now discovers consecutive empty Map declarations and
all empty owning record Map leaves, proves each selected leaf is directly cleared,
checks each post-clear suffix independently for refill/transfer, and renews all
selected native owners per iteration. The original empty owners remain available
for zero iterations and outer errors. Source ownership checks remain unchanged.

Exact O0/O3 disposal passes for paired record fields and separate local Maps,
including continue, normal fallthrough, break and zero iterations. The affected
batch preserves 88 existing composition, 42 scoped-record and four fallible-record
passes. The new witness passes separately after correcting a test-source
replacement that had left its record initializer in the local-Map variant. The
original failed batch and corrected test are both preserved. Focused editor
checks pass for the paired-owner source.

The matched workload adds a 39th paired-owner observation. Its independent oracle
is `2 * value + 904`: three cleanup visits see six total entries before clear,
zero afterward, final captures sum to `2 * value + 301`, and turn is three.
All 39 observations verify untimed on native and Node-after-tsc; source guards,
rehash mutation controls and registration pass. SQLite stores
`runtime/results/2026-10-08T21-29-08-920Z-verify.json` and `tsrust-b5-paired-map-*.log`
with byte-exact read-back. Loose evidence is removed after verification.

B5 remains active. Next extend discovery across nested exception regions and
broader loop shapes, then precise entry/element release and remaining callable
compositions. Rust-style record Set equality/hash remains planned. No family
regression, timing or user consumer refresh has occurred. The scratch CLI includes
the paired-owner repair; user executable consumers remain at gate181.

Continuation prompt: Continue B5 from the multiple empty Map loop owners checkpoint.
Read AGENTS.md, development-workflow.md and the scoped aggregate plan. Root and
record-field paired-owner disposal now pass with existing cases preserved. Extend
nested-region/loop discovery without treating a refill as empty, then precise
entry/element release. Keep focused editor and matched benchmark support ready,
SQLite evidence durable, whole-project formatting automatic and explicit Git
staging. Preserve unrelated fixture semantics. The full goal remains active.

## Nested paired-owner cleanup checkpoint

Catch-region storage discovery now walks nested try, catch and finally bodies,
matching the existing no-catch discovery behavior. A new paired record Map loop
witness failed native E0502 before this change and passes at O0/O3 afterward.
Exact disposal covers normal fallthrough, handler continue/break, nested rethrow
into an outer catch and an outer cleanup break overriding pending continue.

The override exposed a separate conservative loop-proof gap. Post-clear storage
independence now walks blocks and conditionals iteratively, checking every branch
and its condition for references to the selected owner. It continues to reject
refill/transfer from this empty-epoch proof; no source-valid program is rejected
by that proof, since unmatched shapes retain ordinary lowering. No parser or
analyzer recursion/dispatch changed.

The affected batch passes 90 composition, 42 scoped-record and four fallible-record
tests. The added rethrow variant passes separately. Focused editor diagnostics
pass. A 40th matched observation exercises nested paired-owner cleanup; its oracle
is `2 * value + 910`, including six inner-cleanup turn observations in addition
to the paired-owner arithmetic. All 40 observations verify untimed on native and
Node-after-tsc. Source guards, rehash mutation controls, registration and project
formatting pass. SQLite preserves `tsrust-b5-nested-pair-*.log` and
`runtime/results/2026-10-08T21-34-27-759Z-verify.json`, read back byte-exact before
loose copies are removed.

B5 remains active. Broader loop forms, precise entry/element release, remaining
callable compositions and Rust-style record Set equality/hash remain. Family
regressions, timing and user consumer refresh remain pending. Scratch CLI includes
this repair; five user executable consumers remain at gate181.

Continuation prompt: Continue B5 from nested paired-owner cleanup. Read AGENTS.md,
development-workflow.md and the scoped aggregate plan. Nested catch/finally Map
discovery and post-clear conditional independence now pass with exact disposal.
Proceed with broader loop forms and precise Map-entry/array-element capture
release. Preserve retained-owner semantics, source ownership checks, focused
editor behavior, matched benchmark support and SQLite evidence. Format the whole
project before explicit-path source checkpoints. The full goal remains active.

## Retained refill and unrelated scalar Map checkpoint

Conditional refill after cleanup retains its callback until a later explicit
clear. A new exact O0/O3 witness verifies the retained callback result, size,
disposal and capture release. Moving capture mutation before that clear correctly
reports `capture-conflict`. The optional `get` view uses an explicit inner scope;
its original unscoped form correctly reports `view-conflict` under current rules.

An unrelated scalar Map adjacent to two borrowing-record Maps exposed E0502:
loop discovery required even that scalar Map to participate in the scoped owner
clear proof. Candidates now include only Maps with scoped callback payloads,
while discovery may continue past empty scalar Map declarations. Root and record
leaf selection use the same scoped-payload predicate. Paired-owner disposal with
an adjacent scalar Map now passes; no scalar Map is renewed or discarded.

One newly retained red witness remains:
`empty_scoped_record_map_has_a_native_layout_without_insertions`. A never-filled
`Map<string, Holder>` whose Holder field is `ReadOnly<Function<int>>` is rejected
by declaration-only scoped container validation before emission. The earlier
spare borrowing-record Map variation exposed this same limitation. Do not remove
the witness or equate an empty container with an unsupported payload operation.

Investigate empty concrete record environments next. Existing `Analyzer::absent_callback`
creates a distinct callback contract, and `TsrAbsentCallable` supplies an
uninhabited native environment. Reuse that mechanism per empty record instance
rather than changing a shared declaration callback to absent. Check native trait
emission as well: `TsrAbsentCallable` is currently emitted only when a nullable
union mentions callbacks. Preserve normal later insertion/widening and callback
parameters elsewhere. Cover empty size, clear, lookup, iteration, movement and
later insertion before acceptance. Do not simply remove validation and allow
undefined native generic parameters.

The affected batch passes 91 composition, 42 scoped-record and four fallible-record
tests, explicitly excluding the one red empty-layout witness. Focused editor and
registration checks pass. The paired workload now includes the adjacent scalar
Map and observes its unchanged empty size. All 40 observations verify untimed on
native and Node-after-tsc with unchanged independent expectations. SQLite stores
`tsrust-b5-retained-refill-*.log`, `tsrust-b5-spare-*.log`,
`tsrust-b5-empty-scoped-map-red.log` and
`runtime/results/2026-10-08T21-42-25-884Z-verify.json`, read back byte-exact before
loose copies are removed. Formatting and explicit staging preserve owner fixtures.

B5 remains incomplete. Empty scoped record container layout is the next concrete
repair, followed by broader loop forms, precise entry/element release, remaining
callable compositions and Rust-style record Set equality/hash. Family regressions,
timing and user consumer refresh remain pending.

Continuation prompt: Continue B5 from retained refill and the adjacent scalar Map
repair. Read AGENTS.md, development-workflow.md and this handoff. Keep the newly
retained empty scoped record Map witness and implement its concrete absent
callback environment using existing mechanisms where suitable. Preserve populated
Map inference, ownership and callback parameter behavior. Then continue broader
loops and precise release. Keep focused editor/native/matched checks and SQLite
evidence current, format the whole project and stage explicit paths. The full
goal remains active.


## Empty scoped record Map layout checkpoint

Empty local Maps whose record payloads contain borrowing callbacks now receive
concrete absent environments after populated storage inference has finished.
The implementation lives in empty_scoped_maps.rs. It specializes local storage
and iteration carriers without changing shared declaration contracts or parent
storage before later insertions. Native Map entry aliases carry their payload
generic, and absent callback trait implementations are emitted whenever used.
Impossible callback calls retain an explicit result contract.

Matched verification exposed one further defect: absent contracts contributed
unknown callback failures to callers with catches. A focused red native witness
reproduced it. Distinct absent contracts now ground error inference as pure;
populated callback declarations and ungrounded callback cycles remain unchanged.

Acceptance passes 96 composition, 42 scoped-record and four fallible-record
checks (142 total), 42 error-effect unit checks, all eight depth canaries and the
focused editor contract. Source fixtures execute after format round trips.
Project formatting changes zero files. Registration and all 41 matched native
and Node-after-tsc observations pass untimed. Original failed and successful logs
are stored byte-exact in SQLite under tsrust-b5-empty-scoped-map-*.log and
tsrust-b5-empty-map-effects-*.log. The verified result is
runtime/results/2026-10-08T22-05-55-611Z-verify.json. Twenty-nine closed records were
imported, read back and their loose copies removed. No timing or consumer refresh
is claimed. Owner fixture changes remain uncommitted.

The formerly retained empty-layout red witness is now green. B5 remains open:
broader loop forms, precise entry/element release, remaining callable compositions
and Rust-style record Set equality/hash still need implementation. Nested Map or
array callback fields inside empty record payloads are not covered by this slice.

Continuation prompt: Continue B5 from this empty scoped record Map checkpoint.
Read AGENTS.md, development-workflow.md and this handoff. Preserve the accepted
absent environment and failure contracts, populated insertion inference and owner
fixture semantics. Add the next focused witness for broader loop forms or precise
entry/element release, implement in isolated helpers, then native/editor/docs and
matched support. Format the whole project, preserve SQLite evidence and stage
explicit paths. Family regressions, timing and user consumer refresh are pending.


## Counted and post-test loop storage checkpoint

Proven-empty scoped Map storage epochs now support counted for and do-while loops
as well as while loops. Their body scopes release callback environments before
an update or post-test executes. Continue still executes the tail, while break
skips it. Counted loops preserve initializer scope and zero-iteration behavior.
The existing conservative proof requires finally to clear every selected owner
and rejects tail expressions that retain its payload. Independent intervening
local declarations do not prevent discovery of the empty owner. Lexical block
emission keeps visible declaration context for the synthetic counted-loop block.

A focused native witness initially failed with E0502 after cleanup changed a
captured array. O0 and O3 now verify exact disposal and output for a counted loop,
a counted initializer, zero iterations and a post-test loop. The affected batch
passes 97 composition, 42 scoped-record and four fallible-record tests (143 total).
All eight depth canaries and focused editor checks pass. Matched sources now add
two loop observations: all 43 independent observations pass native and
Node-after-tsc verification untimed. Registration includes mutation controls for
both observations and their capture updates. Project formatting preserves tokens
and fingerprints. Failed source-contract evidence remains stored separately from
the corrected pass.

SQLite contains tsrust-b5-counted-map-loops-*.log and
runtime/results/2026-10-08T22-12-20-160Z-verify.json, imported and read back byte-exact
before loose copies were removed. No timing or consumer refresh is claimed.
B5 remains incomplete. For-of storage renewal, precise individual release,
remaining callable compositions and Rust-style record Set equality/hash remain.

Continuation prompt: Continue B5 from counted and post-test scoped Map renewal.
Read AGENTS.md, development-workflow.md and this handoff. Preserve exact disposal,
finally control semantics and owner fixtures. Add a focused for-of renewal or
individual-release witness, then isolated implementation, editor, guide and
matched benchmark support. Keep SQLite evidence and project formatting current;
stage explicit paths. Family acceptance and executable refresh remain pending.


## For-of scoped Map storage checkpoint

Proven-empty scoped Map epochs now renew inside for-of bodies. Shared iteration
emission lives in iteration_emit.rs and keeps normal iterable evaluation,
conditional temporaries, entry adapters, copy/borrow bindings and string handling.
Both ordinary and renewed loops use this path. The renewal proof traverses
constructor operands and record fields rather than treating independent collection
initializers as unknown references. Actual owner references remain disqualifying.

The focused native witness initially failed with E0502 after finally cleared the
Map and changed its captured array. O0 and O3 now verify exact disposal, cleanup,
continue and break for array, Set, Map-entry, Unicode-string and literal-string
iteration, plus empty iteration. A resumed affected batch passes 98 composition
and three string-iteration tests. The preserved original batch already passed
42 scoped-record, four fallible-record and ten for-of tests; its one failing
composition witness is repaired in the resumed batch. Total affected checks:
157 passed. All eight depth canaries, focused editor and benchmark registration
checks pass. Failed originals remain separate evidence.

All 44 matched observations now verify native and Node-after-tsc untimed.
SQLite stores tsrust-b5-for-of-map-*.log and
runtime/results/2026-10-08T22-17-53-107Z-verify.json. Closed originals were imported,
read back byte-exact and removed as loose copies. B5 remains incomplete: precise individual
entry/element release, remaining callable compositions and Rust-style record Set
equality/hash still need implementation. No timing or consumer refresh is claimed.


Continuation prompt: Continue B5 from for-of scoped Map renewal. Read AGENTS.md,
development-workflow.md and this handoff. The while, counted, post-test and
for-of clear-in-finally paths have focused native/editor/matched acceptance.
Next implement precise individual entry/element capture release with independent
failing witnesses and exact disposal checks. Preserve remaining entries, removed
values and ownership semantics. Keep matched support and SQLite evidence current,
format the whole project and stage explicit paths. B5 family acceptance, timing
and user executable refresh remain pending.


## Precise entry release: red witness and implementation plan

The next retained red witness is deleting_one_map_entry_releases_only_its_capture.
Deleting the first of two differently capturing callback entries still reports
capture-conflict on mutation of the first array. Its negative partner protects
the second entry and its O0/O3 observer specifies separate deletion and clear
disposal. The independent Node observable reference passes. No compiler source
has changed in this checkpoint.

Read docs/map-entry-capture-release-plan.md before implementation. Both keyed
capture ownership and point-sensitive native environment narrowing are required;
a source-only loan deletion is insufficient. The existing carrier planner joins
all insertions for the entire function, and record carrier conversion currently
only widens borrowed callable families. Preserve unknown-key aliases, reaching
branch states, moves, lookup loans, order, failures and exact disposal. Avoid
unconditional survivor scans on ordinary deletion. B5 remains unfinished.

Continuation prompt: Continue precise Map entry release from the new red witness
and docs/map-entry-capture-release-plan.md. Read AGENTS.md and development workflow.
Implement keyed ownership plus checked native storage epochs together, then
negative aliases, O0/O3 disposal, editor, guide and matched benchmark support.
The whole composition suite now includes one intentionally red entry-release
witness; prior 157 affected checks passed before it was added. Preserve owner
fixtures and use explicit Git paths. Keep the full goal active.
