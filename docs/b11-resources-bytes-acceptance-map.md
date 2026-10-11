# B11/S07 resources and bytes acceptance map

This is the current implementation map. S07 is not accepted or measured.

| Scope | Status | Authoritative evidence |
| --- | --- | --- |
| Typed native declarations, signatures, imports, aliases and Result payloads | Gate 187 accepted | milestones/187-native-byte-foundation.md |
| Owned byte construction, access, mutation, allocation, growth and range copying | Gate 188 accepted | milestones/188-owned-byte-operations.md; native-owned-bytes fixtures |
| Independent clone, generic/record/array/Map/nullable/Result storage and callable captures | Gate 188 accepted | ownership.tsr; native_declarations.rs; project_intelligence.rs |
| Validation/reservation atomicity and one argument evaluation | Gate 188 accepted | bytes/runtime.rs tests; access.tsr; growth.tsr; copy-append.tsr; ownership.tsr |
| Exact disposal and allocation restoration at O0/O3 | Gate 188 accepted | native_byte_cleanup.rs; cleanup.tsr; exact SQLite gate 188 records |
| Scoped zero-copy readonly/exclusive views, provenance, conflicts and escape diagnostics | Gate 189 accepted with focused evidence; family regression/measurement pending | native-byte-views-implementation.md; resources-bytes-plan.md |
| Strict UTF-8 encoding/decoding and error offsets | Gate 189 accepted with focused evidence | native-owned-bytes/utf8.tsr; utf8_runtime.rs; resources-bytes-plan.md |
| Owned returns through finally, including renewed Map callbacks | Whole/partial returns, nested Map error fields, catch/loop disposal, split callback records and owning captures have focused acceptance; family regression remains pending | exception_return_moves.rs; finally_move_slots.rs; final continuation entry below |
| Opaque resource consuming close, drop and failure ownership | Gate 190 pending | owner consuming-close decision; resources-bytes-plan.md |
| Useful synchronous file proof and private instrumented resource backend | Gate 190 pending | resources-bytes-plan.md |
| Complete regressions, matched measurements and optimization review | Family milestone pending | resources-bytes-plan.md; standard-library-roadmap.md |

All current byte workloads have matching sources, independent oracles, source hashes,
ABI support and harness hooks. Untimed verification is distinct from measurement.
Native thread-transfer metadata is backend eligibility, not source threading acceptance.
The owner chose consuming close even on Err, static rejection of later use/double
close, and a nonthrowing destructor fallback. Bytes does not require explicit close.

The progress entries below preserve earlier checkpoint evidence; the table above
and the final continuation entries describe current status.

### Gate 189 private storage progress

The private native view storage checkpoint emits real slice lifetimes through
nonrecursive records, arrays and Result wrappers. Focused storage/native execution,
scoped-record regression and native editor checks pass (48 tests). Source loans,
returned lifetime inference, recursive wrappers, source factories and their benchmark
support remain pending. This is implementation progress, not gate acceptance or
measurement. See handoff-2026-10-10-native-view-storage.md and the SQLite record
runtime/results/native-view-storage-2026-10-10-checks.json.

### Gate 189 retained loan progress

Native slice loans now integrate with analyzer binding initialization, whole-binding
replacement and ownership access checks through a dedicated module. Seven private
loan checks, 89 tests across ten depth-canary suites and 14 native editor checks pass.
Conditional-source validation also rejects temporary alternatives. No public factory
is enabled. Branch joins, Result-path release, precise projections, source escapes
and lifetime inference remain pending. This is not gate acceptance or measurement.
See handoff-2026-10-10-native-view-loans.md and SQLite record
runtime/results/native-view-loans-2026-10-10-checks.json.

### Gate 189 loan join progress

Native view provenance and retained state now participate in Availability restoration
and joins. Transfers on one path cannot release another reaching path's loan; two
transferred paths keep the old holder dormant. Eleven focused loan checks pass,
including parsed if replacements with private loans injected into the analyzer.
Ten depth-canary suites pass 89 tests with no failures or aborts; 14 native editor
checks pass. Project formatting changes zero files. Result-path release, precise
fields, return lifetimes and public factories remain pending. No timing ran.
See handoff-2026-10-10-native-view-branches.md and SQLite record
runtime/results/native-view-branches-2026-10-10-checks.json.

### Gate 189 Result and value projection progress

Native loan storage paths and nominal variant guards now distinguish success payloads
from Err and unrelated record siblings. Parsed, formatted match bodies in private
analyzer contexts permit owner replacement in Err and reject it in Ok. Seventeen loan
checks, 89 depth-canary tests and 14 native editor checks pass, with no failures or
aborted targets. Formatting changes zero files. Owner field conflicts, projected
moves/replacement, escape checking, return lifetimes and public factories remain
pending. No timing ran. See handoff-2026-10-10-native-view-result-paths.md and SQLite
record runtime/results/native-view-result-paths-2026-10-10-checks.json.

### Gate 189 owner place and record lifecycle progress

Loans distinguish owner field paths from holder storage paths. Disjoint owner fields
remain usable; overlapping fields and whole-owner replacement conflict. Projected
record moves deactivate only their old holder origins, and direct field replacement
installs new origins without discarding sibling loans. Conditional sources retain
distinct fields of one owner. Twenty-three private loan checks, 89 depth-canary tests
and 14 native editor checks pass, without failures or aborts. Formatting changes zero
files. No timing ran. Return lifetimes, escapes, indexed collection transport and public
factories remain pending. See handoff-2026-10-10-native-view-owner-places.md and SQLite
record runtime/results/native-view-owner-fields-2026-10-10-checks.json.


### Current gate 189 integration checkpoint

Private integration now includes owner fields and record lifecycle, returned source
escape checks, carried input slots, named Rust function lifetimes, direct/saved named
function calls, ordinary arrow invocation and consuming native view captures.
Shared owner captures also retain external storage loans and named reference
lifetimes in private provenance/signature controls; source factory integration for
that path remains pending. Captured Results retain named payload lifetimes, including independent view lifetimes
in Ok and Err. Read-only callbacks can inspect captured Result variants, and closure
projection preserves explicit ReadOnly facts. Generated O0/O3 witnesses preserve
pointer identity and exclusive write-through. Alternative inputs and independent
record fields retain separate lifetimes. This supersedes the pending lists in older
checkpoint paragraphs, which retain their original historical scope.

Receiver-owned and mutable capture storage now have private caller-loan and named
receiver-signature controls, including mutation/move/escape conflicts. Complete
source scope/drop and factory-body integration for these contracts remain pending.

Public views remain guarded. Receiver scope/drop enforcement and source integration,
escaped/rebound environments, specialized callbacks, arbitrary stored callable
families, recursive carriers, collection transport
and exceptional completion reconciliation remain pending. Factory/editor/benchmark
acceptance and the S07 family review are still required. See
handoff-2026-10-10-native-view-receiver-contracts.md and
native-byte-views-implementation.md.

### Gate 189 source storage matrix status (2026-10-10, after `ff481c9c`)

