---
title: Set up VS Code
description: Configure paired executables, editor commands and language-server recovery.
---

Editor support for `.tsr` and `.tsrx`: syntax highlighting, compiler diagnostics,
completion, documentation hover, argument help, navigation, formatting and run
commands. The Rust language server shares the compiler's analysis library.

Tsrust and this extension are experimental. The current language server has an
unresolved report of excessive memory use in long workspace sessions. Avoid large
workspace scans until resolved; stop or disable the extension for the workspace
if memory grows. Unexpected exits use bounded recovery: up to four automatic
restarts within the language client's three-minute failure window. A fifth exit
stops recovery and leaves the cause in **Output → Tsrust**. Use **Tsrust: Restart
Language Server** to reset the budget after inspecting repeated failures. Recovery
does not establish that the underlying exit or memory issue is resolved.

## Install

Download the `.vsix` asset from
[GitHub Releases](https://github.com/Forever-Frameworks-LLC/tsrust/releases) and
choose **Extensions → Install from VSIX**. Until releases are available, build
locally using the [extension build instructions](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/editors/vscode/README.md#build-locally). Marketplace publication is not configured.

The extension bundles its JavaScript client and third-party notices. It does
**not** include `tsrust` or `tsrust-lsp`; install both from the matching toolchain
archive or build them from source.

The extension finds executables beside an explicitly configured companion first,
then in `~/.cargo/bin` and absolute PATH entries. A configured path must point to
an executable and is never silently ignored. For a custom installation:

```json
{
  "tsrust.compiler.path": "/absolute/path/to/tsrust",
  "tsrust.server.path": "/absolute/path/to/tsrust-lsp"
}
```

For a [managed installation](../../tools/install-and-update/), configure
both paths to the stable launchers under its `bin` directory. On Windows, include
`.exe`. The launchers select the installed compiler/server pair together.

An `ENOENT` startup warning means a configured path no longer exists. Check both
user and workspace settings for stale build paths such as `target/editor/debug`.
Correct the path to the installed executable, or remove the override to use
discovery. The extension deliberately reports invalid explicit settings instead
of silently running another installation.

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

## Use

- **Tsrust: Run File** saves and executes the current file. The editor play
  button invokes the same command.
- **Tsrust: Check File** runs a compiler check.
- **Format Document** and **Tsrust: Format Document** use the shared formatter.
- Hover and signature help explain supported APIs, argument contracts and errors.
- Completion and navigation work with project imports and unsaved source.
- Quick fixes appear only when the proposed edit passes the relevant check.

`tsrust.checkOnSave` and `tsrust.formatOnSave` enable explicit save actions.
Live diagnostics already run while typing. `tsrust.analysis.debounceMs` adjusts
the delay after edits; its default is 200 ms. Changing executable or analysis
settings restarts the server.

Workspace discovery excludes `.git`, `node_modules`, `target` and symlinks.
Open buffers override disk contents. Project diagnostics include dependencies
and unopened source. Unsupported syntax or an unfinished declaration can still
limit recovery; an absent answer is not proof that an API does not exist.
Report a reproducible compiler/editor disagreement as a bug.
