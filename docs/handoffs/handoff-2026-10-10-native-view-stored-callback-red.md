# Stored view callback TDD checkpoint

Continue from da2eccb4. Read AGENTS.md, docs/development-workflow.md,
native-byte-views-implementation.md and plans/native-view-callable-storage-plan.md.
No production compiler behavior changed in this checkpoint. Gate 189/S07 remain open.

The new isolated views_stored_emit_tests.rs has two complete source/native witnesses.
The mixed record control passes at O0/O3 with pointer identity and independent
selected-owner escape rejection. The callback field positive test intentionally
remains red at callback-escape: Once<Function<Window>> storage rejects a callback
owning a carried view. The latter is a valid one-shot callback, unlike the initial
reusable-contract probe. Earlier probes and their failures are preserved separately
in SQLite; they are not claimed as feature acceptance.

Do not remove callable_storage.rs or callable_dispatch.rs safety guards alone.
The owning family currently lacks native payload lifetime planning. Follow the
new plan: typed scoped owning family metadata, actual-callee source transport,
then safe lifetime-bearing family emission and storage/move/replacement controls.
No full regression, timing or public API acceptance was performed. The warm CLI
still comes from da2eccb4; installed consumers remain at f569190a.

SQLite record: runtime/results/native-view-stored-callback-red-2026-10-10.json.

## Continuation prompt

Implement the stored view callback dependency from plans/native-view-callable-storage-plan.md.
Start with typed scoped owning family metadata and preserve original invocation
modes. Make native_view_record_callback_preserves_selected_input pass through normal
source analysis and O0/O3 emission without weakening physical borrow checks. Keep
native_view_mixed_record_preserves_selected_input green. Add distinct alternative
origins and owner-escape controls before expanding arrays/nullable storage. Keep
public factories guarded, use bounded checks and normal-stack canaries after shared
analyzer changes, then format, store exact evidence in SQLite and checkpoint.
