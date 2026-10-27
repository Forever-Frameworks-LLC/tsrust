# Scoped native callable provenance

Read AGENTS.md, docs/development-workflow.md and
plans/native-view-callable-storage-plan.md first. This checkpoint follows 6fbc7d77.
Gate 189/S07 remain open. Public factories and normal owning source construction
remain guarded; native_view_record_callback_preserves_selected_input is still red.

Scoped owning parameter seeding now traverses family alternatives and capture fields
using the typed Segment::CallableAlternative(origin). Tests cover both access modes,
two alternatives and two independently carried fields. Family return summaries live
in native_view_family_provenance.rs. Named/field/expression call paths register them;
the iterative value walker selects the actual alternative and capture before
transporting its returned view. Construction keeps original owner roots under the
alternative path. A checked-expression control proves a single constructed alternative
returns only its two actual owner loans even when the family contains another origin.
Receiver-storage return borrows remain diagnosed pending their receiver ABI.

native_view_origin_remap.rs updates alternative IDs in binding loans, returned loans,
prepared slots and guard paths during closure compaction. It preserves owner roots,
physical owner fields and record variant IDs. Missing mappings reject without changing
an individual loan or publishing a partially remapped prepared registry.

SQLite evidence: runtime/results/native-view-family-provenance-2026-10-10.json.
No public source acceptance, full regression or timing. Installed consumers remain
f569190a; the warm compiler is rebuilt for checkpoint formatting.

## Continuation prompt

Finish the normal source path rather than adding more private ABI proofs. Scoped
identity, alternative/capture provenance and private native ABI are now present.
Extend native signature rendering and containing-record parameters for scoped owning
families; wire native_view_family_emit into normal family emission and construction
adapters. Connect checked source construction to intern_scoped_native, materialize
record fields with the distinct scoped family identity and preserve the written
ownership contract. Do not admit a carried environment into ordinary lifetime-free
owning storage. Turn native_view_record_callback_preserves_selected_input green
through normal analysis and complete emission, preserving selected-owner escape
rejection. Then alternatives, moves, replacement, arrays, nullable fields and required
invocation/error/argument forms. Keep public factories guarded until observer,
IntelliSense examples and matched benchmark support are complete. Use bounded TDD,
normal-stack canaries after analyzer dispatch changes, project formatting and exact
SQLite evidence.

Focused acceptance: 95 private native-view tests (stored positive witness explicitly
excluded), 114 callable tests, 107 canaries across eleven targets and 16 native editor
checks pass. All commands exit successfully; canary logs contain no aborted targets.
The warm CLI builds without warnings. Project formatting changes no files or hashes.
No consumer refresh or timing is claimed.
