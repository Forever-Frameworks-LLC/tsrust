# Precise indexed capture release

## Direct reassignable removal results

Reassignable local pop/shift results now receive point-sensitive native storage.
Direct assignments evaluate the RHS before dropping the previous result, then
shadow storage only when its concrete environment changes. The isolated
`array_result_storage.rs` module handles result eligibility, nullable environment
specialization, result-use retagging and unsupported join diagnostics. Source
ownership snapshots also cover named result reassignment. Empty removal clears the
old result owner rather than restoring its capture from the snapshot.

Focused checks cover retained mutable results, sentinel replacement, new callback
origins, further removals through empty, normal lexical-block replacement,
surviving and newly captured values, and exact pop/shift disposal at optimization
levels 0 and 3. Affected array coverage passed 30 tests and exposed one previous
block witness: early callback specialization caused native planning to treat that
known result invocation as opaque. The repaired witness passes separately. Three
new reassignable-result tests pass, as do the original four pop/shift checks,
eight depth canaries and focused editor diagnostics and hover. The final source
helper avoids collecting write facts when no relevant mutable union exists.

Fresh captured-record replacement inside branches or loops still needs native
joins. It now produces a source diagnostic instead of generated Rust type errors.
Canonical optional record construction and nullable fields are not expanded by
this existing-concrete-environment specialization. Same-expression result
lifetimes, aggregate result storage and control-flow joins remain required.

The matched row has 119 observations per iteration. The added independent formulas
are `3 * value + 5` for a mutable selected result and `3 * value + 6` for replacement
with a new callback. Source guards, mutation rejection and literal expectations
pass. All five cases pass untimed on native and Node-after-tsc. SQLite evidence
is `runtime/results/2026-10-09T08-58-51-249Z-verify.json`; startup, compilation and
execution timing arrays are empty. This remains a focused B5 checkpoint. No full
regressions, timing or installed-consumer refresh ran.

## Immutable retained removal results

Direct zero-argument `pop()` and `shift()` initializers on known dense local arrays
transfer the removed slot's capture owners to the declared result. Surviving slots
retain independent ownership, and shift reindexes those slots. Normal result moves
and lexical disposal govern the removed capture. Uncertain shapes and other
expression forms remain conservative.

Native planning projects an immutable result into its selected slot's concrete
nullable environment and narrows surviving array carriers. Normal lexical blocks
return the surviving owners into the enclosing scope. Map storage transport remains
active in combined blocks. Callback invocation targets follow specialized result
origins, including fallible calls; the matched workload exposed an error-conversion
mismatch that now has a focused native regression witness. Non-record optional
removal results retain their existing conversion path.

The affected array batch passes 27 tests. Supplementary checks pass for record-field
storage, mixed Map/array blocks and fallible selected-result calls. Exact disposal
passes at optimization levels 0 and 3. The focused editor diagnostic and eight
normal-stack depth canaries pass. Source checks retain rejection while either the
result or surviving elements hold a conflicting capture. Project formatting and
matched untimed verification are recorded separately below when complete.

Remaining work includes reassignable result storage, nullable result assignment
through aggregates, same-expression release, indexed aggregate arrays and
control-flow slot joins. B5 remains incomplete; this checkpoint does not authorize
family acceptance, timing claims or installed-consumer refresh.

## Discarded removal checkpoint

Standalone `pop()` and `shift()` calls now release the removed callback's captures
at the statement boundary. Dense lengths live in an isolated ownership-path
module: moves transfer them, incompatible or branch-only shapes become unknown,
and unmodelled mutations invalidate positional precision. Three shape tests cover
moves, sticky uncertainty and reaching-state joins. A snapshot is taken only for
a direct named/record-field zero-argument removal with a known shape; it is boxed
to keep recursive statement dispatch small. Other expressions retain conservative
ownership. The snapshot currently copies retained owner metadata during analysis;
compiler allocation costs remain subject to the family performance review.

Native epochs move surviving records to their concrete carrier after removal.
Empty arrays drop their old carrier and create an empty environment without
cloning. Focused native coverage includes front removal, repeated removal through
empty, record fields, moves and exact disposal at optimization levels 0 and 3.
Bound results and removal temporaries passed before a later conflicting argument
remain protected. One previous rejection became a valid shift/replacement program;
the revised negative case retains the same capture in the replacement.

