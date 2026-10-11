# B11/S07 resources and bytes acceptance map

This is the current implementation map. S07 is not accepted or measured.

| Scope | Status | Authoritative evidence |
| --- | --- | --- |
| Typed native declarations, signatures, imports, aliases and Result payloads | Gate 187 accepted | milestones/187-native-byte-foundation.md |
| Owned byte construction, access, mutation, allocation, growth and range copying | Gate 188 accepted | milestones/188-owned-byte-operations.md; native-owned-bytes fixtures |
| Independent clone, generic/record/array/Map/nullable/Result storage and callable captures | Gate 188 accepted | ownership.tsr; native_declarations.rs; project_intelligence.rs |
| Validation/reservation atomicity and one argument evaluation | Gate 188 accepted | bytes/runtime.rs tests; access.tsr; growth.tsr; copy-append.tsr; ownership.tsr |
| Exact disposal and allocation restoration at O0/O3 | Gate 188 accepted | native_byte_cleanup.rs; cleanup.tsr; exact SQLite gate 188 records |
| Scoped zero-copy readonly/exclusive views, provenance, conflicts and escape diagnostics | Published with focused array/nullable recursive storage checks; gate 189 pending recursive stored-callable fields and capturing callbacks returning argument views | native-byte-views-implementation.md; resources-bytes-plan.md |
| Strict UTF-8 encoding/decoding and error offsets | Implemented increment; gate 189 still pending views | native-owned-bytes/utf8.tsr; utf8_runtime.rs; resources-bytes-plan.md |
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
