# Paired toolchain installation — focused platform acceptance

2026-10-28. Focused macOS arm64 and Windows x64 acceptance is recorded below;
Linux execution and published-release installation remain pending. No release was
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

Execute the new Linux acceptance through staging CI.
The current process fixture demonstrates platform file/process behavior, while
the separate real consumer witness verifies actual CLI/LSP startup. Neither a
macOS pass nor suffix-only fixtures prove Windows behavior. Published artifact
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
