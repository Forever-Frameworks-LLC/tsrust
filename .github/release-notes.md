Build native tools and services with TypeScript-style syntax, inferred ownership
and a Rust backend. This distribution includes the CLI, language server and an
editable offline UI component catalog, plus the separate VS Code extension.

Use `tsrust new hello` to create a console project, or `tsrust` for the interactive
launcher. Shared browser/native UI projects use `.tsrx` and explicit target settings.

Tsrust is experimental. Language, library and component APIs may change.

Download the archive matching your host, verify it against `SHA256SUMS`, and put
both `bin/` executables on `PATH`. Install the separate VSIX in VS Code.
Windows x64 downloads use a ZIP containing `tsrust.exe` and `tsrust-lsp.exe`.
Windows support is experimental: dev supervision, consuming filesystem close and
shutdown signals remain Unix-only. See `docs/guides/windows.md` in the tagged source.
Native compilation requires Rustup with Rust 1.96.1 and a system linker. These
archives do not include Rust itself. Linux archives target glibc-based systems
built against Ubuntu 24.04; they are not static musl binaries.

The executables are not code-signed or notarized. The LSP has an unresolved report
of excessive memory use in long sessions. The UI library remains incomplete;
Table acceptance is pending. Hosted checks do not certify native GPU interaction
or assistive-technology behavior on a real desktop.

See the installation guide, status section and release guide in the tagged source
for prerequisites, verification commands and test coverage.
