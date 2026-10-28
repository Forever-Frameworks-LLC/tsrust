---
title: Install and update the CLI/LSP pair
description: Install, verify, update and roll back a managed Tsrust toolchain.
---

Implemented with focused macOS acceptance; Windows/Linux acceptance remains open
in [the milestone](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/docs/milestones/paired-toolchain-installation.md).
Do not treat these commands as a published release until the staged release flow
has produced an immutable stable release.

Start with the CLI from an official release archive, verified using the release
API's asset digest. Installation downloads the latest stable archive for this
host and verifies it before executing either consumer:

```sh
tsrust toolchain install --root /absolute/path/to/new/tsrust
```

The parent directory must exist and the destination must be new. Installation
never adopts an existing directory or overwrites a package-manager installation.
Add the new directory's `bin` directory to PATH and configure VS Code to use its
`tsrust-lsp` (`tsrust-lsp.exe` on Windows). Shell profiles and editor settings are
not changed automatically. Both executables are selected together.

```sh
tsrust toolchain check
tsrust toolchain check --json
tsrust toolchain update
tsrust toolchain rollback
tsrust doctor --json
```

Check reports the latest stable version without replacing anything. Update only
selects a newer stable version. Rollback selects the retained previous verified
pair without network access. Update and rollback require a managed installation;
an unmanaged CLI explains how to use its owning package manager or install into
a separate new directory. `tsrust update <crate>` still updates application Rust
dependencies and has no effect on the CLI/LSP installation.

Versions live in separate directories. One atomic installation record selects the
pair. A running compiler or editor server stays on its original executable; new
launches select the committed version. Restart the editor server after an update.
The installer does not stop editor processes or overwrite running Windows files.
An incomplete download or failed verification leaves the selected pair unchanged.
A verified stage left by interruption can be reused, and incomplete owned stages
are discarded on the next attempt. The prior pair remains available for rollback.
Older unused directories are removed when possible; a locked directory can remain
until it can be removed safely. Do not manually edit installation records.

The launcher protocol is versioned. Incompatible future launcher protocols require
reinstallation into a new directory; they are not silently interpreted by an old
launcher. Doctor reports installation ownership, selected consumer versions and
manifest integrity separately from Rust/compiler/linker readiness.

The trust boundary is verified HTTPS to the fixed official GitHub repository and
GitHub's immutable-release metadata and asset digest. A checksum hosted alongside
an arbitrary archive is insufficient. Archives must match the host and clean
release manifest; links, traversal, duplicate paths, missing notices and modified
files are rejected. Both staged consumers must report their manifest versions.
This is not independent code signing or macOS notarization. See GitHub's
[release API](https://docs.github.com/en/rest/releases/releases) and the
[release guide](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/docs/guides/releases.md) for publication and platform limitations.

Supported release hosts are Linux x64/glibc, Windows x64/MSVC, macOS ARM64 and macOS
x64. Installation and updates do not install Rust, Cargo or platform build tools.
Checking, formatting and emission remain tooling-free. Native development still
uses explicit `tsrust setup` preparation. There is no background update check,
telemetry, automatic downgrade or network access during normal compiler/LSP startup.
