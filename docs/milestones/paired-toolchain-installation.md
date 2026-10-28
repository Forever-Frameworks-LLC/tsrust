# Paired toolchain installation — focused platform acceptance

2026-10-28. Focused macOS arm64 and Windows x64 acceptance is recorded below;
Linux acceptance is recorded below; published-release installation remains pending. No release was
published to obtain this evidence. Follow plans/paired-toolchain-updates.md for
remaining platform acceptance and guides/toolchain-installation.md for usage.

## Implemented behavior

`tsrust toolchain install --root NEW_DIRECTORY` creates an owned installation from
an official immutable stable release. `check [--json]` is read-only; `update`
requires a newer stable version; `rollback` switches to the retained verified
pair without network access. Existing package-manager/unmanaged directories are
never adopted or overwritten. Rust tools, shell profiles and editor settings are
not changed. `tsrust update <crate>` retains its existing meaning.

Release metadata is acquired asynchronously over verified HTTPS from the fixed
GitHub repository API. Host, immutable/stable status, exact asset origin, size and
API-supplied digest are required. Downloads and extraction are bounded. The tar
reader is pinned at 0.4.46 with optional xattr disabled. Links, special entries,
traversal, duplicate/case-colliding paths, missing/unlisted files, modified hashes,
wrong host and mismatched executable versions fail before selection. The CLI and
LSP each expose --version; staged bytes must agree with their separate manifest
versions. Trust is in GitHub/repository metadata and HTTPS, not independent signing
or notarization.

A versioned installation record selects one immutable CLI/LSP pair. The existing
CLI also supplies the two small dispatch entry paths, sharing its launcher inode
where hard links are supported. Launchers verify the selected executable and
hold a shared selection lock only until it starts. Unix uses exec; Windows reuses
the existing owned-job process helper so terminating the launcher cannot orphan
the LSP. Updates never overwrite running executable files. Mutations serialize;
interrupted verified staging can be reused; failed stages preserve the selected
pair; rollback revalidates the prior files. Doctor checks the selected manifest
and every recorded file without querying for updates.

## Executed evidence

- Both rebuilt CLI/LSP consumers compiled successfully.
- Seven focused installer tests passed initially and again after review repairs.
  The final pass took 15.84 seconds, excluding compilation.
- A real isolated installation ran the compiler check and the actual LSP initialize,
  hover capability, shutdown and version responses with development tools absent
  from PATH and Rustup home. Doctor distinguished healthy installation files from
  missing Rust tools and detected a tampered LSP.
- Controlled executable fixtures proved pair selection, an older running process
  surviving update, concurrent mutation rejection, rollback, no implicit downgrade,
  bad archive rejection, mismatched executable-version rejection, orphaned verified
  stage reuse, interrupted-record cleanup and owner-file preservation.
- Archive link rejection and pure metadata/version/path/redirect rejection controls
  passed. These are deterministic transport-boundary tests, not a published artifact.
- Backend compiler/LSP all-target diagnostics passed without warnings or errors.
- All 14 existing release-policy and distribution tests passed. Workflow YAML parsed;
  PR tests remain stag-only and manual Windows checks report Toolchain acceptance,
  never the mandatory CI gate.
- The real HTTPS latest-release check returned the expected no-stable-release
  diagnostic; it downloaded no archive and changed no installation.
- Whole-project formatting and format check passed with zero changed files or
  refreshed fingerprints; git diff --check is clean.

Current disposable logs: /tmp/tsrust-toolchain-final-tests.log,
/tmp/tsrust-toolchain-final-check.log and /tmp/tsrust-toolchain-policy-tests.log.
Do not import them into benchmark analytics. No performance claim is made.

## Pending

The live-LSP update witness has macOS, Windows and Linux acceptance below. Published artifact
acquisition remains unexecuted because no stable release exists. Full backend
regressions, matched performance and the wider documentation audit remain separate
goal requirements.

## Windows follow-up acceptance

The Windows toolchain/shared-cache job passed at
`9b6588570a283b14e8d61b5e4892ca33afbe7136`:
[run 40731485167](https://github.com/Forever-Frameworks-LLC/tsrust/actions/runs/40731485167).
It builds both consumers, prepares portable sources, runs all installer,
shared-source and package-management tests, and checks every compiler/LSP target.
This includes the locked-process installation witness and the repaired deep-path
and directory-sync cases. The job and aggregate Toolchain acceptance check both
completed successfully. This result covers that source commit, not subsequent
local editor/documentation changes; it did not publish a release.

## Live LSP update acceptance on macOS

The real consumer test now initializes an installed LSP, updates the selected
pair, verifies new launches select version 1.1.0 fixtures, then opens a newly
formatted document and obtains its integer hover from the still-running original
LSP. It shuts down cleanly, rolls back to the original real pair, and retains the
existing doctor/tampered-file checks. PATH and Rustup home remain empty for the
managed consumers. The successor pair is a version-reporting fixture; this proves
selection and old-server continuity, not execution of a future released LSP.

Focused test 24646 passed in 19.16 seconds after a 5m22 dependency/compiler build,
with no warnings or errors. Disposable log: /tmp/tsrust-live-lsp-update.log.
Windows execution also passed at `86435b8846badcfaf836af9c36da3fa56379f3a6`:
[run 40748893795](https://github.com/Forever-Frameworks-LLC/tsrust/actions/runs/40748893795).
The job and aggregate Toolchain acceptance both succeeded. Installer/process tests
passed 8/8 (39.45s), shared-source tests 5/5 (219.12s), and package-management tests
7/7 (332.51s). Both consumers built and backend all-target checking passed.
The installer witness source is unchanged in local checkpoint e33f4674. This proves
the live protocol sequence and locked-process behavior on Windows, but not the
website's fresh-machine PowerShell setup commands or published artifact acquisition.
Linux acceptance is recorded below. Disposable log:
`/tmp/tsrust-windows-live-lsp-acceptance.log`.


## Linux staging acceptance — 2026-10-28

[Run 40748887685](https://github.com/Forever-Frameworks-LLC/tsrust/actions/runs/40748887685)
completed successfully at `86435b8846badcfaf836af9c36da3fa56379f3a6` on Ubuntu 24.04.
The unchanged installer witness passes with actual LSP initialization, live update,
post-update hover and rollback. All 7 Linux installer cases pass (51.49s);
shared-source tests pass 5/5 (115.57s) and package-management tests 7/7 (191.71s).
Windows has an additional platform-specific process case, so its count is 8.

The same run passes the bounded compiler/CLI baseline, real native SDK replica
integration, complete checker agreement and completion matrices, editor consistency
and stdio. The aggregate CI check succeeds. This is the selected staging baseline,
not every integration target or a release-package/fresh-download test. Subsequent
local changes require their own accepted evidence. Current disposable log:
`/tmp/tsrust-linux-staging-acceptance.log`.
