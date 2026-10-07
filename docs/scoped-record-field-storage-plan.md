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

## Disjoint borrowing prerequisites before parameter routing

Two focused red witnesses now establish separate gaps. The checker rejects
`touch(pair.left, pair.right)` when both parameters use Mutable<Holder>, despite
the statically distinct owning record fields. Temporary argument Loans currently
records binding roots, modes and spans, without field paths. Preserve root-wide
conflicts for whole values, ancestors, unknown indexed places and hidden captures,
but track proven static field projections for disjoint sibling arguments. Isolate
this access-loan representation and overlap logic in a dedicated module. Add
same-field, ancestor/descendant, whole-root and hidden-capture rejection controls
before changing shared access dispatch.

The native bridge has a separate E0499 witness. Calling two mutable getters on
one opaque generic carrier borrows its whole self twice. Routing mutable parameter
bodies directly through those getters would therefore lose existing native field
disjointness. Add whole-record read/mutate transport methods returning concrete
view layouts, then establish one entry view for each borrowed parameter. Native
record and view implementations must split their concrete fields in one borrow.
Generic child carriers must obtain each child view once, not request multiple
mutable child getters. Preserve callback associated types, fixed data constraints,
property-order metadata and field restoration. Do not use unsafe references or
weaken alias checks to make the native witness pass.

Both witnesses remain red and are tracked independently of the original E0502
partial-field disposal witness. These prerequisites are part of the full record
plan, not a replacement for independent storage, capture/completion transport,
editor acceptance or benchmark support.

## Static argument field loans

Temporary argument loans now retain proven static record field paths in the isolated
argument_access_loans module. Legacy root loan insertion remains conservative and
invalidates earlier field proofs for that binding. Two paths overlap when either is
a prefix of the other. Unknown, indexed and hidden capture accesses remain root-wide.
Ordinary field access establishes a projection while temporary loans are active;
fully available reads without loans retain the previous allocation-free path.

The committed disjoint sibling argument witness now passes analysis. Five controls
pass for same-field and whole-root conflicts, ancestor paths, opaque indices, mixed
mutable/shared access, hidden captures and broad-insertion invalidation. Mutable
scalar contracts remain rejected with callback-access because scalars copy; the
initial proposed scalar-loan witness was invalid and was corrected to preserve the
existing language rule. All 64 tests across the eight required depth suites pass.
Exact compilation diagnostics and red/green results are verified in SQLite under
runtime/diagnostics/scoped-record-argument-loans/. Project formatting changes zero
files and fingerprints. Fifteen owner fixtures remain unstaged and unchanged.

This is an analyzer checkpoint, not feature acceptance. The native E0499 bridge
witness and original E0502 independent-field witness remain unresolved. Implement
whole-record view transport and one concrete mutable parameter entry view next;
then complete declaration/access/store/whole-move routing and capture/completion,
editor and matched benchmark support. Existing ReadOnly, Mutable and Once contracts
remain the public capability model. No additional wrapper is introduced. Consumers
remain f1dd284; no timing or full editor acceptance is claimed.

## Whole-record native view transport

Production nominal borrowing bridges now provide tsr_fields and tsr_fields_mut.
Each operation borrows the carrier once and returns a concrete flat view containing
separate references to its static leaves. These internal view structs parameterize
all leaves with associated types, including unsized leaves, so callback environments
remain distinct and concrete scalar constraints remain expressible. Existing nested
view declarations and single-field accessors remain available.

Owned parent records request one child transport per generic child field, then move
the child's independent references into the parent transport view. They do not call
multiple mutable getters on the same generic child. Shared views implement read
transport only; mutable views reborrow their concrete fields for both operations.
No cloning, allocation, unsafe erasure or new public ownership wrapper is introduced.

The new native transport witness was red on missing tsr_fields_mut, then passed.
It holds sibling string references simultaneously, checks mutations, exercises owned,
shared and mutable carriers with distinct String and Vec callback environments,
checks callback result 14, and verifies shared string pointer identity. A generated
native rejection control confirms shared views cannot invoke a mutable consumer.
Ten layout/rendering controls and the existing production metadata adapter pass.
The original separate-getter E0499 witness and independent-storage E0502 witness
remain retained; transport support alone does not route ordinary parameter bodies.
No shared parser/analyzer dispatch changed in this checkpoint.

