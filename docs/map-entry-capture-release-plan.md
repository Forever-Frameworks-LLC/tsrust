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
and timing remain deferred. User-facing consumers are still at gate 181.

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

## Abrupt iteration transport

A focused native witness first failed with E0502 when break/continue followed
known-entry deletion and capture mutation. Each abrupt statement now records its
point-sensitive storage inputs. Selected loop controls convert to the proven
carried survivor family and yield a stop flag plus owned Maps from a labelled
body scope. The enclosing loop restores those Maps before acting on the flag.
Break skips counted updates and condition evaluation; continue runs counted
updates. Do-while retains its first-entry distinction, and for-of retains its
shared iterator evaluation and binding layouts. No surviving entry is cloned.

The focused witness covers while, counted, post-test and array for-of, observable
counted updates, exact O0/O3 disposal and retained-capture errors. Break before
deletion rejects a later mutation because the entry remains live. The affected
Map batch passed 71 checks, the editor witness passes, and all eight normal-stack
canaries pass before the final update guard. Separate body/update representations
preserve existing normal-loop lowering. Abrupt loops whose counted update touches
Map storage require further transport and are not claimed by this chunk.

Matched support appends a 59th observation for abrupt iteration, with independent
arithmetic oracle and rehashed guards for deletion, break, continue and tail
execution. Registration and native/Node-after-tsc verification pass on the initial
implementation. Final guarded-source verification and evidence import remain
pending. B5 remains open: cleanup/nested control transport, different pre-test
entry families, independent Map fields, same-expression/array release and
structural record Set keys still need work. User-facing consumers are unchanged.

Final guarded-source checks pass: abrupt four-form native/disposal witness, normal
iteration regression witness and all eight depth canaries. Final formatting
changes no files/fingerprints. All five independent result cases pass for native
and Node-after-tsc with 59 observations. The final untimed record is
runtime/results/2026-10-09T02-00-12-888Z-verify.json in SQLite. Original red,
intermediate compiler errors, repaired checks and both verification records are
retained independently with byte-exact readback; loose copies are removed.
This is a source checkpoint for abrupt iteration transport, not B5 acceptance.

## Counted Map update transport

The new counted_map_update_keeps_body_payload_until_update_deletes_it witness
first failed in generated Rust with E0502 for the body capture and the temporary
update capture. The body creates a temporary third entry after deleting first;
continue must preserve third until the counted update observes size two and
deletes it. Narrowing to the backedge family at body completion would discard
that still-live entry.

Planning now joins normally completing body storage with each break/continue
input into a separate body-completion family. Update expressions are retargeted
to that family. The update converts to the backedge family only after its effects;
the stopped path skips the update and converts its separately proven survivors.
The emitter shadows the body storage when the native type changes instead of
assigning incompatible families. Normal loops retain their previous lowering.
The blanket prohibition on Map-touching counted updates is removed. Distinct
pre-test entry/backedge and break/backedge payloads still need later transport.

Scalar observations such as console.log(handlers.size) no longer invalidate a
local Map proof. They pass a scalar, not storage. Passing scoped Map storage or
a record containing it to an opaque callee still invalidates the proof.

Focused checks cover observable update order, surviving second entries, third
release, break skipping updates, capture-conflict before deletion, formatting
and exact O0/O3 disposal. The affected Map batch passes 75 tests. One focused
editor witness and all eight normal-stack depth canaries pass. Matched support
appends a 60th observation with independent arithmetic and rehashed source guards.
Final registration, native/Node verification and SQLite import remain pending.
No timing or user-facing consumer refresh is claimed. B5 remains open.

Final counted-update registration and all five independent native/Node result
cases pass with 60 observations. The untimed record is
runtime/results/2026-10-09T02-07-31-272Z-verify.json in SQLite. Formatting
preserves tokens, formats one changed workload and refreshes its fingerprint.
Original failing and repaired checks are retained independently in SQLite with
byte-exact readback; closed loose copies are removed. The implementation chunk
is ready for its source checkpoint; B5 and family measurements remain pending.
Next cover cleanup/nested-control transport and distinct pre-test entry families.

## Exception-phase payload prerequisite

The new map_entry_iteration_control_runs_finally_before_leaving_body native
witness fails before implementation. It deletes a known first entry, mutates its
released capture, then continues or breaks through finally. Cleanup must inspect
and call the surviving second entry before the loop control takes effect. Its
O0/O3 disposal expectations remain asserted; native lowering is not accepted yet.

The isolated map_completion_payloads module records checked payload alternatives
at Body, Handled and Cleaned phase boundaries, separately for normal, return,
throw, and each targeted break/continue. Repeated alternatives are conservatively
joined within their own phase/completion identity. A recorded empty normal state
means reachable empty storage; a missing state means that completion was not
recorded. No phase may substitute its normal payloads for error or control
payloads. Source availability remains governed by storage_exit_availability;
payload metadata alone cannot revive a transferred source binding.

Two focused proof tests cover different control targets, repeated alternatives,
empty-versus-missing states and real checker states. The checker witness has
explicit failures before deletion in both body and handler. Normal/return paths
retain only second; pre-deletion errors retain first and second. Cleanup clearing
records reachable empty normal and return storage. Earlier failed proof assertions
assumed the wrong handler error shape; strengthened explicit-error witnesses
replace those assumptions. Original logs remain separate evidence.

The phase hooks run after checked body/error collection, after catch, and after
finally, without changing native dispatch. Native accessors remain test-only
until lowering consumes this metadata; no unused production API is introduced.
This prerequisite has no new observable method, catalog entry, workload or timing
claim. Existing 60-observation matched support remains current. Final bounded
checks and source checkpoint are pending. B5 and cleanup implementation remain
open. Next plan phase-specific native storage and completion payload layouts,
then route control through cleanup before yielding loop survivor storage.

Final prerequisite evidence: both phase proof tests pass, all eight normal-stack
canaries pass, six existing editor entry-release checks pass, and the affected
Map batch passes 75 tests with exactly the newly retained native cleanup witness
red. The independent transpiled Node runtime reference passes its complete
observable-output expectation. Whole-project formatting changes no files or
fingerprints. Existing matched workload registration passes. Original failed
proof assumptions and the repaired explicit-error tests, plus all closed check
logs, are imported into SQLite and read back byte for byte before loose removal.
No cleanup implementation acceptance, new matched observation, timing or installed
consumer refresh is claimed. The next source chunk must consume phase payload
proofs, preserve per-operation storage types and source availability, and carry
normal/error/control storage through finally before resuming loop control.


### Native storage projections and post-test conditions

Native completion values now consult point-specific Map storage representations.
When live storage differs from the destination family, emission moves entries and
projects callback payloads into that family. An unavailable transferred binding
produces typed empty storage; projection cannot revive its source owner. The
availability regression covers transfer boundaries and storage renewal.

Focused proof tests cover narrowing after deletion, widening before refill and
post-test conditions. The post-test witness initially failed because lexical
lookup retained the last body statement's working family. The loop planner now
records the condition after backedge conversion with the carried family. All four
phase/native-point proofs pass. This is a prerequisite for cleanup transport,
not cleanup acceptance. The retained finally/control native witness remains red;
phase-specific cleanup storage, nested control transport and structural record
Set keys still require implementation. No timing or consumer refresh is claimed.