Gate 189 and S07 remain open. Public `view`/`viewMut` are still guarded. The private
source matrix for views in owning callable storage is now complete except for two
diagnosed forms. Every supported row runs natively at two optimization levels for
readonly and exclusive views, in
`stdlib/bytes/views_stored_emit_tests.rs`.

| Form | Status |
| --- | --- |
| Field move, saved move, single-element array, present nullable field, nested concrete capture, conditional field, returned-record alternatives | supported (earlier checkpoints) |
| Arrays with different callback alternatives: literal, `push`, elements moved between arrays | supported |
| Absent nullable values; absent to present and back; conditional callback or `undefined` | supported |
| Capture of an already stored scoped family, also into a new stored callback | supported |
| Whole-record replacement in the declaring block (new lifetime) and in blocks, branches and loops (joined inputs) | supported |
| Field replacement, including a new alternative in a branch or loop | supported; the result depends on the earlier and the new input |
| Reusable invocation through `&self` and `&mut self` | supported |
| Arguments that own or borrow data; optional, default and rest parameters | supported |
| Results of ordinary types or `void` | supported |
| Throwing callbacks, caught and propagated; mixed throwing and non-throwing alternatives | supported |
| A callback result that borrows an invocation argument | diagnosed (`native-view-source`) |
| A type that contains itself and holds a byte view | diagnosed (`native-view-shape`) |

Not started: public factories, shared readable and mutable observer contracts
(`length`, `at`, `copy`, `toUtf8`, `set` for views), IntelliSense, public fixtures,
matched subarray workloads, and the S07 family run and measurements.

Checks at `ff481c9c`: whole compiler crate 2,924 passed, 0 failed, 4 ignored, no
aborted target; whole editor crate 484 passed, 0 failed, 1 ignored. No timing ran.
Consumers remain at `f569190a`.

### Gate 189 publication status (2026-10-10, later the same day)

Public byte views are published. Gate 189 stays open for recursive stored-callable fields and capturing callbacks
returning argument views. S07 also needs gate 190,
the finally-return repair and family measurements.

| Scope | Status | Evidence |
| --- | --- | --- |
| `ByteView`, `ByteViewMut`, `view`, `viewMut` exported from `std:bytes` | published | `tests/std_bytes.rs`, fixtures `native-byte-views/{read,values,callbacks}.tsr` |
| Shared observers: `length`, `at`, `copy`, `toUtf8`, `view` on `Bytes`, `ByteView`, `ByteViewMut`; `set`, `viewMut` on `Bytes`, `ByteViewMut` | published; typed alternatives in `NativeBinding::argument_alternatives`, shared readable and writable byte traits in the bridges | same fixtures; `stdlib/bytes/emit.rs` |
| Ownership diagnostics through the CLI: escape, owner write, resize, move, read under an exclusive view, write through a readonly view, clone, construction | nine cases | `byte_view_ownership_rules_are_source_diagnostics` |
| IntelliSense: catalog entries with executable examples, hover, signature help with alternatives, import completion, conflict diagnostics | complete for the published surface | `project_intelligence.rs`, `project_diagnostics.rs`, `builtins.json` |
| A callback that returns a view it received as an argument, stored or local | supported, including owned view captures | fixture `callbacks.tsr`; `native_view_capturing_argument_matrix` |
| The same, when the callback also captures a view | supported with precise argument origins and intersections for mixed returns | `native_view_capturing_callback_returns_its_argument`; `native_view_capturing_argument_matrix` |
| Matched workload `native_byte_views` (`Uint8Array.subarray`) | registered; native and Node-after-tsc agree untimed; not measured | `native-byte-views-*.mjs`; SQLite `runtime/results/2026-10-10T23-27-13-709Z-verify.json` |
| A type that contains itself and holds a view (`Chunk { data: ByteView; next?: Chunk }`) | implemented with focused source/native/editor checks; recursive stored-callable fields remain guarded | `recursive-view-storage-plan.md` |

A stored function value (`const read = length;`) keeps the declared parameter type
`Bytes`; the alternatives apply to direct calls of the import.

Checks: whole compiler crate 2,926 passed, 0 failed, 4 ignored; whole editor crate 486
passed, 0 failed, 1 ignored; no aborted target. No timing ran. Consumers remain at
`f569190a`, so installed tools do not have views yet.

### Approved continuation after consumer refresh

This section supersedes older pending lists without changing their historical evidence.
The owner approved nullable recursive links with unique ownership and one shared
lifetime for recursive records holding byte views. Both recursive storage and the
view-capturing callback that returns an argument view remain gate 189 work. Gate 190
and S07 family measurements remain pending. The owned-value return through finally
repair precedes resource cleanup acceptance.

The five workspace consumers, global CLI and local VS Code extension were refreshed
from `4367f4cc` and smoke-tested. SQLite record:
`target/editor/consumer-refresh-2026-10-10.json`. No timing ran. Subsequent recursive
lifetime groundwork is tracked in `recursive-view-storage-plan.md`; it does not
claim public recursive-storage acceptance.


### Recursive array construction checkpoint

Array-recursive readonly/exclusive view records now support source construction,
replacement, append, extraction, whole-record borrowing, aggregate returns and
owning callback captures. All contributing owners share the recursive lifetime;
outer sibling fields remain independent. The public `recursive-arrays.tsr` fixture,
five owner/escape rejection controls, O0/O3 pointer and write-through checks, and
focused editor checks pass. Detailed evidence and repair history are recorded in
`recursive-view-storage-plan.md`.

The matched `native_byte_views_recursive` workload agrees with Node-after-tsc on
five untimed cases. SQLite result:
`runtime/results/2026-10-11T00-30-34-645Z-verify.json`. Workload source acceptance is
true; gate/family acceptance and measured status remain false. Nullable links are
next, followed by capturing callbacks returning view arguments and the finally fix.
Consumers remain at `4367f4cc`.


### Nullable recursive link checkpoint

Nullable recursive records now use unique owning links with iterative clone/drop,
including mutual cycles and byte-view lifetimes. Public fixtures, six ownership
rejections, editor diagnostics and the source/native matrix pass. The new
`native_byte_views_linked` workload agrees with Node-after-tsc on five untimed cases;
SQLite preserves `runtime/results/2026-10-11T01-01-09-264Z-verify.json`.
See `recursive-records.md` for behavior and `recursive-view-storage-plan.md` for evidence.

Recursive stored-callable fields still hit the existing `callable-layout` diagnostic;
this is pending callback integration, alongside capturing callbacks returning argument
views. Gate 189 and S07 remain open. Finally-return repair, resources, family
measurements and the final consumer refresh remain pending. Consumers stay at
`4367f4cc`.


### Capturing callbacks returning argument views

Scoped callback families now give invocation arguments their own named lifetimes,
separate from capture lifetimes. An argument-only return follows the argument owner;
an unused capture does not shorten it. A result selected from a capture or argument
requires both owners. This uses real slices and retains the existing owning capture
requirements. Views remain non-clonable.