The affected batch passed 22 tests and exposed two test expectations. Both repaired
checks pass separately. The new editor test, three shape tests and all eight depth
canaries pass. Pop/shift IntelliSense documents the behavior with native-verified
examples. The matched workload has 115 observations, adding `2 * value + 3`,
`2 * value + 4` and `2 * value + 6` for pop, shift and empty removal. Source and
mutation guards pass. All five matched cases pass untimed on native and
Node-after-tsc. SQLite preserves
`runtime/results/2026-10-09T08-21-40-026Z-verify.json`, with zero timing rows,
and 15 diagnostic records restored byte for byte before loose copies were removed.
All 13 owner-module tests pass. The final project formatter changes zero files
and zero fingerprints.

Retained removal results still conservatively retain the container's capture set.
Precise result provenance, release after the result's last owner, same-expression
release points and aggregate/control-flow compositions remain required. This is
focused implementation progress; B5, its family regressions, timing review and
installed-consumer refresh remain incomplete.

## Record fields and moves checkpoint

The next slice extends epoch planning from a binding ID to an owning record field
path. Native conversion shadows only the selected independent array leaf, keeping
sibling fields in place. Root expression types are rebuilt from the currently
active leaf epochs. Local moves transfer the known slot payloads to the destination
storage after restoring any source carrier needed by the original declaration.

The record-field and move witnesses initially failed generated Rust with E0502.
They now pass with nested records, whole-record moves, moves after narrowing,
sibling scalar mutation and exact disposal at optimization levels 0 and 3.
Twenty-two affected array tests, two editor diagnostic checks and all eight depth
canaries pass. Independent sibling-array capture protection, field extraction and whole-record
moves after narrowing also pass.
The matched workload adds two observations, `3 * value + 10` for field storage and
`3 * value + 4` for a moved array, bringing the row to 112 observations. Source
contracts and mutation guards pass. All five cases pass untimed on native and
Node-after-tsc. SQLite preserves
`runtime/results/2026-10-09T08-07-01-526Z-verify.json`, with zero timing rows, and
15 diagnostic records, all restored byte for byte before removing loose copies.
The final project formatter changes zero files and zero fingerprints.

B5 remains incomplete. Next implement precise element removal and retained result
ownership, then indexed arrays inside aggregates and reaching slot states through
control flow. Nullable and same-expression compositions, structural record keys,
family regressions, timing review and installed-consumer refresh remain pending.

B5 remains incomplete. The first replacement witness is
`replacing_known_array_element_releases_only_its_capture` in
`scoped_record_composition.rs`. Replacing index 1 with a callback borrowing the
first array must release the previous callback borrowing the second array.
Expected native output is `3 3 9`. The independent Node assertion passes;
the baseline source checker rejected mutation of the second array with
`capture-conflict`. The red witnesses and plan are checkpointed in `a79ac42`.

Source ownership is implemented through the isolated
`array_capture_owners.rs` module. Literal elements have separate internal owner
paths, known replacement stages its RHS and replaces only that path, and indexed
reads retain the conservative container view. Reordering and removal operations
collapse positional paths into conservative root loans. Push preserves prior
positions and retains new payloads conservatively. The duplicate unconditional
append in indexed assignment is removed.

The positive source test and six rejection cases pass. Native epoch planning and
emission now live in separate array modules. The original witness passes,
including exact disposal at optimization levels 0 and 3, a later borrowed handle
and repeated replacement. Twenty affected array tests and the new editor
diagnostic test pass. Replay metadata replaces matching epochs instead of emitting
the same conversion twice. Opaque uses and control flow restore the baseline
carrier and stop positional planning rather than guessing reaching slot types.
The eight normal-stack depth canaries pass with the new pipeline.

The matched workload adds one independent observation, `3 * value + 3`, for
known indexed replacement followed by mutation of the released capture. Source
contracts, rehashed mutation guards and literal oracle expectations are updated.
All five matched cases pass untimed on native and Node-after-tsc. SQLite preserves
`runtime/results/2026-10-09T07-57-50-707Z-verify.json`, with zero timing rows, and
14 diagnostic records restored byte for byte before loose copies were removed.
The final project formatter changes zero files and zero fingerprints.
No installed consumer is refreshed
from this unfinished B5 implementation.