Final native-point prerequisite checks: all four phase/point proofs pass, the
projection availability proof passes, all eight depth canaries pass, and the
existing post-test native witness passes. Six editor checks and the matched
workload registration pass. The final cleanup/control witness still fails native
compilation; earlier affected batches retain their 75 passes plus that red.
Whole-project formatting changes zero files and zero fingerprints. Closed check
logs, including the post-condition red and repaired proof, are imported into
SQLite and verified byte for byte before deleting loose copies. B5 remains open.


### Materialized exception phase storage families

The isolated map_completion_layouts analyzer now materializes a concrete Map
payload family for each recorded phase and completion identity. It joins only
payloads reaching that identity, preserving normal, return, error and targeted
control boundaries. Places present in another phase but empty in this phase get
an empty callback environment. Missing completion identities get no layout.
Record field paths remain distinct; layout types describe storage, not renewed
source availability. The planner runs outside recursive exception dispatch.

A new red proof initially found no normal layout. Its repaired witness proves
that normal and return storage retain only the surviving callback, pre-deletion
errors retain the wider family, and normal/return cleanup clearing gets a third,
empty family. The normal layout's payload is checked against the exact single
record in the ownership proof. An absent break completion remains absent.

Five phase/native-point proofs, all eight normal-stack depth canaries and six
editor entry-release checks pass. The selected map_ native batch has 72 passes
and the retained finally/control compilation failure; this is not full compiler
acceptance. Phase layouts are checked metadata with test-only accessors pending
native consumers. They do not yet repair that cleanup witness. No new catalog
entry, workload observation, timing or installed consumer refresh is claimed.
Next consume phase layouts in carried completion emission, specialize cleanup
storage for its incoming family, and resume control only after cleanup. Keep
error and cleanup override behavior, ownership and disposal assertions intact.


Whole-project formatting changes zero files and zero fingerprints. Matched
scoped-record-composition registration passes. Ten closed logs, including both
red witnesses and strengthened proof checks, are imported into SQLite, read back
byte for byte, then removed from scratch storage. B5 remains incomplete.


### Deletion through straight-line exception cleanup

Checked local entry epochs now run within try bodies, handlers and cleanup.
Native carried completion discovers these entry conversions as well as clears.
Normal completion moves a narrowed live Map into the existing common transport
family; unavailable source owners remain typed empty. Plain branch tuple emission
keeps its existing survivor family instead of converting to the function-wide
family. The first experimental shared helper caused two branch regressions;
separating completion conversion repaired both without weakening their witnesses.

Borrowed Map iteration under carried storage now uses an inner typed completion
region. It finishes the iterator scope before transporting the owning Map on
error or escaping control. Local loop controls still act on the iterator loop.
This fixes moving a Map while its iterator temporary still borrows it. The new
native witness checks deletion, mutation of the released capture, surviving
callback invocation in finally, error during borrowed iteration, catch visibility,
cleanup order and exact disposal at both optimization levels. The editor witness
accepts mutation of the deleted capture and rejects mutation of the survivor.

Matched verification exposed a tail-expression boundary defect in an existing
replacement scenario. Short statement spans did not include the final native
conversion. The planner now records exact body-exit storage types, and normal
completion uses that type before projecting into the common transport family.
A focused proof checks the exit against the actual tail replacement epoch.
Phase-specific normal/error/cleanup layouts remain available checked metadata;
separate phase-specialized emission is still required. This chunk does not claim
that the phase layout consumer or the broader B5 family is complete.

The original loop-cleanup witness remains red. Planning its inner try exposed a
moved outer Map on repeated iterations: resuming break/continue must first
reassign the carried loop tuple. The exploratory while recursion is removed;
the failing evidence is retained. Next combine loop entry/back/body transport
with resumed cleanup control, without installing direct source control hooks
inside try that could bypass finally. Preserve cleanup overrides and disposal.


Additional handler and cleanup witnesses now pass native execution and exact
callback disposal at both optimization levels. A named pending completion alone
was insufficient: its inactive storage variants kept capture lifetimes alive.
For phases with nonempty entry narrowing, control and storage leave that owner
scope separately before entering the phase. Clear-only renewal retains its
existing independently inferred normal storage lifetime; unconditionally
combining normal/error storage under one explicit tuple type regressed six
accepted clear/restoration checks. Those failed runs are preserved. Selecting
transport by the checked entry conversion restores all accepted cases; the final
selected batch is 73 passing tests and only the retained loop-cleanup red.

Six phase/native-boundary proofs, eight normal-stack depth canaries and seven
editor entry-release checks pass. Matched support adds finallyEntryMutation,
with 61 scalar observations per iteration and an independent 3*value+10 oracle.
Token-based source guards reject removal of deletion, mutation and survivor
invocation. Registration literal expectations are extended; the initially stale
60-value assertion and repaired run remain separate evidence. Native/Node untimed
verification and final formatting are recorded below when complete. No timing,
full-family acceptance or installed consumer refresh is claimed.


Final matched native and Node-after-tsc correctness verification passes for all
61 observations and all retained input cases. Canonical final record:
runtime/results/2026-10-09T03-05-12-987Z-verify.json. Whole-project final
formatting changes zero files and zero fingerprints. Original failed checks,
repaired checks and both successful untimed records are imported into SQLite
and read back byte for byte before their loose copies are removed. B5 remains
open; this is a focused native transport checkpoint, not a family milestone.


### Cleanup controls and carried iteration storage

The retained finally/control witness now passes. Iteration planning accepts
checked exception regions and records cleanup controls separately from direct
source controls. A source break or continue inside try receives no direct
map_body hook. Resuming cleanup yields the enclosing body tuple only when that
loop belongs to the current completion region; an enclosing finally receives
its pending control first. The isolated map_cleanup_control_emit module owns
this dispatch. Its fallback excludes already intercepted loop targets so native
control flow cannot syntactically skip tuple reassignment on impossible arms.
The first native attempt exposed that moved-owner path; its failure is retained.

Cleanup can replace a pre-deletion error with continue or break. Those edges may
retain a wider payload family than normal execution or initial loop entry.
For cleanup loops, the carrier joins entry, back and break payloads. Proven
emptiness is recorded for the carrier separately from the actual entry and
backedge. Initial populated storage is not discarded merely because the
backedge is empty. A focused native/disposal witness exposed an absent-callback
conversion panic before this repair and now observes size one on first entry,
size zero on the next, and correct callback disposal.

Native execution and disposal pass for while, counted, post-test and for-of
loops, pending continue/break, cleanup overriding both controls, a wider break
family, and nested finally ordering. The nested witness proves inner cleanup
precedes outer cleanup before loop control resumes. Seven phase/native proofs
and all eight normal-stack canaries pass. The affected native map_ batch has
74 passes and no retained red. The separate populated-entry/empty-backedge
witness passes. Eight editor checks preserve acceptance of the deleted capture
and rejection of mutation of a surviving capture. These are bounded checks;
whole compiler/LSP regressions and timing are not claimed.

Matched support adds cleanupIterationMutation as observation 62, with an
independent arithmetic oracle for one and three iterations. Source guards are
scoped to individual helpers where identical control snippets occur in both;
an initially weakened presence guard failed its mutation test and is repaired.
Registration passes. Native and Node-after-tsc untimed verification passes for
all retained cases. Final formatting, immutable record identities and SQLite
retention are recorded below after checkpoint verification.

