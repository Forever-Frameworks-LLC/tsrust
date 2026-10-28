# Tsrust for VS Code

Editor support for `.tsr` and `.tsrx`, powered by the shared compiler analysis.
Install the release VSIX through **Extensions → Install from VSIX**. The CLI and
language server are installed separately; Marketplace publication is not configured.

[Editor setup, commands and troubleshooting](../../website/src/content/docs/start/editor.md)
contains the canonical user guide, including executable discovery, Windows paths
and known long-session memory limitations.

## Build locally

From the repository root, with Node.js 24, npm and the pinned Rust toolchain:

```sh
cargo fetch --locked
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --offline --locked --release -p tsrust-compiler -p tsrust-lsp --bin tsrust --bin tsrust-lsp
npm --prefix editors/vscode ci
npm --prefix editors/vscode run package
code --install-extension editors/vscode/tsrust-0.2.0.vsix --force
```

Keep both `target/release` executables on PATH or configure their absolute paths.
Packaging copies the project license and generates `THIRD_PARTY_NOTICES.txt` for
the bundled client. It publishes nothing externally.

```sh
npm --prefix editors/vscode test
```

The Extension Host integration suite is separate. It uses a temporary VS Code
profile and needs an installed VS Code or permission to download its test build:

```sh
TSRUST_LSP_PATH="$PWD/target/release/tsrust-lsp" \
  npm --prefix editors/vscode run test:integration
```

Set `VSCODE_EXECUTABLE` to select a local VS Code executable and `TSRUST_VSIX_PATH`
to exercise a packaged VSIX. See the [development workflow](../../docs/development-workflow.md)
for compiler/LSP acceptance. Hosted extension tests do not establish native OS
interaction or long-lived LSP memory safety.

The file icon respects the active icon theme. Remote workspaces need executables
on the remote extension host. Uninstall with `code --uninstall-extension tsrust-local.tsrust`.

## License

MIT. Bundled third-party client code retains the licenses in
`THIRD_PARTY_NOTICES.txt` generated during packaging.
