# Handoff: B8 accepted

Date: 2026-10-10. Previous handoff: `docs/handoff-2026-10-10-b7-accepted.md`.
Record: `docs/milestones/184-generic-adapter-compile-time.md`.

## State

- B8 (generic adapter compile time) is accepted at gate 184. No compiler source
  changed. The record holds the measured check times, the passing suites, the rejected
  in-place join experiment and the remaining cost profile.
- Compiler source is unchanged since `2c232b4e`. The five consumers stay at that build.
- The shared release and debug binaries in
  `/private/tmp/tsrust-gate100-editor-target` are built from current source.
- The fifteen protected owner fixtures remain modified, unstaged and untouched.

## Ledger

Accepted: B1 to B8, B15, D1. Next in order: B9 (S04 project editor), B10, B11, B12,
B13, then D2 to D5. Structural record keys for data records need their own row.

## Open items

- Owner decisions: B14 `switch` fallthrough; whether the formatter may rewrite the
  protected fixtures.
- 49 of 441 `node --test benchmarks/runtime/*.test.mjs` tests fail at accepted commits
  (stale pinned identities and one `ReferenceError`). They predate B7 and are tracked
  as a separate task.
- Optional: shared binding storage in `generic_call_validation` if a real program
  exceeds an interactive check time.
- B7 possible later work is listed in `docs/b7-caller-borrowed-acceptance-map.md`.

## Working rules

Follow `AGENTS.md` and `docs/development-workflow.md`. Stage explicit paths. Use
`CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`,
`--offline --locked`, Node 24.11.1 first on `PATH`, and `/opt/homebrew/bin/python3` for
`benchmarks/storage/store.py`. Copy the protected fixtures aside before
`node tools/format-project.mjs` and restore any that change.