Focused evidence: 18 stored-view tests pass, including readonly/exclusive O0/O3
execution, pointer identity, write-through, selected-owner escape controls, reusable
read/mutate callbacks, later and nested arguments, throwing calls and alternative
bodies. All five public view fixtures pass through CLI; editor acceptance and
related-location owner-escape diagnostics pass. The new matched workload
`native_byte_views_callbacks` agrees with Node-after-tsc on five untimed cases:
SQLite `runtime/results/2026-10-11T01-22-59-785Z-verify.json`. No timing ran.

The benchmark oracle dispatch also replaces an accidental comma expression with
explicit workload membership. A non-view byte workload control prevents recurrence.
No existing timing records were rewritten.

A separate probe returning a new view of a borrowed `Bytes` callback argument still
reports `native-view-source`. Its original source and diagnostic are retained as
`tsrust-view-argument-owner-probe.tsr` and `tsrust-view-argument-public.log` in SQLite.
Investigate the `ClosureForward::Once` adapter's missing return summary and borrowed
argument lifetime in trait emission. This is distinct from returning a view argument.
Recursive records with stored callbacks, the owned return-through-finally repair,
resource cleanup acceptance and S07 measurements remain open. Installed consumers
remain at `4367f4cc` until the next approved refresh.


### Callback borrows of owner arguments

A legal equivalent of the earlier borrowed-owner probe, with an explicitly moved
view capture, reached native emission and exposed an undeclared result lifetime.
Trait adapters now map returned storage loans to their actual argument borrow
lifetime. Local and stored callbacks support readonly and mutable `Bytes` arguments
through named view-producing helpers. `callback-owners.tsr` verifies observed bytes,
write-through into the original owner and source rejection of a local-owner escape.
Five focused native view controls and the corresponding editor check pass.

The callback workload now exercises both an existing view argument and a view
created from a borrowed owner argument during invocation. Both pairs of length/byte
observations agree with Node-after-tsc on five untimed cases; SQLite record
`runtime/results/2026-10-11T01-29-57-686Z-verify.json`. The earlier workload record is
preserved unchanged. No timing ran.

The two adjacent source-diagnostic boundaries are now repaired. A terminal `if`
(including lowered `match`) retains incoming native-view slots for capture layout
when both arms exit. Temporary record field calls select the actual checked family,
so its returned-owner and error summaries are available. Direct `viewMut` in a
stored callback and `saveRead(captured).pick(owner)` both execute; escaping the
borrowed owner remains a source diagnostic.

Evidence: all 15 public byte tests, the extended readonly/exclusive O0/O3 callback
matrix, focused editor owner-escape test and four workload registration checks pass.
The callback workload now calls a native view constructor directly and invokes the
owner callback through a temporary returned record. Its five native/Node-after-tsc
untimed cases agree: SQLite
`runtime/results/2026-10-11T01-38-19-462Z-verify.json`. Original red probes and previous
result records remain intact. No timing ran. Recursive stored callbacks, finally,
resources acceptance and S07 measurements remain pending.


### Owned returns through finally: whole-owner slots

The original ignored `return a` failure now executes. Checked transfer snapshots
select owners whose normal path uses them after cleanup. Native stack slots track
whether those values remain initialized; no implicit clone or heap allocation is
introduced. Normal field reads, array mutation, local/deferred initialization and
assignment preserve their original behavior. Calls and record literals may transfer
owners inside return expressions. Nested cleanup and closure bodies keep their
return targets, including a normal path that captures the owner after cleanup.
Borrowed return-call arguments remain ordinary borrowed accesses.

Focused checks cover the original failure, containers, replacement returns, cleanup
throws, nested regions, closure parameters/captures and a borrowing control. Native
O0/O3 instrumentation proves returned allocation identity and exact drop ordering:
cleanup sees a live pending owner; a replacement discards it once. Readonly and
mutable byte-view returns keep pointer identity, mutable write-through and owner
escape rejection. Editor hover and transferred-value diagnostics remain consistent.
The formatter corpus verifies tokens, idempotence and formatted-source contracts.

The matched `exception_completion` workload now also returns an owned string on
both sides of a conditional return through cleanup. Both paths append an observed
cleanup marker. Independent oracle/source guards retain complete result checks and
the original timing protocol; no timing was run during this checkpoint.

One adjacent failure remains: returning an owned record **field** through finally
and using that field on the normal path still produces native E0382. Whole-owner
slots deliberately do not move/drop the record's other fields. Exact source, native
output and diagnostic are in SQLite as `tsrust-finally-partial-owner-probe.tsr`,
`.rs` and `.log`. Finish precise partial-field state before closing the finally
repair. Recursive stored callbacks, resources and S07 measurements also remain open.
Installed consumers remain at `4367f4cc`.

Nine focused return tests pass across retained runs, plus 27 neighboring exception
tests, four native/source cleanup checks, one editor check and three registration
checks. Five native/Node-after-tsc cases agree: SQLite record
`runtime/results/2026-10-11T02-06-29-505Z-verify.json` has an empty execution array.
Exact focused logs are stored with the `tsrust-finally-slots-` prefix. Historical
exception workload records remain unchanged; the new source fingerprints identify
the added owned-return behavior.


### Partial-field returns through finally: direct cleanup routes

The retained partial-field E0382 probe now executes. Plain completion frames run
cleanup at the return site and leave directly, preserving Rust's initialized-field
state. An enclosing catch still receives cleanup errors at its proper boundary.
Nested finally blocks share one typed abrupt route for return and throw; cleanup
does not bypass enclosing cleanup. Frames carrying specialized storage keep their
existing transport. No placeholder field values, whole-record extraction, implicit
clones or unsafe code are introduced.

Checks cover early/normal field returns, nested fields, sibling reads in cleanup,
catch returns, cleanup throws, replacement returns, field restoration, outer break,
continue after restoration and closure capture on the normal path. A cleanup's
internal loop does not become an escaping control destination. Only checked escaping
control targets are emitted: adding hypothetical continue paths would make Rust
consider a moved field live on another iteration. The public `owned-returns.tsr`
fixture uses std:testing assertions and runs through the project CLI. The editor
retains the `partial-transfer` diagnostic for a moved field accessed during cleanup.

O0/O3 native probes prove allocation identity, sibling liveness during cleanup,
and exactly one disposal per field on ordinary and replaced returns. Existing whole
owner and readonly/exclusive view probes still pass. Thirty focused finally unit
checks pass, plus the 38-test exception integration batch and the added fixture and
restoration/control cases. Formatter corpus checks pass. Original failed records
and repaired runs are retained under `tsrust-finally-fields-` in SQLite.

The first inline prototype duplicated return/error alternatives at every nesting
level: six and ten finally levels emitted 134,760 and 1,103,220 bytes. Sharing the
abrupt route reduced these to 77,293 and 86,365 bytes. These are generated-source
sizes, not runtime performance measurements; a bounded growth test guards this.
The matched exception workload now exercises both whole-owner and record-field
returns and observes the retained sibling during cleanup. Timing remains pending.

