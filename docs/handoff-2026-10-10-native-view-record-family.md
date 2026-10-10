# Scoped owning families in native record fields

Read AGENTS.md, docs/development-workflow.md and
native-view-callable-storage-plan.md. This checkpoint follows d14caee9.
Gate 189/S07 remain open. Public factories and normal owning source construction
remain guarded. The positive stored-callback source witness remains red.

native_view_record_parameters.rs indexes native-bearing field positions across record
instances under their nominal declaration. Record definitions, type rendering, named
lifetime signatures, environment types and record-array parameters now use that shared
decision. Ordinary owning families remain lifetime-free; their identities are not
reclassified to make the native representation fit.

callable_layout::declared_storage_families takes a generic-field predicate and skips
those symbolic field types, avoiding unused empty owning-family declarations with
unbound view result lifetimes. Concrete function storage and populated origins remain
emitted. Discovery uses direct loops without temporary per-record vectors.

The normal family test now materializes a Holder instance with a scoped owning field,
then forwards the record through generated named input/return types at O0/O3. Both
alternatives preserve pointer identity; exclusive writes reach the owner. Owner escape,
second invocation and conflicting mutation remain independently rejected. Original red
showed a nongeneric nominal record and an invalid unused symbolic family.

Evidence: SQLite runtime/results/native-view-record-family-2026-10-10.json.
No public source acceptance, full regression or timing. Installed consumers remain
f569190a. The warm CLI is rebuilt for formatting.

## Continuation prompt

Finish normal source construction now. Scoped identity, provenance, normal family
emission and nominal record generic field positions are connected. Do not redo the
private ABI proofs. In checked source storage, use intern_scoped_native for an owned
carried-view environment; preserve the written invocation contract and materialize
record fields with the scoped family identity. Require structural callback compatibility
without certifying lifetime-free storage, and retain the ordinary environment guard for
physical reference captures. Ensure unsupported ABI forms produce source diagnostics
before any emitter assertion. Turn native_view_record_callback_preserves_selected_input
green through normal analysis and complete emission, retaining selected-owner escape
rejection. Then alternatives, moves, replacement, arrays, nullable fields and required
invocation/error/argument forms. Keep public factories guarded until observer,
IntelliSense and matched benchmark coverage is complete. Bounded TDD, canaries after
analyzer changes, project formatting and exact SQLite evidence remain required.

Focused acceptance: 95 native-view tests (stored positive source witness excluded),
114 callable tests and 16 native editor checks passed. The final native-view batch
also checks the allocation-free discovery loop refinement. No shared analyzer dispatch
changed; prior 107-canary evidence remains applicable. The CLI builds without warnings;
project formatting changes no files or hashes. No timing or consumer refresh is claimed.
