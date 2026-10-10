# Gate 189 native byte view integration

This is the implementation route for the still-pending view half of gate 189.
Strict UTF-8 is independent and can use the gate 188 owned-value ABI. It does not
accept gate 189, replace views with copies, or close S07. The read.tsr fixture in
native-byte-views is a retained red CLI probe, not an accepted executable fixture.
Currently it fails because view is not exported; keep that original diagnostic.

## Current architectural facts

Native declarations project into ordinary nominal records. Their private runtime
storage is emitted separately. Signature supports nominal Named/Array/Result shapes, including private borrowed
identities. NativeBinding describes argument access and returned source contracts;
source provenance and lifetime-bearing emission remain pending.
Result bridges currently return owned union alternatives without a source lifetime.
The existing ViewUnion borrows an existing union object: it does not make that
union's owned payload a newly returned native slice.

borrowed_owner_provenance.rs plans callback storage owners. Scoped record parameter
views transport borrowed callback leaves. These are useful precedents, but neither
is a lifetime-bearing arbitrary native byte view. Do not reuse a callback marker
as a byte owner or reinterpret a Bytes owner as a slice. Do not erase lifetimes,
return integer handles, keep a hidden shared allocation, or clone the source.

## Public surface and representation

Introduce nonconstructible native ByteView and ByteViewMut identities. view and
viewMut accept strict half-open ranges and return Result<ByteView, ByteError> and
Result<ByteViewMut, ByteError>. Use the same start-before-end validation and errors
as copy, including empty end ranges. A mutable view writes through to the owner.
Readonly views cannot write; neither view is an independently cloneable owner.

Keep length/at/toUtf8/copy useful for readable byte owners and views; set is useful
for mutable owners/views. Add typed native argument alternatives or bounded native
buffer contracts in shared metadata, with consistent stored-function signatures.
Do not silently substitute a new collection of view-only observer names. No dynamic
any value, representation cast or display-name dispatch is needed.

Generated storage must retain actual Rust &[u8] and &mut [u8] lifetimes. Readonly
and exclusive views need distinct identities. Any record/array/Result wrapper that
stores a view must carry its lifetime as well. Emit borrowed parameters and results
with explicit named source lifetimes where Rust elision is ambiguous. Return an
owned byte buffer only through explicit copy/clone of an owner.

## Integration increments

1. Add returned-borrow contracts to native metadata: source parameter identity,
   readonly/exclusive access and the borrowed result payload. Validate source indices,
   required/non-rest borrowing parameters and mutability before publishing bindings.
   Extend typed native declarations, signatures and recursive projections without
   relying on public aliases. Keep owned native bindings and old callers unchanged.
2. Add small dedicated native-view provenance/loan helpers. Native factory calls
   attach source roots to returned views, including Result success payloads. Propagate
   those roots through bindings, moves, branches, returns and forwarding functions.
   Infer source parameter relationships; no public lifetime syntax is required.
   An owned local or consuming parameter cannot supply an escaping returned view.
3. Extend native record/union ABI emission to carry safe borrowed storage through
   parameters, local wrappers and returns. Reuse existing scoped aggregate machinery
   where it actually preserves the needed lifetime. Do not emit an empty source record
   as a native slice. Unsupported intermediate forms must get source diagnostics.
4. Publish factories and readable/mutable byte contracts only when analyzer and
   backend agree. Add complete catalog help, aliases, completion, hover, signatures,
   parameter docs and diagnostics alongside each supported form. Run normal-stack
   depth canaries after analyzer dispatch changes.
5. Finish nested reborrows, returned/stored/captured views and all exit paths. Then
   add zero-allocation/native pointer-identity controls, matched Uint8Array subarray
   workloads, independent result oracles, source guards/hashes and untimed verification.
   Only accept gate 189 once the complete view and UTF-8 matrices pass.

## Required positive and rejection witnesses

