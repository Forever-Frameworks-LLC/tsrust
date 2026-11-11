# Tsrust Studio

Native developer-tool application under active implementation. Created with
`tsrust new apps/studio --ui`; application source remains `.tsr`/`.tsrx`.

Run `tsrust dev --verbose` in this directory. The current directory is the default
workspace; pass `tsrust dev -- /absolute/project/path` to select another directory.
Dogfooding currently uses this Studio package, not the entire compiler checkout.
When launching through Codex tools on macOS, use
`env -u CODEX_TOOL_CALL_ID -u NO_COLOR tsrust dev --verbose`: a fresh invocation ID otherwise
changes Cargo's effective environment and correctly invalidates native caches.
Clearing the launcher's `NO_COLOR` lets interactive shells detect their PTY's
color support; terminal-specific user preferences still apply.
Ordinary terminal launches can use the standard command above.

Use Edit → Format Document (Option+Shift+F), Project → Check Project
(Cmd+Shift+B), or the compact braces/check controls in the status bar. File → Save All
uses Cmd+S and saves every modified open text file, including non-active tabs.
Each file retains its own format-on-save and disk-conflict checks; failures remain
in Problems while other files continue saving. Check-on-save runs after the batch. Formatting supports .tsr/.tsrx, works on the unsaved buffer and leaves
the result unsaved until Save. An intervening edit, close or reopen rejects an old
formatting reply. Switching tabs does not retarget it. Compiler checks read saved
project files and report their actual diagnostics in Output.

Settings → Formatting exposes the supported rules and format/check on save.
Both save hooks default to off. Global defaults are persisted in
tsrust_studio.config.json. Project overrides in tsrust.config.json are partial:

```json
"format": {
  "indentWidth": 2,
  "organizeImports": true,
  "importWrapThreshold": 5,
  "jsxReturnParentheses": true,
  "jsxChildrenOnNewLines": true,
  "blankLinesAroundBlocks": true
},
"editor": {
  "formatOnSave": true,
  "checkOnSave": true
}
```

Add those fields to the existing project object alongside name/build/schemaVersion.
Indent width accepts 1–8 spaces. Project fields override editor defaults; the
formatter and LSP resolve the nearest project's format rules. Formatting preserves
mixed text and multiline literal layout so that reindentation cannot change their
contents. On-save formatting must finish against the same buffer revision before
Save; failure preserves the draft. A successful save can queue a background check
without changing the user's selected tab or opening Output. These controls use the
existing compiler checker; no separate lint engine is implied.

Files open in closeable tabs. Edit text and press Cmd+S to save all modified files. A dot marks unsaved
changes. Saves check the original disk bytes and reject
conflicting external changes. Unsaved drafts are journaled under
`~/.tsrust/studio/recovery` and restored on restart. The journal also retains
open files and the active tab; clean buffers read current disk contents. Missing
clean files are skipped, while a missing dirty file preserves the journal and
reports recovery failure. Native Dock split axes/measured sizes, tab order and
per-group selection are restored by stable panel values. Cursor/scroll positions
and sidebar expansion are not restored yet. Fast horizontal, vertical and sidebar resizing are verified in the native app. Dirty tabs refuse to close;
explicit discard/close prompts and recovery of deleted files remain unfinished.
Cmd++/Cmd+- adjust shared tree/editor text size between 8 and 48 pixels. Zoom
is retained per workspace in its view-state journal, separately from the base
`fontSize` in Settings. Cmd+0 resets to that configured base; Settings Save/Reload
applies the base while retaining the workspace zoom offset. This changes document
and navigation text, not the entire operating-system window scale.

Recovery currently assumes one Studio writer per workspace and writes
synchronously; large-document performance and multi-window coordination remain
required.

The activity-bar search searches file contents. Its Aa and .* controls enable
case sensitivity and regular expressions. Click the search button to run it and
click a result to open its line. Cmd+P opens filename/path search; Cmd+F searches
the current editor. Project search currently runs synchronously, reads at most
4 MiB per disk file, and reports skipped files. Worker-backed search, filesystem
watching, cancellation and Enter-to-search remain required.

PNG viewing and Markdown source/preview are verified in the app. Other routed
image formats still need decoder acceptance. Image previews start in Fit mode,
follow pane resizing, and provide percentage zoom, 100% natural size and scrollbars
in both directions. Cmd+=/Cmd+- zoom the active image; Cmd+0 restores Fit. On text
tabs those shortcuts control the shared tree/editor text size. Image view changes
have guarded `set_image_zoom` and `pan_image` tools targeting explicit sessions;
`save_files` queues explicit revisions through the same Save All implementation. Settings opens a separate window backed by
`~/.tsrust/studio/tsrust_studio.config.json`. Its searchable, categorized controls
save changes immediately. The formatted JSON view fills the window; use Cmd+S or
its Save action to apply JSON edits. The catalog has two Studio themes and 36 upstream
community variants; source revision, authors, URLs and hashes are in
`src/assets/themes/sources.json`, with upstream license files alongside them.

