# Actual expression callee lifetime transport

Read AGENTS.md, docs/development-workflow.md and
plans/native-view-callable-storage-plan.md. Continue from 60ec23c9. Gate 189/S07 remain open.

Concrete expression callbacks now register native summaries, including conditional
callees and specialized method origins. Captured return loans follow the actual
callee expression via iterative walker tasks. Named calls still resolve their
binding from CallTarget::Closure when the callee name lacks a binding annotation.
Missing capture slots report native-view-call instead of erasing a returned loan.

The new field control is a checked-expression transport test, not a public source
storage acceptance test. It retains the selected owner's identity and rejects an
unknown capture slot. The conditional callback control uses fully parsed/formatted
source with private native type projection. Analysis retains both possible input
origins; normal native emission at O0/O3 verifies each branch's pointer and rejects
owner escape in independent native compilation.

The first view batch exposed four named-call regressions after the generic callee
walker was introduced. Those are repaired by retaining typed target binding identity;
original failed logs remain in SQLite. The missing-slot control also has its own
retained red witness. Final private checks, ten normal-stack canaries plus Result,
and native editor checks are recorded at:
runtime/results/native-view-expression-transport-2026-10-10.json.
No full regression or timing. Public APIs remain guarded; the stored callback positive
witness remains red. Warm CLI rebuilt for formatting, installed consumers f569190a.

## Continuation prompt

Use actual-callee capture transport to integrate prepared owning family alternatives,
then add admitted scoped owning storage and safe lifetime-bearing native family
emission. Source::Receiver still requires a stable binding; do not claim receiver
field storage acceptance from the carried-view tests. Preserve physical reference
rejections, all possible alternative origins and missing-slot diagnostics. Turn
native_view_record_callback_preserves_selected_input green through complete O0/O3
normal emission with owner-escape rejection. Keep public factory guards until the
full view/observer/editor matrix is complete. Use bounded checks, depth canaries
after shared dispatch changes, project formatting and exact SQLite evidence.
