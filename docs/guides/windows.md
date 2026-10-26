# Windows development

Windows x64 support is being validated on GitHub's `windows-2025` MSVC runners.
Hosted compilation and CLI/LSP tests come first; real desktop rendering, keyboard,
accessibility and terminal behavior still need acceptance on a Windows PC.

## Try a hosted build

After a Windows CI job passes, open that run's **Artifacts** section and download
`windows-x64-development-toolchain` and `vscode-extension`. The first artifact
contains a toolchain ZIP; extract both archive layers and keep its notices.
These are development builds, not optimized tagged releases. Artifacts expire
seven days after creation.

Tagged releases add a `tsrust-vVERSION-x86_64-pc-windows-msvc.zip` asset. Verify its
SHA-256 against the release's `SHA256SUMS`, then extract it:

```powershell
Get-FileHash .\tsrust-v0.1.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256
Expand-Archive .\tsrust-v0.1.0-x86_64-pc-windows-msvc.zip -DestinationPath .\tsrust-toolchain
```

Substitute the downloaded version. Compare the displayed digest with the exact
asset's line in `SHA256SUMS` before extraction. Keep `bin\tsrust.exe` and
`bin\tsrust-lsp.exe` together on PATH, or configure their full paths in VS Code.
Install the `.vsix` through **Extensions → Install from VSIX**.

## Native prerequisites

Install Rustup and Visual Studio Build Tools with **Desktop development with C++**,
including the Windows SDK. Select the MSVC host toolchain:

```powershell
rustup toolchain install 1.96.1-x86_64-pc-windows-msvc --profile minimal
```

Checking and formatting do not need Rust. Native compilation needs Rust and the
MSVC linker; archives do not bundle either. Git and Node.js 24 are also needed for
repository development. Native service dependencies can require CMake and NASM;
the hosted setup installs/exposes NASM explicitly.

## Work from source

```powershell
git clone https://github.com/Forever-Frameworks-LLC/tsrust.git
cd tsrust
$env:CARGO_INCREMENTAL = "0"
$env:CARGO_BUILD_JOBS = "1"
$env:CARGO_TARGET_DIR = "$PWD\target"
cargo fetch --locked
cargo fetch --locked --manifest-path crates/tsrust-compiler/native-dependencies/Cargo.toml
cargo build --offline --locked --bin tsrust --bin tsrust-lsp
.\target\debug\tsrust.exe new hello
.\target\debug\tsrust.exe run .\hello\src\main.tsr
```

The repository's `.gitattributes` keeps source files at LF on all hosts so formatter
output and benchmark fingerprints remain stable. Do not renormalize old source or
replace lockfiles merely because the host changes. Keep one Cargo target and one
build owner; avoid enabling incremental compilation in this project.

For editor paths use JSON strings with escaped backslashes, for example:

```json
{
  "tsrust.compiler.path": "C:\\dev\\tsrust\\target\\debug\\tsrust.exe",
  "tsrust.server.path": "C:\\dev\\tsrust\\target\\debug\\tsrust-lsp.exe"
}
```

The LSP drive-path bridge handles local drive letters, spaces, Unicode, escaped
characters and Rust's canonical verbatim drive paths. Remote/UNC file authorities
remain rejected. Long-lived LSP memory pressure is still unresolved; start with a
small project rather than scanning the entire repository immediately.

## Current limits

- `tsrust dev` process supervision is currently macOS/Linux only. On Windows use
  explicit `check`, `build`, `run`, `test` and `format` commands while that work is
  ported. Both native and web development watchers share this limitation.
- Consuming `std:fs` close uses a Unix OS backend. Programs using it are rejected
  on Windows at native compilation; the backend must be ported with its ownership
  and independent-release checks intact.
- Shutdown signal bindings return their explicit unsupported error on non-Unix
  targets. They do not silently simulate Unix signals.
- Native renderer compilation does not prove GPU initialization, input, window
  chrome, accessibility or clipboard behavior. Custom titlebars remain macOS-only.
- Windows ARM, GNU/MinGW releases and the full compiler regression suite are not
  accepted by this x64 MSVC baseline.

The CI baseline deliberately exercises supported operations rather than hiding
platform failures behind `continue-on-error`. Do not describe a queued or failed
Windows job as accepted support. Use the first hosted failures and real PC results
to drive the next portability section.
