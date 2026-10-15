# Rust package workflow requirements

This is a design requirement, not an implemented CLI. The accepted bridge provides
explicit Rust declarations, dependency resolution and owned data adapters. Its JSON
manifest is dependency configuration, not a second handwritten signature language.

## One source of truth

Users must not maintain both JSON and Cargo.toml. A package command owns the project
configuration; Cargo manifests are generated build inputs. Cargo remains responsible
for resolution, checksums, feature unification and vendoring. Do not implement another
registry resolver or silently change the compiler's pinned runtime dependencies.

Adding a curated package installs/selects intentional public bindings. Adding another
crate makes it available to a handwritten Rust adapter; it does not automatically
expose its entire API to Tsrust. Package inspection must distinguish ready bindings,
custom bindings and a dependency with no Tsrust exports. Existing custom adapter code
must never be overwritten by package operations.

## Required operations

- Add a crate, selecting an explicit version or Cargo's current stable selection.
- Enable and disable individual features, replace the declared feature set, and
  enable/disable default features without ambiguous additive-versus-replacement rules.
- Show requested versions, resolved versions, declared/effective features, dependencies
  that enabled those features, supported targets and available binding exports.
- Preview and apply targeted upgrades. Preserve unrelated locked versions where Cargo
  can do so; show every necessary transitive change. Major upgrades, breaking 0.x
  changes, prereleases and yanked releases must not happen silently.
- Remove an unused dependency; report remaining binding references instead of deleting
  user code. Support reproducible offline installs from the lockfile and warm sources.

CLI spelling should follow Cargo conventions where semantics match. Final names and
flags are not accepted yet; do not publish illustrative commands as working features.

## Validation before publication

Resolve a candidate in a private generated build directory. Compile the real Rust
adapters with candidate features, then check affected Tsrust callers and run project
acceptance tests. Compare the exported binding contract, including parameter ownership,
return/error types, struct fields and enum variants. Report compiler diagnostics at
adapter and caller source locations. Do not silently rewrite a public binding to make
an incompatible library upgrade compile.

A successful report distinguishes resolution, native compilation, source compatibility
and executed tests. SemVer is useful guidance, not proof of behavioral compatibility.
Untested behavior and untested targets must remain explicitly unknown. Curated bindings
also report their tested library versions; a version outside that set is not relabeled
as supported merely because compilation succeeds.

A failed or preview-only operation leaves the working configuration, lockfile, adapters
and editor state unchanged. Publication must be crash-safe and coordinate concurrent
readers/writers. Two independent file renames alone do not make a manifest/lock pair
transactional; mixed generations must be prevented or detected with recovery. Reuse
project snapshots/locking mechanisms where appropriate, without keeping source archives.

After successful publication, invalidate native cache identities, refresh editor
signatures/diagnostics and retain only the current dependency closure plus useful warm
artifacts. Package exports must preserve the same exact sources, features, licenses and
lockfile and continue building offline after relocation.

## Acceptance witnesses before implementation is called complete

Cover successful add, repeat add, feature enable/disable/default changes, unified
features, compatible/incompatible upgrades, transitive drift, compiler-pinned dependency
conflicts, stale lockfiles, failed resolution/build/tests, preview mode, missing bindings,
custom adapter preservation, concurrent operations and interruption during publication.
Include a real curated crate and a custom adapter, live editor invalidation, offline
relocation and exact restoration after failed operations. No general promise that every
Rust crate, resource model, macro or build script is compatible with Tsrust.