A separate integration probe uncovered a pre-existing Map callback carrier panic:
`unchecked record carrier conversion Closure(1) to Closure(0)`. A Map of callback
records is replaced between a return path and a normal path before finally reads
it. The previous release compiler reproduces the same panic. Exact source and both
logs are in SQLite as `tsrust-finally-fields-map-probe.tsr`,
`tsrust-finally-fields-map-probe.log` and `tsrust-finally-fields-map-baseline.log`.
Resolve this storage-carrying integration case before closing finally/resource
acceptance. Recursive stored callbacks, gate 190 and S07 measurements remain open.

Five native/Node-after-tsc cases agree for the expanded workload: SQLite
`runtime/results/2026-10-11T02-26-03-593Z-verify.json`. Execution is empty; timing
remains pending. The rebuilt release compiler also executes the assertion fixture.


### Map callback owners across finally: precise direct return transport

The retained Map carrier panic is repaired. Return projections now use only the
source points belonging to their phase. Cleanup without a source return or escaping
control no longer manufactures a return carrier from the pending return's layout.
Disjoint concrete callback origins are excluded in narrowed Map projections, just
like excluded variants of a joined callback family; environments are never converted
into another origin. Direct return transport consumes the precise Map owner before
cleanup and preserves partial-field move state through enclosing plain cleanup.
Return/error dispatch honors the enclosing cleanup route, and cleanup with no source
control exit does not emit hypothetical break/continue paths.

The public owned-return fixture now checks callback invocation on early and normal
paths. Focused tests cover catch returns, a third callback installed by replacement
cleanup, Map storage declared inside an outer try, allocation identity and exactly
one disposal per callback environment at O0/O3. The final exception integration
batch passes 44 tests and the scoped-record finally batch passes 17. The editor
accepts the valid normal-path capture release and still rejects mutation of the
current capture. The matched exception workload includes both Map callback paths,
with independent expected traces ending in 4 or 8; measurement remains pending.

A new native probe exposed an earlier direct-cleanup regression: an owned local
created inside try is disposed before finally on normal exit, but survives into
finally on early return. This is separate from returned-payload and sibling liveness,
which pass. The ignored test
`try_locals_drop_before_finally_on_both_return_and_normal_paths` records the failing
native assertion. Preserve scope-exit disposal, including conditional/partial moves,
before accepting the broader finally repair or resources. Do not replace this with
an early whole-record drop, implicit clone or uninitialized placeholder. Exact red
and repaired evidence is stored in SQLite under `tsrust-map-finally-` and
`tsrust-finally-local-drop-`. Recursive stored callbacks, gate 190 and S07 family
acceptance remain open; installed consumers remain at `4367f4cc`.


Thirty-five finally unit checks pass; the known try-local disposal test is ignored.
The formatter corpus and rebuilt release CLI assertion fixture pass. Five matched
native/Node-after-tsc cases agree in SQLite
`runtime/results/2026-10-11T03-06-13-031Z-verify.json`; `execution` is empty.
The previous release at `eba57bd2` independently reproduces the try-local failure
(`tsrust-finally-local-drop-baseline.tsr`, `.rs`, `.log`).


### Lexical cleanup for regions without partial returns

Direct cleanup is now limited to regions with an actual partial-field return
transfer. Whole-root initialized slots already preserve return availability, so
ordinary and Map completion bodies can retain their lexical scope boundary. The
original try-local disposal test is enabled and passes at O0/O3. Additional probes
verify a conditionally moved local record field and a renewed Map callback owner.
Prior field moves in the body do not falsely classify a later whole-root return as
a partial return; classification uses the checked return transfer's source span.

Regions containing partial returns still use inline cleanup and retain the disposal
bug. `try_locals_drop_before_partial_field_return_cleanup` is the replacement ignored
witness, not an accepted limitation. The next repair must transport the surviving
outer fields on normal completion while keeping body-local Rust drop scopes intact.
A standalone Rust prototype passes eight paths at O0/O3: early/normal completion,
preserved return, replacement return, cleanup error and restored-field continuation.
It checks exact disposal, cleanup order, a borrowed sibling and returned allocation
identity. SQLite retains `tsrust-finally-normal-field-transport.rs` and `.log`.
This is lowering-design evidence, not compiler acceptance of partial-return disposal.
Focused compiler records use `tsrust-finally-lexical-`. No timing ran.


Final focused checks: 38 finally unit tests, 44 exception integration tests and
17 scoped-record finally tests pass. The formatter corpus passes and whole-project
formatting changes no file. Running the partial-return witness explicitly still
fails with `try local survived into finally` (0 observed, 1 expected), as retained
in `tsrust-finally-lexical-partial-residual.log`. No editor behavior or benchmark
source changed; family regressions and measurements remain pending.

### Partial-field completion transport and lexical disposal

Ordinary partial returns now transport surviving fields on Normal completion.
Real body scopes dispose try/catch locals before finally; normal fields are restored
before cleanup, and cleanup survivors are restored directly in final Normal dispatch.
Body, Handled and Cleaned ownership snapshots distinguish fields returned or consumed
by cleanup. Untouched siblings keep their original storage and borrows. Conditional
and unconditional returns, nested cleanup, eligible catches, default parameters,
local records, cleanup replacement and normal field continuation are covered.
No implicit clones, dummy values or manual whole-record drops were added.

The original partial-return disposal witness is enabled. Native O0/O3 checks prove
returned allocation identity and exact local disposal. Forty-three finally unit
checks pass; one Map partial-return disposal witness remains ignored and explicitly
fails when run. Remaining fallback routes include Map carriers, escaping controls,
catches after body-field transfers and specialized/capture record layouts. Resource
acceptance remains open until these routes are resolved and verified.

Focused results: 21 exception-handling tests, 18 return-move tests, 4 generic-call
exception tests, 3 native-region tests, 17 scoped-record finally tests and 6 editor
exception tests pass. Formatter corpus token preservation/idempotence passes.
Whole-project formatting changes zero files and fingerprints. The rebuilt release
passes the public owned-return assertion fixture. Benchmark registration checks
pass (3); five matched native/Node-after-tsc cases agree in SQLite record
`runtime/results/2026-10-11T04-04-21-702Z-verify.json`. Its execution list is empty:
this is untimed verification, not measured acceptance. The workload, independent
oracle, matched TypeScript and mutation guards include conditional finally returns.

Exact red and repaired logs use SQLite prefix `tsrust-finally-field-transport-`.
An intermediate integration failure came from emitting a dead catch and checking
only keyword-sized statement spans for nested transfers; both were repaired, the
affected target rerun, and the previously unrun targets completed. Installed
consumers remain `4367f4cc`. Recursive callable storage, gate 190 and S07 acceptance
remain pending. No full family regression or timing ran at this checkpoint.

### Map partial-return lexical disposal

The ordinary surviving-field plan is now shared with Map completion bodies.
Normal fields travel beside independently typed Map owners, then are restored before
cleanup and again on normal continuation. Eligible catch handlers use the same
transport. The body-normal fields are moved only after selecting Normal, preserving
availability on the catch path. Terminal bodies and unconditionally replacing
cleanup carry no normal fields. Body/catch locals leave their real Rust scope before
cleanup; returned allocations and callback environment types remain unchanged.

