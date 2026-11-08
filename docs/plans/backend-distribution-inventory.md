# Current binding-cache migration — implementation pending validation

The 2026-10-28 section migrates ordinary dependency-bearing Rust bindings to Cargo's
shared sources and stable per-graph native workspaces. Package add/update/features
validate real adapters/callers/tests before atomic publication, without vendoring.
Binding markers carry manifest/lock/crate names rather than machine-local vendor
paths. SchemaVersion 2 fixes new graphs to backend inputs; schemaVersion 1 retains the
legacy full-runtime manifest explicitly, without lock-content heuristics. Explicit bindings resolve --portable reuses the existing source-pack
producer and checksum verifier with the selected graph. Exports stay toolchain-free.

Affected consumers: project binding serialization, explicit resolve/package commands,
package candidate compilation, runtime selection/identity/workspaces, portable pack
materialization/export, CLI help, package documentation and focused regressions.
Validation must prove no vendor copies in normal builds, cache hits, offline failure
with an empty Cargo home, legacy locks, atomic failed/preview edits, and relocated
portable parity/tamper rejection. No new timing or platform acceptance is claimed.

# Backend distribution inventory

Current audit: 2026-10-28, source c5b3d4a9. Follow the backend-only reset and
Rust setup/distribution plan. Inspection proves the mechanisms below, not their
unexecuted platform behavior.

## Current mechanisms and remaining work

- Compiler build.rs has been removed. The CLI no longer embeds the vendor source
  closure. Ordinary backend runtime builds select backend_sources.rs, use the
  shared Cargo home and exact backend lock, and remain locked/offline.
- portable_sources.rs prepares optional vendor packs explicitly and validates
  package identities and file checksums. Unprepared export gives the explicit
  `tsrust setup --portable --yes` instruction; export does not download sources.
- Dependency-bearing bindings still select the vendor path. rust_bindings.rs
  invokes Cargo vendor during explicit resolution, and rust_binding_packages.rs
  copies that graph into runtime/export workspaces. This is the remaining cache
  migration, not a reason to repeat the accepted ordinary-backend migration.
- package-toolchain.mjs handles Windows executable suffixes and records both
  consumer versions. Its README already describes backend/shared-cache setup.
  The release workflow still lacks a Windows artifact job; production paired
  installation, self-update and locked-binary recovery remain unfinished.
  (Superseded 2026-10-28: release.yml now builds x86_64-pc-windows-msvc on
  windows-2022, and hosted runs 40731485167/40748893795 passed Windows paired
  installation and locked-process tests. Published acquisition remains pending.)
- `tsrust update <crate>` is the existing dependency-upgrade command. Preserve
  that interface when adding explicit compiler/LSP update checking and updating.
- CI run 40663277431 passed the selected Linux/macOS suites. Focused async run
  40672007920 adds actual Windows execution but remains pending. Neither run
  establishes release-installation or self-update acceptance.

See [shared backend sources](../milestones/shared-backend-sources.md),
[portable sources and editor agreement](../milestones/portable-source-packs-and-editor-agreement.md)
and [current progress](../agent-progress.md) for evidence and open checks.

## Historical source inventory — 2026-10-27

The following records the pre-migration mechanisms and original research. Claims
about an embedded payload and private Cargo home describe that earlier source,
not the current implementation above.

