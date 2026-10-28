# Staged release pipeline

Owner direction, 2026-10-28. This takes priority over the remaining language polish
checks. Preserve their completed evidence and resume them after this pipeline work.

## Required flow

- Feature PRs target `stag`. Run unit/integration/editor checks on Linux, not a
  release-build matrix. Passing tests and one approval permit merging. For a PR
  authored by Cotter45, the owner may merge without another person's approval;
  tests remain required.
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

Still required: staging CI triggers, promotion artifact/gate/merge/publication
workflow, exact owner-authored review exception, compatible tag permissions,
branch/rules migration, retargeting the existing backend PR, local policy tests,
and documentation. Changes are not live and no release has been authorized for
immediate publication merely to test this setup. Future successful promotions
are explicitly authorized to publish automatically by the owner's requested flow.
