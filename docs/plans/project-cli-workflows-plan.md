# Project CLI workflows: new and dev

Status: accepted on macOS, including installed-consumer acceptance at `57ba7b74`. The owner prioritized this milestone before
further S09 host operations and application workflows. Keep the last working
program running when a development build fails (explicitly approved).

## Supported first configuration

Use one `tsrust.config.json`, found by walking from the working directory to the
nearest ancestor containing it. Version 1 defines a name and build entry/output.
Unknown settings and unsupported versions are errors. Configuration never executes
code. Relative config paths resolve from its directory; source entry paths must
end in `.tsr`, and output must not overwrite project sources or config.

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

`tsrust new <name-or-path>` creates a new directory exclusively. It never overwrites
an existing directory, file or symlink. Names start with an ASCII letter and contain
letters, digits, hyphens or underscores, at most 100 characters. The parent directory
must exist. Scaffold source, an imported greeting module, a native test, README,
config and `.gitignore`. Format source with the compiler's current formatter.
Creation needs no backend compiler and does not install dependencies or initialize Git.

When inputs are omitted, build/run/emit use the configured entry; build also uses
the configured output. Configured run executes from the project root. Check,
format and test default to the discovered root, even from nested directories.
Explicit source paths retain their existing command behavior and cwd-relative
resolution. No config is required for existing explicit-file workflows. No main
function is required; the application template includes one for its greeting.

## Development lifecycle

`tsrust dev [--release] [-- program-arguments]` requires a config. It watches `.tsr`
files beneath the project root and the config, excluding `.git`, `node_modules`,
`target` and symlinks. It does not currently watch imports outside that root.
Polling compares contents and requires a quiet interval; rename, creation, deletion
and same-size edits are observed. Source changes during a build discard that build.

Build each candidate into a private temporary directory. Keep the current program
running through syntax, type, config and backend failures. After a successful current
build, terminate the old process group, reap it and launch the new binary from the
project root. Normal program exit leaves the watcher running. Ctrl+C and termination
stop both build and application process groups. After a brief graceful termination
window, force termination prevents hung children from blocking restart. Temporary
binaries and per-build caches are removed instead of accumulating across changes.

Initial supervision support is macOS/Linux; other platforms need native process-tree
and signal integration before claiming dev acceptance. This is restart-on-change,
not preservation of application state across builds.

## Acceptance and remaining configuration

Require native scaffold build/run/test, stable formatting, ancestor discovery,
invalid config/name protection, last-good preservation, changed import restart and
interrupt cleanup. Run focused existing CLI/format/diagnostic regressions. Refresh
installed CLI consumers and run an installed new/dev smoke before acceptance.

Configurable indentation/quote style, include/exclude policies, target selection
and richer build parameters remain follow-ups. Do not generate ignored settings.
The shared config and CLI implementation should remain the foundation for those
features; separate lint/format config files are not introduced.


## Focused evidence

Scaffold/config tests pass, including native build/run/test from nested directories,
project-root runtime file access, malformed settings and non-overwrite protection.
Development tests exercise a failed initial build followed by repair, invalid config
while a program runs, failed dependency compilation preserving that process, successful
restart, passthrough arguments and Ctrl+C cleanup. A process-group witness verifies
shell descendants are also reaped. Existing acceptance (25), formatting (28) and
project diagnostics (10) tests pass; four new project command tests and one process
unit test pass. The first runtime-root witness used invalid `Ok()` syntax; it was
corrected to the existing `Ok(_)` pattern without changing language behavior.

Original command logs use `tsrust-project-commands-*`, `tsrust-project-config-build.log`,
`tsrust-project-dev-checks.log` and `tsrust-cli-workflow-*` in SQLite. No runtime
performance measurement or new full compiler/LSP regression is claimed for this
CLI-only milestone. The preceding complete diagnostics regression remains recorded.

Installed debug/editor/release/global CLI consumers were atomically refreshed from
`57ba7b74` and hash-verified. The installed CLI passes scaffold creation, nested-root
check/format/test/run/build, imported-source dev restart and interrupt shutdown.
Evidence is retained in SQLite as `tsrust-project-cli-consumer-build.log`,
`tsrust-project-cli-consumer-refresh.json` and `tsrust-project-cli-installed-smoke.log`.
Linux process supervision is implemented but was not independently exercised here.

## New-project editor discovery repair

A real newly scaffolded workspace exposed a distribution gap: the extension required
an explicit server path, while this repository supplied that path only in workspace
settings. The extension now discovers executable files beside an explicit companion
path, in ~/.cargo/bin, or in absolute PATH entries. Explicit invalid overrides remain
errors. Workspace trust remains required. Both CLI and language server were installed
atomically in ~/.cargo/bin from compiler/LSP checkpoint 5408b3b5, and local consumers
were refreshed. Three resolver tests, grammar checks and the real VS Code integration
suite pass with no server-path setting. Original logs are in SQLite under
`tsrust-editor-discovery-*`. The fixed VSIX is installed. The owner's `tsrust_test` window subsequently showed checking enabled. After the
562a6c00 consumer refresh, the language server was restarted through its status item
and the same window showed checking enabled and no Problems. Live-window acceptance
is now verified.

## Test console reporting

Pretty test reports capture console output per test, render it below the result,
label stdout/stderr, indent multiline content, and show original project source
coordinates (including imported helpers). Suite hooks have separate labels;
per-test hooks stay with their test. Failed tests retain their logs. Plain reports
and ordinary console rendering retain their existing output. Source coordinates
come from the checked project graph rather than reading files at execution time.

Focused evidence: five reporter tests, 56 testing-library tests and 12 console
output tests pass. The installed pre-fix CLI reproduces the missing attribution.
Logs `tsrust-test-log-*` are retained in SQLite, including the initial fixture import
path error and its repair. Formatting and installed refresh evidence accompany the
source checkpoint. This user-priority repair does not close S09 or its pending
buffered-file benchmark acceptance.
