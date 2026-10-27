# Shared Cargo sources for compiler-owned backends

Ordinary compiler-owned backend runtimes now materialize only a small manifest,
exact lock, runtime source and offline Cargo config. Cargo uses its real shared home
and a stable native target instead of extracting vendors or using a private Cargo
home. The separate backend lock contains 266 packages versus the retained legacy
closure's 531; retained versions, source identities and checksums are unchanged.
Offline locked all-feature metadata excludes wgpu, winit, vello, parley and taffy.

`tsrust setup --dependencies --yes` explicitly prepares sources with upstream
`cargo fetch --locked`. `--dependencies --offline` checks/prepares from existing
cached sources without download consent. Toolchain installation remains explicit;
combining --offline with --toolchain is rejected. A missing shared source fails the
normal offline build with an actionable setup instruction. No build-time fetch or
implicit toolchain installation was added.

Executable and runtime cache keys include digests of environment inputs and Cargo
configuration. The already locked toml_edit parser discovers string, inline-table
and array-of-table includes recursively, including missing optional files. Cargo
still applies configuration itself. Config content and credential values are not
persisted in identity files. Inputs are rechecked before artifact publication;
changed inputs suppress cache publication. One stable Cargo target remains reusable
across source/config revisions. Retained UI and resolved binding paths keep their
existing vendor protocol and cache identities.

## Acceptance on Windows x86_64

- Application scaffold 3/3 passed with the new CLI; setup integration 2/2 passed.
- Shared-source integration 3/3 passed: empty-cache setup/build fails offline with
  guidance, source emission works without tools, normal JSON builds create no vendor
  or private Cargo home, repeated builds hit caches, changed Cargo config invalidates
  artifacts while reusing the target, and relocated portable exports rebuild offline
  with an empty Cargo home and produce the same native result.
- Nested/optional configuration regression passed. After adding [[include]] support,
  only that affected gate was rerun and passed. A changed nested wrapper or a newly
  created optional wrapper cannot reuse the previous executable; failures preserve
  the working output.
- The changed deployment smoke case passed separately: the executable was copied to
  a runtime-only directory and ran with PATH containing no developer tools and with
  nonexistent Cargo/Rustup homes.
- Actual `tsrust setup --dependencies --offline` succeeded against the existing
  shared Cargo home. No downloads or global toolchain/configuration changes occurred.
- Actual CLI creation at generated-projects/backend-shared-cache, native HTTP build,
  real TypeScript/Rust SDK live/saved description acceptance, and installed-LSP
  entry/operation clean/error/repair all passed. The HTTP test server was stopped.
  Its native workspace has vendor=false and privateCargoHome=false.

Disposable logs: target/shared-source-recheck-current.log,
target/config-table-includes-current.log, target/shared-runtime-deployment-current.log,
target/source-setup-live-current.log, target/shared-http-build-current.log,
target/shared-backend-http-sdk-current.log and target/shared-backend-editor-current.log.
The first compile rejected unsupported LowerHex formatting for the current digest
API; repaired before tests ran. A JSON test fixture initially used unsupported
anonymous generic inference; replaced with the existing std:json constructor API.
Passing gates were preserved; no full compiler/LSP/benchmark matrix was restarted.

## Remaining scope

This is not completion of dependency distribution. The CLI still embeds the legacy
portable source bundle and its build script still vendors that closure. Remove that
payload next while preserving toolchain-free exports, exact offline source packs,
notices and Windows import-library restoration. Custom binding graphs still require
migration to shared registry/Git sources. Compiler/LSP installation, trusted signed
updates, failure recovery and real macOS/Linux acceptance remain open. The server
scaffold still uses generated/server; the server-owned API facility is a separate
required correction. Native docs stack overflow, UI authoring warnings and the full
VS Code Problems inventory remain unresolved. No release, tag, push, package
publication, matched performance measurement or analytics import occurred.
