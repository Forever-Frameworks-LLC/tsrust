# Working in this Tsrust project

Read `tsrust.config.json` before changing entrypoints, output paths, UI targets or
component locations. Tsrust checks ownership and supported language/library APIs;
use its diagnostics and local source instead of assuming every TypeScript or Node
API is available.

## Commands

Run commands from the root defined by the nearest `tsrust.config.json`. Nested
projects use their own configuration:

- `tsrust check`: check project sources and imports.
- `tsrust format`: format the whole project; `tsrust format --check` verifies it.
- `tsrust test`: run assertion-based project tests.
- `tsrust run`: build and run the configured native entry once.
- `tsrust build`: write the configured native executable.
- `tsrust dev`: watch inputs, preserve the last working application on a failed
  build, and replace it after a successful rebuild. Ctrl+C stops its descendants.

Check, format and test a complete change before calling it finished. Preserve
unrelated changes. Fix ownership errors deliberately; do not add clones merely to
silence diagnostics. Keep behavioral tests and examples consistent with the code.

## UI components

For UI projects, read `AGENTS.md` inside the directory configured by
`ui.components` before importing or editing an installed component. It records
the installed module paths, exported signatures, props and available documentation.
The current local source is authoritative after customization.

Use `.tsrx` for JSX components and `.tsr` for ordinary modules. Trace the configured
`build.entry` bootstrap and its imported application components before changing
composition. A fresh UI scaffold uses `src/main.tsr` and `src/app/page.tsrx`;
existing projects may use other paths. Bind application logic in typed handlers
and keep state in the component's existing signal/store pattern.

Use `tsrust ui list --bundled` to inspect the offline catalog and
`tsrust ui add <component> --dry-run` before an installation when its effects are
unclear. Installed components are editable project source. Preserve their notices,
`ui-components.lock.json`, theme and owner changes. Upgrades may create versioned
files; inspect the recorded imports rather than overwriting customized sources.

Native UI needs Rust, a system linker and a graphical desktop. Web UI additionally
needs the pinned Wasm toolchain; select the configured target explicitly with
`tsrust build --target web` or `tsrust dev --target web`. Windows uses native
titlebars; custom titlebars currently require macOS. Do not claim native/web parity
from type checking alone: exercise the affected target and observable behavior.