B5 is still incomplete. This acceptance covers the tested cleanup iteration
paths, not all Return/Throw, labelled/nested-loop compositions or phase-specific
normal/error/cleanup emission. Ordinary loops with different entry/back/break
families remain separate pending work. Stable owned record Set keys should use
the owner's chosen structural equality and hashing, with borrowing keys and
mutable stored keys excluded, explicit cloning for retained copies, and a
consistent numeric-field policy before float-containing records are accepted.
That key feature is not implemented in this chunk.


Final untimed correctness record: runtime/results/2026-10-09T03-29-23-041Z-verify.json. Whole-project checkpoint formatting changes zero files and zero fingerprints. All 27 closed logs, including red proofs, native failures and repaired checks, plus the matched record are imported into SQLite and read back byte for byte before loose removal. No installed consumer refresh, whole-family acceptance or timing is claimed.


### Pending callable exits through Map cleanup loops

Return and throw no longer exclude a checked loop from entry storage planning.
The focused red witness checked successfully but generated Rust retained a
borrow of the deleted capture. Conditional callable exits now use the existing
completion transport, preserving finally execution and disposal before return
or error propagation. Cleanup return, throw and continue retain their ordinary
replacement precedence. Source callable exits receive no direct loop hook.

A separate native red exposed loops with no backedge. Cleanup still needs
working storage for deletion and survivor iteration even when every body path
exits the callable. Planning now retains that entry family. Counted and
post-test loops also need this plan when their tail is unreachable; unchecked
updates and post-test conditions must not be analyzed or emitted. The initial
counted witness failed in generated Rust; the repaired witness executes while,
counted, post-test and for-of forms. A post-test witness removes its unreachable
fallback return rather than changing the language's reachability diagnostic.

Native witnesses verify pending integer returns and errors, cleanup return/error
replacement, cleanup continue cancelling a return, no-backedge exits and a
return expression evaluated before cleanup mutation. Exact callback disposal
is checked at both optimization levels. Cleanup mutation must still account for
failure before deletion: an unconditional mutation there is rejected, while an
explicit cleanup deletion releases the capture before mutation. The failed
unsafe-cleanup attempt is retained as evidence, not accepted behavior.

Final bounded checks pass: 76 selected map_ native/ownership tests, eight
phase/native-boundary proofs, eight normal-stack depth canaries and nine editor
entry-release checks. No whole compiler/LSP regression or timing is claimed.
Matched support adds five return/error cleanup observations with independent
arithmetic oracles, bringing the family to 67 observations per iteration. Its
wrapper narrows a caught integer and rethrows runtime errors; an initially
unnarrowed wrapper failed checking, and both runs remain evidence. Token guards,
mutation rejection and registration pass. Native and Node-after-tsc untimed
verification passes for all five retained correctness cases. Canonical record:
runtime/results/2026-10-09T03-47-01-259Z-verify.json.

Whole-project final formatting changes zero files and zero fingerprints. All
28 closed logs and the matched record are imported into SQLite and read back
byte for byte before their loose copies are removed. The five installed
consumers remain at gate 181; scratch compiler binaries are current. B5 is still
open: labelled/nested-loop compositions, ordinary differing loop edge families,
phase-specialized native storage, broader aggregate/within-expression precision
and structural owned record Set/Map keys remain pending. This checkpoint does
not claim those features or feature-family acceptance.


### Labelled and nested Map cleanup loops

The labelled-loop red checked successfully but failed native capture mutation.
Eligibility and exception storage discovery had both excluded labelled controls,
even though their checked destinations are the same loop targets used by
unlabelled controls. The new map_iteration_shape module inspects resolved targets
and completion boundaries with an iterative walk. The shared
map_exception_storage_plan module discovers conversions for body, handler and
cleanup, including planned nested loops. It replaces duplicated discovery code.

The nested witness exposed two further defects. Completion storage had no local
control destinations, so an inner break incorrectly escaped its owning phase and
ultimately reached the callable's impossible-control arm. Phase emission now
records its local destinations. Once that path ran, an absent inner backedge was
mistaken for an empty break: the survivor disappeared before outer cleanup.
MapIterationEpoch now preserves break emptiness independently, and all loop
forms select the appropriate back or stopped conversion after cleanup.

Counted variants exposed physical exit and lexical scope mismatches. A checked
exception transport returns the common native family, although its body may
narrow storage before leaving. The isolated map_exception_storage_types module
records that normal exit representation without cloning the source tree.
Counted-loop initializer blocks now return storage converted by their labelled
loop, rather than dropping the new owner in the block and using the moved outer
owner afterwards. The original native failures and repaired checks are retained.

Native witnesses pass labelled while, counted, post-test and for-of controls;
nested versions of those loop forms; inner-finally before outer-finally order;
catching an inner error; inner cleanup continue replacing a pending outer exit;
and rejection of mutation of a surviving capture. The base labelled and nested
witnesses verify exact callback disposal at both optimization levels. A raw
storage proof distinguishes absent backedge, live break payload, common phase
exit and the narrowed body representation.

Final bounded checks pass: 78 affected map_ tests, nine phase/native-boundary
proofs, eight normal-stack depth canaries and ten editor entry-release checks.
Matched support adds nestedCleanupEntryMutation with and without inner cleanup
override, bringing the family to 69 scalar observations per iteration. Independent
arithmetic oracles, token guards and mutation rejection pass. Native and
Node-after-tsc untimed verification passes all five retained correctness cases.
Canonical record: runtime/results/2026-10-09T04-05-37-523Z-verify.json.

Whole-project final formatting changes zero files and zero fingerprints. All
23 closed logs and the matched record are imported into SQLite and read back
byte for byte before loose removal. Installed consumers remain at gate 181;
scratch compilers are current. No whole-family regression, timing or B5
completion is claimed. Pending work includes arbitrary labelled blocks, direct
nonlocal controls outside completion regions, ordinary differing loop edge
families, independent native phase layouts, broader aggregate/within-expression
precision and structural owned record Set/Map keys.

Next continuation should add a focused direct nonlocal-control witness in nested
loops before changing eligibility. Source control outside try must not skip a
carried tuple reassignment. Labelled block exits need their own owner transport;
do not treat a labelled block as an ordinary lexical block. Keep source ownership
proofs authoritative, run the eight normal-stack canaries after analyzer changes,
and preserve the 15 unrelated fixtures with explicit Git paths.


### Direct nested nonlocal controls: active red witness

After checkpoint c52310f, `nested_map_direct_outer_controls_preserve_survivors`
adds the ordinary nested-loop counterpart without any try/finally boundary.
Source checking and format round trips succeed, but generated Rust fails with
E0382 (moved Map storage) and E0502 (the deleted capture remains borrowed).
The focused test is intentionally red and is not accepted or committed yet.
Its exact diagnostic log is retained in SQLite as
`tsrust-b5-direct-controls-red.log`, verified by byte-exact readback.

