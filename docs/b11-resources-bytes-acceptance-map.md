# B11/S07 resources and bytes acceptance map

This is the current implementation map. S07 is not accepted or measured.

| Scope | Status | Authoritative evidence |
| --- | --- | --- |
| Typed native declarations, signatures, imports, aliases and Result payloads | Gate 187 accepted | milestones/187-native-byte-foundation.md |
| Owned byte construction, access, mutation, allocation, growth and range copying | Gate 188 accepted | milestones/188-owned-byte-operations.md; native-owned-bytes fixtures |
| Independent clone, generic/record/array/Map/nullable/Result storage and callable captures | Gate 188 accepted | ownership.tsr; native_declarations.rs; project_intelligence.rs |
| Validation/reservation atomicity and one argument evaluation | Gate 188 accepted | bytes/runtime.rs tests; access.tsr; growth.tsr; copy-append.tsr; ownership.tsr |
| Exact disposal and allocation restoration at O0/O3 | Gate 188 accepted | native_byte_cleanup.rs; cleanup.tsr; exact SQLite gate 188 records |
| Scoped zero-copy readonly/exclusive views, provenance, conflicts and escape diagnostics | Gate 189 pending; private loans, return lifetimes, function calls and ordinary arrow invocation integrated; factories, capture/callback/storage completion pending | native-byte-views-implementation.md; resources-bytes-plan.md |
| Strict UTF-8 encoding/decoding and error offsets | Implemented increment; gate 189 still pending views | native-owned-bytes/utf8.tsr; utf8_runtime.rs; resources-bytes-plan.md |
| Opaque resource consuming close, drop and failure ownership | Gate 190 pending | owner consuming-close decision; resources-bytes-plan.md |
| Useful synchronous file proof and private instrumented resource backend | Gate 190 pending | resources-bytes-plan.md |
| Complete regressions, matched measurements and optimization review | Family milestone pending | resources-bytes-plan.md; standard-library-roadmap.md |

All current byte workloads have matching sources, independent oracles, source hashes,
ABI support and harness hooks. Untimed verification is distinct from measurement.
Native thread-transfer metadata is backend eligibility, not source threading acceptance.
The owner chose consuming close even on Err, static rejection of later use/double
close, and a nonthrowing destructor fallback. Bytes does not require explicit close.

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