A local readonly view observes the selected bytes without allocation. A local mutable
view changes its owner, and owner access resumes after the view ends. Empty end views
are valid; negative, reversed and oversized bounds report structured errors without
changing the owner. Nested readonly/exclusive reborrows retain the original owner
and suspend conflicting parent access until the child borrow ends.

Returning a view from a borrowed parameter is valid with source provenance. Returning
one from a local owner or consuming parameter is rejected. Multiple potential borrowed
parameter origins retain every required loan; ambiguous lifetimes must never become
static. An owner-producing temporary cannot back a stored view after its statement.
Immediate temporary use needs a proved scope rather than hidden ownership promotion.

While a readonly view lives, reads are allowed and mutation/resize/transfer is rejected.
An exclusive view blocks conflicting reads, writes, resize and transfer. Overlapping
mutable windows are rejected conservatively unless disjointness is actually proved.
Moving a view transfers its loan; ending it releases the loan. Result Err paths retain
no success payload and must release their source loan. Return, throw, break, continue,
finally and discarded closures must release views correctly.

Scoped aggregate/capture storage may not outlive the owner. Stored factory function
values must preserve the same source relationship as direct calls; erased callable
signatures cannot invent a static lifetime. Alias imports and unfinished editor buffers
must retain useful public help without exposing private identities.

## Closing evidence

Keep original failing probes and focused repairs. Verify formatting tokens,
idempotence and formatted native execution. The native zero-copy control must show
no reservation or hidden owner copy on successful view creation and exact source
storage identity. Mutable writes must reach the owner. Preserve O0/O3 cleanup through
all exits. Add the matched view workload before acceptance, not during the later
family timing run. Full regressions and measurements remain the S07 family milestone.

## Metadata and slice-helper checkpoint

The returned-borrow contract is now explicit in NativeBinding. It identifies the
source parameter, readonly or exclusive access, the nominal payload identity, and
either the direct return or Result Ok slot. Validation rejects absent, optional,
rest, consuming and copy sources, including sources after optional/rest parameters;
an exclusive result requires a mutate source.
The source must describe storage, and the selected result slot must have the exact
payload identity. Export lookup preserves contract errors rather than reporting a
missing export. All current public bindings explicitly retain owned results.

Type projection validates the contract before interning anything. Until provenance
and lifetime emission are connected, it rejects even a structurally valid returned
borrow. This guard is deliberate intermediate safety, not accepted view support.
The first metadata checkpoint used Bytes as a synthetic payload without publishing
such a binding. The identity checkpoint below replaces that placeholder with actual
borrowed declarations and rejects owned payloads explicitly.

The isolated views_runtime.rs backend helpers return actual &[u8] and &mut [u8].
They share checked_range with copy, preserving start-before-end diagnostics and
empty end ranges. Readonly and mutable nested reborrows retain source storage;
mutable writes reach the source. These helpers are not yet emitted or exported by
Tsrust. Their Rust tests do not prove Tsrust source provenance or lifetime checking.
The retained read.tsr probe remains red until that integration is implemented.

## Borrowed identity checkpoint

ByteView and ByteViewMut now have distinct native identities and borrowed
representations with readonly and exclusive access. Neither is source constructible,
independently cloneable, collection-key eligible or independently thread transferable.
They remain absent from source exports until lifetime and source-loan integration is
ready. Their identities intern and project recursively without becoming Bytes or
sharing an arena identity with the other view kind.

Returned-borrow validation now requires a genuinely borrowed payload with exactly
matching access. An owned Bytes result cannot masquerade as a borrow. An exclusive
borrow cannot originate from a readonly native view. Borrowed results cannot omit
their contract, including borrowed storage nested in arrays or Result alternatives.
A native Result Ok borrow contract cannot hide an additional borrow in its error
payload. This restriction concerns the single-source native contract, not a new
language-wide restriction on Result types.

