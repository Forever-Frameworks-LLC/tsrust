<!-- tsrust-docs
{"id":"installation","group":"getting-started","order":10,"status":"ready","targets":["web","native"]}
-->
# Installation

Tsrust includes a CLI, a language server and an offline catalog of editable UI
components. It is experimental; language and library APIs may change.

## Download a release

Open [GitHub Releases](https://github.com/Forever-Frameworks-LLC/tsrust/releases)
and choose the archive matching your host. Download `SHA256SUMS` alongside it.
Release archives are configured for Linux x64, macOS Apple Silicon and macOS
Intel. Until the first release is published, use the source build below.

Verify the selected archive's checksum before extraction. For example, on macOS:

```sh
grep 'tsrust-v0.1.0-aarch64-apple-darwin.tar.gz$' SHA256SUMS | shasum -a 256 --check
tar -xzf tsrust-v0.1.0-aarch64-apple-darwin.tar.gz
```

Substitute the version and target you downloaded. On Linux use `sha256sum --check`.
Put both files from the extracted `bin/` directory on your `PATH`. You can keep
them together in a directory such as `~/.local/bin`:

```sh
mkdir -p ~/.local/bin
cp tsrust-v0.1.0-aarch64-apple-darwin/bin/tsrust ~/.local/bin/
cp tsrust-v0.1.0-aarch64-apple-darwin/bin/tsrust-lsp ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"
tsrust --version
```

Add that PATH setting to your shell profile to retain it. Keep the extracted
license and attributions with any redistributed copy. Archives include a
`manifest.json` recording source commit, both consumer versions and file hashes.
Checksums detect corruption; these binaries are not code-signed or notarized.
Linux archives require a glibc system compatible with Ubuntu 24.04, not Alpine.

## Install the native backend

Checking, formatting and emitting Rust need no external Rust compiler. Building
and running native programs require [Rustup](https://rustup.rs/), **Rust 1.96.1**
and a system linker/C toolchain. Tsrust does not download them implicitly.

```sh
rustup toolchain install 1.96.1 --profile minimal
```

Use Xcode Command Line Tools on macOS, or the distribution's C build tools on
Linux. Building GPU applications on Linux also requires native development
libraries; see [contributor setup](../../../CONTRIBUTING.md).

## Build from source

```sh
git clone https://github.com/Forever-Frameworks-LLC/tsrust.git
cd tsrust
cargo fetch --locked
cargo fetch --locked --manifest-path crates/tsrust-compiler/native-dependencies/Cargo.toml
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --offline --locked --release --bin tsrust --bin tsrust-lsp
```

The repository pins Rust and its dependencies. Rustup can install the pinned
compiler on first use. The two fetch commands prepare the workspace and bundled
native dependency source graphs. Put both executables from `target/release` on
`PATH`. No Node runtime is needed by the generated native application.

## Install editor support

Download the release's `.vsix` file and use **Extensions → Install from VSIX** in
VS Code. The VSIX does not embed the Rust server: keep `tsrust` and `tsrust-lsp`
on `PATH`. The extension also searches `~/.cargo/bin` and beside configured
companion executables. See [VS Code setup](../../../editors/vscode/README.md) for
explicit paths and local packaging.

The current LSP has an unresolved excessive-memory report during long workspace
sessions. Use small workspaces and stop the language server if memory grows.

## Create a project

```sh
tsrust new hello
cd hello
tsrust run
```

Run `tsrust` in a terminal for the interactive launcher. For UI, select targets
explicitly:

```sh
tsrust new my-app --ui --target native --target web
cd my-app
tsrust dev --target native
```

Pass target flags for repeatable scripts. In a terminal, the CLI can ask for
missing target choices interactively. Selecting web does not enable native or
embedded implicitly. Native execution requires a graphical desktop and supported
GPU. Custom native titlebars currently require macOS; other desktop platforms
use the native titlebar.

## Browser prerequisites

Web UI uses WebAssembly and the same checked component source. Install these
additional tools explicitly:

```sh
rustup target add wasm32-unknown-unknown --toolchain 1.96.1
cargo +1.96.1 install wasm-bindgen-cli --version 0.2.129 --locked
tsrust dev --target web --port 4317
```

Web development watches source and Markdown, reuses the compiler session and Wasm
workspace, and reloads browsers after successful builds. Failed builds keep the
previous application available. Dependencies compile offline after setup.

## Add components offline

```sh
tsrust ui add button card input --offline
```

The bundled catalog needs no registry or package-manager download. The first
installation creates missing UI configuration and source directories. Importing
a component alone does not create setup: follow the diagnostic's `tsrust ui add`
command. The catalog is still being completed; inspect `tsrust ui list --bundled`
for the actual available components.

Installation preserves existing source. A conflicting upgrade gets a versioned
filename such as `card-v3.tsrx`; newly installed bundled dependencies use the
selected file. Existing application imports stay under your control.

## Check the project

```sh
tsrust check src
tsrust test
tsrust format --check
```

Checks and editor analysis are read-only. See [project structure](project-structure.md)
and [theming](../ui/foundations/theming.md) for configuration and customization.
