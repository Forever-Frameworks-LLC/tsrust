# Create and develop a Tsrust project

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
exits remain visible. `tsrust dev --verbose` adds process IDs, temporary binary
paths and successful exit messages, plus checking, emission, backend verification,
cache lookup and combined native compilation/linking timings.

Dev shares the validated project cache at `target/tsrust-native` across edits and
sessions. Each running executable remains a private copy, so builds cannot overwrite
its inode. Identical source/toolchain/options reuse an artifact; a new source version
still needs native compilation. The watcher still polls source contents and checking
still starts afresh for each build. Filesystem events, retained compiler state and
separate runtime compilation remain follow-up performance work.
