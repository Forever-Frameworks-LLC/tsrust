---
title: Prepare native build tools
description: Inspect your host and explicitly prepare Rust and shared dependency sources.
---

`tsrust check` and ordinary Rust source emission do not need Rust installed.
Native build, run, test and dev use Tsrust's exact pinned Rust toolchain.

## Inspect this machine

```sh
tsrust doctor
tsrust doctor --json
tsrust doctor --native --json
```

Doctor never installs tools or fetches dependencies. Its versioned JSON report
identifies the host OS/architecture, required Rust version, and available pinned
rustc/Cargo. `--native` additionally compiles, links and executes a tiny program
in an automatically removed temporary directory. This checks host build tools,
not application dependencies or every optional runtime feature. Those remain
explicitly unchecked.

| Exit status | Meaning |
| --- | --- |
| `0` | Rust tool checks and any requested native probe pass, with no managed-installation integrity error. |
| `1` | A requested readiness check or managed-installation integrity check failed. |

An unmanaged CLI is not itself an integrity error. A successful doctor result
does not certify that every application is ready to build.

## Install the pinned Rust toolchain

Install [rustup](https://rustup.rs) first if it is missing, then explicitly prepare
the pinned compiler:

```sh
tsrust setup --toolchain
# Unattended, with permission to download the pinned toolchain:
tsrust setup --toolchain --yes
```

Without `--yes`, toolchain setup asks before invoking rustup; noninteractive use requires
that flag. Setup requests rustup's minimal profile for the exact pin and disables
rustup self-update for this invocation. It does not select a global default,
change directory overrides, run downloaded installation scripts, or install
platform SDKs/linkers. Windows requires the appropriate Visual Studio C++ build
tools; macOS requires its command-line build tools; Linux requires the platform
C toolchain and linker. Run `doctor --native` to test the actual host combination.

On Windows MSVC hosts, native builds use the LLVM linker bundled with the pinned
Rust toolchain to support deep output paths. Visual Studio C++ build tools and
Windows SDK libraries are still required. Exported packages include the same
Windows-only linker setting; macOS and Linux retain their platform linkers.

## Repair a missing native linker

Finding Rustup does not prove that the platform linker and SDK are available.
For a native build error containing **Native linker unavailable**, inspect the
backend error immediately below it for the missing linker name.

1. Install or repair the host build tools listed in
   [installation](../../start/installation/#install-the-native-backend).
2. Restore the required tool directories to the environment used by your terminal
   or editor. Restart that terminal or editor after changing its environment.
3. Run `tsrust doctor --native`, then retry the build from that same environment.

Tsrust does not install platform SDKs or linkers automatically. Checking and
ordinary Rust emission still work without a linker. A failed native compilation
does not publish a new executable to the build cache.

The missing-linker diagnostic applies when Rust explicitly reports that its linker
was not found. Other backend failures retain their details; do not treat every
linking or generated-code error as a missing-tool problem.

## Prepare sources for normal backend builds

After installing the pinned toolchain, prepare compiler-owned backend dependency
sources explicitly:

```sh
tsrust setup --dependencies --yes
# Validate/populate from already cached archives without network access:
tsrust setup --dependencies --offline
# Prepare both tools and sources, with download consent:
tsrust setup --toolchain --dependencies --yes
```

Source preparation delegates to `cargo fetch --locked`. The backend profile excludes
legacy rendering dependencies. Ordinary backend builds use Cargo's shared home,
exact locks and `--offline`; they create no vendor copy or private Cargo home.
A missing source produces a setup instruction rather than an automatic download.

### Reuse cached builds

The compiler hashes Cargo configuration (including nested and optional includes)
and environment inputs into artifact keys;
configuration changes invalidate executable/runtime artifacts while the stable Cargo
target remains reusable. Inputs are checked again before publishing cached artifacts.
Raw configuration contents and environment credentials are
not written into identity files. Package resolution and source storage remain Cargo's
responsibility.

`--offline` applies to dependency preparation and cannot be combined with toolchain
installation. Offline preparation needs no download consent. The shared Cargo home
follows `CARGO_HOME`, otherwise the normal user `.cargo` directory. Its configuration
and ancestor configuration of `TSRUST_CACHE_DIR` (by default the project's
`target/tsrust-native`) apply to backend support builds. The pinned toolchain and
compiler-owned runtime profiles remain explicit. Existing global defaults and
configuration files are never rewritten.

## Prepare sources for portable export

Portable export sources are optional and explicitly prepared, independently of normal
backend development:

```sh
tsrust setup --portable --yes
# Prepare entirely from existing Cargo sources, without network access:
tsrust setup --portable --offline
```

This invokes upstream `cargo vendor --locked --versioned-dirs` once into a staged,
versioned source pack. The pack retains the full legacy export closure, including
original UI dependencies, for compatibility. Normal compiler builds neither vendor
nor embed that payload. Setup validates the exact manifest/lock, Cargo package
identities and all vendor file checksums before publishing the pack by rename.
Existing modified packs are preserved and rejected; failed staging is removed.

Packs live under `tsrust/source-packs` inside the resolved Cargo home.
`TSRUST_SOURCE_PACK_DIR` overrides that location.
Set that variable to a persistent directory if exporting with a different Cargo home.
`emit --package` copies verified sources and licenses from the prepared pack without
invoking Cargo, Rust or the network. An unprepared export reports the setup command.
The resulting package includes its manifest, lock, runtime sources, toolchain pin,
Cargo offline configuration, notices and build instructions; it can be relocated
and rebuilt with an empty registry cache.

### Choose which sources to prepare

- Plain programs need no source pack.
- Retained UI builds use the prepared legacy pack.
- New custom binding graphs use schema 2 and Cargo’s shared sources; schema 1
  retains its legacy dependency graph. Resolve binding dependencies explicitly
  before offline builds. Binding portable exports validate and include their
  exact locked graph.

## Install the CLI and editor server separately

CLI/LSP acquisition is separate: see [managed installation and updates](../install-and-update/).
Doctor reports managed-pair ownership and integrity independently of Rust readiness.

## Related tool documentation

The [rustup profile documentation](https://rust-lang.github.io/rustup/concepts/profiles.html)
defines the minimal profile. Its [basic usage documentation](https://rust-lang.github.io/rustup/basics.html)
documents invocation-scoped `--no-self-update`.

[Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html) defines
configuration discovery and nested/optional includes. Cargo parses and applies
the build configuration. [cargo fetch](https://doc.rust-lang.org/cargo/commands/cargo-fetch.html)
provides explicit source acquisition before offline use.

[Cargo vendor](https://doc.rust-lang.org/cargo/commands/cargo-vendor.html) defines
locked/offline preparation and versioned directories.