The current iteration shape rejects an ancestor target outside a completion
region. The planner also does not descend into nested loops when installing
direct control conversions. Simply removing either restriction is insufficient:
an ancestor break can bypass the inner carried tuple assignment. Implement
explicit propagation through each intervening loop owner, preserving the
source destination and distinguishing normal completion, break and continue.
Do not route an ancestor control through the inner loop's boolean stop flag,
which would lose its destination. Keep ordinary differing edge families and
arbitrary labelled blocks separate until their transport is proved. Extend the
witness with counted, post-test and for-of loops and survivor disposal before
acceptance; update editor, matched workload support and bounded evidence.


### Direct nested Map loop controls accepted

The direct ancestor-control witness now passes. Iteration shape validation accepts
resolved loop destinations, while controls to labelled blocks remain excluded.
Loop planning joins reaching edge families for nested loops and for a loop with
a direct ancestor control. The body-family walk collects nested control input
representations. Direct conversion installation descends through nested loops
but handles only controls whose checked destination names the current loop.
This prevents an inner plan from replacing an ancestor destination.

The red investigation initially proposed forwarding through each intervening
loop. Native execution proves a simpler ownership-preserving transport: convert
the innermost live owner directly into the destination loop's carried body tuple.
All intervening loop scopes are exited, so their moved storage is never used
again. The destination loop performs the normal break/backedge conversion.
Exception boundaries retain their existing completion/finally transport; this
change does not bypass cleanup or introduce cloning.

The focused witness covers inner while, counted, post-test and for-of loops,
ancestor break and continue, local inner break, surviving callback invocation,
format round trips, capture-conflict rejection and exact disposal at both native
optimization levels. The affected batch passes 79 Map tests, nine phase/storage
proofs, eight normal-stack depth canaries and 11 editor entry-release checks.

Matched sources add directNestedEntryMutation, an independent arithmetic oracle,
source guards and mutation rejection. A shared helper guard was scoped to its
function so the new helper cannot mask a deleted snippet from the earlier one.
All five native and Node-after-tsc cases agree on 70 scalar observations per row.
Canonical record: runtime/results/2026-10-09T04-15-23-075Z-verify.json. Sixteen
closed logs and that record are imported into SQLite, read back byte for byte
and removed from loose storage. Final project formatting changes zero files and
zero fingerprints. No timing or whole-family acceptance is claimed.

B5 remains open. Next implement owner transport for arbitrary labelled block
exits with a focused failing witness; keep source ownership and disposal proofs
authoritative. Ordinary differing loop edge families, independent native phase
layouts, broader aggregate/within-expression precision and structural owned
record keys remain pending. Installed consumers remain at gate 181; scratch debug
compiler and benchmark-built native compiler are current. Preserve the 15
unrelated fixture edits and stage only explicit paths.


### Labelled block exits: active failing witnesses

After checkpoint 9e3cdb3, two uncommitted witnesses cover a labelled block whose
normal path and break path retain a second callback entry after deleting the
first. `labelled_map_block_direct_exit_preserves_survivors` tests direct break.
`labelled_map_block_exit_preserves_survivors_after_cleanup` tests break through
finally, including cleanup observation followed by continued use outside the
block. Both check and format successfully, but generated Rust rejects mutation
of the deleted capture with E0502. Both independent Node references produce the
expected output. These tests are active red witnesses, not accepted features.

The current local epoch planner explicitly skips labelled blocks. The lexical
block emitter likewise declines storage results for labelled blocks; plain
control dispatch emits a valueless break. Existing labelled-block analysis joins
normal and break capture owners, but does not retain distinct native storage
inputs for a storage-bearing exit. This is separate from the now-fixed loop
transport, whose targets return carried tuples.

Implementation requirements:

1. Preserve normal and break owner proofs for the labelled block after cleaned
   completions join. Discover eligible local Map places without admitting maps
   whose source ownership remains unavailable on any reaching path.
2. Plan the block's working and output families in an isolated module. Recurse
   through its body with local Map epochs, retaining each direct break input.
   Convert each reaching path to one checked block result family.
3. Emit a labelled block expression returning its Map owner tuple. Direct breaks
   return that tuple; cleaned breaks return it only after every exited finally
   runs. Valueless dispatch cannot exit a storage-bearing block expression.
   Preserve destination identity for nested blocks and loops.
4. Verify skipped bodies, conditional retained entries, nested block destinations,
   cleanup overriding the pending break, exact disposal at both optimization
   levels and surviving-capture rejection. Add editor checks and matched workload,
   independent oracle, scoped token guards and registration mutation controls.
5. Run affected tests and all eight depth canaries, format the entire project,
   retain SQLite evidence and commit explicit paths. B5 remains open until its
   remaining family requirements are fulfilled.

Three closed logs are imported into SQLite, read back byte for byte and removed:
tsrust-b5-labelled-block-red.log, tsrust-b5-labelled-block-direct-red.log and
tsrust-b5-labelled-block-node.log. No compiler source or installed consumer is
changed in this investigation. The two new tests remain uncommitted together with
this plan update; the 15 unrelated fixture edits remain untouched.


### Labelled block source join proof implemented; native transport pending

The labelled-block analyzer now records completion survivor metadata after its
normal/break capture-owner and availability joins. It reuses the existing
record_map_completion_release helper; it does not authorize elimination from an
individual deletion path. The new raw proof checks both deletion on every exit
(one retained payload) and an early break before deletion (both payloads remain).
The valid red showed missing block completion metadata. The repaired proof and
all ten phase/native-boundary tests pass. All eight normal-stack depth canaries
pass on the modified analyzer.

The first attempted proof lacked main and failed entry validation. It was repaired
and rerun red with implementation removed before adding the analyzer change.
Those failed attempts remain distinct immutable records, not acceptance evidence.
The two native labelled-block witnesses still fail E0502 after the metadata fix;
there is no claim of usable labelled-block storage transport yet.

Next add a dedicated block storage plan using this joined proof. Carry the checked
working family into the block, collect point-sensitive direct-break inputs, and
return a common result owner on normal and break paths. Wire cleaned block breaks
through completion dispatch without bypassing finally. The existing epoch walk's
early labelled-block skip and valueless block break must be replaced together.
Keep the block destination distinct from loop stop flags. Verify the retained
native reds plus conditional retained entries and disposal before acceptance.

Seven new closed logs are retained in SQLite, read back byte for byte and removed:
proof-red, proof-first, proof-valid-red, proof-fixed, proofs, canaries and
native-still-red under the tsrust-b5-labelled-block prefix. Compiler changes,
raw proof, native witnesses and plan updates remain uncommitted in-progress work.
HEAD remains 9e3cdb3. Preserve the 15 unrelated fixture edits. B5 remains open.


### Labelled block Map transport accepted

Labelled blocks now retain checked normal/break owner joins and plan their storage
in map_block_storage_plan. The planner recurses with eligible local Map places,
joins all reaching callback payloads, records point-sensitive direct break
conversions and restores the output family at the block boundary. Early exits
retain entries they bypass; newly inserted callback origins remain present.
Source ownership proofs remain authoritative and opaque callee effects do not
create a local elimination proof.

map_block_storage_emit emits a labelled result expression carrying the Map owner
tuple. Direct breaks return that tuple. Cleaned breaks dispatch to the block only
after required finally bodies have completed. Nested block and loop destinations
remain distinct. Exception discovery and lexical outer scopes now recognize
planned labelled-block results. No captured value is cloned implicitly.

