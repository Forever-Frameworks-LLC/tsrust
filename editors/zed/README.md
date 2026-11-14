# Tsrust for Zed

Local language support for .tsr and .tsrx using the shared tsrust-lsp process.
Install the compiler and language server on PATH, then use Zed's “Install Dev
Extension” action to select this directory.

Alternatively configure an absolute server path in Zed settings:

```json
{
  "lsp": {
    "tsrust-lsp": {
      "binary": { "path": "/absolute/path/to/tsrust-lsp" }
    }
  }
}
```

This first integration uses Zed's built-in TSX grammar. Compiler diagnostics,
completion, hover and navigation come from Tsrust. Tsrust-specific match syntax
and ownership modifiers can produce Tree-sitter syntax errors, affecting
highlighting, indentation and outline. A dedicated grammar remains necessary
for complete syntax support.

Build the adapter with the pinned Rust toolchain and wasm32-wasip2 target:

```sh
CARGO_TARGET_DIR=../../target CARGO_INCREMENTAL=0 cargo build --release --target wasm32-wasip2
cp ../../target/wasm32-wasip2/release/tsrust_zed.wasm extension.wasm
```

Run these commands from editors/zed. The generated WASM stays out of Git.
