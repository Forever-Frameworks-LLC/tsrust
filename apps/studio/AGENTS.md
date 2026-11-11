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
  build, and replace it after a successful rebuild with changed application inputs.
  Unchanged live applications retain their state. Ctrl+C stops its descendants.

Check, format and test a complete change before calling it finished. Preserve
unrelated changes. Fix ownership errors deliberately; do not add clones merely to
silence diagnostics. Keep behavioral tests and examples consistent with the code.

## UI components

`.tsrx` files hold JSX components rendered natively by GPUI Kit through
`std:ui`. `tsrust new <name> --ui` creates a desktop app; `tsrust ui list` shows
the components and `tsrust ui add <Component>` copies an editable wrapper into
`src/components/`. `tsrust dev` rebuilds and restarts the window on save, and
`tsrust package` builds downloadable artifacts (see the packaging guide).

## Purposeful layout and configuration — owner, 2026-11-10

Develop visual design alongside editor/storage capabilities. Use a coherent
neutral palette, explicit region dividers, compact tooltip icons, left-aligned
truncated file labels, and shared text zoom for navigation and editor surfaces.
Inspect the running app for spacing, overflow, resize/collapse, empty/error states
and keyboard behavior before claiming the UI is accepted. Format coherent UI
changes during development and keep chrome, navigation, editor panes and models
in separate composable files.

Configuration must cover locale/i18n, theme/text sizing, navigation/activity-bar
side, terminal docking and AI-chat docking. Default terminal placement is a
resizable bottom panel beneath the editor, while navigation keeps full height;
these placements are defaults, not fixed product restrictions. Use existing Kit
docking/layout capabilities and one validated settings model; avoid one-off
position toggles or placeholder panels. Safe editing, saves and recovery remain
required alongside UI work.

## Development-session restarts — owner, 2026-11-11

Restart the dev supervisor whenever needed without requesting permission. Keep
one current development preview running afterward. Once editable buffers are
enabled, preserve drafts across legitimate dev reloads and restarts; restart
authorization does not remove the file-safety requirements.

## Editor completion requirements — owner, 2026-11-11

Settings belong in a separate window backed by tsrust_studio.config.json. Keep
navigation/editor sizing visually consistent. The activity-bar search searches
content with regex and case sensitivity; quick filename opening is a separate
command, without duplicate search icons. Support image viewing, Markdown preview,
distinct JSON highlighting, clear active tabs and selected-tab reveal. Workspace
roots come from launch arguments, with directory-open/drop integration to follow.
Implement editable documents/saving/recovery, native tab/tree drag/drop, resizable
editor splits and a substantial theme catalog with verified persistence. Restore
a useful status bar and terminal/panel toggles. This update explicitly authorizes
clearly labeled planned-feature panels/controls, superseding the earlier ban on
placeholders. Keep the original functional-UI requirements for implemented work.


## Feature implementation and human review — owner, 2026-11-11

Prioritize usable features and live manual acceptance. Batch the smallest relevant
checks after a coherent feature; do not repeatedly run full regressions or unchanged
executables. Keep data-preservation checks, build requirements and meaningful
regressions, but do not let speculative preference testing displace implementation.
Use established VS Code/Xcode/Zed layouts and the existing std:ui/GPUI Kit controls.
Settings must use categorized Settings pages with GUI controls first, custom window
chrome, and optional formatted full-height JSON. Keep file-type icons, consistent
navigation/editor typography, one terminal toggle, and real service integration on
the implementation path. Owner will provide preference feedback through the live app.


## AI action parity — owner, 2026-11-11

Every editor feature must expose a direct action to the integrated AI, covering
all semantic actions available to the user. Route toolbar/menu/shortcut actions
and AI tool calls through the same typed command implementation. Keep behavior,
validation, document transactions, errors and cancellation consistent. A separately
implemented AI approximation, simulated clicks, an exported helper alone or tool
metadata without an executable hookup does not satisfy this requirement.

Command definitions must describe arguments/results, availability, target scope,
preconditions and effects. The trusted host supplies actor identity, capabilities
and approval state; model arguments cannot grant privileges or claim user origin.
Agents must target explicit workspace/document identities and revisions rather
than relying on whichever tab happens to be active. Background work must not
silently change the user's selection, focus or active document.

Bounded reads, search and explicitly requested navigation/view operations may run
within granted scope. Agent edits, filesystem mutations and persistent settings
changes use isolated proposals and the existing review/conflict boundary. Commands
that can write files or execute code run in isolation unless an applicable grant
explicitly permits their effects. Consequential/destructive operations require the
appropriate approval; an agent cannot approve its own proposal, expand its scope
or bypass safeguards through another command, process or agent.

Feature acceptance requires a working UI route and a real registered AI action
route, including relevant guardrail/error/cancellation evidence. Record existing
UI acceptance separately from missing AI parity. Native editor actions such as
selection, undo/redo, tab movement and pane layout require checked programmatic
APIs where those hooks are absent. Clearly labeled planned features remain
unavailable actions with an explanation, not pretend successful tool calls.
