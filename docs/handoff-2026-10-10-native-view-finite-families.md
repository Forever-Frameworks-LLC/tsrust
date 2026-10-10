# Finite native view callable family planning

Read AGENTS.md, docs/development-workflow.md and
native-view-callable-storage-plan.md. Continue from 973f6542. Gate 189/S07 remain open.

The candidate family planner now records the original structural signature and
invocation mode, with distinct carried slots for sorted/deduplicated concrete
alternatives. It rejects reusable storage of consuming environments and unknown
origins. Physical reference captures and unresolved layouts retain existing guards.
The dispatch integration preserves the rejection of native payload lifetimes and
avoids candidate allocations on ordinary fully owned families.

Focused evidence includes the initial failing finite-family witness, its repair,
the private view batch with the intentional stored-callback positive witness
excluded, callable contracts/dispatch and native editor checks. No source storage
admission or native family ABI was enabled. The positive stored callback witness
remains red; its current diagnostic is covered by the private batch. No full
regression or timing. SQLite evidence:
runtime/results/native-view-finite-families-2026-10-10.json.

Installed consumers remain at f569190a. Warm CLI rebuilt for project formatting.

## Continuation prompt

Finish native-view-callable-storage-plan.md step 1 by integrating finite scoped
owning family identity into the registry without conflating it with the existing
borrowed invocation family. Preserve Owned/OwnedOnce invocation and each
alternative's candidate slots. Then add actual-callee provenance transport and
safe lifetime-bearing native emission before admitting values. Turn
native_view_record_callback_preserves_selected_input green with complete O0/O3
emission and owner-escape rejection. Keep ordinary callable behavior, mixed record
controls and public factory guards. Use focused checks, normal-stack canaries for
shared analyzer changes, project formatting and exact SQLite evidence.
