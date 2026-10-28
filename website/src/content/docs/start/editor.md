---
title: Set up VS Code
description: Configure paired executables, editor commands and language-server recovery.
---

Editor support for `.tsr` and `.tsrx`: syntax highlighting, compiler diagnostics,
completion, documentation hover, argument help, navigation, formatting and run
commands. The Rust language server shares the compiler's analysis library.

## Install

Use VS Code 1.100.0 or newer.

Download the `.vsix` asset from
[GitHub Releases](https://github.com/Forever-Frameworks-LLC/tsrust/releases) and
choose **Extensions → Install from VSIX**. Until releases are available, build
locally using the [extension build instructions](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/editors/vscode/README.md#build-locally). Marketplace publication is not configured.

The extension bundles its JavaScript client and third-party notices. It does
**not** include `tsrust` or `tsrust-lsp`; install both from the matching toolchain
archive or build them from source.

## Locate the compiler and language server

An explicit path takes priority and must be an absolute path to an executable.
If a tool has no explicit path, the extension searches in this order:

1. Beside the explicitly configured companion executable, when provided.
2. `~/.cargo/bin`.
3. Absolute directories in PATH.

A broken explicit setting stops that tool's discovery; it is never silently
ignored. For a custom installation:

```json
{
  "tsrust.compiler.path": "/absolute/path/to/tsrust",
  "tsrust.server.path": "/absolute/path/to/tsrust-lsp"
}
```

For a [managed installation](../../tools/install-and-update/), configure
both paths to the stable launchers under its `bin` directory. On Windows, include
`.exe`. The launchers select the installed compiler/server pair together.

Use user settings to share paths across projects. Open a `.tsr` or `.tsrx` file
in a trusted workspace. **View → Output → Tsrust** shows startup and protocol
errors. Run **Tsrust: Restart Language Server** after replacing its executable;
**Developer: Reload Window** reloads extension updates.

On Windows, Cargo installs `tsrust.exe` and `tsrust-lsp.exe` under
`%USERPROFILE%\.cargo\bin`; keep the `.exe` suffixes. The same automatic discovery
works in a fresh project without custom paths. Paths containing spaces or Unicode
are supported. The extension supplies the Tsrust default formatter without
overwriting explicit user settings.

Checking and formatting require no external Rust compiler. **Run File** and
native builds require Rustup, the pinned Rust backend and a system linker.
See [installation](../installation/).

## Work in the editor

- **Tsrust: Run File** saves and executes the current file. The editor play
  button invokes the same command.
- **Tsrust: Check File** runs a compiler check.
- **Format Document** and **Tsrust: Format Document** use the shared formatter.
- Hover and signature help explain supported APIs, argument contracts and errors.
- Completion and navigation work with project imports and unsaved source.
- Quick fixes appear only when the proposed edit passes the relevant check.

## Save actions and diagnostics

`tsrust.checkOnSave` and `tsrust.formatOnSave` enable explicit save actions.
Live diagnostics already run while typing. `tsrust.analysis.debounceMs` adjusts
the delay after edits; its default is 200 ms. Changing executable or analysis
settings restarts the server.

Workspace discovery excludes `.git`, `node_modules`, `target` and symlinks.
Open buffers override disk contents. Project diagnostics include dependencies
and unopened source. Unsupported syntax or an unfinished declaration can still
limit recovery; an absent answer is not proof that an API does not exist.
Report a reproducible compiler/editor disagreement as a bug.

## Fix a missing executable

An `ENOENT` startup warning means a configured path no longer exists. Check both
user and workspace settings for stale build paths such as `target/editor/debug`.
Correct the path to the installed executable, or remove the override to use
discovery. The extension deliberately reports invalid explicit settings instead
of silently running another installation.

Check both paths from a terminal before restarting the server:

```sh
/absolute/path/to/tsrust --version
/absolute/path/to/tsrust-lsp --version
```

In PowerShell, use the call operator for a quoted executable path:

```powershell
& 'C:\path\to\tsrust.exe' --version
& 'C:\path\to\tsrust-lsp.exe' --version
```

If highlighting works but diagnostics do not, inspect **Output → Tsrust**.
Highlighting is supplied by the extension and does not prove the server started.

## Recover from repeated server exits

Open **View → Output → Tsrust** and inspect the error before restarting.
The extension automatically restarts the server after an unexpected exit. If
five exits occur within three minutes, it stops automatic recovery; the first
four exits can trigger restarts.

After addressing the error, run **Tsrust: Restart Language Server**. This starts
a new client and resets its restart history. Restarting restores service but
does not fix the cause of a crash.

## Investigate growing memory use

If the language server's memory keeps growing during your session, save your work
and disable the extension for that workspace. Include your OS, CLI/LSP versions,
workspace size, reproduction steps and relevant **Output → Tsrust** messages when
reporting the issue. Remove credentials and private source from shared logs.

Earlier investigations led to fixes that release unrelated editor-buffer
snapshots and superseded workspace-scan entries. Those fixes have focused tests
and a bounded Windows protocol probe. Long-running macOS sessions have not yet
been verified, so the historical memory report is not considered conclusively
resolved across platforms.