The bounded iterative signature walk also recognizes borrowed parameters. Projection
therefore rejects a view-taking observer before interning anything, even when its
result is owned and it has no returned-borrow contract. This prevents an observer
from bypassing the pending ABI guard. Fixed-storage analysis treats borrowed native
representations as native storage, not as empty source records. Source factories,
loan propagation and lifetime-bearing emission remain the next required work.

## Private storage ABI checkpoint

The emitter now represents private ByteView and ByteViewMut records as actual
`&'a [u8]` and `&'a mut [u8]` fields. Nonrecursive record fields and Result payloads
carry these types through generic storage positions; array aliases retain the slice
lifetime. Borrowing a Result preserves all its non-copy payload type arguments.
Native view storage does not use the callback-only nominal record bridge. Inline
record-array selection does not infer a small value from an opaque view's empty
source schema.

A cached reverse reachability index identifies data shapes containing borrowed
native storage, including cycles, nested array ranks and union alternatives. Programs
without native views allocate no graph. This index describes storage only; it does
not establish source loans or make a recursive storage layout emit correctly.

The private AST witness checks formatted-source idempotence, then compiles generated
Rust at O0 and O3. It stores views in a record, an array and a Result inside another
record, checks exact source pointers, and writes through an exclusive view. A readonly
Result parameter is also emitted and compiled. These tests prove the isolated
nonrecursive storage ABI, not source factory calls, returned lifetime inference,
recursive wrappers, callback capture composition or source ownership diagnostics.
The native projection guard and missing public view exports remain in place.

## Retained source-loan integration checkpoint

Native view loans now have dedicated binding-identity storage, separate from
optional-element views and callback capture markers. Each retained loan records its
source binding, access and diagnostic location. An ordered holder index avoids
allocation or scanning unrelated bindings on ordinary ownership accesses. The
initializer walker is skipped when the function has neither retained views nor a
checked native borrow-producing call.

Analyzer access and replacement checks now consult these loans. Readonly loans block
mutation and transfer; exclusive loans also block reads. Moved holders stop enforcing
the old loan, while the moved value can retain it. Out-of-scope or shadowed holders
are ignored by current lexical identity. Native factory checking validates stable
source alternatives. A conditional cannot hide a temporary behind another named
source. Nested reborrows retain the original owner and the parent view identity.
Binding initializers and whole-binding replacement propagate native loans through
supported value wrappers and aggregate literals.

These are private analyzer integration checks, not complete source-view acceptance.
The tests inject retained loans into real analyzer contexts because public factories
remain guarded. Branch-state merging, Result success/error projections, precise field
provenance, captures, all control-flow exits, last-use release and return lifetime
relationships still need implementation and source-level witnesses. Do not remove
the projection guard based on these checks. Existing source programs do not acquire
new borrow-producing bindings at this checkpoint.

## Loan availability joins

Native loan origins and their retained state now participate in Availability snapshots,
restoration and joins. Whole-binding consumption marks the old holder's loans dormant
without deleting origins needed by the moved value. Initializing the destination
reactivates those origins. A join retains an owner loan if either reaching path retains
it, even when the binding is unavailable because another path transferred it. A join
of two transferred paths does not reactivate the old holder.

The initial holder index could become stale when Availability restored a binding.
It is replaced by a stable binding scan in functions that have encountered native
views. Ordinary functions retain a constant empty fast path with no scan or allocation.
Visible lexical identity still excludes dead or shadowed holders. This supersedes the
ordered-index detail in the preceding checkpoint description.

The focused tests now include parsed if statements checked through the actual analyzer
with private loans injected into the starting context. They verify replacement from
different origins in the two arms, restoration, one-path transfer and two-path transfer.
This is still private integration evidence. Result variant projection and path-specific
success-loan release, precise field paths and return lifetime inference remain open.

## Result and value-relative projections

Retained native loans now carry value-relative storage paths and variant guards.
Native Result factories associate their checked union identity with its success
record, and success loans occupy that record's payload field. This map is allocated
only after a checked native Result borrow call. Source aliases and diagnostic spans
do not establish variant identity.

