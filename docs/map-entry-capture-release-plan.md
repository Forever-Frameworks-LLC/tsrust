# Precise Map entry capture release

## Required behavior

Deleting a known entry releases only its callback captures. Remaining entries,
borrowed lookup results and moved payloads retain their own loans. Replacement
releases the replaced payload after evaluating its key and replacement value.
Dynamic keys and uncertain branch outcomes must retain every potentially live
capture. No implicit clone, shared cell, unsafe cast or object identity is needed.

The first red witness is deleting_one_map_entry_releases_only_its_capture in
scoped_record_composition.rs. It stores callbacks capturing two independent
arrays, deletes first, mutates first, calls second, clears and mutates second.
Source checking currently reports capture-conflict on the released first array.
The paired negative witness must still reject mutation of second. Native O0/O3
Drop observers must show the first closure dropping during delete and the second
during clear. The independent Node observable reference passes.

## Two coordinated changes

Current capture_loan_owners::Place identifies a container and record field path,
not its entries. Map.set attaches every incoming callback to that same owner.
Map.delete has no ownership effect. Separately, scoped_map_carriers joins every
insertion into one native callback environment family for the entire function.
Merely deleting a source loan would allow programs that still fail generated
Rust because the surviving Map type retains the removed capture lifetime.
Implement both layers before accepting the witness.

1. Add an isolated entry-state module. Track canonical stable scalar keys under
   each Map storage path, their actual payload types, and capture owner paths.
   Unknown keys use a conservative bucket. Use exact typed key semantics;
   never use expression spelling as identity. Preserve the current float policy.
   Known replacement stages the RHS before discarding its previous slot.
   Known deletion discards only the selected slot. Unknown writes and deletes
   retain all possible aliases. Moves transfer this metadata, clear removes it,
   branches join reaching states, and loops retain safe fixed-point behavior.
2. Keep point-sensitive storage epochs separate from the existing global
   insertion-family join. Record checked deletion/replacement boundaries and
   the remaining concrete environment types. A surviving owner must acquire a
   native type that no longer mentions a provably removed capture. Reuse
   tsr_owned_items and record carrier conversion to move surviving entries,
   preserving key order and without cloning their payloads. Checked narrowing
   must handle excluded callback variants explicitly; arbitrary narrowing is
   forbidden. Empty survivors reuse existing absent environments.
3. Retarget later constructor, set, get, iteration, move and exception metadata
   to the applicable epoch. Do not change shared nominal declarations. Keep
   error contracts from surviving callbacks and preserve typed failures.

A moving survivor conversion may scan retained entries. Do not quietly turn
ordinary delete into an O(n) operation: emit conversion only where a proven
capture-release boundary requires a new native environment type. Record this
cost in developer evidence and cover it in the matched workload. The family
performance review must assess it before any broad efficiency claim. A later
static-slot representation may remove that cost without changing semantics.

## Acceptance sequence

First establish straight-line known keys, then record-field storage and moves.
Cover deletion of missing and repeated keys, replacement by another origin,
same capture in several entries, dynamic-key aliases and key normalization.
Keep lookup loans live until their scope ends. Follow with branches, loops,
try/catch/finally, fallible callbacks and exact disposal at every release.
Each implementation batch needs its own failing witnesses before source edits.

Add editor diagnostics and verified guide examples. Extend matched Tsrust and
TypeScript workload observations, independent oracle, source contracts, mutation
controls, hashes and existing hooks. Verify untimed native and Node-after-tsc.
Run affected compiler suites and all eight depth canaries after analyzer dispatch
changes. Preserve failed originals and resumed checks in SQLite. Format the whole
project before each source checkpoint. Do not describe B5 as accepted until
remaining array-element release and all named composition requirements are
reconciled and the feature-family acceptance is complete.


## Keyed source ownership work in progress

Uncommitted map_entry_capture_owners.rs now assigns decoded scalar literal keys
separate internal owner paths. Map.set stages its RHS before replacing a known
slot, unknown keys retain conservative container ownership, and Map.delete
removes only the known slot. Existing owner transfer and reaching-state joins
carry these paths. Parameter and captured storage cannot discard external loans.
Recognized number literals preserve existing Map SameValueZero normalization;
unknown expressions and union keys retain the conservative path.

The original red witness now passes source checking and fails native compilation
with E0502. This is expected evidence of the second required layer, not feature
acceptance. Its remaining-entry negative assertion now runs before native
execution. Six additional source cases pass and protect shared captures, unknown
insertion aliases, dynamic deletion, missing deletion, conditional deletion and
replacement retaining the same capture.

Do not commit this as accepted implementation. Native point-sensitive payload
types and checked survivor conversion still need implementation, followed by
full focused editor/matched acceptance. The current source module contains no
payload-type state yet. Add that coordinated metadata next rather than deriving
payload types solely from capture IDs: distinct records may share captures, and
unknown slots or reaching branches must retain their own actual types. Existing
scoped_map_carriers still widens all insertions into a whole-function type.


## Per-entry payload state work in progress

Uncommitted map_entry_payloads.rs now tracks payload-type alternatives separately
from capture loan IDs. Owners carry this state through consuming moves, borrowed
copies, replacement, discard, scope exits and reaching-state joins. Known slots
receive their actual record type; unknown insertions accumulate a conservative
root bucket. Queries select only this Map's immediate slots, excluding payloads
inside nested containers. A state test proves differently typed records sharing
one capture loan survive uncertain deletion and movement independently.

