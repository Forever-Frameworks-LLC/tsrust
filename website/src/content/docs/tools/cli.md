---
title: Use the project CLI
description: Create, check, run and develop native Tsrust projects.
---

Create a backend service with `tsrust new hello --app`, or a console program with
`tsrust new hello`. See [backend applications](../../backend/applications/) for server
operations and independent TypeScript/Rust SDKs. Prepare native tools and shared
dependency sources explicitly using [machine setup](../rust-setup/).

## Interactive commands

Run `tsrust` without arguments in a terminal to open the interactive command menu.
The launcher groups actions into Development, Quality, UI and Project screens,
with help and exit alongside them. The UI screen contains retained legacy tooling;
it is outside the current backend workflow. Development and Quality appear inside projects.
Use arrow keys and Enter, or the displayed Back item to return to categories.
Esc or Ctrl+C cancels.

`NO_COLOR` and `--color never` disable colors; `--color always` explicitly enables
them. For example, `tsrust --color never` opens the launcher without color. Narrow
terminals omit the wordmark and scroll longer selections. `TERM=dumb` uses numbered
prompts with `q`/EOF cancellation. When stdin or stderr is not a terminal, bare
invocation prints help and exits successfully instead of waiting for input.

## Create a project

```sh
tsrust new hello
cd hello
tsrust dev
```

`new` creates source, a test, README, `AGENTS.md`, `.gitignore` and `tsrust.config.json`. It refuses
existing destinations. Use a name beginning with a letter, followed by letters,
digits, hyphens or underscores, with a maximum of 100 ASCII characters.
A destination path is allowed if its parent exists.

`dev` builds and starts the program, watches project source and config changes, and
restarts after each successful build. Build errors leave the last working program
running. Ctrl+C stops the watcher and its processes. Use `--release` for optimized
builds or `--` to pass arguments to the program. macOS, Linux and Windows have native watcher/process support.

## Check and run

| Command | Purpose |
| --- | --- |
| `tsrust check` | Check the project's source. |
| `tsrust format --check` | Report formatting differences without rewriting files. |
| `tsrust test` | Build and run discovered tests. |
| `tsrust run` | Build and run the configured entry. |
| `tsrust build` | Write a native executable at the configured output path. |

Use `tsrust run -- --port 3000` to forward arguments. Your program must interpret
them; the CLI does not add a server or a port setting.

## Project configuration

Commands with omitted inputs use the nearest `tsrust.config.json`, including from
nested folders. `check`, `format` and `test` default to that project root.
Build/run/emit use its entry when you omit a file; build uses its output.
Configured run and dev use the project root as their working directory. Explicit
input paths remain relative to your current directory and work without config.

```json
{
  "schemaVersion": 1,
  "name": "hello",
  "build": {
    "entry": "src/main.tsr",
    "output": "target/hello"
  }
}
```

Config paths are relative to the config directory. Unknown settings are rejected.
Formatting currently retains the standard two-space rules. Configurable formatting,
include/exclude paths and compilation targets are not supported yet.

## Watch files and reuse builds

The watcher tracks source and build inputs below the project root, including
`.tsr`, `.tsrx`, Rust files, JSON/JSONC configuration, text files and lockfiles.
It excludes `.git`, `node_modules`, `target` and symlinks from that scan.
The resolved import closure is also captured; dependencies outside that tree or
inside excluded directories are reconciled by metadata polling.

Development binaries are temporary; `build` writes the named output. A program that
exits normally runs again on the next change. Restarting does not preserve its
in-memory state. The template has a `main` function, but the language does not require it.

See [testing](../testing/) for test discovery, output and failure reporting.

## Read development output

Dev output defaults to one watching message and a build/rebuild duration, followed
by program output. Successful program exits are silent; failed builds and nonzero
exits remain visible.

Dev status uses cyan for information, green for successful
builds, red for failures and yellow for watcher fallback warnings. Colors enable
automatically on a supported stderr terminal; redirected output, `NO_COLOR` and
`TERM=dumb` stay plain. Override with `--color auto|always|never` before `--`.
Program output is unchanged.

Use `tsrust dev --verbose` when diagnosing a slow build or process failure. It adds:

- Process IDs, temporary binary paths and successful exit messages.
- Checking, emission, backend verification and cache lookup timings.
- Runtime preparation, dependency workspace paths and runtime build/publication timings.
- Combined native compilation and linking timings.

## Build caching and watcher fallback

Dev shares the validated project cache at `target/tsrust-native` across edits and
sessions. Each running executable remains a private copy, so builds cannot overwrite
its inode. Identical source/toolchain/options reuse an artifact; a new source version
still needs native compilation.

The watcher uses native filesystem notifications on macOS, Linux and Windows. Notifications
trigger reconciliation of source contents; idle polls do not scan the source tree.
Unchanged file contents remain cached.

Native watcher setup/runtime failures print a message
and fall back to metadata polling. Set `TSRUST_DEV_WATCH=poll` to force that fallback.
macOS uses one descriptor per watched file/directory, so descriptor exhaustion also
falls back visibly.

## Pass program arguments

`run` accepts program arguments after `--`, with either a configured entry or an
explicit source: `tsrust run src/main.tsr -- --port 3000`. Arguments after the
separator are passed directly to the executable, without shell interpretation.
Flags such as `--help` and `--release` after the separator belong to the program;
compiler options belong before it. Empty arguments, spaces and Unicode are
preserved.

Read forwarded arguments with `std:env.args()`:

```typescript
import { args } from "std:env";
function main() {
  const values = unwrap(args());
  console.log(values.length);
}
```

`tsrust run main.tsr -- first "two words"` prints `2`. The array excludes the
executable path. Each call returns independent strings. Native executables reject
non-Unicode arguments with `EnvError` code `NotUnicode`; no lossy replacement or
partial result is returned. The CLI's own command-line parser currently expects
Unicode input. Flag interpretation belongs to application code.

