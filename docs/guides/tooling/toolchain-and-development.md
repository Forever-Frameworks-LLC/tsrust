# Toolchain and development experience

[Documentation index](../../README.md) · [Proposed roadmap](../../roadmap.md) · [Source section map](../../plans/section-map.md)

## Current tools

Use [the project CLI](../project-cli.md) for check, format, test, build and watched
native development. The [VS Code extension](../../../editors/vscode/README.md)
uses the shared compiler analysis for project diagnostics, completion, navigation,
hover, parameter help and validated fixes. See [project editor behavior](project-editor.md)
for its supported boundaries; the original single-file foundation plan below is
historical evidence, not a current feature limit.

Repository and generated native builds use [the pinned Rust toolchain](rust-toolchain-policy.md).
[Machine setup](../../../website/src/content/docs/tools/rust-setup.md) prepares tools and shared dependency sources
explicitly. [Managed installation](../../../website/src/content/docs/tools/install-and-update.md) selects verified
CLI/LSP pairs atomically and supports explicit update and rollback; its platform
acceptance is recorded separately from implementation.

`tsrust build <input.tsr> --release -o <output>` and `tsrust run <input.tsr> --release`
compile generated Rust at optimization level 3; the default is level 0. Both retain
overflow checks. The compiler executable’s own Cargo profile is independent of the
generated program’s profile. Native caches reuse compatible dependencies and
artifacts; changing source still requires the affected native compilation.

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
