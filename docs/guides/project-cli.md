# Create and develop a Tsrust project

Run `tsrust` without arguments in a terminal to open the interactive command menu.
The launcher groups actions into Development, Quality, UI and Project screens,
with help and exit alongside them. Development and Quality appear inside projects.
Use arrow keys and Enter, or the displayed Back item to return to categories.
Esc or Ctrl+C cancels. A compact wordmark, color, action hints and workspace context
provide terminal presentation without changing the commands themselves.

Selection lists have blank space before and after their options. Individual rows
stay compact; short terminals scroll the list. The UI browser filters current components as you type, shows source,
dependency and license details, and returns to browsing without closing the
launcher. Installation is an explicit action. Native/web targets use Space to
toggle one or both choices, then Enter to confirm; nothing is selected implicitly.

`NO_COLOR` and `--color never` disable colors; `--color always` explicitly enables
them. For example, `tsrust --color never` opens the launcher without color. Narrow
terminals omit the wordmark and scroll longer selections. `TERM=dumb` uses numbered
prompts with `q`/EOF cancellation. When stdin or stderr is not a terminal, bare
invocation prints help and exits successfully instead of waiting for input.

`tsrust new` asks for a destination and console/UI project type. `tsrust new --ui`
asks for the destination and UI targets. The destination must be new and its parent
must exist; invalid answers retry before creating files. `tsrust ui` offers add/list,
and `tsrust ui add` lets you select several current components with Space, then
install them together with Enter. Target selection follows before any writes.
These prompts require terminal input and stderr.
`ui add --json`, `--yes` and `--dry-run` never prompt; pass component names
and target flags for scripts, for example `tsrust ui add button card --target native`.
An optional `@version` must match the current bundled version. Older versions are
not offered for new installation; existing installed source remains supported.
`tsrust ui list --bundled` groups components and foundations in a terminal;
redirected text and `--json` preserve their plain machine-readable output.
All catalog views show one current version per component and source variant.
Upgrades preserve the old file and create a versioned filename such as
`card-v3.tsrx`. Newly installed bundled dependencies import their recorded file.
Existing application imports and customized components are never rewritten.

```sh
tsrust new hello
cd hello
tsrust dev
```

`new` creates source, a test, README, `.gitignore` and `tsrust.config.json`. It refuses
existing destinations. Use a name beginning with a letter, followed by letters,
digits, hyphens or underscores. A destination path is allowed if its parent exists.

`dev` builds and starts the program, watches project source and config changes, and
restarts after each successful build. Build errors leave the last working program
running. Ctrl+C stops the watcher and its processes. Use `--release` for optimized
builds or `--` to pass arguments to the program. macOS and Linux are supported.

You can also run commands once:

```sh
tsrust check
tsrust format --check
tsrust test
tsrust run
tsrust run -- --port 3000
tsrust build
```

These commands find the nearest `tsrust.config.json`, including from nested
folders. Build/run/emit use its entry when you omit a file; build uses its output.
Configured run and dev use the project root as their working directory. Explicit
file commands continue to work without config.

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

The watcher reads `.tsr` files below the project root and excludes `.git`,
`node_modules`, `target` and symlinks. Changes to external imports are not watched.
Development binaries are temporary; `build` writes the named output. A program that
exits normally runs again on the next change. Restarting does not preserve its
in-memory state. The template has a `main` function, but the language does not require it.


`tsrust test` groups results by file, marks passing and failing tests, shows failure
sections and reports file/test totals. Durations measure test execution, excluding
compilation. Strict equality failures show expected and received values; object
comparison still uses identity, even when printed contents look alike. Console output from each test and its per-test hooks appears
below that test's result, with stdout/stderr labels and the original source file,
line and column. Imported helpers point to their own source files. Multiline output
is indented beneath its source location. Test names are not repeated beside stream
labels; suite setup/teardown output retains its separate scope label. Output remains
on its original stream; logs are retained when a test fails. Colors follow
terminal support and `--color auto|always|never`. Set `TSRUST_TEST_REPORTER=plain` for
the compact line reporter used by older scripts. Direct `runTests()` calls retain
that compact output unless the pretty reporter is selected through the environment.


Dev output defaults to one watching message and a build/rebuild duration, followed
by program output. Successful program exits are silent; failed builds and nonzero
exits remain visible. Dev status uses cyan for information, green for successful
builds, red for failures and yellow for watcher fallback warnings. Colors enable
automatically on a supported stderr terminal; redirected output, `NO_COLOR` and
`TERM=dumb` stay plain. Override with `--color auto|always|never` before `--`.
Program output is unchanged. `tsrust dev --verbose` adds process IDs, temporary binary
paths and successful exit messages, plus checking, emission, backend verification,
cache lookup, runtime preparation, persistent dependency workspace paths, runtime
build/publication and combined native compilation/linking timings.

Dev shares the validated project cache at `target/tsrust-native` across edits and
sessions. Each running executable remains a private copy, so builds cannot overwrite
its inode. Identical source/toolchain/options reuse an artifact; a new source version
still needs native compilation. The watcher uses native filesystem notifications on macOS and Linux. Notifications
trigger reconciliation of source contents; idle polls do not scan the source tree.
Unchanged file contents remain cached. Native setup/runtime failures print a message
and fall back to metadata polling. Set `TSRUST_DEV_WATCH=poll` to force that fallback.
macOS uses one descriptor per watched file/directory, so descriptor exhaustion also
falls back visibly. The dev-loop milestone includes retained frontend state and separate native runtime
compilation; see [its acceptance and remaining limits](../plans/dev-loop-performance-plan.md).

`run` accepts program arguments after `--`, with either a configured entry or an
explicit source: `tsrust run src/main.tsr -- --port 3000`. Arguments after the
separator are passed directly to the executable, without shell interpretation.
Flags such as `--help` and `--release` after the separator belong to the program;
compiler options belong before it. Empty arguments, spaces and Unicode are
preserved. This does not add a language API for reading process arguments.

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

## UI project entry

`tsrust new counter --ui` generates `src/app/page.tsrx`, exporting `Page`, and an
explicit `src/main.tsr` bootstrap that imports and mounts it. `build.entry` continues
to identify that executable file in `tsrust.config.json`; the component name is not
an implicit executable entrypoint. This is the first root-page convention from
UI_VISION.md. Nested discovery, layouts, loaders and navigation are pending.

The native UI development runtime uses optimized renderer dependencies without
Cargo dependency debug symbols. Application arithmetic remains checked in dev and
release. The first native support compilation can be expensive; subsequent edits
reuse the exact runtime artifact when the runtime/toolchain/profile are unchanged.
`tsrust dev --verbose` reports frontend work, runtime cache status and native build
time so a cache miss can be distinguished from application checking.
