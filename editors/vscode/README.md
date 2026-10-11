# Tsrust for VS Code

Local editor support for `.tsr`: a file icon, syntax highlighting, documentation hover, argument help, autocomplete, source navigation, snippets, and live syntax, type and ownership errors from the actual Tsrust compiler. Errors include related transfer/access locations and appear in Problems. Unsaved files and untitled Tsrust buffers are checked in memory; Rust code generation and rustc are not involved.

## Build and install locally

From the repository root, with its pinned Rust toolchain and Node.js 22 or newer:

```sh
cargo fetch --locked
CARGO_TARGET_DIR=target/editor cargo build --offline --locked -p tsrust-lsp
npm --prefix editors/vscode ci
npm --prefix editors/vscode run package
code --install-extension editors/vscode/tsrust-0.2.0.vsix --force
```

The compiler crate remains dependency-free. The new server has protocol/JSON dependencies; `cargo fetch --locked` downloads them once before offline builds. The VSIX bundles its JavaScript dependencies and their license notices, but does not contain the Rust server binary. Nothing is published to the Marketplace.

Open VS Code Settings and set **Tsrust › Server: Path** to the absolute path of `target/editor/debug/tsrust-lsp`. For this repository checkout:

```json
{
  "tsrust.server.path": "/Users/seancotter/Desktop/dev/learning/tsrust/target/editor/debug/tsrust-lsp",
  "tsrust.analysis.debounceMs": 200
}
```

Set it in user settings to reuse the binary across projects, or workspace settings for this project. After upgrading, run **Developer: Reload Window** once to load the new extension contributions. Open a `.tsr` file in a trusted workspace. Settings changes restart the server. **Tsrust: Restart Language Server** and the Tsrust status-bar entry restart it manually. **View → Output → Tsrust** shows startup/protocol errors. A missing executable leaves highlighting available and explains the required setup. Unexpected server termination leaves a stopped status; there is no automatic restart loop.

Remove the extension with `code --uninstall-extension tsrust-local.tsrust` or the Extensions panel. Remove the Tsrust settings if no longer needed.

## Supported limits

Each file is a separate single-file program. The compiler reports its first error and checks the whole file, so unfinished syntax can hide later errors. Checks run on one background worker; edits replace pending buffers and obsolete results are suppressed, but running checks cannot be cancelled yet. UTF-8 compiler spans are converted to UTF-16 editor positions, including emoji and CRLF. Closing/reopening a document invalidates its old results.

Hover displays source-declared function/callback signatures, adjacent `/** ... */` documentation and line comments, including `@param`/`@returns` text. Signature help highlights the active argument and shows parameter descriptions. Completion suggests visible declarations, declared record fields, keywords, and supported built-ins with docs. Definitions, references, highlights, outline symbols, and folding work within a file; documented function/type/control-flow snippets are included.

These features use a tolerant source index so they also work in unfinished files. Types come from annotations and straightforward literals or declared call results; unknown or compound-expression types stay unknown. This is source assistance, not compiler-proven binding/type inference. Complex callable origins, semantic rename, checked inferred ownership hover, formatter, debugger and cross-file module analysis still need compiler work. The compiler handoffs for those capabilities are tracked in `docs/editor-foundation.md` in the repository. Local macOS is the verified platform; other platforms require their own integration verification. Remote workspaces require a server executable on the remote extension host.

The file icon uses the standard language contribution and respects your chosen icon theme; a theme with its own explicit mapping can override it.

## Verify or develop

```sh
CARGO_TARGET_DIR=target/editor cargo test --offline --locked -p tsrust-lsp
CARGO_TARGET_DIR=target/editor cargo clippy --offline --locked -p tsrust-lsp --all-targets -- -D warnings
cargo fmt -p tsrust-lsp --check
npm --prefix editors/vscode test
```

For the VS Code Extension Host integration suite, point at the installed executable and built server (or omit the executable setting to let the test runner download VS Code):

```sh
VSCODE_EXECUTABLE='/Applications/Visual Studio Code.app/Contents/MacOS/Code' \
TSRUST_LSP_PATH="$PWD/target/editor/debug/tsrust-lsp" \
npm --prefix editors/vscode run test:integration
```

The suite creates temporary settings, extensions and source files, verifies unsaved errors, related locations, Unicode, repair, multiple files, rapid edits, restart, close/reopen, missing-server behavior, documentation hover, signatures, incomplete-source autocomplete, definitions/references and outline, then removes its temporary files. It does not change your normal VS Code profile. Set `TSRUST_VSIX_PATH` to an absolute packaged VSIX path to install and test that package in the temporary profile.

Check current compiler/editor contracts with `python3 editors/vscode/scripts/check-compiler.py` from the repository root. The script fingerprints compiler/LSP sources and the coverage inventory, reruns focused LSP tests only on change (or with `--force`), and stores successful state in `target/editor/compiler-compatibility.json`. No recurring task is installed by this command.

For interactive development, run `npm --prefix editors/vscode run compile`, open `editors/vscode` as a VS Code folder and select **Tsrust Extension** in Run and Debug. Configure the server path in the launched Development Host.

The adapter uses the official [Language Server Protocol](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/) and Microsoft's [VS Code language client](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide). Runtime dependency versions are pinned in `Cargo.lock` and `package-lock.json`; bundled third-party licenses are generated in `THIRD_PARTY_NOTICES.txt`.

### Quick fixes

Use the lightbulb or **Cmd+.** on a syntax diagnostic to insert a missing semicolon or closing delimiter. Fixes preserve comments and are offered only when the edit makes the current document pass compiler checking. Other errors and documents with multiple remaining errors may have no quick fix.

### Run, check and format

Use the play button on a .tsr editor or **Tsrust: Run File** to save and execute it in a terminal. **Tsrust: Check File** runs compiler syntax/type/ownership checks. **Tsrust: Format Document** and standard Format Document use the shared Tsrust formatter. Set `tsrust.formatOnSave` and `tsrust.checkOnSave` to enable automatic actions. Set `tsrust.compiler.path` if the compiler is not beside the configured language server. Commands require workspace trust.

Transferred strings, arrays and records can offer **Clone before transferring (copies data; may allocate)** via Cmd+.; the earlier transfer is edited only if the complete document then passes checking.

For isolated CLI distribution acceptance, build the compiler with `CARGO_TARGET_DIR=target/editor cargo build --offline --locked -p tsrust-compiler`, then run `python3 editors/vscode/scripts/check-cli-distribution.py`. This copies only the CLI to a temporary directory, verifies check/emit with an empty PATH and builds/runs a two-file std:path project from an unrelated directory containing spaces. It requires the installed pinned Rust backend, cleans up its temporary project/cache and saves evidence under target/editor. It does not change a global installation. The separate check-offline-build.py verifies network-denied bundled builds on macOS.
