# Handoff: recursive views and finally cleanup

Current at the commit carrying this note. The original October 10 handoff is
superseded; prior checkpoint evidence remains in Git, SQLite and
`docs/b11-resources-bytes-acceptance-map.md`. S07 is not accepted or measured.

Read `AGENTS.md`, `docs/development-workflow.md`, the acceptance map,
`docs/resources-bytes-plan.md`, `docs/recursive-view-storage-plan.md` and
`docs/standard-library-roadmap.md`. All decisions below are already approved.

## Approved scope and completed work

- Nullable recursive links, including mutual cycles, use unique heap ownership.
  Nonrecursive layouts stay unchanged. No shared ownership or implicit byte copies.
- Recursive records containing views share a real borrow lifetime. Array recursion,
  nullable links, provenance, iterative link destruction/clone and public fixtures
  are implemented. Views remain non-clonable. Recursive records storing callback
  fields still have the explicit callable-layout diagnostic and remain pending.
- View-capturing callbacks can return argument views or views of borrowed Bytes
  arguments. Terminal-branch provenance and temporary record calls are repaired.
- The installed CLI/editor consumers were refreshed from `4367f4cc` with executable
  fixture and LSP smoke checks. Later source work is not installed yet.
- Resource close consumes ownership even on Err. Later use and double close are
  static errors. Drop is a nonthrowing fallback. Bytes and views need no close.
- Whole-owner finally returns were repaired in `ed469b32` using initialized stack
  slots. Partial-field direct cleanup routes landed in `eba57bd2`, preserving
  returned allocations and sibling liveness without cloning or placeholders.

## Current checkpoint: lexical disposal through partial returns

Map callback return projections and cleanup carriers were repaired in `4d887f9d`.
Whole-owner completion scopes were restored in `a26bda43`. This checkpoint extends
lexical disposal to ordinary partial-field returns: Normal completions carry the
surviving fields, while Return carries the returned value. Body locals leave their
real Rust scope before cleanup. Normal fields are restored before cleanup and
survivors restored again directly in the final Normal dispatch arm.

Checked ownership snapshots now distinguish Body, Handled and Cleaned phases.
Separate input/output field sets support conditional cleanup returns and fields
consumed during cleanup. Original storage retains untouched siblings and their
borrows. No cloning, dummy values or manual whole-record drops are introduced.
Parameters and locals receive native mutability only where restoration needs it.

Focused native probes cover conditional/unconditional partial returns, nested
cleanup, eligible catch returns, local record partial moves, allocation identity
and exact disposal at O0/O3. Terminal return/throw bodies need no Normal field
carrier and retain their lexical boundary directly. Escaping break/continue paths
are excluded from that shortcut. Dead catches are not emitted by field transport;
catch transfer guards include nested expression spans, not only statement spans
(which can cover just an if/while keyword).

The original ordinary and Map partial-return disposal witnesses are enabled and
pass. Map completion bodies now carry normal fields beside independently typed Map
owners. Cleanup carries its surviving fields back to the normal continuation.
Eligible catches carry body-normal fields only after selecting the normal branch;
moving those fields into the body's completion prematurely breaks catch ownership
correlation. Body/catch locals leave real Rust scopes before cleanup. Terminal body
returns and unconditional cleanup replacement need no surviving normal fields.

Focused O0/O3 probes cover Map body/catch partial returns, terminal returns,
unconditional cleanup replacement, pointer identity and exact local disposal.
A separate callback-environment probe verifies exact disposal and allocation
identity. Integration cases verify conditional cleanup returns and terminal
branches with different callback origins. See the acceptance map for final counts.

Ordinary field-return loops now carry a separate field tuple for every escaping
break/continue destination. Handled ownership snapshots select fields entering
cleanup; Cleaned snapshots select survivors for the eventual loop exit. Body scopes
end before cleanup. Cleanup can replace a pending return with break. Normal branches
that have no checked completion diverge instead of inventing field initialization.
Nested cleanup, continue/update, labeled exits, normal fallthrough and a Map-backed
loop disposal probe pass. Empty native ownership payloads now render the valid unit
tuple. The original loop-control witness is enabled.