The isolated native_view_projections module handles wrapping, field/element selection,
variant projection, flow feasibility and normalization. Projecting Err discards loans
that require Ok. Projecting Ok removes the satisfied guard; selecting its payload
preserves an unconditional loan in the resulting view. Flow facts on a Result or a
nested record field release a success loan on Err paths only. Unknown or joined paths
retain every potentially live loan. A guarded sibling cannot discharge an unrelated
unconditional loan to the same owner.

An iterative value walker preserves paths through record and array literals,
conditional values, injections and projections. Record field selection no longer
inherits every sibling's provenance. Array element paths remain conservative across
indices. Normalization and Availability joins distinguish paths and guards, so loans
to the same owner in different locations cannot silently collapse into one condition.
Reborrow source projection inherits only the selected value's provenance before
resetting paths relative to the new native view.

Private analyzer tests cover parsed, formatted Result matches: owner replacement is
accepted in Err and rejected in Ok. They also check payload transport, sibling field
selection and guarded/unconditional loans to one owner. These tests inject native
loans into real analyzer contexts; no public factory or end-to-end source/native view
execution is claimed. Owner access still conflicts at whole-binding granularity.
Field-sensitive owner conflicts, projection moves/replacement, source escape and
return lifetime inference remain pending, along with the broader view matrix.

## Owner places and projected record lifecycle

Loans now distinguish the borrowed owner's field path from the value-relative holder
path. A shared or exclusive loan on one record field conflicts with that field, its
parents and its descendants, while disjoint sibling fields remain usable. Whole-owner
replacement still conflicts with every borrowed field. Native functions compute direct
field access footprints even before a partial move has occurred; ordinary functions
retain their existing path-allocation fast path.

native_view_places isolates footprint selection, overlap, source alternatives and
record-field replacement. Conditional borrow sources retain distinct field paths even
when both alternatives have the same owner binding. Indexed sources use the containing
array footprint; this does not prove disjoint byte ranges or array indices. Reborrow
provenance retains the original owner's path and the selected parent storage path.

Consuming a record field deactivates only the old holder loans under that field. The
moved value reactivates its selected origins, and sibling loans remain live. Replacing
a direct record field removes its old origins and installs the new value's prefixed
origins without replacing sibling provenance. Normalization and joins include owner
paths as well as holder paths and variant guards.

Private analyzer witnesses check parsed, formatted sibling mutation, field replacement
and field moves. The moved value blocks its owner inside its scope and releases that
loan when its scope ends. These checks still inject native loan state; they do not
publish factories or prove native execution of source views. Dynamic indexed stores,
collection removal/clear, source escapes, returned lifetime inference, captures and the
remaining control-flow/composition matrix still need implementation and witnesses.


## Returned-source analysis checkpoint

The dedicated native_view_returns module traces returned loan origins through local
aliases to source parameters. Returning a borrow of local owner storage or a consuming
owner parameter reports native-view-escape. Final parameter access inference rechecks
the retained summary. Checked functions preserve the summary for later lifetime emission.
The source trace is iterative and diagnoses alias cycles without recursion.

Loan origins distinguish Storage from Carried lifetime slots. Moving an input view
can preserve its carried lifetime even when the input is consumed; borrowing owned
storage in that input cannot. Normalization keeps these sources separate. This avoids
allowing a consumed mixed record merely because another field contains a view.

Private tests inject origins into real return analysis. Parameter carrier seeding,
backend named lifetimes, forwarding calls and exception completion reconciliation
remain pending. Carried is currently constructed only by these private tests, so a
non-test build reports its unused-variant warning. The retained summary can conservatively
include returns superseded by finally. No public borrowed signature is enabled, no
native returned-view execution is claimed, and gate 189 remains open.

## Input carrier origin seeding

