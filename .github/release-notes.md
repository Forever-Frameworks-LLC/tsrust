This release makes runtime faults unwind by default, adds `Checked<T, E>` results,
and speeds up the development loop with one Rust crate per source module. It
includes compiler/CLI **0.3.0**, language server **0.4.0** and VS Code extension
**0.2.2**. Extension versions are independent of compiler versions.

Changes since v0.2.1:

- **Runtime faults unwind.** Integer overflow, out-of-range indexes, failed
  assertions and length limits now unwind to the nearest `try`, `Checked` function
  or boundary (threads, tasks, UI callbacks, HTTP handlers, test bodies) instead of
  being threaded through every call as a value. Uncaught faults print
  `Uncaught RangeError: <message>` with `at file:line:column` and exit 1. Ordinary
  calls no longer carry error bookkeeping; for example `fib` now runs at the speed
  of hand-written Rust. `TSRUST_FAULT_MODE=value` keeps the previous behavior.
  `throw`, `Result` and `unwrap` are unchanged.
- **`Checked<T, E>`.** Declare a function's result as `Checked<T>` or
  `Checked<T, E>` to receive every fault in its body as
  `Result<T, RuntimeError | E>`. `RuntimeError` is a public type with `name`,
  `code`, `message`, `source`, `line` and `column`.
- **Faster development builds.** Development builds emit one Rust crate per source
  module, so an edit recompiles only the affected crates. Checks reuse earlier
  analysis and the language server's background check; the measured
  edit-to-run loop on the Studio application fell from 39 s to about 18 s.
- **Desktop UI.** View input events (keyboard, pointer, wheel), zoom-aware text
  sizing, queued UI commands that run without waiting for a frame, idle ticks
  that skip unchanged bindings, and a custom-titlebar drag region. A Desktop UI
  section on the documentation site covers getting started, components, state,
  styling, navigation, input and packaging.
- **Runtime performance.** A single-table Map/Set index with hash reuse,
  borrowed `split` loops, cheaper fallible sorts and dead `filter` sources moved
  instead of copied.
- **Editor help.** Hovers and parameter help are shorter and focus on behavior:
  long entries were rewritten one by one and UI component hovers show purpose,
  example and prop names.
- Fixes for closures that relay borrowed callbacks, `Map.set` argument order,
  generic functions in native declarations, and quiet exit on a closed output
  pipe.

The performance page now includes a whole-program comparison with hand-written
Rust and remeasured runtime workloads under the new default.

Validation passed compiler/LSP core tests, the runtime suite groups on each
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
over 1 GB of memory; multi-hour memory behavior is not fully verified. Measured
improvements do not establish a universal overhead bound relative to Rust. Native
desktop UI support has known styling and platform limitations; browser UI remains
unverified. Studio and Worklist are local platform-validation experiments, not
supported application products in this release. No platform developer signing or
Apple notarization is provided.

Documentation: https://forever-frameworks-llc.github.io/tsrust/
