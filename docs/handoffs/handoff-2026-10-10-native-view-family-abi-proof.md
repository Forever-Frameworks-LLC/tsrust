# Private native owning callable family ABI proof

Read AGENTS.md, docs/development-workflow.md and
plans/native-view-callable-storage-plan.md. Continue from f6d3e68d. Gate 189/S07 remain open.

The test-only native_view_family_emit.rs backend renders finite enum alternatives
using typed concrete environment lifetimes. A consuming call joins the required
input lifetime sources separately for each output path. The existing output lifetime
planner was extracted into a shared helper; its named-source constructor is currently
test-only. Source owning storage remains rejected and the backend is not yet wired
into normal emitted callable families.

Parsed/formatted factories and normal closure bodies run in the family proof at
O0/O3 for readonly and exclusive byte views. Both branches preserve pointers;
exclusive writes reach their owner. Independent native controls reject owner escape,
second invocation and conflicting mutation. Invalid invocation and missing capture
slots leave output unchanged. Original red and successful evidence is in SQLite:
runtime/results/native-view-family-abi-proof-2026-10-10.json.

No public source feature, full regression or timing acceptance. Non-fallible consuming
families with scalar arguments are the initial backend scope. Reusable invocation,
data arguments, fallible ABI, source storage, nested family wrappers and public
editor/catalog remain pending. The stored callback positive witness is still red.
Warm CLI rebuilt for formatting; installed consumers remain f569190a.

## Continuation prompt

Connect the prepared family registry, actual-callee lifetime transport and private
native family backend to admitted scoped owning source storage. Preserve its
separation from ordinary lifetime-free owning and borrowed invocation families.
Extend the ABI for required data/fallible/reusable forms rather than silently
replacing them with narrower signatures. Turn
native_view_record_callback_preserves_selected_input green through normal analysis
and full emission, then test alternative input origins, moves, replacement, arrays,
nullable fields and owner escape. Keep the public byte factories guarded until the
complete view/observer/editor matrix is ready. Use bounded TDD, depth canaries after
shared analyzer changes, project formatting and exact SQLite evidence.
