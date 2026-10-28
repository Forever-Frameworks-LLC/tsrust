# Complete owned capturing factory witnesses

Continue from `c4094e28`. Read AGENTS.md, docs/development-workflow.md and
docs/milestones/native-byte-views-implementation.md. Gate 189 and S07 remain open.

Three focused private tests now cover complete source factory bodies for closures
owning Bytes, returning readonly and exclusive Result views. The test helper projects
native annotations and links calls while public preparation remains guarded. Normal
analysis infers capture storage, invocation and return provenance. Normal emission
generates the full factory, closure body and bridge; no summaries or native body
fragments are injected.

O0/O3 execution checks pointer identity and reacquisition. Exclusive writes reach
the buffer and appear in a later view. Independent Rust rejection controls show
both returned views cannot outlive their owning closure. Source callers reject
returning such views and moving the closure while a view is retained. Moving the
closure before taking a view is valid. Both readonly and exclusive forms are checked.

All three focused tests passed. No production edits were required, so the preceding
checkpoint's broader checks remain applicable. Whole-project formatting changed
zero files/fingerprints. No full regressions or timing. SQLite evidence:
`runtime/results/native-view-owned-factories-2026-10-10.json`.
Installed consumers remain f569190a; public APIs and catalog integration stay guarded.

## Continuation prompt

Continue gate 189 with complete shared external and mutable reference capture
bodies, then deeper escaped/rebound environments, mixed callback records and
specialized callback/storage families. Finish readable/mutable byte observer
contracts and public editor/catalog parity before lifting the publication guard.
Preserve real slices and explicit ownership. Use focused TDD, bounded checks,
project formatting, SQLite exact evidence and explicit-path commits. Refresh
consumers before executable checks. Full regressions and timing belong to S07.