Known deletion records its input Map type and remaining payload alternatives in
storage_exit_availability::Exits. The metadata table is allocated lazily to avoid
adding a full HashMap to every ordinary analyzer context. Repeated analysis unions
survivors. A changed input type or owner path marks the proof inconsistent so it
cannot authorize native narrowing. A focused replay test passes. The retrieval
method is currently test-only until the native planner consumes these records.

Native phase conversion is still unwired. The original end-to-end deletion witness
must remain red and this source is not accepted or committed. Next expose the
recorded snapshots to scoped_map_carriers, preserve statement/expression order,
plan a native payload type for survivors, and use a checked moving conversion.
Current record_carrier_conversion only widens borrowed callable families; add
excluded-variant handling only on the proven narrowing path. Keep later lookups,
iteration, storage moves, error contracts and replacement/refill epochs consistent.
Do not rerun unrelated full suites or claim editor/benchmark acceptance yet.

## Native straight-line progress

The native epoch planner now consumes the checked survivor metadata for directly
initialized local Maps. Known deletion rebuilds surviving entries into a payload
type that excludes the released environment. Empty storage drops and recreates
its absent environment. Refills widen the current representation before storing
a new payload. Clear follows the current representation rather than the original
function-wide type. Late parameter carrier planning preserves the checked epoch
metadata. These changes remain uncommitted and are not accepted as the complete
entry-release feature.

The independent-capture witness now passes native execution at O0 and O3, with
exact disposal of the deleted and remaining environments. The empty deletion and
existing replacement/refill/clear disposal witnesses also pass. The resumed
composition suite passes 101 tests. A further survivor-iteration witness exposed
an entry adapter retaining the old representation. Planning the iteration entry
against the current Map payload and then retargeting its body repairs that
witness; both native optimization levels now pass.

Record-field and moved storage, control-flow transport, same-expression release,
replacement narrowing and conversion-cost pruning still need focused witnesses
and implementation. Editor, guide and matched benchmark acceptance remain owed.
Depth checks are running after the iteration adapter repair. Original failed logs
and resumed checks must be preserved in SQLite before closing this batch.

## Direct storage moves and repeated deletion

A new disposal witness first failed generated Rust with E0502 after moving an
initialized Map into another local and deleting one entry. The epoch planner now
transfers its local-storage eligibility across a checked direct owning Map move.
The destination receives the current planned Map type; later deletion narrows the
destination rather than the consumed source. The witness passes at O0 and O3,
preserving the other entry and dropping each callback exactly once.

Repeated deletion of the removed key and deletion of a missing key also pass
native disposal checks. Both return false and leave the surviving callback usable.
Twenty closed native planning, regression, replay, iteration, repeated-deletion
and depth logs were imported into benchmarks/records.sqlite, read back byte for
byte, and removed from scratch storage. The moved-storage green log remains open
for the next evidence import. No acceptance or consumer refresh is claimed.

## Replacement and move order

The reverse move order also passes: delete a known entry, move the narrowed Map,
then call the remaining entry. A new replacement witness failed generated Rust
with E0502 when the previous capture was mutated after replacing its entry with a
callback capturing another array. Known set operations now record their checked
survivor payload alternatives after RHS evaluation and owner replacement. Native
epoch planning consumes these snapshots after the set statement. The replacement
witness passes at O0 and O3 with the old environment dropping during replacement
and the new environment dropping during clear.

The complete affected composition suite and a focused LSP diagnostic witness are
running for this batch. The editor witness accepts mutation of the released
capture, verifies iteration of the surviving entry, and still rejects mutation of
the remaining capture. No new built-in API or documentation signature is added.
Replacement evaluation order, failure transport and scan-cost pruning remain
explicit acceptance obligations; a passing straight-line witness does not close
them.

The initial replacement integration activated native planning for ordinary
insertions and regressed 31 composition cases. That failed run is retained as
`tsrust-b5-map-entry-native-replacement-batch.log`. Snapshot creation now requires
an existing known entry whose payload type differs from the replacement. Ordinary
first insertions and same-environment writes preserve their prior planning path.
The resumed affected suite and post-change depth canaries are running. The focused
editor diagnostic witness passes. These results supersede any inference that the
first replacement implementation was regression-free.

The resumed composition suite passes all 103 tests. All eight normal-stack depth
canaries pass after the replacement repair. The focused editor diagnostic test
passes. Nine additional closed logs, including the original replacement failure,
the 31-regression batch and repaired runs, are stored in SQLite with byte-exact
readback; verified loose copies are removed. Project formatting, benchmark support
and complete entry-release acceptance remain pending. The work remains
uncommitted; do not report B5 as complete.

## Replacement failures and next field witness

A failure before replacement payload construction preserves the previous entry.
The focused test verifies its value remains 3 after catching 7, then instruments
both closure environments at O0 and O3. On failure only the original closure is
constructed and it drops at clear. On success the original drops during set, the
replacement returns 5, and the replacement drops at clear. Mutation of the retained
original capture inside catch still receives capture-conflict. No compiler repair
was required for this valid failure witness. Initial witness attempts used a
retired access spelling and a lookup loan that outlived clear; those diagnostic
logs remain separate evidence of test correction, not compiler regressions.

