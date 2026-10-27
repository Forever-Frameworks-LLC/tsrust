# Gate 189: native byte views and recursive storage

Focused implementation acceptance is complete at source checkpoint `bd4b0d02`.
This review reconciles the approved scope with current source and retained evidence;
it does not rerun or replace completed checks. S07 family regressions, timing,
performance review and final consumer refresh remain pending.

## Requirement mapping

| Requirement | Current implementation and evidence |
| --- | --- |
| Readonly/exclusive zero-copy windows, checked bounds and nested reborrows | `stdlib/bytes/views_runtime.rs`, public `read.tsr` fixture and `native_byte_view_runtime.rs`; O0/O3 pointer identity, zero allocation and write-through controls |
| Shared observer names and writable alternatives | `native_bindings.rs`, `std_bytes.rs`, catalog and `project_intelligence.rs`; `length`, `at`, `copy`, `toUtf8`, `view` accept readable storage; `set`, `viewMut` accept writable storage |
| Provenance, conflicts, transfer and exit release | `native_view_loans.rs`, `native_view_returns.rs`, scope/storage helpers and their unit tests; source/editor negative cases reject local-owner escape, conflicting owner access, resizing, transfer and clone |
| Returned argument views beside captured views | Public callback fixtures and `std_bytes.rs`; borrowed Bytes and view arguments retain their actual sources through direct, stored and temporary record calls |
| Recursive arrays and nullable self/mutual links | `record_runtime_links_tests.rs`, `recursive_storage.rs`, recursive view fixtures and native family witnesses; unique links, unchanged nonrecursive layouts, direct infinite-layout rejection, shared shortest-owner lifetime |
| Callback-only and optional callback fields | `recursive-captured-views.tsr`, `nullable-recursive-callbacks.tsr`, `native_view_family_emit.rs`; empty factories, mixed origins, extraction, 10,000 links on a 64 KiB stack, O0/O3 owner rejection and pointer identity |
| Mutable recursive parameter replacement | `recursive-mutation.tsr`, `native_view_mutations.rs`, `std_bytes.rs`; fourteen O0/O3 variants and ten source controls, including relays, cleanup, function values, closures and tracked captures |
| Strict UTF-8 | `utf8_runtime.rs`, `native-owned-bytes/utf8.tsr`, `std_bytes.rs`; empty/ASCII/multibyte/NUL/BOM, invalid and truncated input offsets, validation before reservation and allocation failure |
| Editor and executable documentation | `builtins.json`, `project_intelligence.rs`, `project_diagnostics.rs`; public names, alternatives, hover/signature/import help and recursive ownership diagnostics |
| Matched benchmark readiness | Runtime manifest, byte-view source contracts, independent oracles, registration tests and TypeScript counterparts; existing untimed results retained in SQLite |

Paths in this table are relative to the compiler/LSP source and test directories
unless explicitly identified as runtime benchmark files. The detailed chronological
commands, failures, repairs and evidence references remain in
`docs/milestones/b11-resources-bytes-acceptance-map.md`.

## Evidence reviewed

The latest parameter-store checkpoint retains 76 affected integration passes,
121 native-view unit passes, five native family witnesses, six editor tests, eight
benchmark registration/guard tests and the formatter corpus. Later capture and
diagnostic changes have their focused repaired passes. SQLite readback confirms
the final matrix, native, editor, view-regression and diagnostic logs all passed.
Whole-project formatting made no changes and the formatted release fixture ran.

SQLite record `runtime/results/2026-10-11T07-57-39-054Z-verify.json` proves agreement
between native and Node-after-tsc for five cases each of callback-only records,
nullable callbacks and parameter stores. It has `verifiedOnly: true` and no timing
samples. Earlier view, recursive and UTF-8 records remain valid for unaffected work.

## Supported boundaries and next gate

Views are not cloneable. Explicit owner copying remains separate. Each present
nullable recursive link uses a unique heap allocation; recursive containers share
one borrow lifetime bounded by every retained owner. No hidden owner, byte copy,
static substitution or erased pointer lifetime is introduced.

Passing a partially transferred record as a whole still rejects. Incompatible
recursive specializations and captured destinations without caller provenance get
source diagnostics rather than guessed identities. These existing ownership and
finite-representation boundaries do not become an unlimited compatibility target.

The owned-return-through-finally defect and the retained disposal witnesses are
repaired and enabled; family regression remains pending. Gate 190 must independently
prove resource close/drop, failure ownership and exact cleanup with a private
instrumented backend and a useful synchronous file slice. Installed consumers remain
at `4367f4cc` until the planned final refresh.
