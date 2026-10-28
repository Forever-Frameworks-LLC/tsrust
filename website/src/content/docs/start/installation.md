---
title: Install Tsrust
description: Install the paired CLI and language server on macOS, Linux or Windows.
---

Tsrust provides a native backend CLI and language server. The language is
experimental; language and library APIs may change. Checking, formatting and
ordinary Rust emission work without a Rust installation.

## Download a release

Choose the archive for your host from
[GitHub Releases](https://github.com/Forever-Frameworks-LLC/tsrust/releases):
Linux x64/glibc, Windows x64/MSVC, macOS Apple Silicon or macOS Intel.
If no stable release has been published, use the source build below.

Verify the archive against the digest on the official release/API before extracting
it. Releases also contain SHA256SUMS. For example, after downloading the matching
archive and checksum file from that official release:

```sh
grep 'tsrust-v0.1.0-aarch64-apple-darwin.tar.gz$' SHA256SUMS | shasum -a 256 --check &&
  tar -xzf tsrust-v0.1.0-aarch64-apple-darwin.tar.gz
```

Substitute the actual version and target. Linux uses `sha256sum --check`; Windows
can calculate the selected archive's SHA-256 with `Get-FileHash -Algorithm SHA256`
in PowerShell, then compare the returned hash with the official digest before
extracting. Stop if verification fails. A checksum from an untrusted source does
not authenticate an archive.
The binaries are not independently code-signed or notarized. Linux archives target
a glibc system compatible with Ubuntu 24.04; Alpine/musl and Linux ARM archives are
not provided.

## Choose an installation directory

Keep the archive's license, attributions and manifest. Add its extracted `bin`
directory to PATH to use the unpacked pair. Restart your terminal if your PATH
change is not visible, then check `tsrust --version` and `tsrust-lsp --version`.

For managed updates and rollback, use that unpacked CLI to install a pair in a
**new directory whose parent already exists**:

```sh
tsrust toolchain install --root /absolute/path/to/new/tsrust
```

Add the new directory's `bin` to PATH. The command never overwrites an existing
installation, changes shell profiles, or installs Rust. On Windows the executables
are `tsrust.exe` and `tsrust-lsp.exe`; preserve their suffixes.

## Check, update or roll back

```sh
tsrust toolchain check --json
tsrust toolchain update
tsrust toolchain rollback
```

Check is read-only. Update verifies the official immutable stable release and
stages a complete pair before switching one installation record. Running tools
remain on their original versions; restart the editor server to select the new
pair. Rollback selects the retained verified pair offline. Package-manager-owned
installations receive upgrade guidance instead of being overwritten. See the
[managed installation guide](../../tools/install-and-update/)
for trust, recovery and platform acceptance.

## Install the native backend

Native build, run, test and dev currently select **Rust 1.96.1 exactly**, through
[Rustup](https://rustup.rs/). A newer default Rust installation does not replace
that selected toolchain. Setup keeps the pin alongside your other Rust versions
without changing the global default.

You also need host build tools:

| Platform | Required host tools |
| --- | --- |
| macOS | Xcode Command Line Tools |
| Linux | C compiler and system linker |
| Windows | Visual Studio Desktop development with C++ workload and Windows SDK |

Tsrust does not install these platform SDKs or linkers implicitly.

After installing rustup, prepare the pinned tools and shared dependency sources:

```sh
tsrust setup --toolchain --dependencies --yes
tsrust doctor --native
tsrust doctor --json
```

Setup preserves global Rust defaults and unrelated configuration. Normal builds
use locked Cargo sources offline; missing sources produce an explicit setup
instruction. Doctor reports CLI/LSP installation integrity separately from Rust,
Cargo and linker readiness. A native probe proves a small host compile/link/run,
not every application dependency or target. Use `setup --dependencies --offline`
when the needed sources are already cached.

Portable source exports are optional. Prepare their exact source pack with
`tsrust setup --portable --yes`; binding exports use
`tsrust bindings resolve file.bindings.json --portable`. Neither ordinary builds
nor `emit --package` silently download sources. Deployed native applications need
no Tsrust, Rust, Cargo, repository or developer cache.

## Build from source

```sh
git clone https://github.com/Forever-Frameworks-LLC/tsrust.git
cd tsrust
cargo fetch --locked
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --offline --locked --release -p tsrust-compiler -p tsrust-lsp --bin tsrust --bin tsrust-lsp
export PATH="$PWD/target/release:$PATH"
```

The workspace pins its Rust toolchain and dependencies. Rustup may install the pin
on first Cargo use. The fetch prepares compiler/LSP build dependencies; application
runtime sources are prepared separately by `tsrust setup --dependencies --yes`.
No Node runtime is needed by the compiler or deployed native application.

## Windows source installation

Use the `x86_64-pc-windows-msvc` Rust host and installed Visual Studio C++/Windows
SDK tools. From the checkout in PowerShell:

```powershell
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_BUILD_JOBS = '1'
cargo fetch --locked
cargo build --offline --locked --release -p tsrust-compiler -p tsrust-lsp --bin tsrust --bin tsrust-lsp
$env:PATH = "$PWD\target\release;$env:PATH"
tsrust --version
tsrust-lsp --version
tsrust setup --dependencies --yes
tsrust doctor --native
```

These PATH settings apply to the current terminal. Persist your chosen PATH using
the operating system or shell configuration. If installing with `cargo install`
instead, keep both consumers together and use Cargo for their upgrades; they are
not owned by the managed updater.

Windows dev uses native directory notifications and owned Job Objects. Failed
builds preserve the previous application. Ctrl+C or Ctrl+Break stops the supervisor
and its descendants. `TSRUST_DEV_WATCH=poll` requests the polling fallback.

## Install editor support

Follow [VS Code setup](../editor/) for the VSIX, executable discovery and recovery
from stale development paths. The extension requires the separate CLI/LSP pair.

## Create a project

```sh
tsrust new hello --app
cd hello
tsrust dev
```

The starter is a native greeting HTTP API on loopback port 4318. Source and access
policy live in `app/`; no frontend project, SDK or public dispatcher directory is
created. Generate independent TypeScript/Rust clients explicitly from its checked
HTTP description. Use `tsrust new hello` for a console project. Bare `tsrust` opens
the interactive launcher in a terminal and prints help for redirected input.

## Check the project

```sh
tsrust check
tsrust test
tsrust format --check
```

See [project structure](../project-structure/) and the
[backend application guide](../../backend/applications/).
`tsrust new` writes a project AGENTS.md describing its commands and ownership
boundaries. Existing owner projects are never overwritten.
