Write backend services with TypeScript-style syntax and Rust ownership. Object
literals infer their shape in HTTP JSON responses, routes and middleware stay in
source, and API clients can be generated from source-defined HTTP contracts.
Object keys and middleware `next()` now provide checked editor help.

This update improves scoped Map compilation across catch regions and generated
array, callback, JSON/schema, string and numeric hot paths. It also repairs
optional native arguments in SQL/SQLite/HTTP and repeated cached executable
replacement. Ordinary JSON parsing follows Rust/Serde defaults; explicitly
bounded APIs retain their documented limits. Generated code uses safe Rust with
checked integer operations.

The performance page includes all 933 language/library workloads with independent
hand-written Rust and Node.js references, followed by ten HTTP comparisons.
Tsrust measured faster than Node.js in 682 of 933 workloads, with a 1.74× median
throughput ratio; hand-written Rust measured faster in 621. The selected nested
callback function is shown in the docs alongside its measured 17.14× Node.js
throughput. These are workload-level results, not universal application speedups.
HTTP tables include text, JSON, parsing/transformation and SQLite cases, plus CPU
rendering with request p99, server CPU/request and sampled RSS. Reference baselines
are reused; dates and provenance remain in repository metadata.

The distribution includes the `tsrust` CLI and `tsrust-lsp` language server, plus
the separate VS Code extension. Applications expose HTTP APIs; bring your own UI
and generate TypeScript or Rust clients with `tsrust api generate`.

Use `tsrust new service --app` to create a backend project, `tsrust new hello` for
a console program, or `tsrust` for the interactive launcher.

Documentation: https://forever-frameworks-llc.github.io/tsrust/

Download the archive matching your host, verify it against `SHA256SUMS`, and put
both `bin/` executables on `PATH`, or install a managed copy with
`tsrust toolchain install --root NEW_DIRECTORY`; managed installations update with
`tsrust toolchain check` and `tsrust toolchain update`. Cargo-owned installations
update through Cargo. Install the separate VSIX in VS Code. Native compilation
requires Rustup with Rust 1.96.1 and a system linker; archives do not include Rust.
Linux archives target glibc systems (built on Ubuntu 24.04), not static musl.

Tsrust is experimental. Language and library APIs may change between minor
versions. No platform developer signing or Apple notarization is provided.
Large-workspace and multi-hour language-server memory behavior is not yet
verified. Native file operations currently support macOS/Linux. The original
Tsrust UI is frozen and is not part of this release.

See the installation and release guides in the tagged source for prerequisites,
verification commands and test coverage.
