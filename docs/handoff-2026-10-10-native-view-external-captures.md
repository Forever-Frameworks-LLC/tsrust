# Complete external borrowed capture bodies

Continue from `ec5c0291`. Read AGENTS.md, docs/development-workflow.md and
docs/native-byte-views-implementation.md. Gate 189 and S07 remain open.

Three focused tests in isolated views_borrowed_emit_tests.rs cover real parsed
readonly and mutable reference capture bodies. Native type projection and call
linkage bypass only the publication guard. Normal analysis and emission produce
the complete source functions, closure methods and native bridges.

Readonly capture: source observe returns the Result view after its local receiver
ends. O0/O3 verifies contents and pointer identity against the external owner.
Independent native compilation rejects escaping the external owner.

Mutable capture: source observe invokes its closure and discards the Result.
O0/O3 also invokes the actual emitted closure directly, writes through the returned
view and checks external owner contents after receiver scope ends. Source analysis
rejects returning that view from observe. Independent native compilation keeps
the external owner alive while rejecting a view escaping the local receiver.

All three tests passed. No production behavior changed, so preceding broader
evidence remains applicable. Project formatting changed zero files/fingerprints.
No full regressions or timing. SQLite record:
`runtime/results/native-view-external-captures-2026-10-10.json`.
Public APIs remain guarded; installed consumers remain f569190a.

## Continuation prompt

Continue gate 189 with deeper escaped/rebound environments, mixed callback records
and specialized callback/storage families against the implementation-plan matrix.
Then finish readable/mutable observer contracts and public editor/catalog parity.
Keep public preparation guarded until the required matrix agrees. Preserve real
slices and explicit ownership. Use focused TDD, bounded checks, project formatting,
SQLite exact evidence and explicit-path commits. Refresh consumers before checking
their executables. Full regressions and timing belong to S07 completion.
