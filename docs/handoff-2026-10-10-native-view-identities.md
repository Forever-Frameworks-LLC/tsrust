# Gate 189 private borrowed identities and representation contracts

Read AGENTS.md, docs/development-workflow.md, docs/resources-bytes-plan.md and
docs/native-byte-views-implementation.md first. This checkpoint follows f569190a,
the logical-negation formatter repair. Gates 187–188 remain accepted. Gate 189 and
S07 remain open. One agent works in this tree; no unrelated owner edits remain.

## Implemented

NativeTypeId now distinguishes Bytes, ByteError, ByteView and ByteViewMut. The two
view declarations use Representation::Borrowed with readonly or exclusive access.
They expose no fields or source construction, independent clone, collection keys or
independent thread transfer. They are intentionally absent from source exports.
Recursive signature projection interns distinct identities, including views nested
in Result and array annotations. This is metadata projection, not accepted source
view storage or execution.

Returned-borrow validation now checks actual representation as well as payload
identity. An owned Bytes payload cannot masquerade as a borrow. Payload access must
match the contract, and an exclusive borrow cannot originate from a readonly view.
The earlier synthetic Bytes payload tests now use the real private view identities.

Borrowed results cannot omit their source contract, even when nested in arrays or
Result alternatives. A native Result Ok borrow contract cannot hide another borrow
in the error payload. This preserves the rule that Err carries no success loan; it
is not a new general restriction on source-language Result types. The missing error
check had a failing assertion before the repair. Its first test attempt also had a
static-lifetime mistake in the test metadata; both attempts are retained.

Signature::contains_borrowed_native walks signatures and native declaration fields
iteratively, with depth and node limits. Primitive shapes take a direct path. Native
binding projection checks borrowed parameters too, so an observer cannot bypass the
ABI guard by returning an owned value. Invalid contracts and unsupported borrowed
shapes fail before interning records. Fixed-storage analysis treats borrowed native
representations as native storage rather than inferring an empty record layout.

The previous safe slice helpers remain unchanged. No view factories, shared byte
argument contracts, source loans, returned lifetime inference or lifetime-bearing
record/union ABI are published yet. Do not remove the projection guard simply to
make imports work. The existing read.tsr probe remains red at missing view export.

## Verification

101 effective focused checks pass across the affected batches: 60 compiler library
checks, 19 native declaration checks, eight native byte project checks and 14 native
editor checks. The library batch initially passed 59 checks; after the additional
error-payload witness, the eight affected return-contract tests passed, adding one
new check to the effective total. This is not a whole-crate regression claim.

Coverage includes representation capabilities, readonly/exclusive identity,
projection deduplication, nested signature shapes, traversal limits, invalid payloads,
readonly source rejection, missing contracts, untracked error borrows and the observer
ABI guard. Existing owned byte execution, stored values, aliases and editor help still
pass. Whole-project formatting changes zero files and zero benchmark hashes. No
recursive analyzer dispatch changed, no depth-canary batch or full regression ran,
and no timing ran. Source-view benchmark support remains pending source execution.

Exact evidence is retained in benchmarks/records.sqlite, read back byte for byte,
with loose copies removed:

- runtime/results/native-borrowed-identities-2026-10-10-checks.json
- runtime/results/native-borrowed-identities-2026-10-10-checkpoint.json

The first record retains original metadata API red outputs and the first green
batches. The checkpoint supplements it with the error-payload test setup failure,
actual failing assertion, repaired eight-test batch, current build/format outputs,
source hashes and still-red CLI source probe. Earlier UTF-8, slice allocation and
formatter records remain untouched.

## Next: source provenance and lifetime integration

Metadata groundwork is ready. Do not spend another increment recreating identities
or the slice helpers. Start the analyzer/backend integration from the retained local
readonly view witness, then complete the required matrix in the implementation plan.

Useful entry points are project_link.rs native import projection and analysis.rs
native-call checking. Direct calls and stored native function values already converge
in the branch that resolves NativeBinding and assigns CallTarget::NativeBinding.
That branch currently checks argument access but retains no returned source loan.
Attach the declared source argument roots there through a dedicated native-view
provenance mechanism. Existing view_sources tracks optional selections; it does not
handle native calls. Do not conflate a movable view value with borrowed_place, which
also describes borrowed access to an existing value.

Propagate roots through Result success, bindings, moves, branches, forwarding returns,
stored factory values and aggregate/capture storage. Retain all possible roots. Allow
borrowed-parameter returns with inferred source relationships; reject escaping local
owners and consuming parameters. Err and all exits must release the relevant loans.
Readonly views permit reads and block mutations/transfers; exclusive views block
conflicting access. Nested reborrows suspend the required parent access.

Extend actual record/union/function emission to carry &[u8] and &mut [u8] lifetimes,
including wrappers, arrays and returned values. emit.rs currently has concrete Bytes
storage but no native borrowed storage branch. Existing scoped-record helpers describe
callback leaf transport; reuse them only where they preserve the actual view lifetime.
No erased pointers, integer handles, hidden owners or source copies are acceptable.

Publish view/viewMut and typed shared byte observer/mutator contracts only when the
analyzer and backend agree. Include full catalog/editor help with that source feature,
normal-stack canaries after analyzer dispatch changes, matched Uint8Array subarray
workloads, independent oracles and native zero-copy/cleanup controls. Complete the
full view matrix before accepting gate 189. Gate 190 remains consuming resource
handles; close consumes even on Err and fallback drop is nonthrowing. Full regressions
and accumulated measurements belong to the completed S07 family milestone.

## Environment and publication

Reuse /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0. Node 24.11.1
must be first on PATH; use /opt/homebrew/bin/python3 for store.py. The warm debug
compiler was rebuilt with this checkpoint. Other warm executables can predate the
last private metadata repair and must rebuild when needed.

All five installed consumers were refreshed and smoke-checked at f569190a after the
formatter repair. They include the previous UTF-8 and slice-helper work, but not this
private identity increment. No installed consumer was replaced here. Disposable native
test outputs use the shared helper's temporary directories; no per-gate target or
archive was created. Keep active Cargo artifacts and the five consumers. The overall
goal remains active.

## Continuation prompt

Continue Tsrust from docs/handoff-2026-10-10-native-view-identities.md. Read AGENTS.md,
docs/development-workflow.md and docs/native-byte-views-implementation.md first.
Gate 189 UTF-8, safe slice helpers, borrowed identities and validated source contracts
are verified. Implement actual source-view provenance and lifetime-bearing ABI next;
keep guards until those layers agree. Do not repeat metadata groundwork or replace
views with copies. Complete compiler/editor/catalog and matched view benchmark support
before accepting gate 189, then continue gate 190 consuming resources. Use bounded
TDD, whole-project formatting, explicit commits and immutable SQLite evidence. Provide
a physical handoff at the next checkpoint.
