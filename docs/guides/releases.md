# CI and GitHub Releases

The repository description can be:

> A TypeScript-style language with inferred ownership, a Rust backend, native backend services and independent HTTP SDKs.

The source repository is intended for
`https://github.com/Forever-Frameworks-LLC/tsrust`. Links use that address; creating
or pushing the repository is a separate owner operation.

## Staged pipeline

Feature pull requests target `stag`. The aggregate **CI** check runs compiler unit
and CLI integration tests, LSP agreement, formatting, Clippy, distribution/privacy
checks and the VS Code extension on Linux. One approval is required; only
`Cotter45` may bypass review, including when merging another author's PR after
reviewing it. Tests remain mandatory. CODEOWNERS continues to request the owner's
review without requiring that the single approval come from that account.

A `stag` to `main` PR runs **Release gate**: optimized CLI/LSP builds and packaged
smoke checks on Linux, Windows and both macOS architectures, plus the VSIX package.
Once every artifact passes, the trusted **Promote and publish** workflow merges the
PR and publishes those exact artifacts. It checks the current head/base and verifies
that the merged source tree matches the built tree. Main requires an up-to-date
release gate and permits merge commits to retain staging ancestry.

These workflow files require a one-time live migration. Until
[the migration checklist](../plans/staged-release-pipeline.md) is complete, the
existing main CI/review rules remain authoritative. The publisher must already
be present on the default branch before the first promotion finishes. Source
configuration alone is not proof that the hosted release path has passed.

## Caching and cost

PR tests use Linux only. Release builds happen for promotion PRs, not every feature
PR. There is no duplicate push-triggered run. Superseded runs are cancelled.
Pinned `Swatinem/rust-cache` caches Cargo downloads and compiled workspace outputs;
platform, Rust version, lockfiles and separate test/release keys prevent incompatible
reuse. npm caches extension package downloads. Cargo still checks freshness and all
required tests run. GitHub scopes PR-created caches to that PR's merge ref, so
repeat runs can reuse them but a new PR may be cold. No speedup is promised before
measurement. Artifacts expire after seven days.

Builds use locked dependencies, one Cargo job, one test thread, disabled incremental
compilation and explicit timeouts. Local development uses local checks by default.
The standard suite is selected acceptance, not the entire regression matrix.
Manual `full_regressions: true` runs the broader workspace suite at milestones;
performance benchmarks and database-service checks do not run automatically.

## Repository setup

Keep read-only default Actions tokens and pin external actions. Protect `stag`
with required CI and a separate one-review rule whose sole bypass actor is
Cotter45. Protect main with required Release gate (strict/up-to-date), a PR
requirement and no review count; it accepts only validated staging promotions.
Keep both branches protected against deletion and force pushes. Allow merge
commits for promotion. Allow the publishing identity to create new `v*` tags while
retaining immutable tags/releases. Do not give that identity a test-check bypass.

The build workflows have read permissions. Only the default-branch publisher has
contents/pull-request write permission, and it never executes downloaded build
artifacts. No workflow uses `pull_request_target` or gives forked PR code a
publishing credential. The publisher uses `GITHUB_TOKEN`; live tag permissions
must be configured for it before this path can work without another credential.

## Validate packages without publishing

Dispatch the Release workflow manually against a branch or commit. It derives the
package version from Cargo.toml, checks source cleanliness, runs packaging tests,
builds both optimized consumers and runs the distributed CLI smoke plus LSP
initialization, capability and clean-shutdown assertions on Linux,
macOS ARM/Intel and Windows x64. Archives remain temporary Actions artifacts;
manual dispatch never creates a tag or publishes a GitHub release. This focused
package check does not repeat the full CI gate and is not release authorization.
Manual validation never merges a PR; pushing a tag does not trigger this workflow.

The Windows artifact path is configured but remains pending native package
validation until a hosted run is recorded. Do not infer Windows release readiness
from suffix-only packaging fixtures or earlier local development installs.

