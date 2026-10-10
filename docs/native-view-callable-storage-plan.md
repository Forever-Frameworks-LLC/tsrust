# Native views in owning callable storage

This is an implementation dependency of gate 189, not a new ownership policy.
A callback may own its environment while that environment contains a view borrowed
from another owner. Owning the callback must not imply that its payload is static.
The original owner's lifetime must follow the callback through storage and calls.
Public byte view factories remain guarded until the complete integration is ready.

## Current evidence

The private source witness in stdlib/bytes/views_stored_emit_tests.rs constructs
`Holder { read: Once<Function<Window>>, marker: int }`, where Window is projected
as the native readonly byte view. Its factory returns a callback consuming the
captured view. The caller supplies two independent inputs and selects the second.
Analysis rejects construction with callback-escape in StorageContract::environment.
This is the retained red witness; it is not an accepted positive feature.

A separate mixed record containing a direct view and an ordinary callback passes.
Its complete emitted code runs at O0/O3, preserves the selected input pointer after
an unrelated owner ends, and independently rejects selected-owner escape. Thus
ordinary record wrapping is not itself the missing dependency.

StorageContract::environment rejects closure.borrowed. Family::checked_storage
also requires lifetime_count zero. These guards protect the existing owning family
representation, whose identity and native ABI do not retain payload lifetime slots.
Removing either guard alone is unsafe and does not implement this feature.

## Implementation sequence and gates

1. Define typed scoped owning family metadata in callable_storage.rs, separate from
   the existing borrowed invocation family. Retain the original Owned/OwnedOnce
   invocation contract and finite concrete alternatives. Describe each alternative's
   carried native-view paths and lifetime slots. Do not reclassify physical shared
   or mutable captures as owned storage. Add unit tests distinguishing an owned
   view payload from an environment borrowing local callback storage. Existing
   wholly owned families must retain their current identity and ABI.
2. Add an isolated native callable provenance helper. Construction must transport
   each alternative's capture loans into family storage without retaining factory
   binding IDs. Extend parameter shape seeding and function return summaries to
   these typed slots. Call summaries must resolve the actual callee expression,
   including record fields and array elements, and join possible alternatives
   conservatively. Validate consumed invocation, replacement, moves and disposal.
   Preserve both the path of stored data and the identity of its original owner.
3. Add a dedicated native callable lifetime emitter. Render actual Rust lifetime
   parameters on the finite environment family and every containing record,
   array and Result. Variant payload types must use the right slots. Connect
   argument and return signatures to the existing named native lifetime planner.
   Keep real slices; no static lifetime, raw pointer, erased handle, hidden owner,
   clone or shared allocation may stand in for lifetime transport.
4. Turn the retained positive witness green through normal analysis and complete
   emission. Execute O0/O3 and independent owner-escape rejection. Add two distinct
   alternatives with different input origins, a field move, field replacement,
   an array element, a nullable field, returned storage and nested capture. Add
   mutable view cases: consuming a callback may transfer its carried exclusive
   view, but cannot return a borrow of storage destroyed by consuming invocation.
   Preserve ordinary local reference-capture rejection and source diagnostics.
5. Connect editor checks for the supported public syntax, including ownership
   diagnostics in imports and incomplete buffers. Record private implementation
   acceptance separately from public byte-view publication. Complete public
   factory/observer catalog entries and executable examples before lifting guards.
   Add matched subarray workload support and its independent oracle as specified
   by native-byte-views-implementation.md. Full regressions and measurements belong
   to S07 completion, not each representation increment.

## Bounded acceptance

Run the smallest failing witness after each coherent change, then the private
native-view batch, relevant callable storage tests and native editor contracts.
Analyzer dispatch changes require all ten normal-stack depth canaries and Result.
Format the entire project before source checkpoints. Preserve original failed and
successful logs byte for byte in SQLite and verify their restoration. Keep public
publication guarded while any required representation or provenance case is open.

## Carried-slot metadata checkpoint

native_view_callable_storage.rs now derives candidate lifetime slots from checked
capture bindings. Each slot retains the capture field, its relative payload path,
Result variant guards and readonly/exclusive access. Construction-context root IDs
and source spans are not part of this layout description. Physical shared/mutable
capture storage, physical storage loans and foreign roots are excluded. Slots are
candidate metadata only; they do not certify that a family can safely store the
value, and finite alternative identity/ABI integration remains pending in step 1.

The owning-storage guard now identifies this case with native-view-callable-storage
rather than claiming the callback fails to own its environment. Existing ownership
and invocation guards remain. Focused controls cover readonly/exclusive payloads,
nested record fields, Result success guards and the retained publication guard.
The positive stored callback witness remains red and must not be counted as accepted.

## Finite family planning checkpoint

The candidate planner now retains the structural callable signature, the original
invocation mode and a sorted, deduplicated list of concrete alternatives. Each
alternative has its own carried slots; capture field numbers are not merged across
different origins. A reusable contract cannot acquire a consuming environment.
Unknown origins are diagnosed, and physical references or unresolved layouts fall
back to the existing storage checks. This is a finite layout description, not yet
a registered source storage type or an emitted native family.

callable_dispatch.rs consults this planner only for families containing a borrowed
environment. Wholly owned ordinary families keep their existing path without an
additional candidate allocation. Candidate native-view families remain rejected
with the pending lifetime-storage diagnostic. Registry identity integration, actual
callee loan transport and lifetime-bearing family emission are still required to
turn the retained positive stored-callback witness green.
