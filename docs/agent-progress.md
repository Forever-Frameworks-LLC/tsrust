# Agent continuation state

Updated: 2026-10-26. Memory investigation and scope reconciliation in progress.

Current owner priority: integrate audited benchmark dataset commit
`b35e527f3dce8242a7155dd87d27b5f32152d476`, validate the public measurements
database read-only, then inventory and prune verified disposable generated outputs.
Do not import cleanup logs or source snapshots into analytics. Heavy continuation
is stopped for this priority change; do not rebuild merely to repopulate caches.

- Repository cloned to D:\ai_dev\tsrust; project skills and continuation goal
  added locally. No push, workflow dispatch, tag or release performed.
- Start with `handoffs/compiler-to-roadmap-continuation.md` and
  `handoffs/compiler-pause-memory-pressure.md`. Historical source checkpoint:
  `1085b9c8`. Verify current HEAD and subsequent commits before relying on it.
- Unresolved: long-lived LSP memory pressure; Table candidate SDK/LSP repair
  acceptance. Earlier macOS process paths, installed consumers and port state do
  not describe this Windows clone.
- Original HEAD: `7c956bd7`; local memory checkpoint: `7b18073a`. Owner changes remain `AGENTS.md`, `.agents/`,
  `docs/agent-goal.md` and this progress file. Preserve these owner/setup files.
- Required-scope checklist: [continuation acceptance](plans/continuation-acceptance-checklist.md).
  Newer S01–S21 evidence supersedes stale compiler-ledger cells; D families still
  require individual reconciliation. Proposals and explicit deferrals stay separate.
- Active evidence: [LSP memory investigation](milestones/lsp-memory-investigation.md).
  Trace found cached snapshots retain unrelated open-buffer revisions. This is a
  hypothesis requiring a focused ownership witness, not a confirmed full diagnosis.
- Windows prerequisites: Node v24.19.0; pinned Rust 1.96.1 installed by rustup;
  Visual Studio 2022 Community present. Approximately 16 GiB physical memory was
  available. No Tsrust consumer or target existed at start. Existing editor-owned
  Cargo metadata/check processes were observed; preserve them and monitor contention.
- Initial one-job, nonincremental CLI/LSP build failed in compiler build.rs because
  offline vendoring lacked `ab_glyph v0.2.32`. Locked native dependency fetch is
  complete. Library built; CLI then exposed an ungated Unix watcher reference.
  Guarded the helper/test in `src/cli/docs_cli.rs`; subsequent CLI/LSP build passed.
  Shared target: `D:/ai_dev/tsrust/target`. Build bounds and reproducer stop
  conditions are recorded before any memory reproducer launch.
- No compiler, LSP, native, browser or benchmark checks run during skill setup.
- `benchmarks/records.sqlite` is absent. Preserve all loose historical evidence;
  do not claim locally verified historical bytes or remove verified-copy candidates.
- Local in-progress checkpoint `7b18073a` preserves the memory section. Whole-project formatter passes (zero source or
  fingerprint changes), after locked tooling installation. CLI version passes.
  No normal editor, preview, full regression or timing started.
- Active source changes: observed-only cached snapshots in `workspace.rs`,
  `workspace_scan.rs`, `project_session.rs`; superseded scan-entry release;
  Windows local file URI conversion in `project_diagnostics.rs`; Windows URI
  regression and portable `tests/support/session.rs` URI generation. Shared
  canonicalization now preserves existing ancestor prefixes for unsaved files.
  `tests/checker_agreement.rs` uses host-absolute virtual paths. Two focused
  retention unit witnesses are ready. `tools/lsp-memory-probe.mjs` adds the bounded
  real-server diagnostic. These changes remain unaccepted.
- Checks passed: `git diff --check`, probe syntax, both retention witnesses,
  Windows URI round trip, all eight workspace-scan tests and the initial 76-request
  protocol probe. Initial failures and repairs are described in the linked record.
  Required three editor suites run in exec session 63793; exact log is
  `target/lsp-memory-editor-checks-current.log`. Expanded 12-cycle memory probe
  passed: 890 requests, 43 samples, 61,849 ms, maximum sampled private commit
  6,787,072 bytes. Preserve both JSON records and the original failed suite log.
  Next: finish those checks, repair only affected failures, finish formatting
  idempotence and make a local accepted checkpoint. Then resume
  exact Table witnesses from the pause handoff. No CI or release action performed.
