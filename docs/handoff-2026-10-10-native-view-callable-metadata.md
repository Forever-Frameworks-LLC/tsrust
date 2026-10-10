# Carried-view callable metadata checkpoint

Read AGENTS.md, docs/development-workflow.md and
native-view-callable-storage-plan.md. Start from the stored callback red checkpoint
e09b71f5. Gate 189 and S07 remain open.

native_view_callable_storage.rs derives candidate carried-view lifetime slots from
checked capture inputs, retaining capture identity, relative paths, Result guards
and readonly/exclusive access. It rejects physical capture references, physical
storage loans and foreign source roots. This helper does not certify owning storage.
callable_storage.rs retains its rejection but now explains the actual pending
lifetime-bearing storage dependency with native-view-callable-storage.

Five metadata controls pass. The existing private view batch (excluding the explicit
stored callback positive red witness) passes, as do owning storage and native editor
contracts. A focused parsed-source test verifies the improved diagnostic. The positive
native_view_record_callback_preserves_selected_input still fails by design; no
family ABI, source transport or new public feature is accepted here.

Exact evidence is in SQLite at
runtime/results/native-view-callable-metadata-2026-10-10.json. No timing/full
regression. Warm CLI rebuilt for formatting; installed consumers remain f569190a.

## Continuation prompt

Continue native-view-callable-storage-plan.md step 1: add finite scoped owning family
identity retaining the original Owned/OwnedOnce invocation contract and candidate
lifetime slots for each concrete alternative. Metadata alone must not bypass the
current representation guard. Then implement actual-callee source transport and
safe lifetime-bearing native family emission. Turn the existing stored callback
positive witness green through normal analysis/emission at O0/O3 with selected-owner
escape rejection. Keep mixed record controls, physical callback borrow checks and
public factory guards. Use bounded checks, formatting and byte-exact SQLite evidence.
