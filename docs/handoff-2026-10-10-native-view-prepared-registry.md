# Prepared native callable registry

Read AGENTS.md, docs/development-workflow.md and
native-view-callable-storage-plan.md. Continue from e82ab6a7. Gate 189/S07 remain open.

A separate native_view_family_registry arena now gives candidate layouts canonical
prepared identities. The owning callable registry retains these only for native
carried environments, in optional storage. Prepared IDs are distinct from source
StoredCallable IDs. Registration still returns native-view-callable-storage and
leaves admitted origins empty. Existing fully owned and borrowed families retain
their separate paths.

The new source-shaped registry test first failed because candidate metadata was
lost. It now verifies sorted alternatives, one-shot invocation, empty admitted
origins, failed compaction atomicity and successful remapping. Two isolated registry
controls prove canonical equal layouts, distinct access slots and capture metadata
moving with swapped origin IDs. Candidate-only registries no longer skip compaction.

Focused checks: 86 private view controls (explicit stored-callback positive red
excluded), 113 callable controls and 16 native editor contracts. Counts overlap.
No public API, emitted family or positive stored callback acceptance. No full
regression or timing. Evidence in SQLite:
runtime/results/native-view-prepared-registry-2026-10-10.json.

Warm CLI rebuilt for formatting. Installed consumers remain f569190a.

## Continuation prompt

Implement admitted scoped owning source storage, actual-callee loan transport and
native lifetime-bearing family emission using the prepared metadata. Keep prepared
identity distinct from the ordinary lifetime-free owning and borrowed invocation
families. Do not remove rejection before lifetime safety is represented end to end.
Turn native_view_record_callback_preserves_selected_input green through complete
normal emission at O0/O3 with independent owner-escape rejection. Preserve ordinary
callable tests, mixed record controls, compaction atomicity and public factory guards.
Use focused checks and normal-stack canaries after shared analyzer changes, project
formatting and byte-exact SQLite evidence. S07 remains the full regression/timing gate.
