# Checked resource observation

Developer reference for the `std:testing` lifetime-acceptance resource.

`std:testing` provides a narrow synthetic native resource for lifetime
acceptance. It does not generalize the external Rust data bridge. Exported Rust
records implementing `Drop` remain rejected because field projection would lose
the native owner's identity and destruction semantics.

Import `TrackedResource`, `trackedResource`, `resourceId`, `closeResource`, `releaseCount`, `releaseOrder`
and `liveCount`. `trackedResource()` allocates native storage with a unique positive
process-local identity. `resourceId(resource)` borrows the live owner. `closeResource(resource)`
consumes it. Ordinary scope exit, replacement and discarded owned values invoke
the same native destructor. Cloning, source construction and field projection
are unsupported. A returned resource transfers ownership to its caller; its
integer identity never extends its lifetime.

The destructor releases its allocation before appending its identity to an
independent audit. `releaseCount(resourceId)` is zero while live and one after destruction.
`releaseOrder(resourceId)` is the one-based destruction position, or zero before release.
Unknown, zero and negative identities return zero. `liveCount()` reports created
minus released resources. There is no reset, release-by-ID or registry-owned
resource. Audit storage contains integers only and grows until process exit;
use fresh bounded test processes. Identity exhaustion fails before reuse.

See [the executable example](../../examples/resource-lifetimes/release.tsr).
Assertions use the public audit without retaining the resource. Negative controls
must reject premature release, missing release, duplicate release and reversed order.

## Transport and cleanup boundaries

This resource is deliberately thread-affine: native storage is non-Send/non-Sync,
and checked metadata rejects worker transfer, worker captures, channel transport
and started tasks containing it. A mutex around the audit does not grant transfer
permission to a resource. Plain owned synchronous calls, local callbacks and direct
local async ownership are separate from thread transport. A local active use must
retain ownership or a checked loan until it finishes; it cannot access a resource
after transfer or release.

The event proves synchronous native allocation destruction, not SQL rollback,
network shutdown or completion of an asynchronous backend destructor. Cancellation
requests, task settlement and backend cleanup completion remain distinct. Existing
Job settlement rules and SQL pool/transaction borrowing continue to apply.

UI owner mutation must stay with its serialized owner executor. Background results
must return through checked completion transport and revalidate runtime, owner and
instance identity. This test facility provides no scheduler, executor dispatch,
concurrent identity allocator or permission to share mutable owner access. UI
readiness additionally requires the B2 state/capture/payload matrix, callback-safe
release, automatic owner destruction and descendant-before-ancestor order.

## UI owner candidate

The UI experiment groups each component's resource state and retained callback in
one registration. Registrations remain in descending creation order: without
reparenting, every descendant is newer than its ancestors. Display reordering and
FIFO delivery do not change this destruction order. Dispatch extracts only the
callback slot, leaving state available through the owner. It restores the callback
before processing deferred retirement, including when the callback throws.

Resource queue entries own accepted payloads until delivery finishes, stale delivery
is discarded, or the owner is destroyed. Rejected enqueue returns the original
resource in `Err`; rejection does not destroy or clone it. Queue payload destruction
has no tree-order guarantee. These are synchronous candidate contracts exercised by
`experiments/ui-tsrust-owner-handles/witnesses/tracked-release.tsr`; the [synchronous UI matrix](../../experiments/ui-tsrust-owner-handles/CHECKED-RESOURCES.md)
is accepted within its explicit transport and executor limits.