native_view_parameters seeds dormant Carried origins for native view inputs before
checking callable bodies. Record fields, nested array ranks and tagged union members
retain separate value-relative paths. Union origins retain their variant guards.
Moving an input view can therefore return its inner lifetime without pretending that
the function's local parameter slot survives. Dormant seeds do not create a conflicting
self-borrow. Subsequent storage of the view activates its ordinary retained loan.

A program-level native declaration flag keeps ordinary callable checking on a constant
empty path. Parameter shape walks and cycle checks are iterative. Recursive input
containers that reach borrowed native storage currently report native-view-shape;
unrelated recursive types remain valid. This is an explicit intermediate limitation,
not acceptance of recursive view layouts. Map/Set storage, callback carriers, capture
seeding, forwarding contracts and named backend lifetimes remain pending.

Formatted source tests now verify consumed view identity, record fields, nested arrays,
Result success guards and independent parameter origins. These use private native type
identities injected into parsed source, because public view exports remain guarded.
They establish source analysis and checked summaries, not returned native execution.
The Carried variant now has a production constructor; its earlier unused warning is
resolved. Gate 189 and S07 remain open and unmeasured.

## Named function lifetime emission

native_view_lifetime_emit assigns a distinct named lifetime to each carried input
slot. Function result slots use their checked origin summary. A result supplied by
one input uses that input's lifetime directly; a result that can come from several
inputs gets its own lifetime with an outlives bound from every possible source.
Unrelated input lifetimes are not combined. Storage dependencies separately name the
borrow on their parameter. These declarations integrate with the shared native
function signature planner and its existing callback and record bounds.

The signature renderer preserves borrowed native storage through record generic
fields, array ranks and union payloads. It does not rewrite anonymous lifetime text
or infer a source from argument order. Ordinary types retain the existing renderer.
The current output grouping uses value-relative paths; recursive layout, callback
composition and forwarding contracts still require further work.

Private parsed-source native witnesses compile at O0 and O3. They check pointer
identity after returning a view, a selected record field, an array and a Result,
exclusive write-through, and alternative input selection. A returned first field
outlives a separate unused field's owner. An independent rejection control confirms
that a selected result cannot outlive either possible source. These witnesses inject
private native identities and call generated functions from a Rust harness. They do
not publish source factories or prove source-call lifetime propagation. Keep the
projection guard until those paths and the remaining gate matrix are complete.

## Source call origin transport

native_view_calls maps checked function return summaries from formal source slots to
actual arguments. A context stores summaries by checked function identity, not by
source span. Repeated calls reuse unchanged summaries; re-analysis replaces changed
ones and removes absent ones. Ordinary programs retain their native-free fast path.

The existing iterative value walker transports carried slots through source calls
and saved named-function calls. Selected input fields retain only their own origins;
nested and sibling calls preserve distinct output paths. Storage dependencies retain
both the actual source place and inherited source loans. Returned capture slots and
recursive calls without a stabilized lifetime summary receive source diagnostics.

A saved named function's native adapter now uses the target's complete named signature,
including its result lifetime. Previously its anonymous result lifetime could bind to
self instead of its input. Private formatted-source native witnesses exercise direct
forwarding and saved named-function forwarding at O0 and O3 with pointer identity.

This does not establish arbitrary stored callable families, callback parameters,
closure methods or captured view returns. Storage-return transport still needs native
factory witnesses, and exceptional completion reconciliation remains open. Public
view factories stay guarded. Gate 189 and S07 are not accepted or measured.

## Ordinary arrow invocation lifetimes

Call summary keys now distinguish named functions, closure identities and specialized
closure methods. Closure source binding IDs are mapped through the checked parameter
list rather than assumed to equal argument positions. Missing summaries remove stale
cache entries. Returned captures remain explicitly outside argument-only transport.

Ordinary closure methods can use the shared native signature planner for view-bearing
arguments and results. This prevents Rust receiver elision from tying an input-backed
result to self. Specialized method dispatch is wired to the same planning route, but
its broader callback matrix still needs witnesses before acceptance.

