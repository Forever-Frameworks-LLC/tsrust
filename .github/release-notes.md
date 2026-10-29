Build native backend services and tools with TypeScript-style syntax, inferred
ownership and a Rust backend. This distribution includes the `tsrust` CLI and the
`tsrust-lsp` language server, plus the separate VS Code extension. Applications
expose HTTP APIs; bring your own UI and generate TypeScript or Rust clients with
`tsrust api generate`.

Use `tsrust new service --app` to create a backend project, `tsrust new hello` for
a console program, or `tsrust` for the interactive launcher.

Documentation: https://forever-frameworks-llc.github.io/tsrust/

Download the archive matching your host, verify it against `SHA256SUMS`, and put
both `bin/` executables on `PATH`, or install a managed copy with
`tsrust toolchain install --root NEW_DIRECTORY`; managed installations update with
`tsrust toolchain check` and `tsrust toolchain update`. Install the separate VSIX
in VS Code. Native compilation requires Rustup with Rust 1.96.1 and a system
linker; these archives do not include Rust. Linux archives target glibc systems
(built on Ubuntu 24.04), not static musl binaries.

Tsrust is experimental. Language and library APIs may change between minor
versions. The executables are not code-signed or notarized. Large-workspace and
multi-hour language-server memory behavior is not yet verified. Windows-specific
`std:path` behavior is documented but was not verified on Windows by hand.
The original Tsrust UI is frozen and not part of this release.

See the installation guide and release guide in the tagged source for
prerequisites, verification commands and test coverage.