The formerly ignored Map partial-return disposal witness is enabled and passes at
O0/O3. New native probes cover catch returns, terminal returns and unconditional
cleanup replacement. The callback-environment probe separately proves exact disposal
and pointer identity. New integration cases cover conditional cleanup field returns
and terminal branches with distinct callback origins. A caught-field regression from
premature synthetic field transfer was repaired and its full affected target rerun.

Final focused results: 47 finally unit tests pass, plus the callback-disposal probe;
21 exception-handling, 20 return-move, 4 generic-call exception, 3 native-region and
17 scoped-record finally tests pass. The formatter corpus passes; whole-project
formatting changes zero files/fingerprints. Rebuilt release runs the public assertion
fixture. Five matched native/Node-after-tsc cases agree in SQLite record
`runtime/results/2026-10-11T04-16-00-803Z-verify.json`; verifiedOnly is true and
execution is empty. Existing workload sources, guards and hashes remain unchanged.
No timing, full family regression or consumer installation occurred.

Next failing witness: `loop_control_partial_returns_keep_lexical_disposal`, explicitly
ignored after reproducing zero local disposals where cleanup requires one. Its source
mixes an early field return with break; break is the first failing native path.
The initial probe called the infallible ABI incorrectly; the corrected Result ABI
probe confirms the runtime disposal defect. Both records are retained. Escaping
controls need ownership payloads for their specific destinations. Catches after
body-field transfers and specialized/captured record layouts also retain fallback
routes. These are unfinished work, not accepted resource semantics. Exact evidence
uses SQLite prefix `tsrust-map-field-transport-`. Gate 190 and S07 remain open.

### Partial-field loop control and lexical cleanup

`field_control_transport.rs` gives each escaping break/continue destination an
independent surviving-field payload. Handled snapshots select input fields; Cleaned
snapshots select fields restored before the eventual loop exit. Normal fields retain
their own payload. Body scopes end before cleanup, including when cleanup replaces a
pending return with break. A completion absent from checked ownership states emits
an unreachable branch instead of inventing initialized fields. Empty native storage
payloads now render `()`, with a focused storage regression assertion.

The original ignored loop disposal witness is enabled. Native O0/O3 probes pass for
break, continue, nested cleanup, cleanup break replacing return, and a Map-backed
loop with distinct callback origins. These check exact local disposal and returned
allocation identity where the return survives. Integration cases cover for-loop
updates, normal fallthrough, labeled continue and conditional cleanup replacement.
The cleanup-break case previously failed native compilation; it now executes with
correct disposal. The Map loop probe already passes with the new control route;
its log retains the initial probe filename ending in `map-residual`.

Focused results: 52 finally unit tests pass, one residual catch witness is ignored;
21 exception-handling, 22 return-move, 4 generic-call exception, 3 native-region and
17 scoped-record finally tests pass. Fifteen loop unit tests, the empty-storage
assertion and formatter corpus also pass. Completed checks are retained. No editor
behavior changed. The public fixture now executes all loop completion modes.

The matched workload now includes partial-field loop returns, break, continue and
normal completion; cleanup traces prove iteration/update behavior. Independent
closed-form expectations, matched TypeScript, source fingerprints and mutation
guards are updated. Registration passes all three checks. Measurement protocols,
iterations and operation units remain unchanged; no timing or full family regression
ran. Exact red/repaired evidence uses SQLite prefix `tsrust-field-control-`.

Next verified failure: `body_field_return_and_catch_keep_lexical_disposal`. One body
branch returns a field and another throws to an empty catch. Native execution still
observes zero local disposals where cleanup requires one. This witness is explicitly
ignored pending an error-field payload restored before catch. Captured/specialized
layouts and catch/control combinations retain fallback routes and need coverage.
Resource acceptance, gate 190 and S07 remain open. Installed consumers stay at
`4367f4cc`; scratch release is rebuilt from current source.

Final release verification: public assertion fixture passes, whole-project formatting
changes zero files/fingerprints, and five native/Node-after-tsc cases agree in SQLite
`runtime/results/2026-10-11T04-28-42-759Z-verify.json`. verifiedOnly is true and
execution is empty. These are untimed correctness checks, not measured acceptance.

### Body field returns through catch and nested cleanup

Ordinary throwing bodies now carry surviving fields with their error payload and
restore them before catch. Normal completion carries its own fields. Nested plain
and loop cleanup frames inherit the error-field contract and forward it without
cloning. Fields moved by error transport but retained on a return travel beside the
pending return; normal/control alternatives also preserve these fields where needed.
This keeps initialization visible when cleanup replaces a pending return with a
different field. Checked-absent normal completions diverge instead of emitting a
phantom cleanup branch that reads moved storage.

The original body-return/catch disposal witness is enabled. Additional native O0/O3
probes cover handler field returns, nested cleanup and caught loop exits, with exact
local disposal and returned allocation identity. Integration checks include normal
fallthrough, error forwarding, and nested/loop cleanup replacing pending returns.
An initial replacement test used fallible inner logging; its ownership rejection
was correct because logging could throw after the field moved. The final source
keeps inner cleanup infallible. Native field-owner lookup was separated from the
Map split-record lookup so ordinary inline record paths select the right owners.

Focused checks: 56 finally unit tests pass with one Map catch residual ignored;
21 exception-handling, 25 return-move, 4 generic-call exception, 3 native-region and
17 scoped-record finally tests pass. The existing Map callback cases remain green.
New public fixture assertions and matched workload cases cover body return, caught
throw, handler return and normal continuation. Independent closed-form expectations,
TypeScript counterpart, mutation guards and fingerprints are ready; all 3 registration
checks pass. Measurement protocols, iteration counts and operation units are retained.
No editor behavior changed. No full family regression or timing ran.

Remaining explicit failure: `map_body_field_return_and_catch_keep_lexical_disposal`
observes zero body-local disposals where cleanup requires one. Map error/normal
carriers still need the field payload integration; the existing guard remains.
Same-region catch/control and captured/specialized layouts also need coverage.
Resource acceptance, gate 190 and S07 remain open. Exact evidence uses SQLite prefix
`tsrust-field-catch-`. Installed consumers remain `4367f4cc`.

Final formatter corpus and rebuilt release assertion fixture pass. Whole-project
formatting changes zero files/fingerprints. Five native/Node-after-tsc cases agree
in SQLite `runtime/results/2026-10-11T04-57-25-403Z-verify.json`; verifiedOnly is true
and execution is empty. This remains untimed correctness evidence.


### Map body field returns through catch

The previously ignored Map body-return/catch witness is enabled. Surviving ordinary
fields accompany the body's error and normal payloads, independently of concrete
Map callback owners. Selecting either path restores its fields before the handler
or cleanup. An empty handler projects its caught callback owner into the normal
cleanup representation explicitly; it has no statement input epoch to provide that
conversion. Body-local disposal now precedes cleanup, and returned allocation
identity is unchanged. O0/O3 witnesses also cover nonempty handlers and handler
field returns. Integration coverage includes normal fallthrough and cleanup replacing
a pending return with another field.

