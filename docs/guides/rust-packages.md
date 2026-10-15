# Rust packages

Use the CLI to manage dependencies. Do not edit generated binding JSON or generated
Cargo.toml files. Public Rust functions and owned structs/enums define the source API;
package configuration only selects Cargo dependencies and features.

```sh
tsrust add regex@1.12.2 --project ./app
tsrust packages --project ./app
tsrust features regex --project ./app
```

The first add creates native.bindings.json and native.rs when needed. Regex supplies
a readable starter adapter exporting Search, SearchResult variants and search. Other
crates become available to custom public Rust declarations; adding a dependency does
not bind its entire API. Existing Rust adapters are never overwritten. The CLI prints
available exports and states explicitly when no bindings exist.

Import the generated module in source:

```ts
import { Search, search } from './native.bindings.json';
```

If a project already has one *.bindings.json file, commands select it automatically.
Use --bindings relative/path.bindings.json when several exist. --source ./adapter.rs
selects an adapter when creating a configuration. --project selects the root whose
callers and tests are validated; otherwise the nearest configured project or current
directory is used.

## Features and versions

```sh
tsrust features regex --enable unicode --project ./app
tsrust features regex --disable unicode --project ./app
tsrust features regex --features std,perf --no-default-features --project ./app
tsrust features regex --default-features --project ./app
tsrust update regex --dry-run --project ./app
tsrust update regex --to 1.13.1 --project ./app
tsrust remove regex --project ./app
```

--features replaces the explicit feature set; --enable and --disable modify that
set. Defaults are controlled separately. Cargo unifies features across dependencies;
disabling an explicit feature does not necessarily disable its effective feature.
Inspection shows effective features and dependency requests. Conflicting options are
rejected. An empty --features argument clears the explicit set.

Add without a version lets Cargo select its stable release. Repeated add preserves
an existing version unless another exact version is supplied. Update without --to
stays within Cargo's compatible range (including the narrower 0.x rules). Breaking
versions and prereleases require an explicit exact target. Published versions are
always exact pins, with a complete locked dependency closure. Compiler-owned rayon,
serde and serde_json pins cannot be overridden through project package commands.

Every modifying command supports --dry-run and --offline. Previews still compile
and run tests, then discard the candidate. --offline prevents registry access and
requires suitable locally cached sources; it does not mean "skip validation".
`tsrust bindings resolve path.bindings.json --offline` restores the currently locked
vendor closure after cloning when Cargo's source cache is warm. Ordinary builds and
exports remain offline and locked.

## Validation and publication

Before applying a change, the CLI resolves through Cargo, shows all added/removed
package versions, compiles the real adapter (even if unused), checks and natively
compiles project callers, and runs discovered .test.tsr/.spec.tsr files. Tests run
with the project root as their working directory. Their normal I/O effects are not
rolled back; the package CLI itself never rewrites caller or test source files.

A failed resolution, adapter build, caller check or test leaves published dependency
configuration and custom adapters unchanged. Changed source inputs abort publication.
Concurrent package commands selecting the same configuration serialize. Configuration
and its Cargo lock are published together in one atomic file generation; legacy
adjacent lockfiles remain readable and are not deleted during migration. Newly staged
unpublished caches are removed after failure or preview. Accepted vendor generations
remain available for builds already using them.

Compilation detects incompatible Rust APIs and source ownership/type changes. Tests
check exercised behavior; neither SemVer nor successful compilation proves untested
behavior. Zero test files are reported explicitly. Validation covers the current
host, not every platform. Registry crates are supported; git/path dependencies and
arbitrary automatic crate-to-language mappings are outside this workflow.

One program uses one managed dependency configuration. Independent configurations
can coexist under a project root and are selected with --bindings. Callers mixing
different dependency configurations are rejected before publication.

Offline integration acceptance uses cached regex 1.12.2, proc-macro2 1.0.107 and
unicode-ident 1.0.26/1.0.27 sources. The CLI's --offline behavior is tested against
that fixed warm set; missing sources fail without changing the project.
