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

## Next coherent implementation

Finish frontend removal and affected backend/API regressions. Then enumerate all
bundle consumers and exact supported dependency profiles; implement doctor and
explicit preparation against isolated caches before switching ordinary builds.
Keep portable export intact until cached/vendor parity is executed. Installer and
updater implementation must use prebuilt artifacts rather than requiring Cargo
to install Tsrust. Cross-platform execution remains pending.