Focused results: 59 finally unit tests pass with none ignored; 21 exception-handling,
26 return-move (25 existing plus the new focused case), 4 generic-call exception and
3 native-region tests pass. Whole-project formatting and the formatter corpus pass.
The public callback fixture and matched workload now throw after renewing the Map,
then record the catch before invoking the callback in cleanup. The independent
closed-form oracle, TypeScript counterpart, mutation guards and source hashes reflect
that behavior; all 3 registration tests pass. Iterations, operation units and timing
protocols are unchanged. No editor behavior changed; no full regression or timing ran.

Same-region catch/control combinations, nested Map error forwarding with ordinary
fields, and specialized/captured layouts still require coverage. Gate 190, resource
acceptance and S07 remain open. Installed consumers remain at `4367f4cc`; scratch
release includes this repair. Exact records use SQLite prefix `tsrust-map-catch-fields-`.

Final verification: 17 scoped-record finally tests pass. The rebuilt release passes
the public assertion fixture. Five native/Node-after-tsc correctness cases agree in
SQLite `runtime/results/2026-10-11T05-08-30-353Z-verify.json`; `verifiedOnly` is true
and `execution` is empty. This is untimed evidence. Whole-project formatting changes
zero files and fingerprints.


### Nested Map fields and same-region catch controls

Ordinary catch-plus-finally now supports destination-specific field transport for
break/continue. The body's error carries surviving fields into catch. Return payloads
retain fields that alternate error paths transfer. Body-local disposal precedes
cleanup, verified at O0/O3 with exact drop counts and unchanged allocation identity.
Integration cases cover handler break/continue, normal continuation and cleanup
replacing a pending return with a sibling field.

Nested Map error projection no longer assumes every destination is a Map. Surviving
ordinary fields accompany the inner error and are restored before cleanup. Precise
Map return carriers also retain surviving fields needed by that cleanup. Error
cleanup carries its normal fields separately, preserving initialization when a
conditional cleanup return consumes a sibling. Impossible normal, infallible-error,
and specialized-control alternatives do not access moved fields. A regression showed
legacy cleanup returns still require their fallback route when no specialized return
carrier exists; the repair preserves that route.

The original panic, intermediate ownership failures and repaired checks are retained
under SQLite prefixes `tsrust-nested-map-fields-` and `tsrust-caught-field-control-`.
The public fixture and matched workload now forward Map errors through an inner
finally and exercise break/continue from catch. Oracle results are unchanged; source
guards, matched TypeScript and hashes include the new control flow. Protocols and
sample counts are unchanged. No editor behavior changed. No full regression or timing ran.

Map-backed same-region catch/control and specialized/captured layouts still need
coverage. Gate 190, resource acceptance and S07 remain open. Installed consumers
remain at `4367f4cc`; scratch release is rebuilt from the current source.

Final evidence: 61 finally unit tests, 30 return-move, 21 exception-handling,
4 generic-call exception and 3 native-region tests pass. The scoped-record batch
retains 16 passes; its one failing legacy-return case passes after the focused repair.
Both nested Map integration cases pass again after that repair. The formatter corpus,
3 registration tests and rebuilt release public assertion fixture pass. Whole-project
formatting preserves tokens and finishes with zero changes/fingerprint updates.
Five native/Node-after-tsc cases agree in SQLite
`runtime/results/2026-10-11T05-21-39-462Z-verify.json`; `verifiedOnly` is true and
`execution` is empty. These checks are untimed, not family measurement acceptance.


### Callback-bearing record leaves and owning captures

Field completion transport now expands callback-bearing records into their independent
native leaves. Checked availability selects each surviving leaf, including leaves of
a returned nested plain record. Source/native field lookup recognizes those names.
Owning captures with Consume mode may use the same transport when they are not
references. Native declarations become mutable where restoration requires it; this
does not permit mutation of a source const or borrowed owner. Map inherited-field
selection distinguishes Maps by target type rather than by native name recognition.

Two new red native disposal probes preceded the repair: a local record containing a
callback, and a plain record owned by a callback. Both now pass at O0/O3 with exact
local disposal and unchanged returned allocation identity. Map/catch loop disposal
already passed; four-mode integration coverage checks callback origins on return,
break, continue and normal continuation. Additional source tests cover nested record
field returns, owning captures through catch/loop cleanup, and split ordinary leaves
through nested Map error/replacement transport. The public fixture combines a captured
callback-bearing record with field returns through finally.

A separate probe attempted to return a borrowed callable inside a record and received
the existing callback-escape diagnostic. The repair preserves that rule; the accepted
aggregate-return test moves a plain child record beside the retained callback. Restored
Option-root and borrowed-storage fallback routes are unchanged. No disposal witness
is ignored. Recursive stored-callable support, gate 190 and S07 acceptance remain open.

Matched exception support now uses a callback-bearing record for retained fields and
an owning callback for caught returns. TypeScript, source guards, mutation checks and
hashes are current; the independent closed-form result stays unchanged. Iterations,
operation units and measurement protocols are preserved. This is focused implementation
acceptance, not timing or a full family regression. No editor behavior changed.
Exact records use SQLite prefixes `tsrust-callback-field-cleanup-`,
`tsrust-captured-field-cleanup-`, `tsrust-owned-capture-field-cleanup-` and
`tsrust-map-catch-control-`. Installed consumers remain `4367f4cc`.

Final focused evidence: 64 finally unit tests pass with none ignored. Return-move
coverage has 32 retained passes, the corrected aggregate-return case passes, and the
new split-field/nested-Map case passes (34 cases total). The adjacent 21 exception,
4 generic-call exception, 3 native-region and 17 scoped-record finally tests pass.
The formatter corpus, 3 registration checks and rebuilt release public fixture pass.
Five native/Node-after-tsc cases agree in SQLite
`runtime/results/2026-10-11T05-33-30-740Z-verify.json`; `verifiedOnly` is true and
`execution` is empty. Whole-project formatting finishes with zero changes. No timing
or full family regression ran.

### Nullable links in callable layout validation

The callable cycle graph and inline-size budget now honor the shared nullable-link
plan. Ordinary recursive records execute beside stored callbacks and with owned
stored callbacks at both native optimization levels. Direct callable-environment
cycles remain rejected. This fixes an unrelated stored callback making otherwise
valid nullable record declarations fail analysis.

Recursive records with specialized view-capturing callback fields remain pending.
The saved source reaches a nominal/concrete native field mismatch after removing
the stale graph rejection. A targeted source diagnostic keeps that combination
from reaching malformed native code. One ignored native fixture preserves the
unfinished behavior; active compiler/editor tests verify its current diagnostic.

Focused checks: 113 callable unit tests, 17 byte integrations (one pending native
fixture ignored), two recursive editor diagnostics, formatter corpus and formatted
release linked-view execution pass. The ordinary callback-chain integration also
passes with and without native optimization. Whole-project formatting is idempotent.
Native and Node-after-tsc agree on five cases each for `native_byte_views_linked`
and `native_byte_views_callbacks`, recorded as untimed verification in
`runtime/results/2026-10-11T05-44-49-657Z-verify.json`. Execution samples are empty.
No full family regression or timing ran; gate 189, gate 190 and S07 remain open.
Evidence logs use SQLite prefix `tsrust-recursive-stored-callback-`. Installed
consumers remain `4367f4cc`.