Expanded witnesses exposed two additional native defects. A post-test body that
cannot reach its tail emitted a non-reaching empty projection after its completion.
It now returns without that projection. A body ending in a checked break emitted
its conversion before the break and again in an unreachable tail; the second
conversion used an obsolete carrier family. Definitely abrupt iteration bodies
now emit a typed unreachable tail instead of repeating conversions or reading
moved owners. Failed logs are retained separately from repaired evidence.

Native witnesses cover direct and cleaned block breaks, nested block destinations,
loop-to-block breaks for while, counted, post-test and for-of loops, an early exit
retaining the first entry, a refill with a third origin, cleanup overriding the
pending completion, mutation after deletion and after the block, survivor conflict
rejection and exact disposal at both optimization levels. The raw join proof
preserves one payload after unconditional deletion and two after an early break.

Final bounded evidence passes 83 affected Map tests, 12 editor entry-release
checks, ten phase/storage proofs and all eight normal-stack depth canaries.
Whole-project formatting changes zero files and zero fingerprints at the final
checkpoint. Matched sources, independent arithmetic oracles, token guards and
mutation controls cover plain labelled blocks and nested-loop block exits.
All five native and Node-after-tsc cases agree on 72 observations per row.
Current record: runtime/results/2026-10-09T04-36-26-109Z-verify.json. The earlier
successful 71-observation record, runtime/results/2026-10-09T04-28-28-494Z-verify.json,
is preserved independently. Twenty-nine closed logs and both records are imported
into SQLite, read back byte for byte and removed from loose storage.

B5 remains open: ordinary differing loop edge families, independent native phase
layouts, broader aggregate/within-expression precision and structural owned
record Set/Map keys remain pending. No whole-family regression or timing is
claimed. Installed consumers remain at gate 181; scratch compilers are current.
Next add a focused witness for an ordinary loop whose entry, backedge and break
retain different callback origins before changing its carrier eligibility.
Keep benchmark support current as it lands and preserve the 15 unrelated fixtures
using explicit Git paths.


### Ordinary loops with distinct Map edge families accepted

The ordinary-loop red passed source checking but retained the deleted capture in
native Rust. The planner previously excluded different entry/backedge and
break/backedge payload sets. It now joins every reaching edge while preserving
entry, backedge and break emptiness independently. Empty environments contribute
no live callable origin when joining control inputs; treating their phantom
callable as an origin caused an internal join panic and is repaired.

A narrowed pre-loop owner exposed a runtime excluded-environment panic. The first
owner held only the old callback, while a later backedge held the new callback.
MapIterationEpoch now records a separate working family. That family joins
physical incoming environments with reaching carrier environments; first and
repeated bodies convert into it before their local deletion/refill transitions.
It must retain physical incoming types because exception cleanup can observe them
before deletion finishes. An intermediate narrower working-family implementation
broke five cleanup witnesses; the repaired join restores them. Source capture
availability remains authoritative and no implicit clone is added.

Native witnesses cover zero/repeated iterations, empty break, an early break
retaining the old entry, replacement origins, while/counted/post-test/for-of forms,
a pre-loop deletion and refill that narrows the initial owner, and exact disposal
at both optimization levels. Raw proofs distinguish entry/backedge origins and
empty break, and prove a narrowed incoming type differs from its working family.
The editor rejects mutation of a surviving capture. The first editor rejection
attempt mutated before capture and was legal; the repaired witness mutates after
the loop while the Map still owns the callback.

Bounded evidence is retained without replacing failed runs. The original affected
batch passed 84 tests. After adding the working-family repair, a later batch
passed 79 and failed five cleanup cases. All five failures and the ordinary-loop
witness pass the six-case focused repair run. Current source also passes 13 editor
entry-release checks, 11 phase/storage proofs and all eight normal-stack canaries.
There is no claim of a new full compiler/LSP or whole-family run.

Matched support adds ordinaryEntryEdges and narrowedOrdinaryEntryEdges with three
observations, independent arithmetic oracles, current token guards and mutation
rejection. Native and Node-after-tsc untimed verification passes all five cases
with 75 observations per row. Canonical record:
runtime/results/2026-10-09T04-55-27-316Z-verify.json. Twenty-eight closed logs and
the matched record are imported into SQLite, read back byte for byte and removed
from loose storage. Final project formatting changes zero files and fingerprints.
Installed consumers remain at gate 181; scratch compilers are current.

B5 remains open: ordinary loops with no backedge and no completion boundary,
independent native phase layouts, broader aggregate/within-expression precision
and structural owned record keys remain pending. No timing is claimed. Next add a
focused ordinary no-backedge break/return witness before removing its eligibility
guard. Preserve the original failed evidence and the 15 unrelated fixture edits,
and commit only explicit paths.

## Ordinary loops without a backedge (2026-10-09)

The ordinary-loop eligibility guard incorrectly excluded loops whose bodies only
break, return or throw. The failing break witness now preserves the live second
entry while releasing the deleted first callback before capture mutation.
While, counted, post-test and for-of variants pass native execution. Return and
throw witnesses also verify exact callback disposal; those already passed before
the eligibility repair and are retained as regression coverage.

The variant batch exposed a second defect in post-test loops. A body without a
reachable tail can still break to the normal exit. Emission previously skipped
the exit projection, leaving a physical Map carrier incompatible with the next
borrowed iterator's checked record binding. The emitter now projects storage when
a break edge reaches that exit, while leaving return/throw-only tails untouched.
The original diagnosis of a for-of defect was incorrect; the failing generated
program was the post-test variant.

Focused acceptance passes 86 affected Map native tests, 14 editor release checks,
12 raw storage/completion proofs and all eight depth canaries. Matched support
adds noBackedgeEntryBreak and noBackedgeEntryReturn, independent arithmetic
oracles, mutation-rejecting source guards and two literal reference rows. All five
native/Node-after-tsc cases pass untimed with 77 observations per row. The canonical
record is runtime/results/2026-10-09T05-12-18-733Z-verify.json. Closed logs and this
record are imported into SQLite, read back byte for byte and removed from loose
storage. Whole-project formatting changes one workload and refreshes its verified
formatting fingerprint. Installed consumers remain at gate 181.

B5 remains incomplete. Next implement independent native phase layouts, then
broader aggregate and within-expression release precision. Structural owned
record Set/Map keys remain pending with the owner's Rust-style equality and
hashing decision. No full regressions or timing are claimed. Preserve the 15
unrelated fixture edits; use explicit Git paths. This is focused acceptance,
not the B5 feature-family milestone.

## Independent normal results after a catch (2026-10-09)

A new native witness showed that a try body and catch handler could both delete
the first entry, yet the joined native Map still retained its capture. Source
analysis correctly permitted a following mutation, but generated Rust rejected
it with E0502. Both paths now return the checked Handled/normal payload layout,
independent of the common storage carried by errors and pending controls.
The exit planner records that same layout for subsequent statements. Native
storage projection is factored through a leaf conversion helper so named Maps
and Maps in record fields use the same conversion.

This checkpoint consumes normal phase layouts for catch regions without finally.
It does not claim independent error/control layouts or cleanup-result emission.
Those remain the next phase-consumer work. Existing finally transport retains
its accepted implementation and lifetime behavior.

