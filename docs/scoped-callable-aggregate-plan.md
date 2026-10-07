# Scoped callable aggregate plan

This continues Gate173 after scoped traversal and nullable transport. The goal is
local record and array storage of callable families that borrow their environments,
with ordinary lifetime checking and no implicit clone. Existing explicitly owning
callback storage contracts must remain owning. No new public ownership wrapper is
planned. Start from failing source/native witnesses before choosing internal layout.

## Record gate

Establish an inferred local record containing a scoped conditional callable, then
invoke the field while its captured owner is live. Cover readonly and mutable
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