### Concrete recursive callback storage

Stored view-capturing callbacks now execute through self-links, mutual links and
arrays when the recursive component uses one concrete callback family. Native
records retain a shared real lifetime and concrete callback fields. The public
fixture is enabled; no byte integration remains ignored. Mutable callback results
write through to their owner. Owner conflicts, shortest-owner escape, cloning and
repeated Once invocation remain rejected.

Focused evidence: 18 byte integrations pass, with the expanded recursive variants
and corrected exact Once diagnostic rerun afterward; four native family tests,
113 callable unit tests, two materialization tests, 42 scoped callable record
integrations, four nullable integrations and two recursive editor tests pass.
The native chain probe checks slice identity and a 10,000-node drop on a 64 KiB
stack at O0/O3, plus Rust rejection of a returned short-owner borrow. Whole-project
formatting, corpus token preservation/idempotence and formatted release fixture
execution pass. These are focused checks, not the S07 full regression.

`native_byte_views_stored_callbacks` now has matched TypeScript, an independent
17-node arithmetic oracle, source mutation guards, hashes and harness support.
Five registration checks pass. Native and Node-after-tsc agree on five untimed
cases in `runtime/results/2026-10-11T05-57-38-811Z-verify.json` and the final source
rerun `runtime/results/2026-10-11T06-00-55-478Z-verify.json`; execution samples are
empty. Logs and reproduction sources use SQLite prefix `tsrust-recursive-concrete-`.

Mixed-family links remain pending: two different factories currently produce a
source type rejection. The original fixed-layout diagnostic remains for unsupported
specializations. Gate 189, resource gate 190 and S07 stay open. Installed consumers
remain `4367f4cc`; no timing ran.

### Recursive callback family union and extraction

Different factories now contribute checked alternatives to one stable family per
recursive record field. Existing concrete links retain their identity. No runtime
chain conversion, extra link allocation, byte copy or shared owner is introduced.
The same local field can be restored after Once invocation using a different
closure. Nonrecursive family identities and independent lifetimes remain unchanged.
Extracted callback returns preserve the recursive component's shared lifetime.

Focused checks: 19 byte integrations, four native family tests, 114 callable unit
tests (including atomic recursive-family admission/remapping), two editor checks,
formatter corpus and formatted release execution pass. Source checks reject a
short owner introduced by a different child factory and a short discarded sibling
when returning the extracted callback. The native 10,000-node O0/O3 probe now
alternates two factories. These remain focused checks, not full S07 regressions.

The stored-callback workload now alternates factories while retaining its 17-node
arithmetic oracle and operation denominator. Native and Node-after-tsc agree on
five untimed cases each for linked views, argument callbacks and stored callbacks
in `runtime/results/2026-10-11T06-15-33-873Z-verify.json`. A source-mutation test
caught a guard that accepted removing one factory's callback because another
factory still contained it. The guard now checks both complete factory bodies;
five registration tests pass. Only the affected stored-callback verification was
rerun, in `runtime/results/2026-10-11T06-16-24-880Z-verify.json`. Both records have
empty execution samples and are retained. Logs use `tsrust-recursive-family-`.

Record-parameter transport remains open: `take(holder:Holder){return holder.read;}`
reports an unresolved `error-effect` contract despite a concrete constructed
argument. Resolve parameter family identity and lifetime provenance together.
Callback-only view storage and nullable callback-field composition still need
verification. Gate 189, gate 190 and S07 remain open; consumers stay `4367f4cc`.

### Recursive callback record parameter transport

Recursive record parameters now retain the concrete stored callback family and
shared owner lifetime. The public `recursive-parameters.tsr` fixture covers owning
transport, readonly access, whole-record returns and callback extraction. Focused
variants add source-order permutations, direct construction, explicit callable
return annotations, mixed factories, nullable child extraction and mutual links.
Both a captured short owner and a discarded sibling short owner reject escape.

Concrete literals and mutually recursive members reuse the existing component.
Recursive records retain whole native storage for borrowing and partial moves.
Discovery does not require native return summaries; definitive checking does, and
reused closures refresh their return-lifetime/storage-exit metadata. No byte copy,
shared owner, lifetime erasure, added link allocation or recursive conversion is
introduced. Incompatible component ambiguity is not resolved arbitrarily.

Focused evidence covers the 21 public byte tests, including repaired affected
recursive cases; 34 record tests; 114 callable tests; four native family tests;
three editor diagnostics tests; five benchmark registration/mutation tests; and
the formatter token/idempotence corpus. The native chain witness now transports
10,000 links through an owning parameter at O0/O3, proves slice pointer identity,
and rejects both short-owner cases on the Rust side.

The matched stored-callback workload now returns its final callback through a
record parameter. Both engines keep the original 17-node observation and operation
denominator. Gate 189 still needs the remaining storage compositions; gate 190,
S07 regressions and family measurements remain pending. Installed consumers remain
`4367f4cc`.

Native and Node-after-tsc agree on five untimed cases in SQLite record
`runtime/results/2026-10-11T06-42-06-526Z-verify.json` (`verifiedOnly: true`, no
execution samples). SQLite also preserves 31 exact logs under
`tsrust-recursive-parameter-*.log`, including failed witnesses and their repairs.
All records were read back byte-for-byte before loose copies were deleted;
`PRAGMA quick_check` passes. Final whole-project formatting changed no files;
the formatted public parameter fixture executes with the release compiler.

### Recursive records containing only captured views

A recursive record no longer needs a direct view field to store owned callbacks
that capture views. The recursive lifetime index follows actual family capture
slots and merges exclusive access. Parameter seeding, loan normalization,
environment counts, native declarations and iterative cleanup share that index.
Recursive symbolic bases remain concrete lifetime-free layouts; specialized
components carry their real shared lifetime. Empty owning families have no error
ABI, while unresolved catch contracts still reject.

The new public `recursive-captured-views.tsr` fixture and ten O0/O3 variants cover
self/mutual/array links, nullable callbacks, record parameters, scalar callback
results, an unused input owner ending early, exclusive writes, local replacement
and capture of a whole record. Short captured-owner escape rejects. The native
10,000-link/64 KiB stack probe also runs with two captured-view callbacks and no
direct view field, checks pointer identity and rejects either short owner at O0/O3.

Focused checks cover the 23 public byte tests through retained and repaired passes,
114 callable tests, four native family tests, four recursive lifetime-index tests,
four environment-count tests, two native capture tests, three recursive runtime
tests, 34 record tests, four editor diagnostics tests and six benchmark registration
and mutation tests. Whole-project token/idempotence and formatted native checks
remain part of this checkpoint.

`native_byte_views_captured_only` has matched TypeScript, independent observations,
source guards/hashes and harness support. It preserves the 17-node construction
and consuming traversal denominator. Basic nullable callback fields execute;
nullable joins across factories and mutable-parameter replacement remain pending.
Gate 189, gate 190 and S07 acceptance remain open. Installed consumers remain
`4367f4cc`.