The next red witness is
deleting_record_map_field_releases_only_its_entry_capture. It preserves a sibling
marker field while deleting one of two independently capturing callbacks. Source
checking succeeds; native compilation currently fails with E0425 for a missing
callback environment type. Direct-root epoch eligibility does not cover field
storage. Field support must preserve the enclosing record representation and all
siblings while changing the Map field's payload environment. The red witness is
not acceptance; keep it while implementing that coordinated path.

The field witness exposed an unrelated planner-boundary defect: snapshot presence
alone caused retargeting in functions with no eligible direct local Map. The
planner now skips such functions unless a previously planned root epoch is needed
by the nested body. This removes the missing C0 type. The field witness now fails
with E0502 on mutation of the released capture, establishing the remaining native
field-lifetime transport requirement. Direct survivor iteration remains covered
by the existing witness and must pass after the boundary repair.

## Record-field native transport

The planner now identifies exact empty-Map storage paths inside local record
initializers. A checked survivor snapshot updates that Map field's type in the
point-sensitive enclosing record. Native epochs use the existing independent
owning-leaf layout: the Map leaf is shadowed with its narrower type, and sibling
leaves remain in place. No record reconstruction, cloning, unsafe cast or sibling
disposal is needed. Original record initializer carriers are preserved during
this phase; replanning them as uninstantiated generic records caused the earlier
missing C0 failure.

The original field witness now passes O0 and O3 with exact callback disposal and
its sibling marker preserved. The affected composition suite is running. Field
refill, nested record paths, multiple Map siblings and moved enclosing records
still need focused witnesses. The direct-root pre-write phase has not yet been
generalized to field refills. This is implementation progress, not acceptance.

The affected composition suite passes 105 tests after field transport. All eight
normal-stack depth canaries pass. Six additional closed logs, including native
transport failures and the repaired field, composition and depth checks, were
imported into SQLite, read back byte for byte, and removed from scratch storage.
The focused editor witness has not yet been expanded to record-field release.
Whole-project formatting, matched workload support and source checkpoint remain
pending. This batch does not accept B5 or the complete Map entry-release plan.

## Field refill progress

A field refill witness exposed an unchecked Closure(2)-to-Closure(1) conversion.
Pre-insertion widening now follows exact local storage paths, including record
fields. The planner updates the enclosing point type, while native emission
widens only the independently stored Map leaf before inserting the new payload.
Set emission resolves the root through field receivers so stale carrier wrappers
can be replaced by the checked current payload conversion.

The deletion/refill witness passes at O0 and O3 with exact disposal of all three
environments and an unchanged sibling marker. A focused LSP witness accepts field
release, refill and iteration, and rejects mutation of the surviving capture.
The affected composition suite is running. The guide now explains straight-line
release, replacement failures, conservative dynamic keys, lookup scopes and the
current survivor-scan cost. Nested paths, sibling Maps and field iteration still
need their native disposal witnesses; editor checking alone does not prove native
iteration. Matched support, cost pruning and the source checkpoint remain owed.

Field refill's affected composition suite passes 105 tests. The focused editor
witness and all eight normal-stack depth canaries pass. Six closed field-refill
and editor/depth logs are verified in SQLite and their loose copies removed.
No full family regression or timing run was performed. All entry-release source
changes remain uncommitted pending coherent checkpoint support and formatting.

## Native nested paths and enclosing moves

Native field iteration after refill passes at O0 and O3, preserving key order and
callback disposal. A nested record witness also passes before the move variant:
`outer.inner.handlers` deletes one callback, refills with another, iterates the
survivors, preserves a string sibling and outer integer marker, and clears both
remaining environments exactly once.

The same witness now includes moving the enclosing record before its first Map
insertion. That variant fails generated Rust with E0308: the source record still
contains an absent Map environment while the destination expects its later joined
callback family. Local epoch eligibility now follows a checked owning record move
through a carrier wrapper, but global `scoped_map_carriers` propagation handles
Map-valued leaves only. A record-valued name initializer does not currently expose
its nested Map leaves to that propagation. The next repair must propagate paired
source/destination Map paths across the owning record move before constructor and
carrier retargeting. Preserve sibling fields and current checked payload metadata.
Do not bypass the type mismatch or remove the moved witness. It remains red and
prevents acceptance of complete enclosing-record move support.

## Enclosing record move repair

The isolated map_storage_edges module now enumerates checked Map storage edges
inside direct Map moves, record name moves, record literal fields and existing
array-literal paths. Nested record paths pair source and destination leaves
without cloning source expression trees. Global scoped Map carrier planning joins
both endpoints before adapting constructors. Point-sensitive entry eligibility
continues following the owning record move.

The retained moved-nested-record witness now passes at O0 and O3. Both the unmoved
and moved variants delete, refill and iterate nested Map storage, preserve sibling
string and integer values, and drop every constructed callback exactly once.
The complete affected composition suite is running. This repair does not prove
conditional transport, same-expression release, generic aggregate moves or scan
cost pruning; their existing acceptance obligations remain open.

