---
title: Status and supported scope
description: Understand Tsrust's experimental status and current backend focus.
---

Tsrust is experimental. Language and library APIs can change, and platform
acceptance is tracked separately from implementation.

## Current focus

- Native command-line programs and backend HTTP services.
- Shared compiler and language-server analysis, editor help and formatting.
- Checked request data, explicit access policy and independent TypeScript/Rust SDKs.
- Explicit Rust dependency preparation, offline native builds and portable exports.
- Paired CLI/LSP installation with verified updates and rollback.

The release pipeline targets Linux x64/glibc, Windows x64/MSVC, macOS Apple Silicon
and macOS Intel. A configured target is not proof that a particular release has
passed acceptance: consult its published assets and release notes.

## Compatibility

Source files use `.tsr`; retained UI source uses `.tsrx`. Familiar syntax does not
imply full TypeScript or JavaScript compatibility. Ownership checks reject invalid
transfers and conflicting access. Integer arithmetic is checked in optimized
builds too. Individual APIs document their string offsets and ownership behavior.

Original UI implementations remain in the repository for historical compatibility.
They are outside the active backend scope and are not the renderer for this site.

## Adoption

Use a pinned release or a specific source commit for a project. If no release
assets are available, follow the [source-build instructions](../../start/installation/#build-from-source).
Retain [API compatibility baselines](../../backend/api-compatibility/) for rolling
deployments, and test against your actual operating system and native dependencies.
Report reproducible problems through
[GitHub Issues](https://github.com/Forever-Frameworks-LLC/tsrust/issues).
