# Tsrust handoff to Claude — 2026-10-28

The owner requested this handoff and stopped Codex continuation. The goal is
unfinished. This document supersedes the next-step instructions at the top of
`docs/agent-progress.md`; its older entries remain historical evidence.

## Start here

Repository: `/Users/seancotter/Desktop/dev/learning/tsrust`.
Branch: `codex/language-cli-docs-polish`.
Last implementation/docs checkpoint before this handoff: `ec92fbc75e28ed21092ac647f4e60783300ec1be`.
The handoff itself is a separate documentation-only checkpoint. No push or release
was performed during cleanup.

Read `AGENTS.md`, `docs/development-workflow.md`, `docs/agent-goal.md`, then this
handoff. Use `docs/agent-progress.md` to locate detailed evidence, rather than
repeating its old continuation instructions. It is long and includes superseded
plans. Inspect current source/status before acting.

## Most recent owner correction — highest immediate priority

The owner said:

> im basically just seeing a lot of example how to documents appearing instead of
> actual documentation of the std lib and how the language works - it would be
> better to have an interactive guide of how tos than to just have a bunch of
> articles hogging up the search results

The current site over-fragments language topics into many small walkthroughs,
especially callable/generic features. Do not continue producing those articles.
Their verified facts/examples are reusable evidence, not an approved final docs
architecture. The initial `docs/plans/public-documentation-site.md` already proposed
broad language-reference topics; implementation drifted from that intent.

The requested direction is:

- Make language semantics and standard-library API reference the primary docs.
- Consolidate how-tos into an interactive guided learning experience; do not index
  every tutorial step as another competing reference article.
- Keep examples within reference entries where they explain an operation, without
  making examples the whole documentation model.
- Plan the revised structure in Markdown, then implement it incrementally. Preserve
  useful audited contracts and migrate links/anchors safely. No empty placeholders.
- Every page still needs factual and presentation review. Reorganization does not
  establish correctness, and a successful build does not establish a completed audit.

No revised navigation, consolidated reference, interactive guide or API-reference
generator has been implemented. Codex stopped after inspecting options. The
information-architecture skill was read, but no `.design` artifacts were created.

Useful implementation facts: Astro 7.4.2, Starlight 0.42.7; site configuration is
`website/astro.config.mjs`, content is `website/src/content/docs/`. Installed
Starlight schema supports frontmatter `pagefind: false`. Reuse its navigation,
styles and accessibility instead of adding another framework. Do not hide useful
reference content before its replacement exists. Preserve old URLs through the
migration, then remove superseded public source copies.

`crates/tsrust-lsp/src/builtins.json` has 103 groups / 894 entries, including
signatures, parameter documentation, behavior, ownership and examples. It is a
useful canonical source for a synchronized stdlib reference. Some group keys are
internal names; do not expose them as public types without mapping them. The
existing library overview lists 23 public module names, but detailed API reference
coverage is still incomplete. `std:path` (7 exports) and `std:env` (3 entries) were
only inspected as possible starting points; no generation work was done.

## Full goal and remaining work

Finish the finite backend-only language/CLI/docs/platform polish from PR #2. Do
not expand into native UI or arbitrary TypeScript compatibility. The original fan
stop was explicitly lifted: the owner is using a different computer.

- Async testing registration and HTTP body/SDK work have substantial accepted
  evidence in `docs/agent-goal.md` and the progress record; preserve that evidence.
- Shared dependency caches and paired CLI/LSP installation/update have implemented
  workflows and focused macOS/Linux/Windows evidence. Remaining distribution
  checks must be reconciled, not assumed complete.
- Windows and Linux installer/locked-binary checks passed previously. The newer
  isolated editor UTF-16 fixture and fresh toolchain smoke/acquisition paths still
  need their outstanding platform executions. Exact release-build commands and
  actual published acquisition remain unproven where marked pending.
- GitHub Pages was selected and configured, but a successful production deployment
  and public URLs remain pending. Local preview/build is not deployment evidence.
- Complete the finite remaining docs inventory in the migration/audit plans, using
  the corrected reference/learning architecture. Old pending/unsupported claims
  often contradict newer compiler behavior; check source before copying them.
- Retain completed family correctness/performance records. Do not restart full
  matrices or timing protocols after routine documentation changes.

