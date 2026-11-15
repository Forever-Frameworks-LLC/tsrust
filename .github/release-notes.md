This patch release improves compiler, language-server and native development workflows.
It includes compiler/CLI **0.2.1**, language server **0.3.1** and VS Code extension
**0.2.2**. Extension versions are independent of compiler versions.

Changes since v0.2.0:

- Reuse error-effect analysis within a solve and share unchanged ownership-flow
  state to reduce repeated work and temporary allocations while preserving checks.
- Improve development watcher scheduling and native-runtime cache reuse.
- Improve Tsrust editor support, including Zed syntax for match expressions and
  ownership modifiers. Zed support is in the source repository; the VSIX is for
  VS Code.
- Fix native GPUI compilation on Linux and Windows by restricting macOS window
  positioning to macOS. UI dependencies remain separate from backend builds.
- Preserve case-distinct standard-library examples in generated acceptance suites.
- Correct generated API documentation and formatting-source identity handling on
  the performance page without changing recorded benchmark measurements.

Validation passed compiler/LSP core tests, all 143 runtime suite groups on each
supported host, host/process-boundary tests, tooling, documentation and the VS Code
extension checks. Release packaging additionally verifies optimized CLI/LSP archives
and smoke tests before publication. Existing explicitly ignored tests are not
counted as passes. CI timings are not application-performance benchmarks.

Download the archive matching your host, verify it against `SHA256SUMS`, and put
both `bin/` executables on `PATH`. Supported prebuilt hosts are Apple-silicon macOS,
Windows x64 and Linux x64 (glibc, built on Ubuntu 24.04). Native compilation requires
Rustup with Rust 1.99.1 and a system linker; archives do not include Rust. Install
the separate VSIX in VS Code, or follow the source repository's Zed extension guide.

Tsrust remains experimental. Large-workspace analysis can still temporarily use
over 1 GB of memory; multi-hour memory behavior is not fully verified. The measured
development improvements do not establish a universal overhead bound relative to
Rust. Native desktop UI support has known styling and platform limitations;
browser UI remains unverified. Studio and Worklist are local platform-validation
experiments, not supported application products in this release. No platform
developer signing or Apple notarization is provided.

Documentation: https://forever-frameworks-llc.github.io/tsrust/
