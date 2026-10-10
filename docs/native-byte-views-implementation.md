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