Enclosing record moves now pass the complete affected batch: 106 composition
checks, 42 scoped callable-record checks and four fallible record-callback checks.
All eight normal-stack depth canaries and both entry-release editor witnesses
pass. Seven closed storage-edge, regression, depth/editor and CLI-build logs are
verified in SQLite and their loose copies removed. The current debug CLI is
rebuilt from this uncommitted source; the five user-facing consumers have not
been refreshed. Whole-project formatting is running with that CLI before the
next source checkpoint. Benchmark support and performance-cost pruning remain
pending; no feature-family acceptance or timing claim is made.

Whole-project formatting completed successfully with the current CLI: zero files
changed and zero benchmark formatting fingerprints changed. The source checkpoint
still needs matched entry-release workload support and its untimed verification.

## Matched workload support

The existing matched scoped_record_composition workload now retains 47 scalar
observations per iteration. Three added observations cover direct known-key
release, moving a nested record before release/refill/iteration, and replacement
success or failure selected by the input value. Both language sources execute the
same operations. The independent oracle uses arithmetic formulas 3*value+8,
4*value+40, and 3*value+14 on failure or 3*value+10 on success.

The source contract checks the new operations and final observer row. Registration
includes rehashed mutation controls removing deletion, released-capture mutation,
record movement, field clear, replacement failure observation and replacement
clear. The registration test passes. Semantic source hashes were refreshed by an
explicit text edit to the existing manifest row; the measurement protocol and
sample counts are unchanged. The row remains unmeasured and family acceptance
pending. Native and Node-after-tsc verification is running without timing.

Untimed workload verification currently fails source checking in
movedFieldEntryRelease at fieldFirst mutation after deletion. Initial suspicion
was loss of the delete effect in a ternary predicate. Focused direct and nested
ternary witnesses pass at O0 and O3, including a called nested-record scenario.
An isolated exact workload helper passes source checking, as does a loop caller.
Keeping every other helper declaration while replacing the original workload body
with one helper call also passes. The failure therefore depends on preceding
workload composition; the ternary explanation is disproven. Keep the original
47-observation workload intact and reduce the earlier operations to identify the
shared analysis state or carrier interaction. No source checkpoint, benchmark
verification success or acceptance is claimed while this failure remains.

## Checked closure origin defect

Temporary ownership tracing identified the composition failure. On replay,
attach_payload_capture_loans indexed the checked closure table with the raw parser
closure ID. The workload's second field callback therefore acquired captures from
an unrelated earlier closure, including the moved record itself. Its surviving
entry retained a false nested owner of the deleted first callback. The traced
closure spans differed from their expression spans, proving the ID mismatch.

Capture attachment now resolves the checked closure origin, matching native
emission, with the checked Type::Closure slot as fallback. It does not use the
parser ID for capture lookup. Temporary tracing is removed. The original unchanged
47-observation workload passes source checking with the rebuilt debug compiler.
Affected ownership suites and native/Node-after-tsc untimed verification are
running. Their results must be inspected before checkpointing this repair.

The origin repair passes 152 affected compiler tests and seven focused capture
ownership units. Native matched verification now passes source analysis and fails
Rust compilation at a later callback error adapter: the narrowed callback returns
TsrCallbackError0<Record47>, while emitted transport matches
TsrStorageError21<Record47>. This is a separate point-sensitive callback failure
contract mismatch, not a recurrence of the capture-owner defect. Preserve the
native failure and inspect callback call-target/error metadata after narrowing.
No workload observation may be removed to avoid this composition. Untimed native
and Node verification remains incomplete; source checkpoint remains pending.

## Narrowed callback error targets

A focused helper witness reproduces the native error mismatch when its caller has
try/catch. Without typed exception transport the same source executes normally;
with transport the old StoredCallableField target expects a storage-family error
while the narrowed concrete callback returns its callback error contract.

Retargeting now changes a surviving concrete closure's call target to
ConditionalClosure along with its callee type. Existing native error registration
then selects the surviving closure contract rather than the removed family.
The focused helper and caught variant pass. An added bounds-failure variant checks
exact O0/O3 disposal before catch; that check and the resumed matched verification
are running. The original matched failure remains separate evidence.

## Source checkpoint evidence

The callback error-target repair passes the focused normal and caught helper
cases. A bounds failure from the surviving callback drops both constructed
environments exactly once before catch at O0 and O3. The affected batch passes
107 composition checks, 42 scoped callable-record checks and four fallible
record-callback checks, plus all eight normal-stack depth canaries and both focused
entry-release editor witnesses.

The 47-observation workload passes untimed native and Node-after-tsc verification
at all five configured correctness cases, including zero iterations and the full
2048-iteration case. The exact JSON is preserved in SQLite as
runtime/results/2026-10-08T23-38-22-518Z-verify.json. Seven closed error-contract,
regression and verification logs are also stored with byte-exact readback, and
verified loose copies are removed. The first default multi-engine attempt stopped
at the sandbox-denied tsx IPC socket after native compilation succeeded; its log
is retained separately. No timing or all-engine verification is claimed.

This is a source checkpoint of the implemented straight-line release path. B5
and the complete entry-release plan remain open. Priorities next: conversion-cost
pruning with focused generated-code and native witnesses; conditional and loop
transport; same-expression release; additional key, alias and failure combinations;
then array-element release and family acceptance. Five user-facing consumers still
require an explicit refresh from the completed checkpoint before testing them.

## Demand-driven survivor conversion

