# Staged release pipeline

Owner direction, 2026-10-28. This takes priority over the remaining language polish
checks. Preserve their completed evidence and resume them after this pipeline work.

## Required flow

- Feature PRs target `stag`. Run unit/integration/editor checks on Linux, not a
  release-build matrix. Passing tests and one approval permit merging. Only Cotter45 may bypass the review requirement, including when reviewing and
  merging another author's PR. The owner accepted this native actor-based bypass;
  tests remain required against the current staging base.
- A `stag` to `main` PR builds release artifacts on Linux, Windows and macOS.
  Preserve both currently configured macOS architectures unless explicitly changed.
  No arbitrary branch may use this promotion path.
- Successful promotion automatically merges and publishes the artifacts already
  validated by that PR. Do not rebuild after merge. Bind artifact provenance to
  the tested source tree and recheck the PR head/base before promotion.
- Use the Cargo version as release identity; reject existing version/tag collisions
  before expensive builds. Version changes remain explicit source changes.
- Cache Cargo sources and compiled outputs by platform, toolchain, lockfiles and
  profile. Test and release caches are separate. Preserve failed-run dependency
  work; caches never replace checks or determine release provenance.
- Avoid duplicate push/PR test runs and avoid using hosted jobs during routine
  local development. Keep explicit non-publishing validation available.

## Current findings and remaining implementation

The existing setup had no Rust cache. A pinned Swatinem/rust-cache step now exists
locally, including workspace outputs and failure saves. The repository already
allows this action. GitHub limits PR-created caches to that PR's merge ref; no
cross-PR hit or speedup is promised without measured evidence.

Existing main rules require CI and one code-owner approval, with an owner review
bypass. There is no stag branch. Release tags currently restrict creation to
maintainers/admins. Workflows and rules must be migrated together without leaving
main open or enabling publication from an unvalidated feature branch.

Local implementation now separates staging tests, read-only promotion builds and
trusted default-branch publication. The publisher checks the run, PR head/base,
merge parents, archive manifests and final merged tree before publishing. It can
resume after a merge or partial draft upload without replacing a published release.
All 14 focused local policy and packaging tests passed; they cover stale sources, failed/manual runs,
artifact substitution, tag collisions and merge refusal.

Still required: deploy the workflows to main, create stag, migrate branch and tag
rules, and retarget the backend PR. The default-branch publisher must exist before
the first promotion run; merely adding it in the promotion PR cannot activate it.
Keep the existing main protections until that bootstrap has passed CI. Main must
then require Release gate with strict up-to-date checks and allow merge commits;
stag must require up-to-date CI plus the separate owner-bypass review rule. Both branches
remain protected against deletion and force pushes.

The owner authorized the bootstrap and tag-policy migration on 2026-10-28.
PR #2 merged as caf5284b, with exactly the previously tested source tree. Its
redundant main push CI was cancelled to conserve Actions usage. This bootstrap PR
contains only the staged pipeline, packaged-LSP smoke and accepted API scaffold/
SDK regression changes; unfinished language/cache work is excluded.

Run required CI by manual dispatch on this bootstrap PR's exact head (the new
PR trigger targets stag). Release preflight rejects this non-stag source before
platform builds. Keep current main protections until bootstrap CI passes and the
PR merges. Then create stag at main, move CI/review requirements to stag, require
strict Release gate on main and enable merge commits. GitHub rejected its built-in
Actions identity as a tag-creation bypass actor, so the owner authorized removing
the creation restriction while retaining immutable existing tags/releases. Writers
may create new tags; automation must never overwrite a tag or published release.

Changes are not live and no release has been authorized for immediate publication
merely to test this setup. Future successful promotions are explicitly authorized
to publish automatically by the owner's requested flow.

Optional async-only checks report Async acceptance; manually dispatched release
builds report Package validation. Neither may satisfy a protected PR gate.
Staging CI also runs native SDK authorization/replica integration and the
compiler/LSP API-project regressions; bootstrap must include their source fix.