For focused async language acceptance, dispatch CI with `async_acceptance: true`.
It runs compiler/native async registration, editor contracts and backend all-target
diagnostics on Windows, macOS and Linux. It does not replace the default CI matrix.

## Prepare a version

1. Complete the intended feature section and its affected acceptance. Resolve or
   explicitly disclose known issues in the release notes. The current LSP memory
   report and unverified Table work remain release limitations, not accepted fixes.
2. Update `crates/tsrust-compiler/Cargo.toml` and its `Cargo.lock` entry. A tag must
   exactly equal that version prefixed with `v`, including any prerelease suffix.
   LSP and extension versions are independent; update them when their behavior
   changes. Toolchain manifests record the LSP version separately.
3. Update [.github/release-notes.md](../../.github/release-notes.md) with the
   current prerequisites and known issues. Review the generated change list.
4. Format, run affected checks and commit. Run
   `node tools/check-release.mjs vVERSION` from a clean checkout. Untracked and
   staged changes both prevent release preflight.
5. Merge the version change into `stag` through its required tests/review, then
   open a PR from `stag` to `main`. Preflight rejects a version whose tag already
   exists before starting expensive builds. No manual tag push is needed.

## Promotion artifacts

The build matrix compiles optimized CLI and LSP executables on their host runners:

| Runner | Archive target |
| --- | --- |
| Ubuntu 24.04 x64 | `x86_64-unknown-linux-gnu` |
| macOS 15 ARM | `aarch64-apple-darwin` |
| macOS 15 Intel | `x86_64-apple-darwin` |
| Windows Server 2022 x64 | `x86_64-pc-windows-msvc` |

Each archive contains the two consumers, an MIT license, upstream notices,
version/source metadata and per-file SHA-256 digests. Dependency sources are not
embedded in the executables. Explicit setup prepares shared compilation sources
or optional portable export packs. The package tool collects notices from the
workspace, backend and portable dependency closures using `cargo vendor --locked`.

The packaged CLI runs a fresh project outside the checkout: scaffold, check,
format, native execution, tests and bundled JSON. Only after all hosts pass does
publication create a GitHub Release containing four archives, one VSIX and
`SHA256SUMS`. GitHub also provides the tagged source archives. Tags with a
prerelease suffix create prereleases; ordinary semantic version tags create
regular releases. Existing releases are never overwritten automatically.

Only the publish job has `contents: write`; build/test jobs have read permissions.
External actions are pinned by commit and Dependabot proposes monthly action
updates. No workflow uses `pull_request_target` or gives forked PR code a
publishing credential.

## Verify an archive

Download the archive and `SHA256SUMS` from the same release. From that directory:

```sh
# Linux: verify the selected asset's digest.
grep 'tsrust-v0.1.0-x86_64-unknown-linux-gnu.tar.gz$' SHA256SUMS | sha256sum --check
# macOS: substitute the selected macOS archive name.
grep 'tsrust-v0.1.0-aarch64-apple-darwin.tar.gz$' SHA256SUMS | shasum -a 256 --check
```

Verify before extraction. Digests detect corruption; they do not authenticate an
untrusted download source. The first release pipeline does not code-sign,
notarize or generate build attestations. Do not claim those properties.

Native application compilation still requires Rustup with Rust 1.96.1 and a
system linker. Linux binaries are built on Ubuntu 24.04 with glibc; Alpine/musl,
Linux ARM archives are not provided. Windows x64 packages require MSVC/Windows
SDK build tools for native application compilation. macOS unsigned executable policy
may require local approval. No `curl | sh` installer, registry publication or
Marketplace upload is configured.

## Recovery

A failed test or build publishes nothing. Inspect the failed job and rerun only
failed jobs when safe. A changed head/base requires a new promotion build. If
publication fails after merging, rerun the publisher while its seven-day artifacts
still exist: it verifies the merged tree and resumes the draft upload. An already
published release is accepted only if its asset names and digests exactly match;
it is never overwritten. If artifacts have expired or published bytes differ,
prepare a new version. Published release assets remain attached to their release.
