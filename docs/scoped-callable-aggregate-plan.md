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
