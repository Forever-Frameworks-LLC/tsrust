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