Next, establish one concrete transport view at each eligible borrowed parameter
entry. Route selected field access through its independent references and rebuild
nested views only when needed for borrowed child arguments. Then integrate independent
local and owned-parameter storage, stores, whole moves, property ordering and all
capture/completion boundaries. Complete editor and matched benchmark acceptance
before claiming the record feature accepted. Consumers remain f1dd284 and fifteen
owner fixtures remain untouched. Project formatting changes zero files and hashes.
No performance timing or full editor acceptance is claimed.

Exact native red/green, rejection, component and retained storage diagnostics are
verified in SQLite under runtime/diagnostics/scoped-record-whole-transport/.

## Parameter entry and leaf access components

The isolated scoped_record_parameter_views emitter module now derives one transport
operation for an eligible borrowed scoped-record parameter. Shared entry uses
let record_fieldsN=valueN.tsr_fields(); mutable entry uses the corresponding mutable
transport with a mutable view binding. Eligibility follows the checked binding's
mode and input type, nominal declaration and validated static layout. Scalars,
ordinary non-scoped records and explicitly owned parameters keep their existing ABI.
A bare unused record parameter can be inferred as borrowed; explicit Owned is needed
for the ownership boundary control. This existing inference was not changed.

The leaf-place helper validates a complete numeric leaf path before returning a
place rooted at its entry view reference. It rejects unknown paths and record
subtrees, which still need an explicit nested-view assembly operation. A native
component witness uses both read and mutable entry output, reads callbacks before
and after updates with result 6, and holds sibling mutable string references at once
through generated leaf-place expressions. Mode and path controls pass. Source is
formatted and a stable second pass is required before analysis.

Neither helper is connected to ordinary emission yet. The new routing witness
ordinary_borrowed_record_parameters_use_one_entry_view remains red on missing mutable
entry output. Signature generic constraints, associated callback/scalar bounds,
entry initialization for ordinary and completion bodies, selected leaf access and
nested borrowed arguments must be integrated together. Do not merely add an unused
entry borrow: current aggregate access would conflict with that borrow, and ordinary
parameter types cannot yet accept reconstructed borrowed child views.

No shared dispatch changed; previous depth evidence remains the latest evidence.
Original getter and independent-storage native failures remain open. Full editor,
guide, property-order/capture/completion coverage and matched benchmarks remain owed.
Consumers remain f1dd284. Fifteen owner fixtures remain untouched. This is a component
checkpoint with a retained integration red, not feature acceptance.

Exact entry API red, native component success, corrected ownership control and
ordinary routing red are verified in SQLite under
runtime/diagnostics/scoped-record-parameter-entry/. Project formatting changes zero
files and fingerprints.

## Ordinary scoped-record parameter routing

Ordinary source function signatures now use nominal read/mutate bridge parameters
for eligible borrowed scoped records. Each callback leaf has its own associated
trait bound rather than sharing a callback environment type across sibling fields;
concrete leaves retain equality constraints. Callback view slots also require Sized,
matching the existing nested view declarations. Redundant callback generic parameters
from these record inputs are excluded from the ordinary signature. The existing
result and other parameter bounds remain in place.

Parameter initialization establishes one flat transport view. Static leaf storage
selection starts from the corresponding view reference and retains subsequent index
and union selection behavior. Borrowed whole records and static record subtrees are
rebuilt as nested shared/mutable views from those independent references. Empty
records remain opaque leaves. Source parameter routing deliberately excludes inherited
capture slots: these are not source argument bindings and may not have a local valueN.
Their existing capture transport remains unchanged and still needs its own integration.
Layout eligibility is factored to avoid formatting an unused entry statement during
bound, leaf and subtree lookup.