- Table prerequisite read-only dry-run failed on Windows:
  `tsrust ui add table --cwd docs --dry-run --json` reports
  `UI path must be a nonempty relative path without '.' or '..': src/components/ui\table.tsrx`.
  The installer joins host paths before calling its portable path guard. No Table
  file or lock was changed. Repair generated path handling without weakening path
  traversal/symlink protection after checkpointing the memory section.
- Installer prerequisite repair is now implemented but unvalidated in
  `src/cli/ui_cli.rs` and `src/cli/ui_attributions.rs`: normalize only generated
  component/versioned/attribution paths to portable separators. Authored config
  and catalog paths still pass the original strict guard. Existing versioned
  installation, local-edit preservation and lock-idempotence witnesses remain
  acceptance requirements. No docs source or lock has changed.
- The checker-agreement test was stopped after a third Unix-only virtual path
  was found inside `callable_field_agrees`; its output/abnormal exit remain in
  the current log. Source now uses the same host-absolute URI helper there too.
  This was a missed fixture, not an accepted agreement result. The no-fail-fast
  run continues completion_matrix and editor_consistency in session 63793.
  Preserve their results and rerun checker_agreement only, with `--nocapture`
  to expose receiver progress. The bounded memory probe has exited normally.
- Completed editor run: editor_consistency passes all 27 tests in 39.66 seconds.
  The complete completion matrix evaluated every row/mode, with eight failures
  confined to the std/project import-path rows; all other row evidence is retained.
  Root fixes in `src/position_completion.rs` use canonical host paths for the
  compiler-confirmed module probe and shared file-URI decoding for sibling files.
  The new encoded-URI sibling witness passes. Affected import-path rows and the
  module-inventory check pass (three tests, 7.31 seconds); project_diagnostics
  passes all 17 tests (0.81 seconds). Logs: `target/lsp-import-path-acceptance.log`.
- Current checker_agreement retry runs directly from its immutable built test
  executable, with `--test-threads=1 --nocapture`, in exec session 57435. Progress
  and exact output: `target/lsp-checker-agreement-final.log`. The owner changed
  priority while this run was at receiver `Time`; it was stopped, not passed.
  Preserve the completed receiver progress and do not describe it as a full pass.
- Current one-job Cargo command, session 50071, builds/runs the focused compiler
  CLI installer witness. Log: `target/windows-ui-installer-checks.log`. It is
  rebuilding the compiler library for the dev-dependency feature combination;
  sampled rustc private commit remains below 2.3 GiB. No LSP binary refresh occurs
  during the checker run. It completed in 14m 53s. The generated-path witness,
  versioned install/idempotence/owner-source witness and Kbd upgrade witness pass.
- Both exact Table editor witnesses still fail: TableProps.colSpan hover has the
  correct type but lacks its `1000` behavior documentation; the multi-import generic
  inheritance fixture has null member hover. Do not weaken those assertions.
  `target/table-editor-probe.json` reproduces null base/override/generic member
  hover with empty diagnostics. A focused formatted header/checked-index regression
  is added in `src/project_intelligence.rs` but not run yet. Trace parameter access
  wrappers and inherited field documentation provenance before repairing source.
- The focused inheritance witness exposed an invalid old fixture: the compiler
  rejects redeclaring `value` on an interface extending a type that already has it.
  The valid documentation cases now use an independent local field declaration;
  a separate strict rejection preserves the duplicate-inherited-field boundary.
  The initial strict failure is in `target/table-inherited-help-unit.log`.
  The next compile failed because ProjectAnalysis has no Debug implementation;
  changed the test's `unwrap_err()` to `err().unwrap()`. This final witness remains
  unrun. Table documentation behavior itself is not repaired or accepted.