The no-scan red witness constructs two entries with independent borrowed captures,
deletes one, iterates the surviving entry and clears the Map without reusing the
released capture. Its generated source originally rebuilt surviving entries.

Checked release snapshots now retain the roots of capture loans removed by the
successful operation. Replayed snapshots union these roots conservatively. The
isolated map_entry_demand module computes later source uses once per checked body,
and nested planning retains the enclosing body's demand. Named calls retain
conversion conservatively because lexical capture access can be implicit at their
call sites. Callable values retain their captures through checked loan owners.
Source traversal of nested closure bodies may conservatively keep extra conversions;
this proof does not promise minimal conversion in every program.

Native epoch planning skips deletion/replacement narrowing when there is no later
use of a released root and no later opaque named call. It also avoids converting
a Map before insertion when joining the incoming payload leaves its existing
payload unchanged. The latter check belongs in epoch planning: changing global Map
canonicalization caused a nested record refill carrier mismatch, which the existing
native witness caught. Global canonicalization remains unchanged.

Focused checks preserve successful deletion/replacement, later capture mutation,
iteration, record siblings, nested refill and typed callback errors. No APIs or
IntelliSense signatures change. The existing 47-observation matched workload,
independent oracle and source contracts remain the behavior reference. This is an
efficiency checkpoint, not B5 acceptance or a claim that all updates are constant
time. Conversions required for later capture access can still scan survivors.

Checkpoint evidence: the initial 108-test composition batch passed 107 and caught
one nested refill regression. After the planner-only repair, that witness and
all 63 Map-filtered composition tests pass, including the no-scan native O0/O3
disposal witness. One survivor snapshot replay unit and eight normal-stack depth
canaries pass. Two focused entry-release editor checks pass. The 47-observation
workload passes all configured correctness cases for native and Node-after-tsc,
untimed, with original JSON stored as
runtime/results/2026-10-08T23-49-40-057Z-verify.json. Failed and repaired logs are
preserved separately in SQLite with byte-exact readback. Whole-project formatting
changes zero files and zero fingerprints.

## Conditional normal-completion joins

A focused red witness deleted the same known entry in both arms of an if statement
and then changed its captured array. Source analysis accepted the program, but
native Rust retained the removed environment in the Map payload type. Conversion
now occurs after the normally completing if statement, using the joined reaching
payloads rather than either arm's local snapshot.

The isolated map_entry_branch_release module gathers candidate roots from checked
release snapshots and records the actual joined payloads. It handles entries
introduced inside a branch as well as entries present before the branch. This does
not require another clone of the pre-branch capture owners. Exit storage keeps a
lazy collection of survivor proofs per conditional join, so independent Maps and
separate Map fields in one record can narrow at the same source point. Replayed
proofs remain conservative about changed inputs and union their payloads.

Native emission uses a shared post-entry-epoch helper after ordinary statements
and conditional emission, including the existing clear/storage return transport.
The focused clear/delete witness caught the missing post-conversion on that custom
path. Normal completion can therefore carry empty storage from one arm and a
surviving environment from the other. Returning arms do not contribute surviving
entries. Conditional reinsertion and branches that leave an entry untouched still
retain their captures and reject conflicting mutation.

Native O0/O3 witnesses cover both branch outcomes, two Maps sharing one capture,
two Map fields with an observed record sibling, clear/delete, early return,
replacement by distinct callback origins and branch-local insertion/deletion.
The final composition batch passes 113 tests. All eight depth canaries and three
focused entry-release editor checks pass. The matched workload now has fifty
observations, adding both-branch deletion, mixed clear/delete and paired shared
capture release. Tsrust and TypeScript sources, independent arithmetic oracle,
token contracts, rehashed mutation guards, exact registration expectations and
manifest source hashes are updated. Existing oracle/run/profile/report hooks are
retained and verified by the focused registration test.

Whole-project formatting changes only the matched Tsrust workload and proves
unchanged tokens before refreshing its fingerprint. Final native and Node-after-tsc
verification passes all five configured correctness cases. SQLite retains the
exact final JSON as runtime/results/2026-10-09T00-02-52-098Z-verify.json and the
preceding verification record, together with original failed and repaired logs.
Every imported record is read back byte for byte before its loose copy is removed. This remains focused implementation progress,
not full B5 acceptance. Loop joins, release inside a branch before its join,
same-expression release and array-element release remain open. No timing or full
compiler/LSP family regression is claimed.

## Loop completion transport in progress

The do/while release witness initially failed generated Rust. Completion proof
collection is now shared through map_entry_completion_release (renamed from the
branch-only module), and loops record the actual joined exit payloads after their
existing ownership joins. Native post-epoch emission is connected to ordinary
while, counted/post-test, for-of and custom empty-loop emission. The ordinary
while case in the zero/repeated-iteration witness executes with correct O0/O3
disposal. These changes are uncommitted and not accepted.

The counted-loop variant still fails native borrow checking. Temporary tracing
shows a completion proof at its checked for span, but no corresponding planner
visit. The parser lowers for into a synthetic Block. scoped_map_entry_epochs
recurses into that block with cloned results, then returns early because the Map
is declared in the enclosing body and no prior epoch exists. Its later converted
representation would also have to leave the block through native return transport.
The source proof itself retains the correct surviving payload. Tracing is removed.