Four parameter-view controls pass, including the previously red ordinary entry
witness. A native source program calls mutable child consumers through a parent
parameter, invokes distinct callback environments through readonly parameters, and
checks exact output 14 and left!/right! mutations. A complementary pure try/finally
program checks finally mutation and exact output done/right!. The checker sibling
argument witness is strengthened with native compilation and passes. All 64 required
depth controls pass. The scoped_callable_records project batch is again 19 passed,
with only its original E0502 independent-storage witness failing. Initial inherited
capture and unsized-slot regressions were repaired; their diagnostics are preserved.

Remaining boundaries are explicit. A new native red witness
scoped_parameter_can_borrow_a_plain_nested_record fails E0308: a scoped parent produces
a mutable child view, while a consumer of an ordinary callback-free record still
expects its owned-record ABI. Extend nominal borrowing compatibility to that child
consumer without reconstructing owned values or cloning. Parameter ABI coherence
must cover these ordinary child records as well as scoped children. The previous
plain-record ABI control documents the current boundary and must be revised only
with a corresponding native compatibility test when that boundary is implemented.

A try/finally variant retaining runtime-fallible array-index callback captures also
exposes unresolved error contracts for abstract record callbacks. An intermediate
variant with unused fallible callback fields reaches native Result-versus-scalar
callback trait errors. Pure completion routing is verified; fallible callback error
contract and carrier convergence remain open. Do not claim full completion parity
from the pure control or discard these failures. Original separate-getter E0499,
independent local storage, property ordering, captures, opaque composition, full
editor/guide and matched benchmark acceptance remain pending. Consumers stay f1dd284.
Fifteen owner fixtures remain untouched. No timing or feature acceptance is claimed.

Exact routing red/green, corrected regressions, completion boundaries, plain-child
red and depth evidence are verified in SQLite under
runtime/diagnostics/scoped-record-parameter-routing/. Project formatting changes
zero files and fingerprints.

## Plain child borrowing compatibility and nominal join registration

Native emission now computes the nominal borrowing family once. It includes each
validated scoped record root and its static record descendants, including ordinary
callback-free children. Parameter eligibility uses that family set, so a function
accepting a plain child can also accept its reconstructed shared or mutable view.
Unrelated ordinary record parameters retain their previous ABI. Empty descendants
remain opaque leaves and retain their owned-record parameter representation. The
family set and support declaration caches belong to one emission.

The previous E0308 plain-child witness now passes. It checks mutable child updates,
readonly length access, direct and deeper nested children, and exact native output
7/changed. Six parameter controls pass, including empty children, ownership/mode
boundaries, distinct callbacks and pure completion routing. The production metadata
adapter passes. All 64 required depth controls pass. The project scoped-record batch
remains 19 passed with its original E0502 independent-storage failure.

A property-order witness exposed two independent boundaries. The isolated witness
borrowed_plain_child_preserves_constructor_property_order still fails E0609 because
the borrowed view lacks key_order. It requires constructor order second,first to
survive the child view, not declaration order. Implement metadata transport for each
ordered nested node before claiming this borrowing family complete; Object.keys,
values/entries, console inspection and spread/rest must retain observable ordering.

The other failure was stale array join registration for generic/scoped record
metadata. The new record_join_emit module emits only nominal record registrations,
with the same generic field slots as the owned struct declarations. Tuple generic
slots receive TsrJoinText bounds, and tuple fields delegate nullable formatting to
the existing union join implementations. This removes references to nonexistent
specialized record names and missing generic arguments. Its new native witness
passes exact output a,b and 1; all four array_join integration tests pass, including
nullable leaves, nested commas, captures, allocation/disposal and argument loans.

The property-order witness was separated from join so each boundary has independent
evidence. No new public ownership wrapper, cloning or unsafe erasure is introduced.
Fallible callback contracts, captured-record entry transport, independent local and
owned-parameter storage, opaque composition, editor/guide and matched benchmark
acceptance remain open. Consumers remain f1dd284. Fifteen owner fixtures remain
untouched. This is an implementation checkpoint, not record feature acceptance.

