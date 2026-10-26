# Agent continuation state

Updated: 2026-10-26. Memory investigation and scope reconciliation in progress.

- Repository cloned to D:\ai_dev\tsrust; project skills and continuation goal
  added locally. No push, workflow dispatch, tag or release performed.
- Start with `handoffs/compiler-to-roadmap-continuation.md` and
  `handoffs/compiler-pause-memory-pressure.md`. Historical source checkpoint:
  `1085b9c8`. Verify current HEAD and subsequent commits before relying on it.
- Unresolved: long-lived LSP memory pressure; Table candidate SDK/LSP repair
  acceptance. Earlier macOS process paths, installed consumers and port state do
  not describe this Windows clone.
- Current HEAD: `7c956bd7`. Owner changes remain `AGENTS.md`, `.agents/`,
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
- Preparing a local in-progress source checkpoint. Whole-project formatter passes (zero source or
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
