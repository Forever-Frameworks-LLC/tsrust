# Handoff: B9 accepted

Date: 2026-10-10. Previous handoff: `docs/handoff-2026-10-10-b8-accepted.md`.
Record: `docs/milestones/185-project-editor.md`. Map:
`docs/b9-project-editor-acceptance-map.md`. Guide: `docs/project-editor.md`.

## State

- B9 (S04 project editor) is accepted at gate 185.
- B14 is closed by owner decision: `switch` fallthrough stays rejected.
- The owner allowed the formatter to rewrite every file (2026-10-10). The fifteen
  fixtures that used to stay modified and unstaged are committed in their formatted
  state (`fa69d75c`). There are no protected files any more; the tree is clean after
  this gate's commit.
- The five consumers are refreshed from this gate's source commit.

## Read this first: regression summaries

Two compiler suites aborted with a stack overflow in every whole-crate run from the B5
carrier work to B7 acceptance, and the summaries did not show it. Gate 185 repairs the
cause and its record explains it. After each `cargo test` run, check all three:

```
grep -E "error: test failed|overflowed its stack|FAILED" <log>
```

and that no suite is missing a `test result` line. The depth tests
(`rest_normal_depth_and_over_limit_keep_the_original_worker_stack` and the last loop of
`exact_math_static_typing_arity_capture_and_constant_write_diagnostics_are_preserved`)
are the canaries for frame growth in `analysis.rs`. Keep recursive checker functions
small: move work around the recursive call into `#[inline(never)]` functions.

## Decisions taken without the owner in gate 185

- A closed-file quick fix is guarded by a content comparison at request time. A change
  between the offer and the application by the editor is not detected. The guide says
  so.
- Header loading drops an imported name that is not exported instead of failing, so
  one unfinished name no longer removes all project help. Strict checking still
  reports it.
- Two open URIs of one file share the most recent text. This was the existing behavior
  and is now recorded as a defined limit.

## Not done

- The packaged-extension suite (`npm run test:integration` in `editors/vscode`) was not
  run; it opens a VS Code window.
- Clippy with warnings denied stops on older style findings in the compiler crate.
- 49 of 441 `node --test benchmarks/runtime/*.test.mjs` tests fail at accepted commits
  (stale pinned identities and one `ReferenceError`); tracked as a separate task.

## Ledger

Accepted: B1 to B9, B15, D1. Closed by decision: B14. Next in order: B10 (S06 tagged
`Result`, exhaustiveness, typed propagation), B11, B12, B13, then D2 to D5. Structural
record keys for data records need their own row.

## Working rules

Follow `AGENTS.md` and `docs/development-workflow.md`. Stage explicit paths. Use
`CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target`, `CARGO_INCREMENTAL=0`,
`--offline --locked`, Node 24.11.1 first on `PATH`, and `/opt/homebrew/bin/python3` for
`benchmarks/storage/store.py`.