| Dependency class | Current mechanism | Required change / preserved guarantee |
| --- | --- | --- |
| Distributed CLI/LSP | Workspace crate dependencies; compiler build.rs also embeds native dependency source closure | Separate executable build dependencies from user compilation sources; tools must launch without Rust installed. |
| Ordinary application runtime | runtime_build::Prepared::with_dependencies materializes native_dependency_bundle, writes stable native-support workspace, invokes pinned Cargo offline/locked | Use normal shared registry/Git cache and exact lockfiles after explicit preparation. Preserve source/cache identity and artifact restoration. |
| User Rust bindings | rust_binding_packages overlay of resolved manifest/lock/vendor; separate package/cache ownership protocol | Inventory graph validation and ownership before changing resolution; no implicit fetch. |
| Toolchain and platform | main.rs backend_tool uses rustup run with pinned RUST_BACKEND_VERSION, no install; verify_backend validates exact rustc version | Reuse detection in doctor/setup; report Cargo, target, linker/SDK/native prerequisites separately. Preserve check/emit without Rust. |
| Portable source | package_export materializes dependency bundle with source replacement and notices | Keep vendoring, relocation/offline rebuild and compatible-toolchain requirement; add explicit target/prerequisite metadata. |
| Installation artifacts | package-toolchain.mjs copies already built CLI/LSP, hashes files, records source/toolchain/versions, copies notices | Retain matched compiler/LSP pair and notices; inspect artifact validation before installing/updating. |

## Historical mandatory-vendor coupling located

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

## Historical installation and update gaps

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

## Source payload migration in progress

Checkpoint 2d3a6e09 accepted ordinary backend shared-cache builds. The next section
removes compiler build.rs and its build dependencies, eliminating implicit Cargo
vendoring and include_bytes source payloads. Explicit setup --portable creates an
optional staged source pack using Cargo vendor; exports validate/copy it without
Rust or network access. Original UI source and portable dependency metadata remain
in place. Binding runtime overlays still need their separate shared-cache migration.
See the current agent progress for acceptance rather than the initial inventory's
historical descriptions above. Installer/signature/paired-update work remains open.

## v0.2 backend package contract crosswalk

Static reconciliation at `dfc711783046`: all54 common direct dependencies below have identical native/backend manifest declarations and matching identities/checksums in both locks. The11 native-only retained UI dependencies are intentionally absent from the backend manifest. Root workspace lock presence is not required for a dependency used only by a generated runtime (for example `hostname`). This is source/lock bookkeeping, not a fresh Cargo-resolution or platform result.

The table records direct requested features; resolved transitive features remain defined by the selected build graph. `default` means backend defaults remain enabled. MSRV/license values are declarations from the cached selected package manifests, not compilation or legal-review results. The URL dependency additionally uses the maintained compiler-owned patch in `crates/tsrust-compiler/patched-dependencies/url`; an upstream archive does not prove that patched source identity.