Carrier narrowing scans and moves the retained array. Repeated replacement may
also widen back to its baseline family before storing and narrowing again. These
conversion costs require the B5 performance review; they are not timing evidence.

The companion rejection cases protect surviving captures, a borrowed element
handle, and an unknown replacement index. A borrowed handle currently rejects
replacement with `view-conflict`; that existing storage protection must remain.

## Source ownership

Introduce an isolated indexed-owner module. Dense literal slots need distinct
internal paths, including nested arrays and record fields. Preserve these paths
through moves and reaching-state joins. Resolve only checked constant indices;
unknown indices retain every possible capture. Stage replacement values before
discarding the previous slot, including self-assignment and fallible evaluation.
Remove the redundant conservative append in indexed assignment only once the
precise replacement path is complete.

Index identity changes under shift, unshift, splice, reverse and sort. Model the
operation exactly or collapse its slots into conservative container ownership
before changing their positions. Never keep stale slot identities. Pop and shift
also transfer the removed payload into a result: discarded results, bound
results, nullable results and temporary arguments need separate lifetime checks.
Do not release a removed capture before its last result owner is gone.

## Native storage

The whole-function array carrier currently mentions all inserted environments.
Source loan removal alone therefore cannot establish acceptance. Add independent
point-sensitive array epochs after carrier planning. Track actual payload types
under carrier wrappers, join the surviving concrete environments and move
surviving elements into their checked destination representation. Retarget later
indexing, field invocation and moves to that epoch without changing public
nominal contracts. Prove every excluded carrier alternative unreachable; do not
cast or clone to hide a lifetime.

Start with straight-line constant-index replacement, then field storage and
moves. Follow with removal and reordering, branches, loops, exceptions and
completion transport. Keep uncertain paths conservative. Measure any survivor
conversion scans at the B5 family milestone rather than claiming ordinary
constant-time mutation is unchanged.

## Acceptance

Require focused source and native witnesses, disposal at optimization levels 0
and 3, editor diagnostics and guide examples. Extend the existing matched
composition workload, independent oracle, source guards and fingerprints during
implementation. Run untimed native and Node-after-tsc verification. Run the eight
depth canaries after analyzer dispatch changes. Format the complete project
before committing; preserve unrelated fixture semantics and use explicit paths.
Full family regressions, timing review and installed consumer refresh remain
pending until the complete B5 scope is ready.

### Retained-result checkpoint evidence

All five matched cases pass untimed on native and Node-after-tsc. The workload
has 117 observations per iteration; the added independent formulas are
`3 * value + 4` for result-scope disposal and `3 * value + 5` for result
independence. Source contracts, mutation guards and literal oracle expectations
pass. SQLite preserves `runtime/results/2026-10-09T08-38-50-598Z-verify.json`,
with empty startup, compilation and execution timing arrays. Twenty-three
diagnostic logs, including original failures and repaired checks, were imported
and restored byte for byte before deleting their loose copies. A supplementary
explicit thrown-callback test passes after narrowing the caught runtime-error
union; its original rejection and repaired result are also retained. All four
existing pop/shift regression tests pass. Final editor hover and diagnostics,
eight depth canaries and zero-change project formatting pass. Pop/shift catalog
examples include retained-result scope release, with native outputs checked by
the focused tests. No full regression, timing or installed-consumer refresh ran.

Mutable-result evidence also covers self-assignment and preservation of a moved
result owner. Pop/shift catalog examples verify direct sentinel replacement with
outputs `3 9` and `5 9`; exact disposal is checked independently. Nineteen closed
diagnostic logs, including original failures and repaired checks, and the matched
verify record were imported into SQLite, restored byte for byte, and removed from
loose storage. Project formatting is token-preserving and idempotent; the final
checkpoint formatter changes zero files and fingerprints.

After the small eligibility-scan optimization, the current debug compiler's
emitted Rust matches the verified workload prefix byte for byte under the
harness's stdout trimming and entry-point rename. The initial comparison
incorrectly required an extra newline; its corrected comparison and final
zero-change formatter log are also stored and restored exactly in SQLite.
No new native verification run or timing was needed for unchanged generated code.
