---
title: Use Rust packages
description: Add Rust adapters, select features and validate dependency changes.
---

Use the CLI to manage dependencies. Do not edit generated binding JSON or generated
Cargo.toml files. Public Rust functions and owned structs/enums define the source API;
package configuration only selects Cargo dependencies and features.

## Add a dependency

Start with an existing project at `./app` and
[prepared native build tools](../rust-setup/). These commands can fetch registry
sources; use `--offline` when the needed sources are already cached.

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

For a source file beside `native.bindings.json`, import the adapter exports:

```ts
import { Search, search } from './native.bindings.json';
```

## Select the project and adapter

If a project already has one `*.bindings.json` file, commands select it automatically.
Use --bindings relative/path.bindings.json when several exist. --source ./adapter.rs
selects an adapter when creating a configuration. --project selects the root whose
callers and tests are validated; otherwise the nearest configured project or current
directory is used.

## Change features

```sh
tsrust features regex --enable unicode --project ./app
tsrust features regex --disable unicode --project ./app
tsrust features regex --features std,perf --no-default-features --project ./app
tsrust features regex --default-features --project ./app
```

--features replaces the explicit feature set; --enable and --disable modify that
set. Defaults are controlled separately. Cargo unifies features across dependencies;
disabling an explicit feature does not necessarily disable its effective feature.
Inspection shows effective features and dependency requests. Conflicting options are
rejected. An empty --features argument clears the explicit set.

## Preview and update versions

```sh
tsrust update regex --dry-run --project ./app
```

This previews a compatible update without publishing it. Remove `--dry-run` to
apply the validated change. To select a particular published version, use
`tsrust update regex --to VERSION --project ./app`, replacing `VERSION` with the
exact version you intend to use.

Add without a version lets Cargo select its stable release. Repeated add preserves
an existing version unless another exact version is supplied. Update without --to
stays within Cargo's compatible range (including the narrower 0.x rules). Breaking
versions and prereleases require an explicit exact target. Published versions are
always exact pins, with a complete locked dependency closure. Compiler-owned rayon,
serde and serde_json pins cannot be overridden through project package commands.

Every modifying command supports --dry-run and --offline. Previews still compile
and run tests, then discard the candidate. --offline prevents registry access and
requires suitable locally cached sources; it does not mean "skip validation".

## Restore locked sources

For the generated configuration above:

```sh
tsrust bindings resolve app/native.bindings.json
```

This explicitly fetches the exact locked
sources into Cargo's shared cache. Add `--offline` to require already-cached sources.
Ordinary check/emit/build/dev commands never download or create vendor copies.
Native builds reuse compiled support for the same locked graph and account for
Cargo configuration/environment changes. Missing sources require explicit resolution.
New binding files use schemaVersion 2, whose runtime graph contains backend
dependencies. Version 1 remains the explicit legacy full-runtime format; its locks
are preserved without inferring a format from their incidental package contents.
Unknown versions fail before resolution. New package commands create version 2;
commands on an existing version 1 file preserve its format and exact graph.

## Export a portable source package

From the project directory, prepare its source pack explicitly. Replace
`path.bindings.json` and `main.tsr` below with your binding configuration and entry
file:

```sh
tsrust bindings resolve path.bindings.json --portable --offline
tsrust emit main.tsr --package -o exported
```

Portable preparation uses Cargo vendor, verifies locked package/file checksums and
retains licenses. Export then needs neither Cargo nor network access, and fails
with preparation guidance if the exact pack is absent or changed. Changing the
binding graph requires a matching source pack. Keep the same Cargo home or set
TSRUST_SOURCE_PACK_DIR explicitly when exporting without the toolchain. The exported
package includes its own vendor directory and can rebuild offline after relocation.

## Remove an unused dependency

```sh
tsrust remove regex --project ./app
```

Removal undergoes the same validation as an update. First remove the dependency's
uses from your Rust adapter and project source; otherwise compilation rejects the
change. Use `--dry-run` to preview removal.

## What validation changes

Before applying a change, the CLI:

1. Resolves through Cargo and shows added and removed package versions.
2. Compiles the real Rust adapter, even when no caller uses it.
3. Checks and natively compiles project callers.
4. Runs discovered `.test.tsr` and `.spec.tsr` files.

Tests run
with the project root as their working directory. Their normal I/O effects are not
rolled back; the package CLI itself never rewrites caller or test source files.

### Failed changes and concurrent edits

A failed resolution, adapter build, caller check or test leaves published dependency
configuration and custom adapters unchanged. Changed source inputs abort publication.
Concurrent package commands selecting the same configuration serialize. Configuration
and its Cargo lock are published together in one atomic file generation; legacy
adjacent lockfiles remain readable and are not deleted during migration. Temporary validation files are removed after failure or preview. Cargo's shared
source and compiled caches may retain work from those explicit commands; published
configuration and adapters do not change. Historical vendor directories are preserved
but ordinary builds no longer consume them.

### Limits of validation

Compilation detects incompatible Rust APIs and source ownership/type changes. Tests
check exercised behavior; neither SemVer nor successful compilation proves untested
behavior. Zero test files are reported explicitly. Validation covers the current
host, not every platform. Registry crates are supported; git/path dependencies and
arbitrary automatic crate-to-language mappings are outside this workflow.

One program uses one managed dependency configuration. Independent configurations
can coexist under a project root and are selected with --bindings. Callers mixing
different dependency configurations are rejected before publication.