| Package/pin | Requested features | Declared MSRV | License |
| --- | --- | --- | --- |
| `futures-util=0.3.34` | `std`, `sink` | 1.71 | MIT OR Apache-2.0 |
| `tokio-tungstenite=0.29.0` | `connect`, `handshake`, `rustls-tls-webpki-roots` | 1.63 | MIT |
| `webpki-roots=0.26.11` | `default` | undeclared | CDLA-Permissive-2.0 |
| `pin-project-lite=0.2.17` | `default` | 1.37 | Apache-2.0 OR MIT |
| `tracing=0.1.44` | `std` | 1.65.0 | MIT |
| `tracing-subscriber=0.3.23` | `fmt`, `json` | 1.65.0 | MIT |
| `icu_provider=2.3.1` | `sync` | 1.88 | Unicode-3.0 |
| `icu_collator=2.3.1` | `default` | 1.88 | Unicode-3.0 |
| `icu_decimal=2.3.0` | `default`, `alloc` | 1.88 | Unicode-3.0 |
| `icu_locale=2.3.1` | `default` | 1.88 | Unicode-3.0 |
| `icu_segmenter=2.3.0` | `compiled_data` | 1.88 | Unicode-3.0 |
| `fixed_decimal=0.7.3` | none | 1.82 | Unicode-3.0 |
| `tokio-util=0.7.20` | none | 1.85 | MIT |
| `regex=1.13.1` | `default` | 1.65 | MIT OR Apache-2.0 |
| `hostname=0.4.2` | none | 1.74 | MIT |
| `memchr=2.8.3` | `default` | 1.61 | Unlicense OR MIT |
| `tempfile=3.27.0` | `default` | 1.63 | MIT OR Apache-2.0 |
| `percent-encoding=2.3.2` | `default` | 1.51 | MIT OR Apache-2.0 |
| `base64=0.23.1` | `default` | 1.71.0 | MIT OR Apache-2.0 |
| `nix=0.31.2` | none | 1.69 | MIT |
| `sha2=0.11.1` | `zeroize` | 1.88 | MIT OR Apache-2.0 |
| `subtle=2.6.1` | `default` | undeclared | BSD-3-Clause |
| `uuid=1.28.0` | `std` | 1.89.0 | Apache-2.0 OR MIT |
| `getrandom=0.4.3` | `default` | 1.85 | MIT OR Apache-2.0 |
| `chrono-tz=0.10.4` | `std` | 1.65 | MIT OR Apache-2.0 |
| `chrono=0.4.45` | `std` | 1.62.0 | MIT OR Apache-2.0 |
| `bigdecimal=0.4.11` | `default` | undeclared | MIT/Apache-2.0 |
| `futures-core=0.3.34` | `default` | 1.36 | MIT OR Apache-2.0 |
| `sqlx=0.9.0` | `runtime-tokio`, `tls-rustls`, `postgres`, `mysql`, `sqlite`, `any`, `migrate` | 1.94.0 | MIT OR Apache-2.0 |
| `rusqlite=0.39.0` | `default`, `bundled`, `hooks`, `extra_check` | undeclared | MIT |
| `matchit=0.8.4` | `default` | undeclared | MIT AND BSD-3-Clause |
| `axum=0.8.9` | `http1`, `http2`, `tokio`, `multipart` | 1.80 | MIT |
| `tower-http=0.6.11` | `fs` | 1.64 | MIT |
| `tokio-rustls=0.26.6` | `aws_lc_rs`, `tls12` | 1.71 | MIT OR Apache-2.0 |
| `reqwest=0.13.5` | `rustls`, `stream`, `http2`, `multipart` | 1.85.0 | MIT OR Apache-2.0 |
| `http-body=1.1.0` | `default` | 1.61 | MIT |
| `http-body-util=0.1.5` | `default` | 1.61 | MIT |
| `hyper-util=0.1.21` | `default`, `server-auto`, `tokio`, `service` | 1.85 | MIT |
| `tokio=1.53.3` | `default`, `rt-multi-thread`, `time`, `sync`, `net`, `signal` | 1.71 | MIT |
| `rayon=1.11.0` | `default` | 1.80 | MIT OR Apache-2.0 |
| `url=2.5.8` | `default` | 1.63 | MIT OR Apache-2.0 |
| `serde=1.0.229` | `default`, `derive` | 1.56 | MIT OR Apache-2.0 |
| `serde_json=1.0.151` | `default`, `raw_value` | 1.71 | MIT OR Apache-2.0 |
| `flate2=1.1.10` | `zlib-rs`, `runtime_detection` | 1.67.0 | MIT OR Apache-2.0 |
| `form_urlencoded=1.2.2` | `default` | 1.51 | MIT OR Apache-2.0 |
| `hmac=0.13.0` | `zeroize` | 1.85 | MIT OR Apache-2.0 |
| `hkdf=0.13.0` | none | 1.85 | MIT OR Apache-2.0 |
| `rand=0.10.3` | `std`, `std_rng`, `sys_rng`, `unbiased` | 1.85 | MIT OR Apache-2.0 |
| `argon2=0.6.0` | `alloc`, `password-hash`, `zeroize` | 1.85 | MIT OR Apache-2.0 |
| `zeroize=1.9.1` | `alloc` | 1.85 | Apache-2.0 OR MIT |
| `tower=0.5.3` | `util` | 1.64.0 | MIT |
| `async-compression=0.4.51` | `tokio`, `gzip`, `zlib` | 1.83 | MIT OR Apache-2.0 |
| `compression-codecs=0.4.47` | `gzip`, `zlib` | 1.83 | MIT OR Apache-2.0 |
| `compression-core=0.4.34` | `default` | undeclared | MIT OR Apache-2.0 |

Evidence boundaries:

