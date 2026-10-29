---
title: Install and update the CLI/LSP pair
description: Install, verify, update and roll back a managed Tsrust toolchain.
---

A managed installation keeps the CLI and language server on the same release and
retains the previous pair for rollback. It is separate from
[preparing Rust build tools](../rust-setup/).

:::note[Release availability]
Installation requires a published, stable, immutable official release for your
host. If none is available, the command reports that and leaves existing
installations unchanged. Use the [source installation instructions](../../start/installation/)
until release archives are available.
:::

## Install into a new directory

Start with the CLI from an official release archive, verified using the release
API's asset digest. Installation downloads the latest stable archive for this
host and verifies it before executing either consumer:

```sh
tsrust toolchain install --root /absolute/path/to/new/tsrust
```

The parent directory must exist and the destination must be new. Installation
never adopts an existing directory or overwrites a package-manager installation.

Add the new directory's `bin` directory to PATH. Configure both executable paths
in [VS Code](../../start/editor/#locate-the-compiler-and-language-server) to the
managed `bin/tsrust` and `bin/tsrust-lsp` launchers; on Windows, both end in `.exe`.
Shell profiles and editor settings are not changed automatically. Both executables are selected together.

## Check for an update

Run these through the managed CLI launcher:

```sh
tsrust toolchain check
tsrust toolchain check --json
```

Check contacts GitHub for the latest stable version without downloading or
replacing the executable pair. It also works from an unmanaged CLI, where it
reports that upgrades belong to the package manager or a separate installation.

## Update or roll back

```sh
tsrust toolchain update
# Return to the retained previous pair, if one exists:
tsrust toolchain rollback
```

Update selects only a newer stable version. Rollback revalidates and selects the
retained previous pair without network access. Both require a managed installation.
They preserve package-manager installations and explain how to create a separate
managed destination when invoked from an unmanaged CLI.

`tsrust update <crate>` updates application Rust dependencies; it does not update
the CLI or language server.

### Running processes and interrupted updates

Versions live in separate directories. One atomic installation record selects the
pair. Restart the editor server after an update so it uses the new version.

- **Already running:** a compiler or editor server stays on its original executable.
  New launches select the committed version. Installation does not stop editor
  processes or overwrite running Windows files.
- **Download or verification fails:** the selected pair stays unchanged.
- **Installation is interrupted:** a verified stage can be reused; incomplete owned
  stages are discarded on the next attempt. The prior pair remains available for rollback.
- **An old directory is locked:** cleanup leaves it until it can be removed safely.

Do not manually edit installation records.

## Verify the installed pair

```sh
tsrust doctor --json
```

Doctor checks the selected pair's manifest and file integrity without an update
request. It reports Rust readiness separately; missing Rust tools can make the
overall command fail even when the installed CLI/LSP files are intact.

The launcher protocol is versioned. Incompatible future launcher protocols require
reinstallation into a new directory; they are not silently interpreted by an old
launcher.

## What installation verifies

The trust boundary is verified HTTPS to the fixed official GitHub repository and
GitHub's immutable-release metadata and asset digest. A checksum hosted alongside
an arbitrary archive is insufficient.

Before switching versions, installation verifies:

- The archive matches the host and clean release manifest.
- Entries contain no links, traversal, duplicate paths, missing notices or modified files.
- Both staged executables report the versions declared in their manifest.

This is not independent code signing or macOS notarization. See GitHub's
[release API](https://docs.github.com/en/rest/releases/releases) and the
[release guide](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/docs/guides/releases.md) for publication and platform limitations.

## Supported hosts and remaining prerequisites

Supported release hosts are Linux x64/glibc, Windows x64/MSVC, macOS ARM64 and macOS
x64. Installation and updates do not install Rust, Cargo or platform build tools.
Checking, formatting and emission remain tooling-free. Native development still
uses explicit `tsrust setup` preparation. There is no background update check,
telemetry, automatic downgrade or network access during normal compiler/LSP startup.