Ordinary body-field returns with a reachable catch now retain lexical disposal.
The throwing body carries surviving fields with its error and restores them before
the handler. Normal fields have a separate payload. Nested cleanup forwards these
error fields; loop completion frames preserve them independently of control owners.
When a field moves on the throw path but survives a return, it accompanies the
pending return too. Normal/control paths carry those same surviving fields where
needed, preventing loss of initialization proof across alternatives. Impossible
normal completion arms diverge. Body/handler returns, nested cleanup replacement
and caught loop exits now pass focused native/integration checks.

The Map body-return/catch disposal witness is now enabled. Map body error payloads
carry surviving ordinary fields beside their precisely typed callback owners; normal
payloads carry their own fields. Each selected alternative restores fields before
catch or cleanup. Empty handlers explicitly project the caught callback type into
the normal cleanup representation, since they have no statement input epoch.
Body scopes end before cleanup on all covered paths, without cloning returned data.
Native O0/O3 witnesses cover empty and nonempty handlers and handler field returns.
A four-mode integration case covers early return, handler return, caught continuation,
normal fallthrough and cleanup replacing a return with another field.

Ordinary same-region catch/control transport now carries the body's error fields
into the handler and uses destination-specific field payloads for break/continue.
Returns retain any surviving fields needed after alternate error transfers. A native
O0/O3 witness verifies body-local disposal before cleanup, and integration tests cover
break, continue, normal continuation and cleanup replacing a return with a sibling.

Nested Map error forwarding now recognizes ordinary field destinations, carries them
through the inner body, and restores them before cleanup. Map return carriers also
retain surviving ordinary fields required by that cleanup. Normal completion of
error cleanup carries fields separately so a conditional cleanup return does not hide
their initialization. Impossible normal/error/control alternatives no longer read
moved fields. Legacy cleanup returns remain valid when there is no specialized
carrier. Native disposal and allocation identity, nested catch, and cleanup replacement
are covered. The original callback-disposal regression remains part of acceptance.

Map-backed same-region catch/control now has native disposal and four-mode output
coverage without an additional implementation change. Field transport also handles
callback-bearing records stored as independent native leaves, including nested plain
record subtrees. Owning callback captures may transport fields when their checked mode
is Consume and they are not references. Native bindings become mutable only to restore
transported owners; source const/borrow rules are unchanged. Map error/return field
classification uses the target type, so split ordinary leaves are not mistaken for Maps.
Native O0/O3 probes verify exact disposal and allocation identity for both layouts;
source fixtures cover captured catch/loop cleanup and nested Map replacement.

One aggregate-return probe hit the existing callback-escape diagnostic for returning
a borrowed callable inside a record. This repair does not relax that source rule;
aggregate field transport is checked with a plain nested record beside a retained
callback. Restored Option-root and borrowed-storage fallback paths are unchanged.
All current disposal witnesses are enabled and pass. Resource acceptance is still open.

Public fixtures and matched `exception_completion` now exercise body field returns,
caught throws, handler field returns and normal continuation, in addition to loops
and Map cleanup. Matched TypeScript, independent closed-form oracle, mutation guards
and hashes are current. Latest records use SQLite prefixes `tsrust-callback-field-cleanup-`,
`tsrust-captured-field-cleanup-`, `tsrust-owned-capture-field-cleanup-` and
`tsrust-map-catch-control-`. The matched Map error crosses an inner finally before
catch; loop break/continue now leave a catch in the same cleanup region. The independent
oracle, guards, counterpart and fingerprints are current. The field-retention stage
now uses a callback-bearing record; caught field returns run inside an owning callback. A discarded test variant correctly failed ownership checking:
fallible logging in inner cleanup could throw after the field had already moved.
The final replacement tests keep that inner cleanup infallible.

See the acceptance map for final counts and untimed verification. No timing or full
family regression ran. Installed CLI/editor remain `4367f4cc`; scratch release is
rebuilt from this checkpoint before acceptance.

