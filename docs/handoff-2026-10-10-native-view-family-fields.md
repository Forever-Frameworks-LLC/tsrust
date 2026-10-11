# Native view family independent fields

Read AGENTS.md, docs/development-workflow.md and
plans/native-view-callable-storage-plan.md. Continue after cbaee6c6. Gate 189/S07 are open.

The private native family backend now proves separate output lifetimes for a record
containing two views. Both factory alternatives execute at O0/O3 for readonly and
exclusive views. A first field survives the second field owner's scope; both pointers
are checked and exclusive writes reach the survivor. Independent Rust rejection
proves that the second field cannot escape its owner. A test-only emitter defect was
also repaired: equal slot counts no longer permit wrong access or capture paths.
Complete slots are compared with the checked closure before output is appended.

Evidence: SQLite runtime/results/native-view-family-fields-2026-10-10.json.
No public source acceptance, full regression or timing. Installed consumers remain
f569190a. The positive stored-callback source witness is still red.

## Continuation prompt

Connect scoped owning native-view families to normal analysis and emission. The
private ABI now has simple, Result and independent multi-field controls. Stop adding
ABI proofs as a substitute for source integration. Preserve a separate typed scoped
owning identity from lifetime-free owning and borrowed-invocation families. Transport
captured loans through construction, parameter seeding, actual callee fields and
family alternatives. Turn native_view_record_callback_preserves_selected_input green
through the normal source path, retaining owner-escape rejection. Then cover moves,
replacement, arrays, nullable fields and required reusable/fallible/data argument
forms. Public factories stay guarded until observers, editor examples and matched
benchmark support are complete. Use bounded TDD and project formatting; preserve
exact failed and successful evidence in SQLite.
