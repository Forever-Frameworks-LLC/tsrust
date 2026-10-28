# Tsrust

**Write TypeScript-style code. Ship native backends.**

Tsrust is an experimental language and compiler with inferred ownership and a
Rust backend. Build command-line tools and HTTP services without a JavaScript
runtime in the deployed program. Generate independent TypeScript or asynchronous
Rust SDKs from a checked HTTP API description, and bring your own frontend.

[Get started](docs/content/getting-started/installation.md) ·
[Documentation](docs/README.md) · [Examples](examples/README.md) ·
[Releases](https://github.com/Forever-Frameworks-LLC/tsrust/releases) ·
[Contributing](CONTRIBUTING.md)

```ts
function main() {
  const languages = ['TypeScript', 'Rust'];
  console.log(languages.join(' + '));
}
```

Save as `hello.tsr`, then run `tsrust run hello.tsr`. It prints `TypeScript + Rust`.

## What you can build

- Native tools with checked integers, collections, files, processes, async tasks
  and explicit resource cleanup.
- HTTP services with routing, middleware, request-body readers, validation,
  SQLite and pooled PostgreSQL/MySQL/SQLite access. Authentication is application policy.
- Independent named TypeScript/Rust clients and OpenAPI documents from the running
  service or a saved description. Client generation never modifies the server.
- Rust adapters with checked function/type boundaries and explicitly resolved,
  locked Cargo dependencies.

The compiler and language server share diagnostics, completion, hover, signatures,
navigation and formatting. Original UI source and explicit UI commands are
[retained separately](docs/guides/tooling/retained-ui-setup.md); frontend templates,
framework integrations and UI expansion are outside the active backend scope.

## Install

Official release archives contain the paired CLI/LSP, source identity, hashes and
notices. Targets are Linux x64/glibc, Windows x64/MSVC, macOS ARM64 and macOS x64.
The VS Code extension is a separate VSIX. If no stable release is available, use
the source build below. See [installation](docs/content/getting-started/installation.md)
for verification, platform prerequisites and editor configuration.

Once a stable release exists, an official CLI can create a managed installation:

```sh
tsrust toolchain install --root /absolute/path/to/new/tsrust
```

Add that directory's `bin` to PATH. Explicit `toolchain check`, `toolchain update`
and `toolchain rollback` manage the verified pair; existing package-manager
installations are preserved. See the [managed installation guide](docs/guides/toolchain-installation.md)
for trust, recovery and the exact platform acceptance status.

For a source build:

```sh
git clone https://github.com/Forever-Frameworks-LLC/tsrust.git
cd tsrust
cargo fetch --locked
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --offline --locked --release -p tsrust-compiler -p tsrust-lsp --bin tsrust --bin tsrust-lsp
```

Put both executables from `target/release/` on PATH. The repository pins Rust
1.96.1; Rustup may install that toolchain on first Cargo use. Checking, formatting
and ordinary Rust emission need no external Rust compiler. Native build/run/test/dev
require the pinned Rust toolchain and host linker. Prepare their sources explicitly:

```sh
tsrust setup --toolchain --dependencies --yes
tsrust doctor --native
```

Normal builds run offline against shared Cargo sources. Portable source exports
use a separately prepared, verified source pack; they are optional for ordinary
development. Deployed native programs need neither Tsrust nor Rust/Cargo.

## Start a project

```sh
tsrust new hello --app
cd hello
tsrust dev
```

The backend starter listens on loopback port 4318. Server source and policy live
under `app/`; no frontend project or public SDK is created. In another terminal:

```sh
tsrust api generate --url http://127.0.0.1:4318/api/description --out js-client
tsrust api generate --url http://127.0.0.1:4318/api/description --language rust --out rust-client
```

Use `tsrust new hello` for a console project. Run `tsrust` for the interactive
launcher, or use explicit commands in scripts:

```sh
tsrust check
tsrust test
tsrust format --check
tsrust build --release
```

See [backend applications](docs/guides/application-projects.md),
[project structure](docs/content/getting-started/project-structure.md) and
[CLI commands](docs/guides/project-cli.md).

## Status and boundaries

Language and library APIs can change. Tsrust does not compile arbitrary existing
TypeScript, Node packages or React applications. Ownership checks reject invalid
transfers and conflicting access; strings use documented UTF-8 byte semantics;
integer overflow is checked in development and release programs.

Async test registration has executed on Linux, macOS and Windows. Broader installer
and release-platform acceptance is recorded separately in the
[paired installation milestone](docs/milestones/paired-toolchain-installation.md)
and [active goal](docs/agent-goal.md). Configuration is not proof that a platform
has passed. The LSP still has an unresolved report of excessive memory use in long
sessions; use smaller workspaces until that issue is resolved.

Performance depends on the workload. Read the [measurement protocol](benchmarks/runtime/README.md)
and [comparison limits](docs/measurements/runtime-comparisons.md). The
[historical report](BENCHMARKS.md) is a snapshot. Shareable samples and provenance
live in the [benchmark-only SQLite dataset](benchmarks/storage/PUBLIC_DATA.md).

## Explore

- [Language and standard library](docs/guides/language/standard-library.md)
- [Ownership](docs/guides/language/ownership-and-memory.md)
- [HTTP behavior](docs/guides/http.md) and [Rust packages](docs/guides/rust-packages.md)
- [VS Code setup](editors/vscode/README.md)
- [Contributor checks](CONTRIBUTING.md) and [release automation](docs/guides/releases.md)

## License

[MIT](LICENSE), Copyright © 2026 Forever Frameworks LLC.
Third-party dependencies retain their own licenses; preserve
[attributions](attributions/README.md) and the notices included in distribution archives.