Then finish recursive stored callback fields, gate 190's resource backend and
synchronous file proof, and S07 family acceptance: complete compiler/editor
regressions, matched measurements, regression review, justified optimization and
consumer refresh. Continue through the existing S08–S21 roadmap after S07. Preserve
completed evidence; rerun affected checks and resume unrun measurement groups.

## Recursive stored callback continuation

The same-family stored callback path now executes. Concrete materialization clones
one finite recursive component and retargets its nullable/array edges. Emitter
record identity, union aliases, owning-field patterns and captured loan projection
now agree on that component. Empty array/absent initializers directly use the
concrete destination; they do not recursively convert a nonexistent payload.
Nested consuming field extraction uses the existing split local storage.

Public checks cover self-links, mutual links, arrays and mutable view callbacks;
owner move, short-owner escape, clone and repeated-Once calls are rejected. A native
probe verifies slice pointers, native short-owner rejection and 10,000-node cleanup
on a 64 KiB stack at O0/O3. `native_byte_views_stored_callbacks` has a matched
TypeScript source, closed-form oracle, mutation guards, hashes and harness support.
See the acceptance map for exact counts and untimed records.

Mixed factory links and local callback replacement now execute. A stable family
is keyed by recursive nominal record and field; alternatives are checked before
admission. Nonrecursive families stay independent. Returning an extracted callback
uses the recursive component's shared lifetime, including the shortest owner.
The matched stored-callback workload now alternates two factories and keeps the
same operation denominator and independent oracle. Original untimed records remain
intact; see the acceptance map for the new records.

Recursive record parameters now retain the concrete callback family and shared
owner lifetime. The public `recursive-parameters.tsr` fixture exercises owning
transport, readonly field access, whole-record return and callback extraction.
Additional checks cover both declaration orders, direct construction, explicit
callback result annotations, mixed factories, nullable child extraction and mutual
record parameters. Both the captured owner and a discarded sibling owner reject
escape. The native 10,000-node probe now consumes the chain through a parameter,
checks slice pointer identity and rejects both short-owner cases at O0/O3.

Discovery collects the concrete component before parameter binding. It does not
require native callee lifetime summaries; definitive checking does. Rechecked
closures now replace return-lifetime and storage-exit metadata. Recursive records
retain whole native storage so readonly borrows and partial moves agree. Cache
normalization reuses concrete parameter literals and other members of an already
materialized component; it does not introduce recursive conversion or allocations.
No arbitrary component is selected when several incompatible instances exist.

Callback-only recursive view records now execute without a dummy direct view
field. Lifetime detection follows actual owned-family capture slots across every
record/capture/runtime enumerator. Symbolic bases have concrete lifetime-free
layouts, while their specialized components use one shared lifetime. Empty owning
families need no error ABI; unresolved catch contracts still reject. Ten public
source variants run at O0/O3, and the native 10,000-link witness now also contains
only two capturing callbacks, with both short-owner rejections.

Next: complete nullable callback joins across factories and replacement through
mutable recursive parameters. Basic nullable fields execute, but `tsrust-storage-composition-nullable-join.tsr`
rejects a link from a second factory with `Expected Holder | undefined, found Holder`.
The retained
`tsrust-storage-composition-replacement.tsr` probe still reports `Expected closure,
found closure` because native field replacement currently excludes parameters. The finite-layout guard
remains for unsupported specializations. Then finish resource gate 190, S07
acceptance and the roadmap. Installed consumers remain `4367f4cc`; no family timing
has run. See the acceptance map for this checkpoint's exact evidence.

## Working environment

One agent in this tree. `CARGO_INCREMENTAL=0`,
`CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`, cargo `--offline --locked`.
Node 24.11.1 is first on PATH; use `/opt/homebrew/bin/python3` for SQLite blob imports.
Format the whole project before every source checkpoint with
`TSRUST=/private/tmp/tsrust-gate100-editor-target/debug/tsrust node tools/format-project.mjs`.
Keep focused TDD and milestone-only heavy checks. Import/read back exact closed
records before deleting loose copies. Prune obsolete native/test outputs and retain
current executable consumers. Do not create archive directories or snapshots.