Exact native child success, controls, metadata, project/depth evidence, property-order
red and join red/green are verified in SQLite under
runtime/diagnostics/scoped-record-plain-child/.


## Ordered borrowing views and explicit snapshots

Ordered records now carry property-order metadata through readonly and mutable
parameter transports at every nested record node. Mutable transports borrow the
original metadata exclusively; readonly transports borrow it without allocation.
Repeated child records retain separate order arrays. Nominal specializations agree
on metadata presence before support declarations are emitted.

Console inspection accepts borrowing views through the existing record traits.
Explicit record copies, spread and rest reconstruct owned records from borrowed
leaves and copy their order metadata. Object.values and Object.entries also copy
nested record owners rather than attempting to clone borrowing views. These paths
remain subject to the existing cloneability rules; callback-bearing snapshots are
not newly supported. No implicit clone or public ownership capability is added.

Fifteen parameter-view controls and four Object.values/entries integration tests
pass. The retained rest batch passes 23 tests and the record console batch passes
two tests. The project batch still passes 19 tests with the original independent
owned-field storage E0502 failure. Original failed runs remain evidence, including
metadata mutability, console view adaptation and nested snapshot failures.
All 64 depth canaries pass after the snapshot changes. Project formatting changes
zero files and zero benchmark fingerprints.

This is a foundation checkpoint, not scoped-record feature acceptance. Independent
local and owned-parameter field storage, captured-record transport, opaque
composition, fallible callback contracts, editor/guide and matched benchmark
acceptance remain open. Executable consumers remain at f1dd284. Fifteen protected
owner fixtures remain unstaged and untouched. Diagnostic evidence is stored under
runtime/diagnostics/scoped-record-order/ in benchmarks/records.sqlite.


## Independent local storage and record capture transport

Owned scoped-record locals now destructure once into independent native leaves.
Static selections move or borrow those leaves directly. Whole owning moves rebuild
nominal records without cloning. Whole replacement uses destructuring assignment
and evaluates the replacement before committing fields. A typed initializer keeps
absent callable union parameters determined before destructuring. Ordered nested
nodes retain independent metadata storage and borrow it through views.

Readonly and mutable record captures now store borrowing views of these leaves,
with the existing capture and environment lifetimes. Capture bodies establish a
single field transport rather than assuming a concrete owned record reference.
The original independent-field E0502 witness is green. Both live-capture rejection
controls remain unchanged. All 20 scoped-record project tests and 15 parameter-view
controls pass. The original four integration failures are preserved as evidence;
replacement, optional callable inference and two capture view failures are repaired.

This is an implementation checkpoint, not full scoped-record acceptance. Owned
parameters and deferred declarations still use their existing aggregate storage.
Their independent representation needs dedicated witnesses and entry/assignment
integration next. Opaque composition, mutable capture/disposal controls, fallible
contracts, editor/guide and matched benchmark acceptance remain open. No consumer
refresh or timing is claimed. Fifteen owner fixtures remain untouched.

All 64 depth canaries pass. Project formatting changes zero files and zero
fingerprints. Exact red/intermediate/green records are verified in SQLite under
runtime/diagnostics/scoped-record-independent/.


## Owned parameter entry and deferred record storage

Scoped owned record parameters now destructure into independent leaves after
parameter default initialization, preserving the existing nominal argument ABI.
Inherited capture slots remain excluded from owned parameter entry. Deferred local
records declare independent leaf variables and metadata arrays before assignment.
Callback leaves use inferred native types so nominal callback slots do not name
undeclared generic parameters. Existing definite-assignment checks still govern
all reads and replacements; assigning the record commits leaves only after its
replacement expression is evaluated.

A new deferred declaration witness exposed missing independent locals, then
undeclared callback generic types, and finally an incorrect local callback receiver.
The repaired native witness proves the left capture is released independently of
the right capture after a deferred record assignment. Local callbacks now borrow
owned native receiver storage when invoked instead of assuming an existing native
reference. The existing 20 project controls and this new deferred witness pass.
An explicit Owned<Pair> parameter with readonly callback fields also passes native
execution and an entry-storage source assertion. Its first proposed witness used
ordinary owned callback fields, which are outside the scoped borrowing family;
that witness was corrected rather than broadening claims from its failed assertion.