- Existing B09/B11/B12/B13/B16/B17/B19 plan sections and family admission metadata retain primary-source decisions, archives/licenses/notices, target-filtered graph checks and direct operation profiles. Keep those exact source/lock identities; they do not automatically cover a later aggregate lock. B11 and B17 historical admission metadata may still describe dependency-only stages despite later private profile success recorded in the sole handoff.
- Current production-three-lock audit: RustSec database `309392b18f6b17022f82c701f300433126901068` (commit dated2026-11-06), all three commands exit0 and report zero known vulnerabilities. This is an advisory snapshot, not a claim that the whole project has no warnings or future vulnerabilities. Exact audited lock SHA256s are:
  - workspace: `f3b431a58dd0af8259fb8f710d753c91fecf20261324acb65a0f86a427ec5273`;
  - native: `a32781de184e9d1d5052ebb032e31485c93a922c670a0b512939c6f30c1771d5`;
  - backend: `b3ce935e86b85b8e746b74d61eede955dd1c84bf7d9b551c7d3516fd6bfb11f1`.
- The backend lock has no audit warnings and contains none of the six warned packages. Both workspace and native reports retain four unmaintained warnings: `ttf-parser0.25.1` (`RUSTSEC-2026-0192`), `zbus_names4.3.5` (`RUSTSEC-2026-0371`), `zvariant5.15.1` (`RUSTSEC-2026-0370`), `zvariant_derive5.15.1` (`RUSTSEC-2026-0372`); both also report yanked `async-executor1.14.0` and `clipboard-win5.4.1`. These warnings are not ignored, upgraded away or described as a clean whole-project audit.
- Warning activation is retained desktop UI: `winit → sctk-adwaita → ab_glyph → owned_ttf_parser → ttf-parser`; `accesskit_winit → accesskit_unix → async-executor`; `accesskit_unix → zbus → zbus_names`; `accesskit_unix → accesskit_atspi_common → zvariant → zvariant_derive`; and `arboard → clipboard-win`. Native dependencies select these roots only through `ui-native`/`ui-acceptance`; `tsrust-ui` selects them through its explicit `native`/`acceptance` features. Compiler consumers disable `tsrust-ui` default features. The backend manifest excludes these desktop roots entirely. Lock reachability is not proof that an optional feature was activated in a particular binary; future UI revival must revisit these retained maintenance/yank findings. The existing B16/B17 private audits remain separate exact-lock evidence.
- Declared MSRVs are not all-target compiler proof. The common direct packages include undeclared MSRVs (`webpki-roots`, `subtle`, `bigdecimal`, `rusqlite`, `matchit`, `compression-core`). Do not substitute a guessed value. Archive/source integrity and notices remain closure-wide requirements, including transitive packages.
- Optional features and empty backend defaults are source-level selection contracts, not measured minimal-program build-time/size evidence. B20 still requires current four-host native/package/installed-consumer acceptance and the accumulated minimal-dependency performance review.

## Current per-export crosswalk refresh

`crates/tsrust-compiler/examples/public_api_inventory.rs` derives exact identities
and native signatures from the compiler registry, joins the current editor catalog
and reference routes, and discovers bundled testing/logging/iterator exports.
Its output lists existing B01–B19 witness target/case selectors independently of
per-export acceptance. B19 includes adapter-generation workflows, rather than a
fabricated `std:rust` export family. `tools/domain-reference-inventory.mjs` remains
upstream declaration comparison; it is not the current host-export ledger.

Each export maps its exact catalog examples and explicit witness review state.
Filesystem and URL catalog-example executions retain reviewed965de83f passes only
while their exact group example digests match; other positive/negative/alias/
platform coverage remains explicitly unreconciled.
Catalog examples are source witnesses with unreviewed execution, never inferred
passes. Reconcile these with retained exact-case evidence before changing status;
a family test or module count alone cannot establish a per-export pass. Keep
private native lookup identities distinct from user-importable source exports.
This source update has not produced a fresh compiled inventory or completed that
reconciliation. Existing `globals-and-imports-coverage.json` remains its separate
naming-review record; do not replace its reviewed identities with coverage claims.
