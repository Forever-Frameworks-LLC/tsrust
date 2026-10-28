# Compiler and analysis library

This crate provides the `tsrust` command and the `tsrust_compiler` Rust library.
It checks Tsrust source and generates Rust; the pinned Rust backend compiles that
output into native applications. The implemented [language server](../tsrust-lsp/)
uses this same library for diagnostics and editor intelligence.

For application development, start with [installation](../../website/src/content/docs/start/installation.md),
[the CLI guide](../../website/src/content/docs/tools/cli.md) and
[modules](../../website/src/content/docs/language/modules.md).
This README covers the compiler crate rather than duplicating the language reference.

## Build and test

Follow [contributor setup](../../CONTRIBUTING.md#set-up) to prepare the pinned Rust
toolchain, linker and locked dependencies. From the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --offline --locked \
  -p tsrust-compiler -p tsrust-lsp --bin tsrust --bin tsrust-lsp
```

The workspace has external dependencies; offline builds require explicit source
preparation first. The compiler and LSP can check source without an external Rust
compiler. Native build/run commands require the exact backend release recorded by
`RUST_BACKEND_VERSION`, currently Rust 1.96.1. See the
[toolchain policy](../../docs/guides/tooling/rust-toolchain-policy.md).

Use the [development workflow](../../docs/development-workflow.md) to select affected
checks and meaningful milestone regressions. Do not run the complete suite after
every routine edit. Compiler integration targets live in [tests/](tests/);
shared formatting/native helpers live in [tests/support/](tests/support/).

## Library entry points

The public entry points and types are defined in [src/lib.rs](src/lib.rs).

| API | Purpose |
| --- | --- |
| `check(&str)` | Parse and check standalone source in process; returns `Analysis` or the first strict `Diagnostic`. It does not invoke Rust or load a project from disk. |
| `diagnostics(&str)` | Collect recoverable source diagnostics without exposing a recovered tree to code generation. Successful analysis returns its warnings. |
| `Analysis::emit_rust()` | Generate Rust from successfully checked source. |
| `Analysis::emit_rust_with_native_features()` | Generate Rust and report selected native binding features. |
| `Analysis::contracts()` | Return inferred parameter-access contracts. |
| `Analysis::closures()` | Inspect capture storage/access, invocation, escape and parameter contracts. |
| `Analysis::generic_records()` | Inspect materialized generic-record provenance and structured type arguments. |
| `Analysis::warnings()` | Return warnings from the checked program. |
| `Analysis::to_json()` | Serialize the analysis summary. This does not substitute for project diagnostics. |

For imports and project analysis, use the [project APIs](src/project/mod.rs) and
[`ProjectSession`](src/project/session.rs). The session retains parsed inputs and
invalidates dependencies; standalone `check` is not a project loader. Source spans
use half-open UTF-8 byte offsets. The LSP converts these to editor positions,
including UTF-16 surrogate pairs, and checks current unsaved text.

Recovery diagnostics are editor feedback, not permission to execute invalid code.
Strict checking must succeed before ordinary emission or native compilation.

## Source layout

- [syntax/](src/syntax/) owns tokenization, parsing and source syntax.
- [analysis/](src/analysis/) owns type/access checking and inferred contracts.
- [project/](src/project/) owns module loading and project sessions.
- [emit/](src/emit/) generates Rust from checked programs.
- [stdlib/](src/stdlib/) supplies compiler-owned library declarations and runtime support.
- [cli/](src/cli/) implements commands, native builds, setup and paired toolchain installation.

See the [source layout guide](../../docs/guides/compiler-source-layout.md) for
feature-specific modules and shared boundaries.

## Behavior and acceptance evidence

[Language guides](../../docs/guides/language/) describe individual supported
features and explicitly marked research. Current public guidance lives in
[the documentation source](../../website/src/content/docs/). Compiler-owned
IntelliSense signatures and executable examples are maintained in
[the LSP catalog](../tsrust-lsp/src/builtins.json).

[Milestones](../../docs/milestones/) retain historical acceptance, including the
original ownership/control-flow proofs and later feature families. Their test
counts describe those checkpoints, not the current complete suite.
[Benchmark documentation](../../BENCHMARKS.md) distinguishes measurements from
correctness acceptance. The [public dataset contract](../../benchmarks/storage/PUBLIC_DATA.md)
defines shareable measurements; historical private logs must not be published.
