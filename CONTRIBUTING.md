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
cargo fetch --locked --manifest-path crates/tsrust-compiler/backend-dependencies/Cargo.toml
cargo fetch --locked --manifest-path crates/tsrust-compiler/native-dependencies/Cargo.toml
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR="$PWD/target"
export RUST_TEST_THREADS=1
cargo build --offline --locked --bin tsrust --bin tsrust-lsp
npm --prefix editors/vscode ci
```

The three Cargo fetches prepare the workspace, backend and retained portable
source graphs. Subsequent builds can run offline. Ordinary application compilation
does not fetch packages silently. Before binding/package-management tests, prepare
their additional locked graph:

```sh
cargo fetch --locked --manifest-path crates/tsrust-compiler/tests/dependencies/Cargo.toml
```

Before tests that export portable sources, prepare their pack explicitly:

```sh
export TSRUST_SOURCE_PACK_DIR="$PWD/target/portable-source-packs"
"$CARGO_TARGET_DIR/debug/tsrust" setup --portable --offline
```

Keep that source-pack path fixed for tests that isolate `CARGO_HOME`.

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
cargo clippy --offline --locked -p tsrust-compiler -p tsrust-lsp --lib --bins -- -D clippy::correctness
```

Formatting verifies token preservation and refreshes pinned benchmark source
fingerprints when applicable. Executable fixtures also need formatting
idempotence and behavior checks through the shared test helpers.

## CI and broader checks

Pull requests targeting `stag` run the Linux compiler/CLI acceptance, native SDK
integration, LSP transport and compiler agreement, Clippy correctness checks,
formatting, distribution/privacy checks and VS Code extension tests. Other Clippy
warnings are reported rather than treated as accepted cleanup. Retained renderer
checks run only with explicit full regressions. The default workflow is a bounded
baseline, not every compiler integration target or a performance run.

Promotion PRs from `stag` to `main` use the separate [release workflow](.github/workflows/release.yml)
to build and validate macOS, Linux and Windows packages. Successful protected
promotion can merge and publish through [promotion automation](.github/workflows/promote.yml).
Manual async and Windows toolchain acceptance modes do not publish releases.
See [CI](.github/workflows/ci.yml) for the exact selected targets.

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
remove obsolete outputs after acceptance. `benchmarks/measurements.sqlite` holds
one row per workload and engine with a short JSON history of run dates and
timings; add runs with `npm run store:ingest` in `benchmarks/runtime`. Do not add
routine logs or source snapshots to it, or upload private logs in a pull request.

## Submit

Explain the problem, resulting behavior and verification. Include remaining
limits, and distinguish checks from benchmark measurements. Contributions to
Tsrust are under the repository's [MIT license](LICENSE); upstream files retain
their own notices. Report security issues as described in [SECURITY.md](SECURITY.md).
