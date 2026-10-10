# Scoped owning native view family identity

Read AGENTS.md, docs/development-workflow.md and
native-view-callable-storage-plan.md. This checkpoint follows 299dd3e1.
Gate 189/S07 remain open. Public view imports and owning source construction stay
guarded; the original stored-callback positive witness is still red.

StorageContract::ScopedOwned(signature, origins) now has a distinct canonical registry
identity. native_view_storage_registry.rs validates carried layouts before interning,
retains the original invocation contract, rejects physical reference captures and
keeps native layout metadata. It does not masquerade as a borrowed invocation family
or populate the ordinary owning identity. Failed origin remapping is atomic; valid
remapping updates both canonical identity and captured layout. The constructor is
not yet called by normal source admission and is explicitly marked for integration.

native_view_emit::Shapes now includes concrete closure captures and stored family
alternatives in its reverse reachability graph. Containing records retain native-view
reachability through callbacks, including cycles. Unpopulated ordinary owning families
remain lifetime-free. No recursive analyzer dispatch was changed.

Evidence: SQLite runtime/results/native-view-scoped-identity-2026-10-10.json.
No full regression or timing acceptance. Installed consumers remain f569190a.

## Continuation prompt

Connect the new scoped owning identity to normal source construction, provenance and
emission. Use native_view_storage_registry::intern_scoped_native rather than admitting
carried views into the ordinary owning identity. Add typed alternative/capture paths
to parameter seeding and native loan transport; construction must preserve actual
owner origins. Extend native signature rendering and containing-record parameters to
scoped families, then wire the private native family backend into normal emission
with construction adapters. Do not remove the guard before those pieces work together.
Turn native_view_record_callback_preserves_selected_input green through normal analysis
and emission, retaining owner-escape rejection. Continue alternatives, moves,
replacement, arrays, nullable fields and required invocation/error/argument forms.
Keep public factories guarded until observer/editor/benchmark support is complete.
Use bounded TDD, depth canaries after analyzer dispatch changes, project formatting
and exact SQLite evidence.

A second reachability defect was caught before acceptance: invalid closure IDs could
alias family graph nodes. Every typed node now checks its own range before mapping
into the combined graph. Original failing and repaired evidence is retained.

Focused acceptance: 93 native-view tests, 114 callable tests and 16 native editor
checks passed. The warm CLI builds without warnings. Project formatting changed no
files or fingerprints. Native-view and editor checks were supplemented after the
range-validation repair; earlier passed evidence was retained. The stored positive
source witness was explicitly excluded from the native-view batch and remains open.
