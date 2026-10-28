# Rust setup and dependency distribution

Owner-requested roadmap addition, 2026-10-27. Status: partially implemented and
accepted on Windows; full distribution/platform acceptance remains open. Doctor,
explicit pinned-toolchain/source setup and ordinary shared-cache backend builds
are implemented. See ../milestones/shared-backend-sources.md and ../agent-progress.md
for exact evidence. Optional exact portable packs, relocated offline source export,
Windows import-library restoration and removal of the compiler vendor payload are
now accepted in [portable sources and editor agreement](../milestones/portable-source-packs-and-editor-agreement.md).
Binding-cache migration has focused macOS acceptance; production install/update
and cross-platform distribution acceptance remain open. This follows
the backend-only direction.

The subsequent [backend-only reset](backend-only-reset.md) removes frontend
orchestration prerequisites and adds reliable cross-platform installation and
explicit latest-version check/update. The dependency migration here remains
required. Original Tsrust UI is retained source, outside normal backend distribution;
React/Electron/Expo tooling is no longer part of installer acceptance.

## Product contract

Distribute Tsrust and its LSP as prebuilt binaries through the established release
mechanism. The installer prepares the environment; Tsrust remains the compiler
and application CLI. Separate installing these binaries, preparing a build machine,
resolving application dependencies, exporting portable source and deploying apps.

Ordinary development uses Cargo registry/Git sources and the shared Cargo cache,
exact generated lockfiles and stable generated workspace identities. Explicit
setup or dependency resolution may fetch dependencies. Routine check, emit,
build and dev must not silently install tools or download dependencies. Preserve
existing toolchain-free operations. Native builds use locked, offline resolution
against prepared caches and give actionable setup/resolution instructions when
prerequisites are missing. Do not copy a vendor tree into every ordinary project
or installation. Preserve binding validation and dependency ownership boundaries.

Portable offline source export retains vendoring. Compiled backends run without Tsrust, Cargo, Rust, this repository or developer caches;
document actual platform runtime requirements separately.

## Priority and implementation sequence

1. **Inventory before migration.** Trace installation, toolchain detection,
   generated Cargo workspaces, binding resolution, runtime bundling, native builds,
   portable export, release artifacts and offline tests. Classify dependencies
   linked into CLI/LSP binaries, application compilation dependencies, user binding
   dependencies and toolchain/target/linker/SDK/native library prerequisites.
   Record every guarantee currently dependent on mandatory vendoring. Preserve
   unrelated work and historical evidence.
2. **Define readiness and explicit preparation.** Add or consolidate `tsrust doctor`
   and `tsrust setup`, reusing equivalents if found. Doctor reports supported Rust,
   Cargo, targets, linker prerequisites and dependency readiness, with actionable
   failures and machine-readable output. Setup uses rustup/Cargo, preserves existing
   toolchains, global defaults and unrelated configuration, and supports an explicit
   unattended flag. A private managed toolchain is optional follow-up, not required
   for the first implementation. Settle commands and readiness schema before wiring
   application dev preflight; package-manager choice remains a separate concern.
3. **Migrate ordinary builds as one coherent section.** Explicit setup/resolution
   prepares exact application graphs; normal builds use shared caches with locked,
   offline behavior. Account for features, optional dependencies, supported targets,
   native prerequisites, binding graphs and Windows import libraries. One successful
   default fetch does not establish all-profile offline readiness. Preserve shared
   Cargo target reuse, deterministic output and existing ownership checks.
4. **Retain portable export.** Include generated Rust and runtime sources,
   manifests/locks, vendor sources, Cargo source configuration, license notices and
   toolchain/target/platform metadata. Require compatible Rust and platform build
   tools; do not bundle Rust by default. Prove relocated offline rebuilding and
   equivalent native behavior for cached and vendored builds before acceptance.
5. **Finish distribution and documentation.** Update CLI help and separate install,
   machine preparation, application resolution, portable export and deployment
   guides. Validate prebuilt CLI/LSP and deployed applications without development
   tooling. Record OS/architecture acceptance only where executed. Cross-platform
   packaging and distribution remain explicit pending gates until tested.

Finish the current coherent starter layout section before changing dependency
materialization. Inventory and setup-contract design are next prerequisites for
production native build/dev preparation. Distribution migration must be accepted
before claiming production installation readiness. Do not delay writing focused
witnesses until the end, but batch their execution after coherent implementation.
Do not build a custom package manager, dependency mirror or toolchain manager.

## Dependency consumer inventory — refreshed 2026-10-28

The source paths below were inspected after the accepted shared-backend/portable
source work. Ordinary backend builds already avoid mandatory vendor copies.
Binding migration has focused local acceptance (see agent-progress.md); do not
repeat ordinary-backend migration or remove the portable export producer.