Next implementation must carry eligible Map paths into lexical/synthetic blocks,
propagate changed storage types back to the enclosing sequence, and extend native
block return transport to include checked entry epochs. Keep loop iteration-body
planning separate: converting outer storage inside an iteration requires loop-carried
transport and is not authorized by this completion proof alone. Retain zero-body
and early break/continue ownership states. Do not merely emit a conversion in an
inner block and leave the original outer storage type alive.

New tests retain post-test and unconditional-break paths, zero/repeated while,
counted and for-of array/string variants, and negative zero-body and early
break/continue cases. The for variants and final combined checks remain pending.
No matched workload expansion, source checkpoint or loop acceptance is claimed.

## Loop completion source checkpoint

The synthetic-block transport defect is repaired. Eligible Map paths and changed
storage types now pass through unlabeled lexical blocks; native block returns
transport the renewed Map representation. Loop completion proofs use the actual
joined exit payloads. Zero iterations and early break/continue retain potentially
live captures. Native witnesses cover post-test and unconditional-break loops,
zero/repeated while and counted loops, and array/string for-of loops at O0/O3,
including callback disposal order. Conversion inside iterations remains pending.

The initial normal-stack depth run passed closures and overflowed in for-of.
Inspection of compiled frames showed loop_body retained 6,592 bytes. Moving its
post-recursion join into loop_body_completion reduced that frame to 3,456 bytes.
The failed for-of canary and the six unrun canaries then passed on the ordinary
worker stack. No stack-size increase was used.

A strengthened no-scan witness exposed a separate demand boundary defect: reads
inside a loop were mistaken for reads after completion because the statement span
covered its keyword. The existing expression walk now computes the full statement
end. The witness passes O0/O3 and verifies no survivor conversion is emitted.

The earlier coherent composition batch passed 116 tests. After the final demand
repair, all 67 Map-filtered tests pass, and the separate new no-scan witness passes.
Four focused editor checks pass. Matched support has 53 observations with paired
TypeScript, independent arithmetic oracles, source contracts and rehashed mutation
guards. Final native and Node-after-tsc verification passes all five correctness
cases, untimed. Whole-project formatting changes no files or fingerprints; the
current registration check passes. Final verification record:
runtime/results/2026-10-09T00-46-58-730Z-verify.json.

This is a source checkpoint, not B5 acceptance. Loop-carried refill/widening,
in-iteration release, array-element release, same-expression release and broader
composition remain open. Full family regression and timing are deferred. The
five user-facing consumers remain at gate181; scratch compiler builds are current.

## Loop refill follow-up in progress

The loop completion checkpoint is committed as 2f0dc05. A new native witness
then exposed a compiler panic when a narrowed Map was refilled in a loop by a
new callback origin. Loop storage now widens before entry to include both existing
survivors and checked incoming payloads. Several writes at the same boundary
preserve the original conversion input while accumulating the output family.

The counted-loop variant exposed duplicate emission because the synthetic Block
and its Loop share a source span. Pre-conversion now emits at the inner statement
only, and block return transport includes widened storage. The native witness
passes zero/repeated while and counted loops, including two incoming origins.
These follow-up edits are uncommitted. All 68 affected Map checks pass; editor,
additional loop forms, disposal checks, matched support and final formatting are
still owed before acceptance. Preserve the original failed and repaired logs.

Owner direction for record Set keys is Rust-style value equality and hashing with
stable owned keys, rather than JavaScript identity. Eligible fields, float policy,
mutation restrictions and ownership must receive focused witnesses and explicit
documentation before implementation acceptance. This remains independent open B5
work; no record Set support is claimed by these Map changes.

## Loop refill source checkpoint evidence

The follow-up now covers zero/repeated while, counted for, array for-of and
post-test loops. The multiple-origin variant verifies both inserted callbacks and
preserved key order. Instrumented O0/O3 native execution verifies replacement
and final clear disposal. All 68 Map-filtered composition checks pass. Two focused
editor checks pass, including rejection of mutation of a newly captured value
before clear. No parser or recursive checker dispatch changed in this follow-up.

Matched support now has 54 observations. The added counted refill observer retains
the surviving callback and two incoming origins after known-key narrowing. Paired
TypeScript, independent arithmetic oracle, full observation contract, rehashed
mutation guards and exact registration expectations are current. Whole-project
formatting changes one workload and its token-preserving fingerprint; registration
passes afterward. Native and Node-after-tsc verification passes all five cases,
untimed. SQLite preserves runtime/results/2026-10-09T00-55-57-942Z-verify.json and
original failed/repaired evidence, read back byte for byte before loose removal.

This remains focused B5 progress. Release inside an iteration or branch before
completion, same-expression release, precise array-element release, record Set
keys and broader error/alias/composition checks remain open. Full family regression
and timing remain deferred. User-facing consumers are still at gate181.

## Release inside branch source checkpoint

Two red O0/O3 disposal witnesses demonstrated native borrow errors when a branch
deleted an entry and then changed its released capture before completion. The
isolated map_entry_branch_transport planner now plans each arm with independent
storage types and the enclosing capture demand. Nonabrupt arms record restoration
to the incoming common storage family; abrupt arms keep their exit disposal.
Native tuple transport emits these restoration conversions before normal joins.
The lazy restoration table lives in storage_exit_availability and follows existing
checked-storage cloning. Existing ownership joins remain authoritative.