This is another representation checkpoint, not full record acceptance. Defaulted
and destructured owned parameter combinations, branch/loop/exception deferred
initialization, mutable record captures, disposal, opaque composition and fallible
callback contracts still need focused boundary coverage. Editor/guide and matched
benchmark acceptance remain open. No consumers or protected owner fixtures change.

All 15 view controls and 64 depth canaries pass. Formatting changes zero files
and fingerprints. The parameter entry assertion is isolated from main local
record storage. Exact evidence is verified in SQLite under
runtime/diagnostics/scoped-record-entry/.


## Branch-assignment carrier witness: implementation pending

The mutable record capture boundary passes native execution without another
implementation change. Its new witness mutates an independent string field,
invokes the captured mutable callback repeatedly, releases the record capture,
then invokes the original field and finally mutates its original scalar owner.
Expected assertions are 1, 2, changed, 3 and 7.

A separate deferred record branch/loop witness fails native E0308. Both branch
assignments satisfy the same nominal readonly callback contract but construct
Closure0 and Closure1. Independent leaf type inference chooses Closure0 after
the first assignment; the other assignment requires Closure1. The source is valid
and remains unchanged. This is a common-carrier gap, not a definite-assignment
failure or permission to merge capture ownership.

Record construction currently specializes each callback field independently in
analysis.rs scoped record construction (around lines 9220-9270). Conditional
callable expressions already select common borrowed families through
common_borrowed_conditional_family in conditional completion (around line 9045).
Branch statements assigning a record do not apply that conversion to the common
storage leaf. The next implementation must collect compatible origins for each
static record field storage slot, register their common borrowed carrier, and
normalize every assignment and later selected invocation to that carrier. Reuse
the existing callable contract checks and dispatch family; do not erase types,
clone captures or make source branches share a JavaScript capture cell. Keep
point-sensitive capture loans and independent disposal even though the native
carrier covers multiple possible origins. Nested fields, replacement and moved
record inputs require the same slot identity and conversion discipline.

Both new witnesses use formatting round trips and std:testing project execution.
This red-test checkpoint is not feature acceptance. No compiler source changed.
Continue implementation from the unchanged branch witness before adding unrelated
features. Earlier completed local, parameter and capture checkpoints stay closed.

Exact mutable-capture success, branch failure and project formatting evidence
are verified in SQLite under runtime/diagnostics/scoped-record-control/.


## Nested branch carriers and variable-cardinality borrowed families

Borrowed family identity now stores the complete sorted, deduplicated origin set.
Native declarations and dispatch support more than two concrete origins while
preserving the existing Left/Right ABI for two-origin families. Registration checks
every origin against the same argument, access, cardinality and result contract.
Compaction remaps the full origin set. This compiler metadata change does not clone
runtime environments or introduce a new public ownership capability.

Record branch normalization now collects assignments through nested blocks, loops,
if arms and exception regions. Compatible record literals materialize a common
nominal instance whose callback fields use the joined borrowed family. Existing
construction and replacement checks compare nominal identity and the written
callback contract before branch normalization, allowing another compatible concrete
origin without structural subtyping. Already wrapped literals retarget their storage
wrapper rather than retaining a redundant intermediate family owner.

All 25 scoped-record project tests pass. The nested three-origin test executes each
branch separately, observing 3, 7 and 11 and releasing the original borrowed array
before mutation. All 64 depth canaries pass. This is a literal branch representation
checkpoint, not complete record acceptance. Record-valued RHS expressions, independent
field replacement, sequential assignments outside branches and common carriers for
owning environments still need direct witnesses and conversion support. The normalizer
currently joins record literal expressions; do not claim arbitrary record assignment
coverage. Preserve point-sensitive capture loans and independent field disposal.
No consumers or owner fixtures change; no timing or full editor acceptance is claimed.

