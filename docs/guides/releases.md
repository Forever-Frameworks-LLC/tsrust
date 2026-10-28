# CI and GitHub Releases

The repository description can be:

> A TypeScript-style language with inferred ownership, a Rust backend, native backend services and independent HTTP SDKs.

The source repository is intended for
`https://github.com/Forever-Frameworks-LLC/tsrust`. Links use that address; creating
or pushing the repository is a separate owner operation.

## Configured repository policy

The repository is public at the address above. As of 2026-10-26, `main` requires
a pull request, the aggregate **CI** check, resolved review conversations and
linear history. Squash is the allowed merge method; merged branches are deleted
automatically. Auto-merge and update-branch suggestions are enabled.

[CODEOWNERS](../../.github/CODEOWNERS) assigns every path to `@Cotter45`.
A separate review ruleset requires one approval and code-owner review, dismissing
stale approvals after changes. Only `Cotter45` can bypass that review rule when
merging a pull request, allowing the owner to merge their own work without a
second reviewer. GitHub's bypass follows the merging actor, not the author:
the owner can also explicitly bypass review on someone else's PR. Contributors
cannot bypass it. The personal review bypass does not bypass CI or permit direct
pushes to `main`.

Only repository maintainers/admins can create or modify `v*` release tags.
Immutable releases protect published release assets and associated tags.
Private vulnerability reporting, Dependabot alerts/security updates, secret
scanning and secret push protection are enabled. The unused wiki is disabled;
documentation lives in this checkout.

Existing Actions policy retains selected actions, mandatory commit pinning,
read-only default tokens and disabled workflow approval of pull requests.
Default artifact/log retention is 30 days; workflows explicitly retain their
temporary artifacts for seven days. Repository protections are live settings,
not automatically restored by cloning or forking the source.

## One-time repository setup

1. Push the source to the intended repository with `main` as the default branch.
2. Enable GitHub Actions and allow the pinned official GitHub actions. The
   workflows use `GITHUB_TOKEN`; no personal token or publishing secret is needed.
3. Run CI and make its aggregate **CI** check required in a branch ruleset.
   Protect `main` and release tags from unauthorized writes.
4. Enable private vulnerability reporting so the security policy's private form
   is available. Choose repository visibility and Actions spending limits.
5. Set the description above and topics such as `compiler`, `typescript`, `rust`,
   `native`, `language-server` and `ui`. No deployed website is assumed.

Changing account settings, enabling branch protection or publishing the first tag
is not performed by committing workflow files. GitHub-hosted execution must pass
before calling this release pipeline verified.

## What CI runs

[CI](../../.github/workflows/ci.yml) runs on pull requests, pushes to `main`, manual
dispatch and release calls. Linux x64 and macOS ARM run selected compiler/CLI
acceptance, LSP transport and checker agreement, renderer model checks, Clippy and
project formatting. A separate job tests and packages the VS Code extension.
Builds use the committed toolchain and locked dependency graphs, one Cargo job, one test
thread and disabled incremental compilation. Jobs have explicit timeouts. Build
outputs are not cached; uploaded artifacts expire after seven days.

The default suite is a baseline, not the entire regression matrix. Dispatch CI
with `full_regressions: true` at a milestone to run all default workspace test
and doctest targets. Database service tests remain opt-in. Hosted model checks
cannot replace real GPU, browser, window and assistive-technology acceptance.
Performance benchmarks do not run automatically or overwrite accepted evidence.

## Validate packages without publishing

Dispatch the Release workflow manually against a branch or commit. It derives the
package version from Cargo.toml, checks source cleanliness, runs packaging tests,
builds both optimized consumers and runs the distributed CLI smoke plus LSP
initialization, capability and clean-shutdown assertions on Linux,
macOS ARM/Intel and Windows x64. Archives remain temporary Actions artifacts;
manual dispatch never creates a tag or publishes a GitHub release. This focused
package check does not repeat the full CI gate and is not release authorization.
Tag-triggered releases still require the complete CI gate before building.

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
5. Push a tag on that commit:

   ```sh
   git tag -a v0.1.0-rc.1 -m "Tsrust 0.1.0-rc.1"
   git push origin v0.1.0-rc.1
   ```

   The example requires the compiler version to be `0.1.0-rc.1`; do not tag the
   current `0.1.0` source with a mismatched version. Prefer prereleases while the
   language is experimental.

## What a tag does

[Release](../../.github/workflows/release.yml) checks version and source cleanliness,
then calls the same CI gate. Failed checks prevent artifact builds and publication.
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
the affected workflow jobs when safe. Once a release exists, do not move its tag
or replace its assets; publish a corrected version. If publication itself fails,
inspect whether the release already exists before rerunning: the workflow refuses
to overwrite it. Seven-day workflow artifacts are temporary; published release
assets remain attached to the release.