Witnesses cover returning arms, falling-through arms, clearing before completion
and retaining the surviving entry across the join. All 70 Map-filtered composition
tests pass. Eight normal-stack depth canaries pass. One editor witness accepts
released-capture mutation and rejects mutation of a still-captured sibling.

Matched support has 55 observations. Paired sources, independent oracle, exact
registration values, whole-output token contracts and rehashed mutation guards
cover deletion and capture mutation inside a branch. Whole-project formatting
changes one workload and one token-preserving fingerprint. Registration passes;
native and Node-after-tsc verification passes all five cases, untimed. SQLite
retains runtime/results/2026-10-09T01-01-45-311Z-verify.json and failed/repaired logs,
read back byte for byte before loose removal.

B5 is incomplete. Next verify branch refill after prior narrowing, including new
callback origins and aliases; do not assume restoring the incoming family can
represent newly introduced origins. Also complete in-iteration and same-expression
release, precise array-element release, record Set key rules and broader error
composition. Full family regressions and timing remain deferred. User-facing
consumers remain at gate181; scratch compilers contain current source.

## Branch refill after narrowing checkpoint

A new witness compiled but panicked in native execution: restoration from a
branch-local callback family to the previously narrowed incoming family excluded
the newly inserted origin. Conditional pre-entry widening now includes checked
incoming payloads, just as loop widening does. Arm restoration therefore targets
a family that represents both the untaken arm and newly inserted callbacks.

Native witnesses cover both branch outcomes, a moved Map, multiple incoming
origins and exact O0/O3 disposal. All 71 Map-filtered composition tests pass. One
editor check accepts mutation after clear and rejects mutation while the new
entry may retain its capture. No recursive checker dispatch changed.

Matched support now has 56 observations. Paired sources, independent oracle,
whole-output token contracts, exact registration values and rehashed mutation
guards cover two incoming origins after prior narrowing. Whole-project formatting
changes one workload and one token-preserving fingerprint. Registration passes.
Native and Node-after-tsc verification passes all five cases, untimed. SQLite
preserves runtime/results/2026-10-09T01-04-58-652Z-verify.json and original failure
and repair logs; byte-exact readback precedes loose removal.

B5 remains open. Next requirements include in-iteration release, same-expression
release, precise array-element release, record Set equality/hash key constraints
and broader alias/error composition. Family regression and timing remain deferred.
User-facing consumers are still at gate181; scratch compiler builds are current.

## In-iteration entry release: active red witness

HEAD f04c9f9 remains the last accepted source checkpoint. The new
map_entry_release_inside_iteration_preserves_refill_and_survivor witness checks
an outer Map with a persistent second entry. Each iteration inserts and deletes
a first entry, changes its released capture, and continues. Zero iterations must
preserve the second entry; repeated iterations must observe changed first values.
The source checker accepts this program, but native Rust reports E0502 because
its persistent Map representation still carries the first callback environment.
A negative companion asserts that mutation of the surviving second capture is
still rejected. The witness is uncommitted; no implementation is claimed.

The existing completion conversion occurs after the loop and cannot end this
iteration's native borrow before capture mutation. Simply planning the body with
inherited paths is insufficient: widening back to a family containing the removed
environment would carry its reference lifetime through the backedge. Likewise,
map_empty_loop_epoch cannot be reused by discarding storage: the second entry is
nonempty and must survive, including its exact disposal and key order.

Implement an isolated iteration transport plan with a stable carried family
containing the actual entry and all reaching backedge payloads. Collect entry,
normal body, continue and break ownership payloads through the existing joins;
never derive reachability from a deletion snapshot alone. Widen inside an iteration
before incoming writes, narrow before conflicting capture access, and move actual
surviving storage into the carried representation at backedges. Preserve break,
continue, post-test/update, zero-iteration, return, throw and finally behavior.
For-of must retain iterable ownership and its existing control targets. Independent
Map fields require independent transport without touching record siblings. No
implicit clone or survivor drop is allowed.

Acceptance must include while, counted, post-test and for-of variants, negative
retained/early-exit paths, O0/O3 disposal, affected Map composition, editor behavior,
matched TypeScript/oracle/contracts and whole-project formatting. Run all eight
normal-stack canaries after recursive checking/dispatch changes. Family regression
and timing remain deferred until B5 is complete. Keep current consumers unchanged
until rebuilding from a source checkpoint. Original native failure evidence is
retained in SQLite under tsrust-b5-map-iteration-release-red.log.

### Iteration entry versus backedge proof

Inspection of finish_loop_body confirms that pre-test completion payloads join
entry, normal body, continue and break owners. In contrast, tail_loop explicitly
omits a zero-body exit for do-while. Its final completion proof can therefore omit
an environment present on the first body entry. Do not use final completion
payloads alone as the uniform first-entry representation. Record actual entry
and backedge payloads separately before the ownership states are consumed.

A second native witness covers post-test zero-limit and repeated execution. Its
zero-limit case must run once; this prevents lowering through a pre-test shortcut.
A stable second callback survives all iterations in both witnesses. The initial
while witness and this post-test witness remain red. No compiler source changed
in this investigation.

