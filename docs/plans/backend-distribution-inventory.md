# Backend distribution inventory

Initial source inventory, 2026-10-27. This is investigation evidence, not migration
acceptance. Follow the backend-only reset and Rust setup/distribution plan.

| Dependency class | Current mechanism | Required change / preserved guarantee |
| --- | --- | --- |
| Distributed CLI/LSP | Workspace crate dependencies; compiler build.rs also embeds native dependency source closure | Separate executable build dependencies from user compilation sources; tools must launch without Rust installed. |
| Ordinary application runtime | runtime_build::Prepared::with_dependencies materializes native_dependency_bundle, writes stable native-support workspace, invokes pinned Cargo offline/locked | Use normal shared registry/Git cache and exact lockfiles after explicit preparation. Preserve source/cache identity and artifact restoration. |
| User Rust bindings | rust_binding_packages overlay of resolved manifest/lock/vendor; separate package/cache ownership protocol | Inventory graph validation and ownership before changing resolution; no implicit fetch. |
| Toolchain and platform | main.rs backend_tool uses rustup run with pinned RUST_BACKEND_VERSION, no install; verify_backend validates exact rustc version | Reuse detection in doctor/setup; report Cargo, target, linker/SDK/native prerequisites separately. Preserve check/emit without Rust. |
| Portable source | package_export materializes dependency bundle with source replacement and notices | Keep vendoring, relocation/offline rebuild and compatible-toolchain requirement; add explicit target/prerequisite metadata. |
| Installation artifacts | package-toolchain.mjs copies already built CLI/LSP, hashes files, records source/toolchain/versions, copies notices | Retain matched compiler/LSP pair and notices; inspect artifact validation before installing/updating. |

## Mandatory-vendor coupling located

- `crates/tsrust-compiler/build.rs` runs Cargo vendor offline/locked over
  native-dependencies and embeds vendor files plus Cargo.toml/lock/src into the CLI.
  Compiler build therefore currently needs the full cached source graph, including
  the retained UI closure. Ordinary backend distribution must stop inheriting it.
- `cli/native_dependency_bundle.rs::materialize` creates a fresh directory, writes
  embedded files and Cargo config replacing crates.io with vendor and forcing
  offline mode. Failure cleans the directory. Preserve fresh-directory ownership.
- `cli/runtime_build.rs` currently calls that materializer for dependency-backed
  runtimes, keys native cache identity on bundle identity/toolchain/platform/profile,
  and shares a stable workspace for non-binding builds. It sets a private CARGO_HOME,
  so changing only source replacement will not use the owner's shared cache.
  Native feature selections include HTTP/async/threads/SQLite and retained UI paths.
- Packed runtime artifacts restore Windows import libraries during linking. Keep
  this acceptance when switching dependency source preparation.
- Source export and resolved bindings also consume bundled/vendor sources. They
  must be disentangled from ordinary runtime preparation, not globally disabled.

## Installation and update gaps

The inspected release workflow currently lists Linux x64, macOS ARM64 and macOS
x64 artifact jobs. Windows installation acceptance is not established by that
matrix. Artifact packaging currently documents manual PATH setup and unsigned/
unnotarized binaries; it also describes old web-UI prerequisites. That documentation
needs to become backend-only. SHA256SUMS detects corruption but does not by itself
establish an independent release authenticity chain.

No self-update implementation was established by this initial inventory. Before
adding one, inspect all existing install/release tools, define trusted metadata,
host/architecture matching, package-manager ownership and transactional replacement.
Windows locked compiler/LSP handling needs a real isolated-install witness.
No release or remote workflow has been triggered for this work.

Upstream design references reviewed on 2026-10-27:
[GitHub release API](https://docs.github.com/en/rest/releases/releases) supplies
release/assets metadata, including artifact digests; this is a possible discovery
source, not an independent signing root. The
[TUF specification](https://theupdateframework.github.io/specification/latest/)
defines trusted metadata, version/expiry checks and rollback protection. Evaluate
maintained implementations before choosing the updater trust mechanism; no custom
signing protocol or production trust keys have been introduced. Installer layout,
Windows replacement and permission handling still require implementation/tests.

Further upstream installer/update research, 2026-10-27:

- [cargo-dist configuration](https://axodotdev.github.io/cargo-dist/book/reference/config.html)
  offers shell/PowerShell installation and an optional updater; package-manager
  installations are upgraded by their package manager.
- [axoupdater](https://github.com/axodotdev/axoupdater/blob/main/README.md) can be
  embedded as a library and uses cargo-dist install receipts. Adopting it would
  require reconciling existing packaging/installation with those receipts; it is
  not a drop-in replacement for the current manual PATH distribution.
- [self_update](https://docs.rs/self_update/latest/self_update/index.html) offers
  release backends, checksums and optional archive signatures. Its
  [signature interface](https://docs.rs/self_update/latest/self_update/update/fn.verify_signature.html)
  uses Ed25519 verifying keys and supports a key-rotation overlap. These features
  do not by themselves prove transactional replacement of Tsrust plus a locked LSP.

No installer/updater crate is admitted yet. Prefer upstream installation and
replacement mechanisms, then add only Tsrust's pairing/ownership policy. Validate
paired replacement, failure recovery, metadata trust and Windows LSP locks before
choosing the integration. Do not weaken verification to unsigned installation
merely because the current release workflow publishes only checksums. Do not create
production signing keys or trigger a release as part of local acceptance.

Setup implementation must keep installed-tools, cached-sources and proven-native
build readiness distinct. rustup's supported installation path should preserve the
owner's default toolchain and existing Cargo configuration. Explicit setup may
prepare a supported toolchain and graph; routine compilation must not fetch. When
moving to the real shared Cargo home, account for config/profile/wrapper inputs in
native cache identity rather than accidentally reusing artifacts produced with a
different configuration. Platform prerequisite checks remain separate from fetching
sources. Missing rustup/linkers/SDKs need actionable platform-specific instructions.

## Next coherent implementation

The completed direct materialization search found only runtime_build.rs,
package_export.rs and tests/native_dependency_bundle.rs consuming materialize;
build.rs produces native_dependency_files.rs. Binding resolution is in
cli/rust_bindings.rs, with contract/cache identities in project/rust_dependencies.rs.
Bindings currently declare exact registry versions, not arbitrary Git URL syntax;
do not expand that syntax merely to migrate Cargo's cache. Runtime overlays require
matching manifests/locks and a shared cache lock before copying vendor sources.

One compatibility guarantee needs an explicit migration decision: package_export.rs
currently promises export without invoking Cargo/rustc because the executable embeds
all vendor sources. Removing that embedding must not silently replace this with a
networking/toolchain-dependent export. Preferred approach to investigate: ordinary
runtime materialization writes only manifests/locks/runtime sources and uses the
real Cargo home; explicit portable preparation can use Cargo vendor once into a
versioned optional cache, from which exports copy/validate sources without invoking
Rust. An unprepared portable export reports the exact preparation command. Keep
this separate from normal setup/build caches and document any changed prerequisite.
Do not write a replacement Cargo vendor implementation or silently vendor during
ordinary development. Preserve existing portable/binding ownership and relocation
tests, including exact notices, before changing their guarantees.

Finish frontend removal and affected backend/API regressions. Then enumerate all
bundle consumers and exact supported dependency profiles; implement doctor and
explicit preparation against isolated caches before switching ordinary builds.
Keep portable export intact until cached/vendor parity is executed. Installer and
updater implementation must use prebuilt artifacts rather than requiring Cargo
to install Tsrust. Cross-platform execution remains pending.