| Area | Current source to inspect | Migration question |
| --- | --- | --- |
| Toolchain detection | `crates/tsrust-compiler/src/cli/main.rs`, `verify_backend`, `RUST_BACKEND_VERSION` | Preserve check/emit without Rust; diagnose the exact supported toolchain. |
| Ordinary backend sources | `cli/backend_sources.rs`, `backend-dependencies/Cargo.toml`, `backend-dependencies/Cargo.lock` | Implemented: shared Cargo home, pinned graph, offline private support configuration, configuration/environment cache identity. Dependency-bearing bindings also use shared Cargo sources; unbound original UI retains its portable path. |
| Portable materialization | `cli/native_dependency_bundle.rs`, `cli/portable_sources.rs` | Explicit portable preparation produces a separate source pack. Export verifies pinned metadata and Cargo package/file checksums before copying; ordinary shared backend builds do not require it. |
| Runtime compilation | `cli/runtime_build.rs`, `cli/native_workspace.rs` | Implemented for ordinary backends: locked/offline builds, stable dependency target across runtime revisions, serialized private workspace and cached artifacts. Binding graphs now select stable workspaces by their exact manifest/lock identity. |
| User bindings | `project/rust_dependencies.rs`, `cli/rust_binding_packages.rs`, `cli/rust_bindings.rs`, `cli/package_validation.rs` | Explicit resolution fetches locked shared sources; optional --portable prepares the existing checksummed pack format. Schema 1 retains its legacy runtime graph; new schema 2 selects backend dependencies. Native validation and atomic package changes remain required. |
| Source export | `cli/package_cli.rs`, `cli/package_export.rs` | Portable export verifies and materializes the selected full graph directly, including binding dependencies, then emits standalone runtime/application sources, locks/configuration, notices and prerequisite metadata. It never invokes Cargo or downloads. |
| Build-time bundling and tests | Compiler build.rs removed; cli/portable_sources.rs and native dependency/package/offline tests | Ordinary builds use shared sources; optional exact portable packs preserve export notices and relocation. |
| Distribution | `tools/package-toolchain.mjs`, `tools/check-release.mjs`, `tools/tests/distribution.test.mjs`, `.github/workflows/release.yml` | Inventory supported artifacts/install paths without triggering publication. |

`cli/setup_cli.rs` now implements `doctor [--json] [--native]` and explicit setup
for toolchains, shared dependencies and portable sources. Doctor distinguishes
toolchain readiness, an executed native link/run probe, and unchecked application
dependencies. These commands do not prove every target, binding graph or platform
ready. Remaining work must preserve their explicit preparation boundary.

The binding migration must update the package marker/graph representation,
resolution, runtime selection/materialization, artifact identity and source export
together. Normal builds must fail with explicit resolution guidance when shared
sources are absent; they must not fall back to implicit fetching or mandatory
vendoring. Portable export must continue working independently after relocation.

## Release/update integration findings — 2026-10-27

Read-only local inspection confirms the current release workflow selects Linux x64,
macOS ARM64 and macOS x64, with no Windows artifact job. Toolchain manifest schema 2
already records host target, separate compiler/LSP versions, source commit and each
file's hash/size. Published SHA256SUMS detect corruption; the package README explicitly
does not claim distributor authenticity, code signing or notarization. Preserve
these distinctions when implementing paired installation and latest-stable updates.
No workflow, release, remote policy or signing trust was changed by this inspection.

Required remaining decisions: establish authenticated release metadata before code
replacement; add isolated Windows packaging/execution and corresponding macOS/Linux
acceptance without public release publication; define package-manager ownership and
interruption/locked-process recovery. The development CLI's guarded local refresh
is acceptance of a local consumer, not proof of this production update mechanism.

## Acceptance matrix

Rows define completion criteria; implementation and platform evidence are recorded
in the linked milestones/current progress. Do not reset completed evidence or treat
partial Windows checks as whole-platform acceptance. Use isolated temporary homes/caches;
never delete or mutate the owner's caches to simulate missing prerequisites.

| Witness | Required evidence |
| --- | --- |
| Prebuilt tools | CLI and LSP launch with Rust/Cargo absent from the test environment. |
| Toolchain-free compiler | Existing supported check/emit operations still work without Rust. |
| Doctor | Missing and incompatible Rust/Cargo/target/linker/cache cases yield actionable human and machine-readable results. |
| Setup | Explicit preparation succeeds; existing toolchains, defaults and unrelated configuration remain unchanged. |
| Resolution | Explicit resolution prepares the selected locked graph, features and target. |
| Cached build | Prepared application builds offline; no vendor copy or implicit fetch. |
| Missing/stale graph | Missing cache and dependency edits fail with the exact explicit resolution instruction, without downloading. |
| Portable source | Export relocates and rebuilds offline with compatible platform tools and complete notices/metadata. |
| Behavioral equivalence | Cached and vendored builds pass the same independent native behavior oracle. |
| Deployment | Native backend runs without repository, toolchain or developer cache. |

Preserve completed checks and original evidence. Separate implementation acceptance
from pending platform/distribution validation. Run whole-project formatting and
local source checkpoints as required. Do not push, trigger remote CI or create
release tags merely to exercise installation. Apply the project remote-validation
decision before any necessary remote check; public release needs owner authorization.

## Decisions and questions to resolve during implementation

- Reuse an existing resolution command or introduce one? Settle exact setup,
  resolution, unattended and JSON flag spellings with help and executable tests.
- What readiness levels distinguish installed tools, resolved graph, cached
  sources and proven build prerequisites? Never label an unbuilt target ready.
- How are feature/target selections and optional dependencies recorded and
  invalidated when manifests, locks or supported toolchain versions change?
- How do registry and Git caches, offline configuration, native prerequisites and
  Windows import-library restoration interact with the stable workspace identity?
- Which lock refreshes require explicit resolution, and how does the diagnostic
  preserve user-owned manifests/configuration without automatic edits?
- Which OS/architecture prerequisites and runtime libraries are actually supported?
  Record executed acceptance separately from planned release artifacts.
- Does existing portable export need an explicit mode/version marker to preserve
  older exports and distinguish cached development from vendor source packages?
- Is an optional private Rust toolchain justified after normal rustup integration
  works? Do not introduce a separate package manager or unrelated frontend toolchain.