When entry and backedge families differ, consider a distinct first-iteration path
followed by a loop carrying the proven backedge family. This can end initial
borrow environments before mutation while retaining exact condition evaluation,
zero-body behavior and survivors. Any such lowering must preserve continue/update,
break/finally, condition failures and iterable advancement. Do not hide these
requirements behind a larger common carrier that retains the eliminated borrow.

The strengthened post-test witness seeds a first-entry callback before the loop,
then replaces and deletes it inside the first body. Source checking succeeds and
native borrow checking fails. Both original and strengthened failure logs are
in SQLite as tsrust-b5-map-iteration-post-test-red.log and
tsrust-b5-map-iteration-post-test-entry-red.log. No live checks remain.

## Loop edge metadata checkpoint

The checker now records Map payloads separately at Entry, Back and Break edges.
Payload snapshots group decoded entry paths under their owning Map paths while
keeping record fields independent. Ordinary loop completion records normal and
continue owners before joining entry/exit state. Tail loops record first entry,
reachable checked update/condition backedges and break owners before exit joins.
Existing ownership joins are unchanged. Replay unions alternatives conservatively;
empty reachable recorded edges are distinguishable from absent edges.

The focused unit verifies that removing an entry at the backedge does not remove
its first-entry or break payload, and that replayed incoming origins do not leak
into entry facts. Its strengthened version verifies an empty reachable backedge.
All eight normal-stack depth canaries pass after the checker hooks. Five editor
entry-release checks pass. The Map composition batch passes 71 tests and retains
exactly the two expected native failures: in-iteration while release and the
post-test first-entry witness. Whole-project formatting changes no files/hashes.
Original logs are imported into SQLite and read back byte for byte before removal.

This is a prerequisite source checkpoint, not iteration or B5 acceptance. The
new edge APIs are not consumed by native lowering yet; their temporary dead-code
annotations mark that outstanding consumer explicitly. No new matched workload
or timing is claimed for metadata alone. Implement carried storage using these
separate facts next, preserving first-iteration evaluation and surviving entries,
then repair both native witnesses and expand control/error/loop-form coverage.
The five user-facing consumers remain at gate181; scratch debug compiler is current.

## Native iteration transport in progress

The ordinary while witness is now green, including exact O0/O3 disposal. The
isolated map_iteration_plan and map_iteration_emit modules use separate checked
Entry and Back payloads. For equal nonempty families on a normally completing
body, native storage narrows before loop entry. Each body can widen before an
insertion and narrow after deletion, then moves its surviving Map back through a
tuple assignment. This preserves the existing second callback and its key order;
it does not clear, clone or discard surviving entries.

Planning requires a consistent checked release candidate and invalidates local
paths passed to opaque callees before selecting iteration transport. The initial
path supports ordinary while bodies; abrupt controls, nested loops/try regions and
differing first-entry/backedge families are deliberately not claimed yet. The
new metadata is consumed by lowering, while the separate edge-presence query is
still needed by the forthcoming empty/different-entry path.

The current Map-filtered batch passes 72 tests and retains only the expected
post_test_map_iteration_release_keeps_first_entry_state_distinct failure. The
ordinary while mutation witness and its retained-sibling rejection pass. All eight
normal-stack canaries pass. Original green, batch and canary logs are preserved
in SQLite under tsrust-b5-map-iteration-transport-*.log with exact readback.
These compiler edits are uncommitted. No implementation acceptance, benchmark
expansion, timing or user-facing executable refresh is claimed.

Next implement distinct first-entry transport for post-test/populated entry cases,
then targeted continue/break and cleanup transport, counted/for-of forms, independent
record Map fields and failures. Complete editor and matched workload support and
whole-project formatting before accepting the implementation chunk. HEAD fd2aa00
remains the prerequisite source checkpoint; preserve the fifteen fixture edits.

## Normal iteration transport verification

Native transport now covers normally completing while, counted for, do-while
and array/string for-of bodies, including a local record Map field and empty
backedge storage. Do-while emits its first body using the original entry family;
subsequent bodies use the separately proven backedge family. Conditions execute
after each body, and body-local storage leaves scope before counted updates.
Surviving entries retain their key order without implicit cloning. O0/O3 disposal
witnesses cover the while and post-test paths.

The final Map batch passed 72 checks and exposed one additional empty post-test
defect: iteration callback calls retained a concrete closure target after their
payload became absent. Retargeting to the absent callback contract repairs the
focused post-test witness. All eight normal-stack depth canaries passed. One
focused editor check covers accepted while/count bodies and surviving-capture
rejection. Matched support now has 58 observations, including counted and
post-test transport; verification results remain pending until recorded.

These changes remain uncommitted until final formatting and matched verification
finish. B5 is not accepted. Abrupt break/continue, nested control/cleanup, differing
pre-test entry and backedge families, multiple independent Map fields, same-
expression release, precise array element release and structural record Set keys
remain open. The owner selected Rust-style equality and hashing for record Set
keys. No timing or user-facing consumer refresh is claimed.

Final bounded verification: benchmark registration passes; all five independent
result cases pass for native and Node-after-tsc with 58 observations. The untimed
record is runtime/results/2026-10-09T01-51-04-431Z-verify.json in SQLite. Final
project formatting changes zero files and zero fingerprints. Closed logs, including
the original failed batch and repaired focused check, are imported into SQLite
and read back byte for byte before their loose copies are removed.
