# Working in this Tsrust project

## Studio product direction — owner, 2026-11-10

Use established native editor layouts from VS Code, Visual Studio, Zed and
IntelliJ. Every visible control, tab and toolbar item must perform its real
action, including in demos. Do not add decorative app-name/filename headers,
placeholder controls, pretend tabs or website-style branding. File names belong
in functional document tabs; actions belong in compact editor toolbars/menus.
Validate behavior in the running app, not only its appearance.

Reuse existing `std:ui`/GPUI Kit components before building custom controls. Use
Tree/TreeItem for project navigation. The workspace directory itself is not an
extra tree row: show its contents directly. Keep the main view free of convenience
button bars; put commands in the editor's native interactions, menus or the
appropriate dedicated surface. Do not add filler merely because it is clickable.

## Initial editor test workspace — owner, 2026-11-10

Use this `apps/studio` package as the initial open directory when developing and
testing Studio. Do not initially open/index the entire Tsrust checkout. Expand
the workspace only when a concrete workflow requires it.

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

`.tsrx` files hold JSX components rendered natively by GPUI Kit through
`std:ui`. `tsrust new <name> --ui` creates a desktop app; `tsrust ui list` shows
the components and `tsrust ui add <Component>` copies an editable wrapper into
`src/components/`. `tsrust dev` rebuilds and restarts the window on save, and
`tsrust package` builds downloadable artifacts (see the packaging guide).