A reported LSP retention issue was repaired and a bounded 30-minute macOS stress
check passed. The old blanket warning was removed. This is not proof of every
possible long-session behavior. HTTP performance evidence and original SQLite
records already exist; no new measurements were taken during recent docs edits.

The last retained nested-function guide still contains unaudited owned/scoped
selection, single-use call scopes, generic replacement and mutable factory arrays.
Its source-flow paragraph was investigated, but is not a mandate for another page.
Other language/library/operational inventory remains pending too.

## GitHub and release constraints

Original PR #2 was bootstrapped with owner approval. Last recorded draft delivery
PR is https://github.com/Forever-Frameworks-LLC/tsrust/pull/4, base `stag`, remote
branch `codex/backend-polish-staging`. Last recorded remote head was
`61ee67a44152437848a53d0fea6499e7ffec56ec`; its tree matched local `1b375f06`.
Later local work, including the recent docs commits, has not been pushed.
These are last-recorded values, not a fresh remote-state check. Revalidate before
any push. Do not force-push or trigger duplicate CI runs.

Owner-approved policy: branch-to-stag PRs run tests and require one review, with
owner review bypass; stag-to-main promotion compiles macOS/Linux/Windows artifacts,
then auto-merges/publishes if successful. New tags may be created by repository
writers; existing tags/releases must remain immutable. Prefer caching, minimal
permissions and dependable artifact reuse. Avoid unnecessary GitHub Actions usage.
No concrete new release version was selected; do not invent one or publish a new
release as part of this handoff.

## Validation and cleanup state

Recent accepted docs evidence is indexed in
`docs/plans/public-documentation-content-audit.md` and
`docs/plans/public-documentation-migration.md`. Their content-audit acceptance does
not mean the owner approved the fragmented article architecture.

The unfinished `language/callable-control-flow.md` article was deleted; its sidebar
addition and source-guide edits were reverted. No unfinished implementation remains.
Before deletion its two intentional rejection examples returned `arity`, and two
native examples printed `4 1` / `7` and `7 1`. Focused compiler flow checks passed
15 tests and editor diagnostics passed 4. Disposable logs, if still present:
`/tmp/tsrust-flow-doc-contracts.log` and `/tmp/tsrust-flow-doc-editor.log`. These are
optional current-run evidence, not durable analytics or required deliverables.

Cleanup rebuilt the restored 66-page site successfully (`npm --prefix website run
build`), removing the deleted article from generated output. The last full internal
link/anchor pass on that same committed site also passed. The agent-started docs
preview on port 4322 was stopped, its browser tab closed and viewport override reset.
No known compiler/test job is running. Restart preview if needed with
`npm --prefix website run dev -- --host 127.0.0.1 --port 4322`.

198 disposable native program cache entries (274 MiB) were pruned from
`target/tsrust-native/v2`; root lock files, shared native runtime cache and Cargo
target were preserved. Native programs rebuild when needed. Source, owner edits,
retained UI, fixtures and historical evidence were not removed.

## Working rules

One agent in this tree at a time. Inspect status first. Use the existing Cargo
target, `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`, and focused section-level checks.
Do not touch the owner's VS Code instance. Format the entire project before each
source checkpoint with `TSRUST="$PWD/target/debug/tsrust" node tools/format-project.mjs`.
Keep native behavior, diagnostics, IntelliSense and examples consistent when code
changes. No new heavy suite for prose alone.

No source snapshots/archive directories. SQLite remains canonical analytics;
never publish ignored `benchmarks/records.sqlite`. Public measurements must use
`benchmarks/storage/public.py` and its provenance allowlist. Preserve original
records and distinguish unmeasured work from measured work. Keep the handoff and
finite acceptance checklist current; do not claim completion from partial evidence.

## Claude progress — 2026-10-28

Architecture recorded in `docs/plans/documentation-architecture.md` (reference
first, one interactive Learn track per journey, chapter merge table and rules).

- Standard-library and built-in reference: `website/tools/generate-api-reference.mjs`
  renders all 103 catalog groups (minus frozen `std:ui` and the internal testing
  runtime) into 28 generated pages before every `dev`/`build`. Output is ignored
  by Git. The docs workflow now also triggers on `builtins.json` changes.
