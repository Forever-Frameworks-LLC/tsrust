# Contributing to Tsrust

Tsrust is experimental. Start with a small reproducible example or discuss a
proposed public API in an issue before a large change. See [documentation](docs/README.md)
for current behavior and the [development workflow](docs/development-workflow.md)
for acceptance requirements.

## Set up

Install Rustup, a native linker/C toolchain, Node.js 24 and npm. The repository
pins Rust 1.96.1. Linux native UI builds also need the development packages listed
in [.github/actions/setup/action.yml](.github/actions/setup/action.yml).

```sh
cargo fetch --locked
cargo fetch --locked --manifest-path crates/tsrust-compiler/native-dependencies/Cargo.toml
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR="$PWD/target"
export RUST_TEST_THREADS=1
cargo build --offline --locked --bin tsrust --bin tsrust-lsp
npm --prefix editors/vscode ci
```

The two Cargo fetches prepare separate locked dependency graphs. Subsequent
builds can run offline. The compiler embeds native library sources when built;
it does not fetch packages silently during ordinary application compilation.

## Make a change

Keep source changes focused. Preserve ownership, cancellation, cleanup and error
behavior. Language or library changes need compiler and native witnesses plus
matching LSP documentation, completion, hover and diagnostics. Use `std:testing`
for executable assertions. Add the regression while implementing, then run the
related checks after completing the section.

```sh
# Choose the affected integration target and exact regression name.
cargo test --offline --locked -p tsrust-compiler --test acceptance forward_calls_and_transitive_consumption -- --exact
cargo test --offline --locked -p tsrust-lsp --test stdio framed_transport_unicode_diagnostics_repair_and_lifecycle -- --exact
node --test tools/tests/distribution.test.mjs
npm --prefix editors/vscode test
```

Before a source checkpoint, format the entire project with the built compiler:

```sh
TSRUST="$CARGO_TARGET_DIR/debug/tsrust" node tools/format-project.mjs
"$CARGO_TARGET_DIR/debug/tsrust" format . --check
cargo clippy --offline --locked --workspace --lib --bins -- -D warnings
```

Formatting verifies token preservation and refreshes pinned benchmark source
fingerprints when applicable. Executable fixtures also need formatting
idempotence and behavior checks through the shared test helpers.

## CI and broader checks

Pull requests and pushes to `main` run compiler/CLI acceptance, LSP transport and
compiler agreement, native renderer model tests, Clippy, project formatting,
distribution checks and VS Code extension tests. The workflow is deliberately a
bounded baseline, not every compiler integration target or a performance run.
See [CI](.github/workflows/ci.yml).

Run full regressions at feature-family milestones or with the CI workflow's
`full_regressions` dispatch input:

```sh
cargo test --offline --locked --workspace --lib --bins --tests --no-fail-fast
cargo test --offline --locked --workspace --doc
```

The second invocation is separate because offline package witnesses can replace
shared dependency artifacts. Native GPU/OS interactions require a real desktop.
Browser Wasm, VS Code Extension Host, SQL services and performance measurements
have their own explicit runners; the default CI does not claim those checks.
Database integration is opt-in with `node tools/check-async-sql.mjs --docker`.
Reuse accepted evidence unless relevant SQL or task-lifecycle behavior changes.

Keep one Cargo target and one build owner, disable incremental compilation, and
remove obsolete outputs after acceptance. The public
[measurement database](benchmarks/storage/PUBLIC_DATA.md) contains benchmark data
and general hardware specifications only. The ignored `benchmarks/records.sqlite`
is a legacy mixed archive; never upload it. Do not add routine logs or source
snapshots to analytics, or upload private logs in a pull request.

## Submit

Explain the problem, resulting behavior and verification. Include remaining
limits, and distinguish checks from benchmark measurements. Contributions to
Tsrust are under the repository's [MIT license](LICENSE); upstream files retain
their own notices. Report security issues as described in [SECURITY.md](SECURITY.md).