Layout settings use `navigationSide` and `activityBarSide` (`left` or `right`),
plus `panelPosition` and `chatPosition` (`left`, `right` or `bottom`). Changes apply
immediately to the current workspace. Older theme/font configuration
files retain their meaning: omitted layout fields default to left navigation and
activity bar, bottom task/terminal panel and right AI Chat. The chat toggle opens
an independent region, so chat and terminal can remain visible together.

The Language selector immediately applies and persists `locale` (`en` or `pt-BR`).
Reload rereads the saved configuration and discards unsaved JSON edits. Older
settings default to English. Catalogs in `src/assets/locales`
cover app controls, accessible names/tooltips, search summaries and owned errors.
File names, file contents, theme names, configuration keys and external technical
diagnostics retain their original text. Catalog keys use English message IDs as
fallback; `node ../../tools/check-studio-locales.mjs` checks coverage and matching
interpolation placeholders.

The bottom status bar reports the selected file type, UTF-8/unsaved state and the
current unconnected language-service state. Cmd+J and the terminal icon toggle the
task/terminal panel at its configured location. The native terminal starts a shell
in the workspace directory. New Terminal and Close share the panel header; the
resizable right session list switches independent shells and closes individual
sessions. Its action menu interrupts the foreground job, copies a selection or
restarts an exited shell. The terminal viewport fills its region without an inset
frame. Editor and panel tab labels follow configured text size and zoom.
Problems lists source-labeled structured diagnostics and operation errors. Debug,
Source Control, AI Chat and Extensions still describe unfinished integrations.
LSP diagnostics remain pending the language-service connection.

Native document tabs support reordering and split controls. Named Open tabs menus
reveal clipped files. Required work includes tree file moves, full pointer and
manual-scroll acceptance, pane/view persistence, LSP integration, terminal lifecycle
and accessibility acceptance, broader locale/RTL coverage, full IME acceptance and remaining editor/agent
workflows. This is an active development checkpoint, not complete editor acceptance.
See `docs/plans/tsrust-studio.md` at the repository root for full scope and evidence.


Source layout:

- `src/main.tsrx`: application entry and service lifetime.
- `src/components`: native surfaces grouped by workbench, explorer, editor,
  settings and tasks.
- `src/lib`: document/recovery/workspace models, settings/actions, explorer
  search/file decoration and localization.
- `src/services`: background compiler/process orchestration.
- `src/assets`: bundled icons, themes and translations with provenance/licenses.
- `tests`: stable runtime suites importing the same application models.

Project → Check Project (`Cmd+Shift+B`) starts a named task terminal running the
actual public CLI. Project → Format Project formats open buffers through revisioned
proposals without saving, and runs the public formatter over unopened source files
in a separate task terminal. Unsaved buffers do not block formatting. Task rows show running/success/failure;
output remains available after exit. Cmd-click a `path:line:column` location
(Ctrl-click outside macOS) to open its workspace file and highlight the rest of
the destination line. External URLs and files outside the workspace are rejected.

Edit → Format Document (`Option+Shift+F`) formats an explicit revision of the
current unsaved buffer. File → Save (`Cmd+S`) retains the disk-conflict/recovery
boundary. Settings → Formatting offers independent format/check switches for save
and editor focus loss: neither enabled means manual only; both enabled means both
triggers. Focus formatting changes the draft without saving. Background checking
reads saved files, never silently saves drafts, and does not open panels or change
the selected document. Output retains background-check output and cancellation;
Problems shows its structured compiler diagnostics with source `check`, separate
from the future LSP source.

Global defaults live in `tsrust_studio.config.json`. A project's
`tsrust.config.json` may override `format` rules and `editor.formatOnSave`,
`editor.checkOnSave`, `editor.formatOnFocusChange`, `editor.checkOnFocusChange`.
Rules include indentation width (1–8 spaces), JSX-return parentheses, separate
component children, and blank lines around control-flow blocks. Project fields
are partial overrides. The `TSRUST` environment variable selects the compiler;
otherwise Studio resolves `tsrust` through PATH. Automated checks are bounded to
five minutes and 4 MiB output. Manual task terminals use native scrollback and can
be interrupted or removed through their session actions.

Unsaved files have a filled-circle marker in document tabs and the tree. The
status bar shows their count and a menu of relative paths, including tabs clipped
out of view. Problems has Copy All beside Close, copying complete source/location/
message text. Saving compares disk against the originally loaded disk baseline,
not against the draft: ordinary edits save normally. If another process changed
the saved file, a plain-language comparison popup offers Keep editing, Discard
draft and reload, or Overwrite with my draft. Explicit overwrite still verifies
that the disk snapshot and draft revision have not changed since comparison.
The same guarded resolution and clipboard actions have executable AI tool routes;
the actual integrated agent transport remains pending.

Within each import block, formatting groups `std:` imports first, then other paths after one blank
line, sorting each group by path. Declarations keep their bindings and attached
comments; bare side-effect imports are ordering boundaries. More than five named
bindings wrap one per line by default. `organizeImports` disables reordering;
`importWrapThreshold` accepts 0–100, with 0 disabling wrapping. These rules use the
same global/project overrides and guarded formatting actions as other rules.