- First language chapter merged: `language/optional-values/` replaces
  borrowed-optional-fallback, optional-chaining, nullish-coalescing and
  nullish-assignment. Old routes redirect to chapter anchors. All ten examples ran
  with the current debug compiler with matching output; five rejection claims
  were checked with `tsrust check`.
- Link checker now validates redirect pages against their destination anchor
  (unit test added). Site build: 91 pages + redirects; link/anchor check passed.

Not done: remaining nine chapter merges, the first Learn track, folding library
guides and backend how-tos into tracks, and the CI/installation/platform/deployment
reconciliation listed above. Nothing was pushed; no GitHub run was triggered.

## Distribution, CI and deployment reconciliation — 2026-10-28

Checked against workflow source and read-only `gh` queries; no run was triggered.

| Requirement | Current state | Evidence |
| --- | --- | --- |
| Release artifacts for macOS, Linux, Windows | Workflow builds `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`. Never executed; no release exists. | `.github/workflows/release.yml` matrix; `gh release list` empty |
| Windows paired install, locked binaries, shared caches | Passed on hosted Windows. An earlier failure (40724567590, 90382caa) was fixed by 9b658857. | Runs 40731485167 (9b658857), 40748893795 (86435b88) |
| Async testing on three platforms | Passed. | Run 40672007920 (c5b3d4a9) |
| PR CI on the staging branch | Linux compiler/LSP, VS Code extension and docs build passed for PR head 61ee67a4. | Runs 40761578381, 40761578389 |
| macOS | Local focused acceptance only (see `docs/agent-goal.md`). | Local evidence |
| GitHub Pages | Configured (`build_type: workflow`, public URL reserved) with zero deployments. Deploy runs only on `main` after promotion. | `gh api repos/.../pages`, empty deployments list |
| Install/self-update from a published release | Pending: needs a release, which needs owner authorization. | — |
| Windows editor UTF-16 fixture | CI now has a dispatch-only `editor_acceptance` mode that runs the VS Code integration and recovery suites (including the surrogate-pair fixture) on Linux and Windows. Not yet run: needs a push and an owner-approved dispatch. | `.github/workflows/ci.yml` job `editor_integration` |

Remote branch `codex/backend-polish-staging` (PR #4, draft, base `stag`) is still at
61ee67a4. Local history is not a descendant of it (the PR branch was rebuilt
from the same trees), so compare trees before any push. All local commits since
then, including this documentation restructure, are unpushed.

## Release candidate status — 2026-10-29

- Editor acceptance passed on Linux and Windows (CI run 40816862762), including
  the surrogate-pair UTF-16 diagnostic fixture.
- The five package-export tests pass locally once `tsrust setup --portable
  --offline --yes` has prepared source packs.
- Owner chose a pre-release. Version is `0.1.0-rc.1`. The toolchain installer
  now accepts `MAJOR.MINOR.PATCH-rc.N` (ordered before the stable version);
  update discovery still considers only stable releases, so self-update from a
  published release still needs a stable release to verify.
- PR #4 (`codex/backend-polish-staging` → `stag`) was merged by the owner with
  the review bypass (squash commit cf4e729c). Authors cannot approve their own
  PRs; the bypass is the owner's path for agent-opened PRs.
- PR #5 (`stag` → `main`) built all four targets and the extension (Release run
  40861023720). Promote (run 40864132322) merged it and published the
  `v0.1.0-rc.1` prerelease with four archives, `tsrust-0.2.0.vsix` and
  `SHA256SUMS`. The Documentation workflow (run 40864208241) deployed
  https://forever-frameworks-llc.github.io/tsrust/.
- Release smoke test on macOS arm64: the archive matched `SHA256SUMS`;
  `tsrust --version` printed `0.1.0-rc.1`, `tsrust-lsp --version` printed
  `0.2.0`; `doctor` and `doctor --native` passed; `tsrust run` executed a native
  program correctly. `tsrust toolchain install --root` refused as designed: "No
  published stable official toolchain is available yet".
- Pages smoke test: the home page, Learn, Language, globals guide, generated
  globals and `std:sql` reference returned 200; the old
  `reference/builtins/globals/` path redirects to `reference/globals/functions/`.
- Still open: managed install and self-update need the first stable release.
  Linux, Windows and Intel macOS archives were built and checked in CI, not
  installed by hand. This handoff commit is local only and reaches `stag` with
  the next PR.