Private formatted-source tests verify ordinary arrow forwarding and alternative input
origins with a scalar capture. Generated native tests exercise both at O0 and O3,
checking exact source pointers. Captured native views, arbitrary stored callable
families, abstract callbacks and receiver lifetime relationships remain pending.
No source factory or public view API is enabled by this checkpoint.

## Captured input provenance

Returned call summaries now distinguish invocation arguments from lexical captures.
The captured route selects the original binding's carried view loans and projects
them through the closure's returned slot. Capture binding IDs are not interpreted
as argument positions. A private formatted-source witness checks a zero-argument
closure capturing a local alias of the second enclosing parameter.

This is analyzer groundwork only. Captured owner storage still requires a receiver
lifetime contract. Native capture layout, generated result lifetime relationships,
escaped closure environments and arbitrary stored callable families are not yet
verified. The public borrowed-signature guard remains in force; this increment does
not accept captured view execution or close gate 189.

## Native capture lifetime slots

Native view leaves now contribute inner lifetime slots to capture environments,
separately from any reference used to borrow the captured value itself. The shared
signature planner accepts named capture sources and uses those environment names
for returned carried origins instead of declaring unconstrained method lifetimes.
Record fields preserve their individual slots. No slice is copied and no owner is
hidden in the generated environment.

Private formatted-source native witnesses cover consuming closures returning a
captured read-only view, a selected view field from a captured record, and a
captured writable view. O0/O3 execution checks pointer identity, independent sibling
lifetimes and writes reaching the original owner. A separate rejected Rust harness
checks that a captured result cannot outlive the original owner. Lifetime counting
also checks native array leaves without changing ordinary recursive array capture
counts.

Borrowing receiver contracts, captured owner storage, Result capture layouts,
escaped and rebound closure environments, specialized callbacks and arbitrary
stored families still require their own matrix. The source factories remain guarded
and gate 189 remains open.

## Captured Result payloads

Native view unions use the complete capture environment shape rather than the first
non-nullish alternative. Owned union fields and read-only union views retain named
lifetime arguments for their non-copy variants. Private native witnesses cover a
consuming `Result<ByteView, int>` capture on both Ok and Err paths, and a read-only
callback matching either path without taking the caller's Result. O0/O3 execution
checks success pointer identity and the unchanged error payload.

Closure context projection now remaps explicit ReadOnly parameter facts alongside
Mutable facts. Losing the ReadOnly fact previously made a captured Result match
request ownership. An ordinary Result regression checks a borrowed record payload
and a string error through a captured match, independently of native view metadata.

These checks do not yet establish variant-distinct lifetimes when both Result
payloads contain views, borrowed receiver returns, or escaped callback environments.
The full view plan and public signature guard remain in force.

## Variant-distinct carried lifetimes

Guarded input slots now retain their nominal variant identities as part of the
carried lifetime source, independently of the value's current output guards.
Unguarded inputs keep the existing compact carried-path form. Native signature and
capture planning use both storage paths and variant guards; Ok and Err payloads
with the same field spelling no longer share a lifetime slot. Output lifetime
grouping also distinguishes guarded variants. Call transport rejects contradictory
source variants before projecting their payload paths.

The formatted native witness covers a captured `Result<ByteView, ByteView>` with
independent payload lifetimes. At O0/O3, each selected variant can outlive the unused
variant's shorter lifetime and retains its original pointer. Checked provenance
contains exactly two distinct sources rather than cross-joining alternatives.
Separate rejected compilations verify that neither selected variant can outlive its
own byte owner.

This increment does not establish every nested union/container or escaped callback
composition. Receiver return contracts and the remaining gate matrix stay open, and
the public factory guard remains enabled.

## Views borrowed through shared captures

