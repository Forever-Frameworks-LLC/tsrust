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

`.tsrx` files hold JSX components. GPUI Kit UI support is in development
(`tsrust ui` and `tsrust new --ui` report that); applications expose HTTP APIs today.
