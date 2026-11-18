---
title: Status and supported scope
description: Understand Tsrust's experimental status and current native application scope.
---

Tsrust is experimental. Language and library APIs can change, and platform
acceptance is tracked separately from implementation.

## Current focus

- Native command-line programs and backend HTTP services.
- Experimental GPUI Kit desktop applications with `.tsrx`, signals and editable component wrappers.
- Shared compiler and language-server analysis, editor help and formatting.
- Checked request data, explicit access policy and independent TypeScript/Rust SDKs.
- Explicit Rust dependency preparation, offline native builds and portable exports.
- Paired CLI/LSP installation with verified updates and rollback.

Prebuilt toolchains target Linux x64/glibc, Windows x64/MSVC and macOS Apple Silicon.
Intel macOS requires a source build. A configured target is not proof that a particular release has
passed acceptance: consult its published assets and release notes.

## Standard-library development

Tsrust v0.2.0 completes the standard-library checklist. It includes:

- Owned byte buffers, binary codecs, file/directory operations, streams and compression.
- Path, environment and OS helpers, signals, child processes, TCP/UDP and DNS.
- Binary and streaming HTTP requests, multipart forms, middleware, SSE and WebSockets.
- Reusable regular expressions and explicit-locale text and number operations.
- Owned iterator pipelines and expanded collection, numeric and date operations.
- Incremental hashes, HMAC/HKDF and password hashing/verification on explicit CPU pools.
- Structured logging, async coordination and testing hooks, assertions and profiling.

Find the APIs in the [standard-library overview](/tsrust/library/overview/) and
[test runner and profiling guide](/tsrust/tools/testing/). Releases before v0.2.0
do not include these APIs.

## Fault unwinding and faster development builds

In v0.3.0, runtime faults (overflow, bounds, assertions, length limits) unwind to
the nearest `try`, `Checked` function or boundary instead of returning through
every call, and uncaught faults report their file, line and column.
`Checked<T, E>` functions return faults as values; `TSRUST_FAULT_MODE=value`
keeps the previous behavior. Development builds use one Rust crate per source
module. The release includes LSP 0.4.0 with shorter, behavior-focused hovers.

## Compiler and editor maintenance

The v0.2.1 patch improves repeated compiler analysis, ownership-flow allocation,
development watcher scheduling and native-runtime cache reuse. It includes LSP
0.3.1, Zed syntax support and corrected API/performance documentation. Recorded
benchmark measurements are preserved; these improvements do not establish a
universal overhead bound relative to Rust. Large-workspace analysis can still
temporarily exceed 1 GB, and multi-hour memory behavior is not fully verified.

## Compatibility

Source files use `.tsr`; UI source uses `.tsrx`. Familiar syntax does not
imply full TypeScript or JavaScript compatibility. Ownership checks reject invalid
transfers and conflicting access. Integer arithmetic is checked in optimized
builds too. Individual APIs document their string offsets and ownership behavior.

## Desktop UI

Use `tsrust new desktop --ui` for a desktop project and `tsrust ui add` for editable
GPUI Kit component wrappers. Backend builds do not acquire the native UI runtime.
The repository gallery contains 71 component pages, 358 component examples and
live guides. Native styling and component API coverage have explicit limitations;
complete upstream parity and browser support are not claimed. The renderer is
experimental, and platform-specific application acceptance remains separate from
prebuilt compiler/LSP availability.

The [Desktop UI guides](/tsrust/ui/getting-started/) cover authoring, state,
styling, navigation, input and packaging; the [UI reference](/tsrust/reference/library/ui/)
lists every component, prop and function. Studio and Worklist
are local platform-validation experiments, not supported applications delivered
with the toolchain.

## Adoption

Use a pinned release or a specific source commit for a project. If no release
assets are available, follow the [source-build instructions](/tsrust/start/installation/#build-from-source).
Follow [HTTP rollout guidance](/tsrust/learn/first-service/#evolve-the-api) while older
clients remain deployed, and test against your actual operating system and native dependencies.
Report reproducible problems through
[GitHub Issues](https://github.com/Forever-Frameworks-LLC/tsrust/issues).
