# Toolchain and development experience

[Documentation index](../../README.md) · [Proposed roadmap](../../roadmap.md) · [Source section map](../../plans/section-map.md)

## Workstream scope

**Editor implementation plan:** [VS Code foundation](editor-foundation.md) scopes highlighting and unsaved single-file diagnostics against the current analysis API. Its C0–C5 handoff matrix records compiler obligations and capabilities that must wait. Whole-file first-error diagnostics can ship before recovery, aggregation or incremental analysis.

**Implemented editor foundation:** `crates/tsrust-lsp` is a Rust language server sharing the compiler's [in-process analysis library](../../../crates/tsrust-compiler/README.md). The locally installable [VS Code extension](../../../editors/vscode/README.md) provides file icons, highlighting, unsaved syntax/type/ownership diagnostics, source documentation hover, argument help, autocomplete and same-file navigation, alongside document versions, UTF-16 position conversion and stale-result suppression. Whole-file first-error checking remains the bounded contract; recovery, diagnostic aggregation, cooperative cancellation, semantic metadata and project analysis are deferred under C1–C5. See [acceptance evidence](../../milestones/editor-foundation-acceptance.md).

**Toolchain decision:** [Pin Rust 1.96.1 for both compilation stages](rust-toolchain-policy.md). Repository builds use `rust-toolchain.toml`; native `tsrust` builds explicitly select that release through Rustup. Installer-managed distribution is future work.

**Native profiles:** `tsrust build <input.tsr> --release -o <output>` and `tsrust run <input.tsr> --release` compile generated Rust at optimization level 3. The default is optimization level 0. Both enable overflow checks. The compiler executable's Cargo profile is independent of the generated program's profile; the CLI acceptance suite checks backend selection, exact flags, native output, and checked failures in both profiles.

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Core proof; ongoing.

**Scope:** Scope CLI commands, editor/debugger integration, toolchain pinning, and iteration speed.

**Planning deliverable:** A minimal check/build/run workflow and measured compile/edit latency; later tooling backlog.

**Dependencies to coordinate:** [compiler-and-modules](../language/compiler-and-modules.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-20"></a>

## 20. Build System

One unified CLI.
Potential commands:
lang new
lang run
lang build
lang test
lang check
lang fmt
lang check
lang add
lang remove
lang update
lang inspect
lang convert
lang doctor
Potential production build:
lang build --release
Output:
dist/my-api
A native executable.
Research:
* cross-compilation
* static linking
* musl
* Windows
* macOS
* Linux
* ARM
* Docker
* WASI/WASM
* debug symbols
* PGO
* LTO

<a id="core-21"></a>

## 21. Development Experience

Developer experience is a first-class product goal.
Provide:
* formatter
* linter
* language server
* autocomplete
* rename/refactor
* go-to-definition
* inline types
* dependency diagnostics
* ownership visualization
* compiler explanations
* native debugger integration
* profiler integration
Optional ownership-debug command:
lang inspect ownership
Potential output:
UserService.getUser

config      borrowed
request     borrowed mutable
user        owned
cache       shared
response    moved into return
This should be educational, not required during normal development.

<a id="core-34"></a>

## 34. Debugging and Stack Traces

Ensure native compilation does not destroy developer ergonomics.
Research:
* source-level stack traces
* async stack traces
* local variable inspection
* panic/crash reporting
* mapping optimized native frames back to source
* VS Code debugging
* LLDB/GDB integration

<a id="core-38"></a>

## 38. Toolchain Version Management

Avoid Python/Node-style environment confusion.
Potential model:
lang toolchain install 1.4
lang toolchain use 1.4
Project pins:
language = "1.4.2"
Research compatibility policies:
* semantic versioning
* editions
* compiler channels
* stable/beta/nightly
Rust editions are worth studying.

<a id="core-46"></a>

## 46. Hot Reloading

Node developers expect fast development loops.
Investigate:
lang dev
with:
* incremental compilation
* automatic restart
* state preservation where possible
* fast diagnostics
Hot module replacement for backend applications may not be necessary initially.
Compile speed matters greatly to adoption.

<a id="core-47"></a>

## 47. Incremental Compilation

This should be researched early.
A native compiled language that takes 30 seconds after every edit will fail the Node-like developer-experience goal.
Investigate:
* module-level incremental compilation
* cached intermediate representation
* rustc incremental mode
* parallel compilation
* dependency artifact caching
Set compile-time benchmarks early.
