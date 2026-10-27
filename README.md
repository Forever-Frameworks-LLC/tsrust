# Tsrust

> Product direction (2026-10-26): Tsrust is pivoting to native backends with
> generated named TypeScript clients for React web, Electron and Expo applications.
> Use [the active roadmap](docs/roadmap.md) and [application goal](docs/agent-goal.md).
> The custom UI implementation described below is preserved and frozen. Generated
> clients and managed frontend templates are planned/in progress, not released.

The current source adds `tsrust new hello --app`: a native greeting API, generated
named SDK and editable React/Vite/shadcn landing page. Use `--target backend` for
API-only generation. See [application projects](docs/guides/application-projects.md)
for the actual commands and ownership boundary. Fresh backend/web creation has
focused acceptance; Electron/Expo scaffolding and coordinated integration
commands remain in progress. Existing task applications are integration examples.

**Write TypeScript-style code. Ship native programs.**

Tsrust is an experimental language and compiler with familiar TypeScript syntax,
inferred ownership and a Rust backend. Build command-line tools, HTTP services
and reactive interfaces without a JavaScript runtime in the generated native
program.

[Get started](docs/content/getting-started/installation.md) ·
[Documentation](docs/README.md) ·
[Examples](examples) ·
[Releases](https://github.com/Forever-Frameworks-LLC/tsrust/releases) ·
[Contributing](CONTRIBUTING.md)

```ts
function main() {
  const languages = ['TypeScript', 'Rust'];
  console.log(languages.join(' + '));
}
```

Save as `hello.tsr`, then run `tsrust run hello.tsr`.

## What you can build

- **Native tools.** Checked integers, strings, collections, files, processes,
  async tasks and explicit resource cleanup.
- **Services.** Hono-style routing and middleware over Axum, HTTP clients,
  validation, SQLite and pooled PostgreSQL/MySQL/SQLite access. Authentication
  policy stays in application code.
- **Interfaces.** Reactive `.tsrx` components for browser WebAssembly and a
  custom-drawn GPU desktop renderer. Themes and editable local components install
  from the bundled offline catalog.
- **Rust integration.** Import supported functions and types from `.rs` files
  without introducing a JavaScript bridge.
- **Editor tooling.** Shared compiler diagnostics, completion, hover, signatures,
  navigation and formatting through the LSP and VS Code extension.

See [runnable examples](examples/README.md) and the
[language and library guide](docs/standard-library.md).

## Install

Download the archive for your host from
[GitHub Releases](https://github.com/Forever-Frameworks-LLC/tsrust/releases).
Releases include `tsrust`, `tsrust-lsp`, source identity, checksums and notices.
Put both executables on `PATH`. The VS Code extension is a separate `.vsix` asset.

Native compilation requires **Rustup with Rust 1.96.1** and a system linker.
Checking, formatting and Rust emission do not require the Rust backend. This is
not a self-contained Rust installation. See [installation](docs/content/getting-started/installation.md)
for source builds, archive verification and web prerequisites.

Until a release is published, build from this repository:

```sh
git clone https://github.com/Forever-Frameworks-LLC/tsrust.git
cd tsrust
cargo fetch --locked
cargo fetch --locked --manifest-path crates/tsrust-compiler/native-dependencies/Cargo.toml
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --offline --locked --release --bin tsrust --bin tsrust-lsp
```

The executables are in `target/release/`. The repository pins its Rust toolchain;
Rustup installs it on the first Cargo invocation if needed. Keep these executables
on `PATH` to use the commands below.

## Start a project

```sh
tsrust new hello
cd hello
tsrust dev
```

Run `tsrust` without arguments for the interactive launcher. Explicit commands work
in scripts:

```sh
tsrust check
tsrust test
tsrust format --check
tsrust build --release
```

For a UI project, choose targets explicitly:

```sh
tsrust new counter --ui --target native --target web
cd counter
tsrust ui add button card --offline
tsrust dev --target native
# Or, after installing the web prerequisites:
tsrust dev --target web --port 4317
```

`src/main.tsr` is the executable entrypoint. The UI scaffold imports its root
`Page` from `src/app/page.tsrx`. Your configuration and editable components belong
to your project. See [project structure](docs/content/getting-started/project-structure.md).

## Status and boundaries

Tsrust is **experimental**, with changing language and library APIs. It is not a
compiler for arbitrary existing TypeScript, Node packages or React applications.
Source uses `.tsr` and `.tsrx`; JavaScript compatibility is deliberate and partial.

Ownership checks reject invalid transfers and conflicting access. Strings use
UTF-8 byte positions where documented. Integer overflow is checked in both debug
and release programs. These semantics can differ from JavaScript; consult the
[behavior guides](docs/README.md), rather than assuming drop-in compatibility.

The UI component library and paired documentation app are still under development.
The current LSP has an unresolved report of excessive memory use during long
sessions; avoid opening very large workspaces until that issue is resolved.
macOS has the most development coverage. CI and release workflows target Linux
x64 and macOS; Windows distribution is not configured yet. Native GPU rendering
and accessibility still need real desktop checks beyond hosted CI.

Performance depends on the workload. [Benchmark methodology](benchmarks/runtime/README.md)
and [comparison limits](docs/runtime-comparisons.md) explain the measurements.
The large [historical report](BENCHMARKS.md) is a snapshot, not a claim about every
current feature.

Individual timing samples and allocation counters are available in the
[benchmark-only SQLite dataset](benchmarks/storage/PUBLIC_DATA.md), with source
identities, toolchain versions and general hardware specifications.

## Explore

- [Installation and first project](docs/content/getting-started/installation.md)
- [CLI and project configuration](docs/guides/project-cli.md)
- [VS Code setup](editors/vscode/README.md)
- [Language and standard library](docs/standard-library.md)
- [Ownership](docs/ownership-and-memory.md)
- [UI direction and implementation scope](UI_VISION.md)
- [Contributor checks](CONTRIBUTING.md) and [release automation](docs/guides/releases.md)

## License

[MIT](LICENSE), Copyright © 2026 Forever Frameworks LLC.
Third-party components and dependencies retain their own licenses. See
[attributions](attributions/README.md); distribution archives also include
applicable dependency notices.