A read-only closure capture may return a read-only view of its external owner's
storage. Its call summary now distinguishes this physical owner dependency from a
captured value that already carries a view lifetime. The returned loan retains the
outer owner root and owner-field footprint, including relevant inherited origins.
It blocks owner mutation and transfer while retained. Native signature planning
uses the capture's existing shared-reference lifetime for this storage dependency.
The result need not borrow the shorter-lived local closure value itself.

Private metadata injection verifies analyzer transport and owner conflicts. A
separate native control uses the compiler's actual planned signature with a safe
String-to-byte-slice body: at O0/O3 the view retains the owner's pointer after the
local closure value is dropped, and owner escape is rejected. This is signature and
provenance acceptance, not an end-to-end source-factory acceptance witness. Source
factories remain guarded.

Captures that own the storage or hold a mutable reference still require receiver
lifetime contracts. Returning exclusive access through a shared capture remains
rejected. Escaped/rebound environments and the full callback/storage matrix remain
open; gate 189 is not accepted.

## Receiver-owned storage contracts

Returned storage borrowed from an owned or mutable capture now identifies the
caller's stable closure binding as its physical owner, with a capture-field
footprint. It does not retain the moved-from outer variable. Shared results permit
read-only invocations while blocking mutation and transfer; exclusive results also
block reads. Returning a view of a local closure's storage is diagnosed as an escape.
Consuming invocations remain rejected for these storage-borrow summaries; transferred
capture borrows still need separate planning.

The shared native signature carries an explicit receiver mode. Ordinary and
specialized method headers name a receiver lifetime and use it for returned storage
slots. Other function signatures keep their existing receiver-free form.

Private analyzer controls check receiver identity, owner conflicts, release and
consuming-invocation rejection. Native controls use the actual planned headers with
safe Vec-backed bodies for both owned storage and mutable owner references. O0/O3
checks verify pointer identity, shared concurrent reads,
exclusive write-through and reacquisition after the view's scope ends. Independent
negative compilations reject receiver escape, owner mutation/moves and overlapping
exclusive calls.

These remain private provenance/signature controls, not complete factory-generated
source acceptance. Source scope-exit/drop enforcement, unused consuming closure
validation, mixed callback-record contracts and escaped/rebound environments require
further witnesses before publication. The public borrowed-signature guard remains.

### Lexical holder lifetime checks in progress

The retained-borrow scope hook now checks native holders that survive the current
block. A holder cannot retain storage declared in that block when the block ends.
Replacing the view before exit is valid, and a local holder may expire alongside
its owner. Diagnostics identify the retained view and the source owner location.

Shadowing hides a name without destroying its storage. Native access checks now
retain the identities of shadowed holders until their lexical scopes end. This
supersedes the earlier statements that shadowed holders are excluded: only dead
holders should release their loans. The tracking allocates only for programs with
native borrowed types and an actual shadowed binding.

The new private witnesses parse and format source blocks and exercise real call,
assignment and scope analysis against an injected borrowed factory interface.
They do not provide complete public factory-body acceptance. Exceptional exits,
loop completions and escaped callback environments still need their own coverage.

The next private source batch verifies break and continue exits, typed throw/catch
paths and finally replacement through the existing exception callable checker.
Dangling outer holders are rejected; dead local holders release access; replacing
an outer view in finally releases its owner, whereas retaining it through catch
continues to block mutation. These controls pass without further production edits.
They cover these concrete paths, not all completion joins or borrowed throw payloads.

Final analysis now validates returned storage-borrow contracts for every checked
closure and specialized method, including unused closures. The same storage access
validator is used when mapping call summaries. A consuming invocation cannot return
a borrow of owned or mutably captured storage that its invocation destroys. Shared
capture storage may return readonly access, and an exclusive receiver view requires
mutable invocation. Carried views remain distinct from borrows of physical capture
storage and are not rejected merely because an invocation consumes its environment.

The private unused-closure witness checks a real parsed consuming closure and then
injects its returned native storage summary before final analysis. It proves the
guard runs without a call, with shared-invocation and carried-view positive controls.
Complete source factory-body acceptance remains pending.
