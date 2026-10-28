# Rust setup and dependency distribution

Owner-requested roadmap addition, 2026-10-27. Status: partially implemented and
accepted on Windows; full distribution/platform acceptance remains open. Doctor,
explicit pinned-toolchain/source setup and ordinary shared-cache backend builds
are implemented. See ../milestones/shared-backend-sources.md and ../agent-progress.md
for exact evidence; optional portable packs and all binding/install/update work
are not implied complete by those results. This follows the backend-only direction.

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

## Located source paths; full inventory pending

These are investigation entry points, not a completed dependency audit:

| Area | Current source to inspect | Migration question |
| --- | --- | --- |
| Toolchain detection | `crates/tsrust-compiler/src/cli/main.rs`, `verify_backend`, `RUST_BACKEND_VERSION` | Preserve check/emit without Rust; diagnose the exact supported toolchain. |
| Mandatory dependency materialization | `cli/native_dependency_bundle.rs` under the compiler source | Identify embedded files, vendor source replacement and offline Cargo configuration. |
| Runtime compilation | `cli/runtime_build.rs`, `cli/native_workspace.rs` | Separate portable materialization from ordinary cached builds; retain identity, locks and artifact reuse. |
| User bindings | `cli/rust_binding_packages.rs`, `cli/rust_bindings.rs`, `cli/package_validation.rs` | Preserve exact graphs, binding validation, cache locks and ownership. |
| Source export | `cli/package_cli.rs`, `cli/package_export.rs` | Keep portable vendor/notices/relocation guarantees and add prerequisite metadata. |
| Build-time bundling and tests | `crates/tsrust-compiler/build.rs`, native dependency/package/offline tests | Distinguish binary build dependencies from user application dependencies. |
| Distribution | `tools/package-toolchain.mjs`, `tools/check-release.mjs`, `tools/tests/distribution.test.mjs`, `.github/workflows/release.yml` | Inventory supported artifacts/install paths without triggering publication. |

Initial inspection found ordinary runtime builds materialize the bundled vendor
sources and use offline/locked Cargo; portable export also materializes the bundle.
Binding packages overlay manifests, locks and vendor sources. Inventory must verify
all consumers before removing mandatory bundling from ordinary paths. Existing
toolchain guidance is not proof that doctor/setup already exist.

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