Native and Node-after-tsc agree on five untimed cases for each of the linked,
stored-callback and callback-only workloads in SQLite record
`runtime/results/2026-10-11T06-59-38-616Z-verify.json` (`verifiedOnly: true`, no
execution samples). The formatted public fixture also executes with the release
compiler. SQLite retains the exact `tsrust-storage-composition-*` logs and probes,
including the two remaining failures: nullable joins from different factories
(`Expected Holder | undefined, found Holder`) and mutable-parameter replacement
(`Expected closure, found closure`).

Forty logs/probes and the untimed result were imported and read back byte-for-byte
before deleting loose copies. SQLite `PRAGMA quick_check` passes. Final
whole-project formatting changes no files; obsolete native/test outputs are pruned.

### Gate 189 nullable recursive callback joins

Nullable callback fields now share a stable family across populated and empty
factories, owning parameter extraction and later local replacement. Constructors
and discovered returns retain the same recursive component. Field replacement
also materializes recursive components rather than expanding their links.

An empty callback-only result quantifies an independent output lifetime. It does
not borrow unused inputs or substitute static. Direct functions, forwarding calls,
closures and trait adapters preserve that contract. Filling an empty record with
a short-lived capture still rejects escape. The public nullable fixture and ten
source variants cover null/undefined, factory order, empty leaves, callback-only
records, optional extraction and replacement at O0/O3. The native stress witness
now includes nullable callbacks, checks allocation identity, drops 10,000 links on
a 64 KiB stack and rejects either short owner at O0/O3.

Focused acceptance retains 24 public byte tests through completed and repaired
passes, 321 affected callback/record/nullable tests, four native family tests, five
editor tests, seven benchmark registration/mutation tests and the formatter corpus.
The new closure fixture and owner rejections were rerun after the trait repair;
completed unaffected checks were retained. Whole-project formatting is unchanged,
and the formatted public fixture executes with the current release compiler.

`native_byte_views_nullable_callbacks` includes an empty leaf and 16 parents
alternating two capturing factories. Its matched TypeScript, independent arithmetic
oracle, source guards, hashes and ABI/harness registration are complete. Native and
Node-after-tsc agree on five untimed cases each for this workload, stored callbacks
and callback-only records. Exact evidence is SQLite record
`runtime/results/2026-10-11T07-26-39-414Z-verify.json` (`verifiedOnly: true`, zero
execution samples). This is correctness evidence, not measurement.

SQLite retains exact `tsrust-nullable-family-*` logs/probes, including original
failures and repaired passes, with byte-for-byte readback before loose-file removal.
Mutable recursive parameter replacement remains pending. Gate 189, resource gate
190 and S07 acceptance remain open; installed consumers remain `4367f4cc`.

Forty-three logs/probes and the untimed result were imported and verified before
removal. SQLite `PRAGMA quick_check` passes. Twenty-six obsolete test/native outputs
were pruned (369,763,335 bytes); current CLI/editor consumers were retained.


### Gate 189 mutable recursive parameter stores

Intact recursive records now accept callback replacement through mutable
parameters, including nullable and callback-only fields. Source checking retains
mutation summaries independently of the final control-flow state, so early return
and cleanup writes remain visible. The native signature requires each incoming
view source to outlive its destination slot. Callers retain those owners after the
call. Closure and function-reference adapters keep the same bounds; checked capture
destinations receive the same caller effects. No byte copy, shared owner, static
substitution, unsafe lifetime erasure or placeholder is introduced.

The public `recursive-mutation.tsr` fixture and fourteen source variants execute
at O0/O3. They cover readonly/exclusive views, nullable fields, relays, early return,
finally, borrowed Bytes, function references, direct closures, captured source and
destination storage, and returned installers. Ten source controls retain owner
escape/conflict and partial-transfer diagnostics. The original replacement probe
consumed its field before borrowing the whole record; its remaining rejection is
intentional. Captured destinations without caller provenance reject rather than
using an unrelated binding identity.

A native witness replaces a callback in a 10,000-link chain on a 64 KiB stack,
checks both retained and replacement slice pointers and rejects a short replacement
owner or sibling owner at O0/O3. Focused evidence includes 76 integration tests
(26 bytes, 35 callable fields, six callable results, nine parameter replacements),
121 native-view tests, six editor diagnostics tests, eight benchmark registration
and mutation tests, and the formatter corpus. Later capture cases and diagnostic
wording were checked with focused repairs; unaffected completed evidence is retained.

`native_byte_views_parameter_stores` has matched TypeScript, independent arithmetic
observations, mutation guards, source hashes and ABI/harness support. It builds an
empty leaf plus 16 parents, replaces each callback with a second-byte view through
a mutable parameter, and consumes the chain. Native and Node-after-tsc agree on
five untimed cases each for this workload, nullable callbacks and callback-only
records. SQLite record `runtime/results/2026-10-11T07-57-39-054Z-verify.json` is
correctness-only (`verifiedOnly: true`, zero execution samples). The final follow-up
changes only diagnostic wording, preserving that successful execution evidence.

Exact logs and probes use the `tsrust-recursive-replacement-*` SQLite prefix.
Whole-project formatting and formatted release-native execution pass. Gate 189
final review, resource gate 190 and S07 family acceptance remain pending. Installed
consumers remain `4367f4cc`.

Thirty-three logs/probes and the untimed result were imported, read back byte for
byte and removed from loose storage. SQLite `PRAGMA quick_check` passes. Twenty-one
obsolete test/native outputs were pruned (188,979,411 bytes); current consumers were
retained.

### Gate 189 scope review

Focused implementation acceptance is complete at `bd4b0d02`; see
`milestones/189-native-byte-views.md` for the requirement mapping. The current table,
roadmap and all nine gate 189 workload acceptance flags now agree. Historical
pending entries remain chronological evidence. Measurement flags remain false.
SQLite readback reconfirmed the latest repaired matrix/native/editor/diagnostic
passes. This review preserves completed checks; it does not claim a new complete
regression or timing pass. Gate 190 is the next implementation step.

### Gate 190 private resource backend

`stdlib/resource.rs` now defines a unique backend owner with consuming explicit
close and a nonthrowing fallback that discards reported close failures. Taking the
backend before explicit close prevents a second release during wrapper destruction,
including when close returns Err. Borrowed access does not expose ownership.

The private instrumented backend checks exact release order for scope exit, early
return, replacement, Result failure, callback failure, unused closure disposal,
explicit successful/failed close and panic unwinding. Three unit tests pass; two
native integration tests rerun the release witnesses at O0/O3 and reject native
reuse, clone, comparison and hashing. The original missing-backend failure and
repaired results use the `tsrust-resource-backend-*` SQLite prefix.

This runtime is currently test-only and is not a published source handle or a
completed source capability/ABI integration. The next step is the concrete file
backend and registry integration, followed by generated Tsrust cleanup witnesses,
source/editor rejections, catalog examples and matched benchmark support. Gate 190
and S07 remain open. No regression milestone or timing pass ran.

Whole-project formatting changes no files or fingerprints. The token/idempotence
corpus and formatted recursive-mutation native fixture pass. Six logs were imported
and read back exactly before removal; SQLite integrity passes. Two completed test
executables were pruned; current consumers were retained.