The focused witness passes successful and caught paths, mutation inside the
handler, nested record-field storage, and a throwing handler that retains both
entries until unwinding. Disposal is checked exactly at both optimization levels.
Editor checks accept released captures and reject both retained survivors and a
handler that omits deletion. Three failed field-test setup attempts are preserved:
the record needed a top-level named declaration and a field terminator. They were
witness setup defects, not accepted compiler behavior. Benchmark guard failures
from reused throw literals are also retained; the new helper uses a distinct
literal so the older exact-count guards keep their existing meaning.

Evidence: 87 affected Map native tests pass, supplemented by the repaired and
expanded focused witness. Fifteen editor release tests, one strengthened editor
negative check, 13 phase/storage proofs and eight depth canaries pass. Matched
support adds phaseJoinedEntry with successful and caught observations, independent
arithmetic oracles, literal reference rows and mutation rejection. All five
native/Node-after-tsc cases pass untimed with 79 observations per row. Canonical
record: runtime/results/2026-10-09T05-17-56-492Z-verify.json. Closed evidence is
imported into SQLite, read back byte for byte and removed from loose storage.
Project formatting changes the workload and refreshes its fingerprint after token
preservation; the final formatter pass changes zero files. Installed consumers
remain at gate 181. No full regression or timing claim is made.

Next continue independent phase layouts through finally and abrupt completions.
Then finish broader aggregate/within-expression precision and structural owned
record Set/Map keys with Rust-style equality and hashing. B5 remains open. Preserve
the 15 unrelated fixture edits and use explicit Git paths.

## Cleaned normal results after finally (2026-10-09)

The initial native witness already passed on the previous checkpoint. Its log is
named tsrust-b5-finally-result-red.log, but it is passing exploratory evidence,
not a runtime red. The focused storage-layout witness fails against the previous
implementation: following statements receive Map(6), while the checked cleaned
normal result is Map(7). The baseline sources were held only in process memory
while running that red witness and the working changes were restored afterward;
no archive or source snapshot was created.

The exit planner now selects Cleaned/normal for regions with finally and
Handled/normal otherwise. Both catch-plus-finally and finally-without-catch emit
normal cleanup results with the checked cleaned layout. Error and pending-control
cleanup continuations retain their existing common transport. This completes the
normal cleanup-result consumer, not independent abrupt layouts.

Native witnesses cover successful and caught bodies, no catch, propagated errors,
return from cleanup overriding pending completion, and Map storage in a nested
record field. Exact disposal is checked at both optimization levels. The affected
Map batch passes 88 tests, supplemented by expanded and record-field runs. All
16 editor entry-release checks, 14 phase/storage proofs and eight depth canaries
pass. The editor continues rejecting mutation of a surviving callback capture.

Matched support adds finallyNormalJoinedEntry with an independent arithmetic
oracle, a literal reference row and mutation-rejecting source guards. All five
native/Node-after-tsc cases pass untimed with 80 observations per row. Canonical
record: runtime/results/2026-10-09T05-24-29-449Z-verify.json. Closed logs and the
record are imported into SQLite, read back byte for byte and removed from loose
storage. Project formatting changes one workload and refreshes its fingerprint
after token preservation; the final formatter pass changes zero files. Installed
consumers remain at gate 181. No full regression or timing claim is made.

Next consume checked abrupt phase layouts. Start with error storage separated
from normal results; preserve per-operation availability and handler entry types.
Then handle return, break and continue storage by resolved destination, including
cleanup override and disposal. Do not combine independently checked payloads into
one storage tuple merely to satisfy the backend. Broader aggregate/within-expression
precision and Rust-style structural owned record Set/Map keys remain pending.
B5 is still open. Preserve the 15 unrelated fixture edits; use explicit Git paths.

## Body error storage and handler entry

Try/catch bodies without finally now transport checked Body/Throw Map layouts
independently from normal and control results. Explicit throw, fallible calls and
nested completion dispatch use per-operation error projections. Handler analysis
starts with the corresponding error layout. A separate emitter helper owns this
projection logic; no implicit clone is introduced.

The initial native witness already passed; the failing witness was the emitted
error envelope contract, which still used the common owner type. Focused checks
exposed clear/refill regressions. Clear-only regions now participate in storage
planning, exit conversion is based on actual Map operations, and eligible places
follow explicit moves into record fields. Treating every layout key as an emitted
storage conversion was incorrect for nested regions with no Map operations and
was removed. Original failures and repair runs are retained in SQLite.

All 89 affected Map native tests, 16 phase/storage proofs, 17 editor entry-release
checks and eight depth canaries pass. New native coverage checks direct throws,
fallible calls, nested finally, record fields and exact disposal at both
optimization levels. Existing moved-field clear/refill and borrowed get coverage
passes. The standalone borrowed-get type proof was positive coverage, not a red
witness. No complete regression or timing run is claimed.

Next separate errors through finally, outgoing handler errors and cleanup errors,
then return/break/continue storage by resolved destination. Rust-style structural
owned record Set/Map keys remain subsequent work. B5 remains open. Installed
consumers remain at gate 181; preserve the 15 unrelated fixture edits.

Matched verification exposed three borrowed-get mismatches in loop-local Maps.
A focused while-loop native witness reproduced the failure. Clear-only layouts
were bypassed by an early local-storage guard, and loops without carried Map
epochs only recursively planned for-of bodies. Those paths now plan while,
counted and post-test bodies as well. The new witness executes all three forms.
The original 89-test pass is preserved; the repaired batch includes this witness.

Final repair acceptance: 90 affected native tests pass, with the 16 storage
proofs, 17 editor checks and eight depth canaries rerun on the repaired source.
Matched native/Node-after-tsc verification passes all five cases, with 81
observations per row. Canonical record:
runtime/results/2026-10-09T05-44-53-954Z-verify.json. The errorPhaseEntry helper
has an independent arithmetic oracle, literal reference rows and deletion/mutation
source guards. Existing ABI, driver and report support applies unchanged.
Project formatting is token-preserving and idempotent; its final pass changes
zero files. Closed evidence is imported, read back byte for byte and removed
from loose storage. Focused acceptance is complete; family measurements and
complete regressions remain deferred until B5 is ready.

## Handler entry with finally

A finally clause no longer disables precise Body/Throw transport into a catch
handler. The handler's initial native Map type now matches the checked error
payload, and the body error envelope uses that layout. Normal cleanup, outgoing
handler errors and pending controls retain their existing independent paths.

The new failing phase proof exposed the old common Map type at handler entry.
Native coverage adds finally observation to the existing successful/caught
scenario and verifies exact callback disposal at both optimization levels.
Editor coverage checks successful analysis, rejection of surviving-capture
mutation and rejection when failure can precede deletion, with and without
finally. Matched support adds finallyErrorEntry, an independent arithmetic
oracle and mutation-rejecting source guards. The first registration attempt
found the old full observation row contract; it was updated for the added row
element. This is a support-contract repair, not a native semantic defect.

This chunk covers body errors entering a catch when finally follows it. Errors
propagating through finally without a catch, outgoing handler/cleanup errors and
return/break/continue layouts remain pending. B5 remains open; no family timing
or complete regression run is claimed.

