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

The bundled Tsrust grammar extends TSX with match expressions, block and value
arms, variant bindings, literal/wildcard patterns and contextual match identifiers.
Highlighting covers ownership annotations (ReadOnly, Mutable, Owned and Once),
Tsrust built-in types and typed JSX. Indentation, bracket and outline queries use
the same grammar. Semantic diagnostics remain compiler-owned.

The grammar source is in grammar/ and pins the upstream TypeScript grammar.
Generated parser C and scanner sources are checked in for reproducible builds.
To regenerate and validate it, run npm ci, npm run generate and npm test from
grammar/. If npm skips dependency install scripts, run
node node_modules/tree-sitter-cli/install.js before generating. Native parser
validation writes Tree-sitter's normal user cache.

For a local WASM grammar build, set WASI_SDK_PATH to a WASI SDK installation and
run npm run build:wasm from grammar/. This writes grammars/tsrust.wasm. The SDK
compiler invocation matches Zed's grammar builder.

The manifest pins the grammar's source commit. While that commit exists only
locally, Zed's automatic remote grammar checkout cannot fetch it. For this local
development setup, build both WASM files, then symlink this extension directory
as tsrust beneath Zed's extensions/installed directory. Rebuild Zed's generated
extension index when adding grammar metadata. Publishing requires making the
pinned grammar revision available in the repository.

Build the adapter with the pinned Rust toolchain and wasm32-wasip2 target:

```sh
CARGO_TARGET_DIR=../../target CARGO_INCREMENTAL=0 cargo build --release --target wasm32-wasip2
cp ../../target/wasm32-wasip2/release/tsrust_zed.wasm extension.wasm
```

Run these commands from editors/zed. The generated WASM stays out of Git.