Thirty-one borrowed-related unit controls and ten callable storage controls pass.
Project formatting changes zero files and zero fingerprints. Exact intermediate
and final records are verified in SQLite under runtime/diagnostics/scoped-record-many/.


## Record-valued conversion and sequential replacement

A checked RecordCarrier node now converts compatible nominal records into joined
storage carriers. Its isolated emitter evaluates the source once, moves original
leaves, reconstructs nested records and forwards property-order metadata. Existing
borrowed-family payloads dispatch into the destination family by their actual origin.
No cloning, unsafe conversion or public syntax is introduced. Capture-loan payload
traversal treats the conversion as ownership-preserving, and the internal node keeps
the original source-depth accounting.

Branch assignments can now join record-valued RHS expressions rather than requiring
both RHS nodes to be literals. The new native witness executes both selections and
uses a side-effecting comma expression to prove one RHS evaluation. Different source
constructor orders survive the conversion and Object.keys observes the selected order.
Expected callback results are 3 and 11; the original arrays are mutable after all
owners are disposed.

Sequential local record writes also normalize their compatible native field carriers.
Construction checks the declared callback contract rather than requiring the previous
concrete origin. Checking records the actual replacement instance at each program
point. Local write normalization runs after statement checking, retaining the existing
point-sensitive loan decisions. The replacement helper now reconciles live capture
loans after staging and transferring the RHS, removing discarded old owners while
retaining independently moved or borrowed owners.

The sequential native witness observes 3 before replacement and 11 after replacement,
then mutates the released first array while the second callback remains live. The
27-test project batch, seven capture-loan unit controls and all 64 depth canaries pass.
This is an implementation checkpoint, not whole record acceptance. Normalization of
sequential writes currently covers owning local binding writes in one block. Cross-block
writes, independent callback field replacement, borrowed parameter replacement,
record-valued capture conversion and owning callback carrier composition still need
boundary witnesses and implementation. Complete default/exception/disposal, opaque
composition and fallible contracts before editor/guide and matched benchmark acceptance.

A separate rejection control verifies that replacement cannot release a capture
owned by a moved-out callback. Formatting changes zero files and fingerprints.
Exact red/intermediate/green evidence is verified in SQLite under
runtime/diagnostics/scoped-record-conversion/.


## Callback field replacement and cross-block local writes

Local callback field replacement now checks the declared nominal callback contract,
rather than demanding the preceding concrete environment origin. After access checks,
analysis materializes the replacement field instance at that program point while
preserving the types of sibling fields. Existing capture ownership replacement keeps
its field path, so releasing the old field does not release the sibling or another
owner of a moved-out callback.

The storage normalizer now collects relevant whole-record and callback-field writes
through nested blocks, branches, loops and exception regions. It ignores scalar writes.
Whole and selected-field writes join compatible carriers by binding identity and static
field names. A whole record initializer can convert its selected callback to the common
carrier while forwarding the remaining leaves and order metadata. Subsequent field
writes wrap their original environments in that same carrier. Ordinary source callback
ownership capabilities and nominal record identity remain unchanged.

The new field native witness observes 3 before replacement, 7 after replacement and
11 from the unchanged sibling. The released first array can mutate while both remaining
callbacks stay usable. The cross-block whole-record replacement witness also passes.
Thirty project tests and seven capture-loan unit controls pass. Three separate rejection
cases retain conflicts for the moved-out original callback, the new callback and the
unchanged sibling. This is a representation checkpoint, not whole record acceptance.

Borrowed parameter replacement, captured carrier replacement, expression-level stores,
loop-dependent origin changes, defaults/exception disposal and owning callback carrier
composition still require focused coverage. Native conversion currently preserves the
existing borrowed family representation; do not claim all owning or opaque callbacks
are supported. Finish those boundaries and fallible contracts, then editor/guide and
matched benchmark acceptance. No consumer refresh or timing is claimed.

All 64 depth canaries pass. Formatting changes zero files and fingerprints.
Exact red/green and retained-conflict evidence is verified in SQLite under
runtime/diagnostics/scoped-record-writes/.