The affected native batch initially passed 89 cases and failed the existing
outer_map_catch_loop_releases_iteration_captures witness. The no-carried-epoch
loop path dropped inherited outer Map eligibility, so a handler refill could
retain the narrowed input family rather than add its new callback origin.
Preserving eligible outer places repairs the witness, including record-field
storage and exact disposal. An exploratory contextual-carrier change did not
repair the defect and was removed; no general carrier conversion was changed.
The initial matched run exposed the same failure during compiler emission.
Both original failures and focused repair evidence are retained.

The next affected batch passed 89 cases and exposed one post-test direct-break
regression: an outer Map was narrowed both inside the body and at the checked
loop completion, moving the same owner twice. The fallback now retains inherited
places except when a consistent releasing loop completion already owns their
conversion. The post-test witness passes again. Matched verification on the
intermediate repair passed and is retained as an intermediate record; it does
not replace the affected native failure or final-source verification.

Final acceptance: 90 affected Map native tests, 17 phase/storage proofs, 17
editor entry-release checks and eight depth canaries pass on the final source.
The full native batch includes the newly expanded finally/disposal witness.
Matched native/Node-after-tsc verification passes all five cases with 82
observations per row. Canonical final record:
runtime/results/2026-10-09T05-57-32-074Z-verify.json. The earlier passing
intermediate record is runtime/results/2026-10-09T05-54-22-483Z-verify.json.
Closed logs and both records are imported into SQLite, read back byte for byte
and removed from loose storage. Project formatting is token-preserving and
idempotent; the final formatter pass changes zero files. Installed consumers
remain at gate 181. No timing or whole-regression claim is made.

Next consume Body/Throw storage through finally without a catch, then outgoing
handler and cleanup errors. Cleanup entry must respect the pending completion's
Map layout; normal results, error results and resolved control destinations must
not be forced into one common tuple. Preserve disposal order, replacement and
cleanup override. Structural owned record keys and the wider B5 family are
still pending.

## Independent cleanup entry for pending errors

Finally without a catch now carries Body/Throw Map storage separately into
cleanup. A dedicated map_cleanup_phase_plan module plans the cleanup AST and
native snapshots from the checked Handled/Throw input, which equals the body
error input when no handler runs. Only the error-entry plan is materialized and
consumed in this chunk. The plan interface retains the completion identity for
later normal/control consumers. Native planning snapshots are reset before
replanning; source ownership facts remain intact. Plans share their registry
through Arc when contexts are cloned, and store binding-type differences rather
than copies of every binding's metadata. Nested plans remain separate.

The failing witness initially found no error-entry cleanup plan. The proof now
checks its input type and emitted FinallyIncoming error envelope. Native coverage
checks normal and thrown body outcomes, cleanup iteration, cleanup-time mutation
of a released capture, record fields and exact callback disposal at both
optimization levels. Editor coverage rejects mutation of a surviving capture
and rejects a body error that can precede deletion.

The first native run exposed conversion into an absent callable environment.
An empty Map environment has no payload to convert: changing its representation
now drops the old empty owner and constructs the checked empty destination.
Empty environments are retained in native storage snapshots, preventing fallback
to a stale nonempty type. Matched verification exposed this snapshot omission
in the new nested outer-catch helper.

The first affected batch passed 90 cases and failed the existing cleanup-deletion
witness because the pending completion enum kept a capture loan alive. Finally
now separates normal, error and control owners into independently typed options
inside a completed enum scope. Exactly one option is live; every option is
fully consumed or dropped before cleanup runs. The owner tuple is destructured
directly, avoiding a residual tuple temporary that can retain a loan. This
preserves phase types instead of merging them into one common storage tuple.
Original failures and repair runs are preserved in SQLite. A mutation variant's
initial source substitution missed the formatter's `} finally {` spelling; the
assertion caught the no-op, and the witness now executes the intended source.

Cleanup outgoing errors, resumed pending-error storage after cleanup and distinct
return/break/continue result layouts still use their existing transport and
remain pending. This acceptance concerns precise body-error transport into
cleanup; it does not claim all abrupt phases complete. B5 remains open.

Final acceptance: 91 affected Map native tests pass, supplemented by the expanded
cleanup-mutation and record-field witness. All 18 phase/storage proofs, 18 editor
entry-release checks and eight depth canaries pass. Matched native/Node-after-tsc
verification passes all five cases, with 83 observations per row. The new
finallyPendingError helper includes a nested outer catch, cleanup iteration and
clear, normal-only work after cleanup, and a distinct caught-error observation.
Its oracle is independent arithmetic; literal reference rows and mutation guards
are registered. Existing ABI/driver/oracle/run/profile/report hooks apply.
Canonical record: runtime/results/2026-10-09T06-12-33-163Z-verify.json. Closed
logs and the record are imported into SQLite, restored byte for byte and removed
from loose storage. Project formatting preserves tokens and is idempotent; the
final pass changes zero files. Installed consumers remain at gate 181. No full
regression or timing claim is made.

Next consume checked outgoing handler/cleanup error layouts and the cleaned
pending-error result. The new error-entry Plan supplies phase-local native
metadata; do not fall back to the joined cleanup AST for that path. Extend the
independent owner transport to resolved return/break/continue destinations, with
cleanup override and disposal witnesses. Structural owned record keys remain
subsequent work. B5 is not complete.


### Handler rethrow storage

A catch body that removes a known Map entry and then throws carries its checked
survivor layout to the enclosing handler. Error forwarding projects that layout
into the enclosing error destination instead of reusing the ordinary statement
input layout. Native transport retains an initialized owner independently of
source-name availability; forwarding does not make a transferred source binding
usable again. The error dispatch helper accepts only an already selected Throw
completion, so unrelated return and loop-control storage is not converted.

Catch entry consumes the pending completion into independent normal, error and
control owners before selecting a branch. This matters for owners with disposal:
an inactive completion alternative must not retain the deleted entry's capture
loan while the outer catch mutates that capture. Native witnesses cover direct
Map bindings and record fields, survivor invocation, released-capture mutation
and exactly-once disposal at both optimization levels. Editor checks accept the
released capture and reject mutation of the surviving capture.

Focused evidence: 92 affected Map tests and 19 storage proofs pass. The expanded
record-field/disposal witness and isolated error dispatch pass separately. The
new editor diagnostic witness passes. No analyzer/parser dispatch changed, so
this chunk does not repeat the depth canaries or run a family regression.
The matched workload adds handlerRethrowEntry with independent arithmetic
2 * value + 3, literal reference rows and mutation guards; each row now has 84
observations. Native and Node-after-tsc verification is untimed. Canonical final record:
runtime/results/2026-10-09T06-23-10-157Z-verify.json. All five cases pass on both
engines. Closed logs and both matched records are imported into SQLite, restored
byte for byte and removed from loose storage. Project formatting changes zero
files on its final pass; diff checks pass.

Still pending: handler errors with finally, cleanup errors, resumed pending
errors after cleanup and distinct return/break/continue destination layouts.
Structural owned record keys follow those capture-release repairs. B5 remains
open; installed consumers remain at gate 181.


### Cleanup outgoing error transport without catch

The no-catch finally emitter now consumes the checked Cleaned/Throw Map layout
for cleanup errors and for the normal completion that resumes a pending error.
It attaches that layout to each cleanup error envelope and projects the selected
outgoing owner into the enclosing error destination. Return and loop-control
owners retain their separate existing representation.

The original execution witness already passed on the previous checkpoint. A
stronger native-layout witness first missed the specialized path; adding a
released-capture write activates it and proves the actual defect: checked Map7
was widened to Map6 in the cleanup error envelope. That focused witness fails on
the previous emitter and passes with this change. Runtime witnesses cover resumed
and replacement errors, released and surviving captures, record-field storage
and exactly-once disposal. Editor diagnostics retain surviving-capture conflicts.

The matched cleanupOutgoingEntry helper observes both error identity and the
surviving callback. Its independent result is 2 * value + 30, plus one when
cleanup replaces the pending error. The workload now observes 85 values per row;
literal reference rows and mutation guards cover the added behavior. Timing
remains deferred. Final checks and the canonical record are recorded below.

This slice concerns finally without its own catch. Catch-plus-finally still uses
its joined pending owner path and needs separate outgoing handler/cleanup plans.
Distinct return/break/continue layouts remain pending. B5 remains open, followed
by Rust-style structural owned record keys. Installed consumers remain at gate 181.


The affected Map batch completed with 92 passes and one loop-cleanup failure.
The failure exposed unrelated return/control arms being generated for an already
selected error when the enclosing function had no carried Map owner. Selected
errors now use the error-only dispatch even without an outer storage tuple. The
failed loop witness passes after repair; passed batch evidence is retained rather
than rerunning the entire group. All 20 phase/storage proofs, the new editor
witness and expanded record-field/disposal checks pass. Parser/analyzer dispatch
is unchanged; no depth, full-family or timing run is claimed.


Final matched verification passes all five cases on native and Node-after-tsc,
with 85 observations per row. Canonical record:
runtime/results/2026-10-09T06-30-32-665Z-verify.json. Original failed checks,
repair checks, final checks and both matched records are preserved in SQLite,
restored byte for byte and removed from loose storage. Final project formatting
changes zero files and fingerprints. Source diff checks pass.

Next: consume independent handler/cleanup error plans in catch-plus-finally,
then destination-specific return/break/continue owners and cleanup overrides.
Keep those phases separate rather than widening all pending owners together.


### Independent catch-plus-finally owners

Catch-plus-finally now has a dedicated emitter module,
map_catch_finally_storage.rs. Body completion is consumed before entering catch;
handler completion is then consumed into independent normal, error and control
owners before entering cleanup. Errors use the checked Handled/Throw layout and
the existing phase-local cleanup plan. Cleanup failures and resumed errors use
the checked Cleaned/Throw layout. Normal and control owners remain separate;
inactive alternatives cannot retain a deleted entry's capture borrow.

Two baseline witnesses fail before this implementation: the handler error
transport widens checked Map7 back to Map6, and generated Rust rejects a capture
write in cleanup after catch removes the only entry borrowing that capture.
Both pass with independent owners. The former joined-owner/clear-all special
case and unused dispatch wrappers are removed, reducing shared exception code.

Focused execution covers normal body completion, handled failure, handler
rethrow, cleanup replacement, surviving callback invocation, direct Map bindings
and record fields. Disposal is checked exactly once at both optimization levels.
Editor diagnostics accept released-capture writes and still reject writes to a
surviving capture. The affected Map batch has 94 passes; an additional expanded
completion-mode witness passes separately. All 21 phase/storage proofs pass.
No parser or analyzer dispatch changes; no depth/full-family/timing run is claimed.

The matched catchFinallyEntry helper covers all four completion modes. Independent
results are 2 * value + 3, +4, +43 and +44 respectively. Literal reference rows and
mutation guards include each mode, handler deletion, rethrow and cleanup override.
Each workload row now contains 89 observations. Final verification and SQLite
record details follow below when the run closes.

Still pending: destination-specific return/break/continue storage, cleanup
control overrides and remaining aggregate/expression precision. Rust-style
structural owned record keys follow the capture-release work. B5 remains open;
installed executable consumers remain at gate 181.


Final matched verification passes all five cases on native and Node-after-tsc,
with 89 observations per row. Canonical record:
runtime/results/2026-10-09T06-37-22-450Z-verify.json. Closed logs and the matched
record are stored in SQLite, restored byte for byte and removed from loose
storage. Final project formatting changes zero files and fingerprints; source
diff checks pass. The current compiler build has no warnings.

Next: resolve return/break/continue owners by their actual destination, preserve
cleanup overrides and disposal, then complete the remaining B5 aggregate and
expression witnesses before the family regression/performance review. Structural
owned record keys retain the accepted Rust-style value semantics.


### Control cleanup planning foundation

The checker already distinguishes Return, Break(target) and Continue(target) in
its exception snapshots. Cleanup planning previously materialized only Throw.
It now materializes a separate native cleanup plan for every abrupt Handled
completion, preserving the complete resolved target in the existing plan key.
Two focused baseline witnesses fail because those plans are absent. They pass
after the change: a narrowed return owner seeds cleanup correctly, and two
separate break destinations plus one continue destination retain their respective
Map survivors. The destination witnesses deliberately use different surviving
callback origins, so a merged plan cannot satisfy the assertions.

This is planning infrastructure, not acceptance of precise native control
transport. Existing emitted return/break/continue carriers still use common
storage. The next implementation must consume the plans without re-widening
owners and must preserve pending return values and cleanup overrides.

The remaining transport design is explicit:

1. Generate a control carrier with a separate typed storage payload per resolved
   completion destination. Use generic payload parameters so local enum
   declarations do not capture anonymous closure lifetimes.
2. Attach each checked phase layout to return and control emission, as error
   emission already does. Preserve source-name availability independently of
   initialized native owners; never revive a transferred source binding.
3. Fully consume the carrier before entering cleanup. Select the plan by the
   exact Return/Break/Continue kind and target, then emit its phase-local body and
   native metadata. Do not choose a plan only by the general control category.
4. Project cleaned storage into the actual enclosing destination. Local loop and
   labeled-block destinations must receive their checked carried layout; an
   enclosing finally receives control before a local destination is resumed.
5. Verify returned owned values, both loop control kinds, multiple labels, nested
   cleanup, overriding return/error/control and exactly-once disposal. Add matched
   observations and editor diagnostics with each observable behavior change.

Control transport remains open. No new runtime capability or performance result
is claimed by this planning checkpoint. Its checks are recorded below when closed.


Bounded verification passes: 95 affected Map native tests, 23 phase/storage
proofs, 21 editor entry-release checks and all eight depth canaries. Project
formatting changes zero files and fingerprints; diff checks pass. Existing
matched workload support is unchanged at 89 observations per row. A focused
untimed native/Node-after-tsc verification checks the current compiler; the final
record is recorded below after the run closes. No new timing is performed.


Final matched verification passes all five cases on both engines. Canonical record:
runtime/results/2026-10-09T06-43-43-721Z-verify.json. Original red witnesses,
closed checks and the matched record are preserved in SQLite, restored byte for
byte and removed from loose storage. Source acceptance concerns control cleanup
planning only; the destination-specific native carrier remains the immediate
next implementation. B5 and the broader compiler completion plan remain open.
