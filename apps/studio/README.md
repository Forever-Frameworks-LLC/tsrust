# Tsrust Studio

Native developer-tool application under active implementation. Created with
`tsrust new apps/studio --ui`; application source remains `.tsr`/`.tsrx`.

Run `tsrust dev --verbose` in this directory. The current directory is the default
workspace; pass `tsrust dev -- /absolute/project/path` to select another directory.
Dogfooding currently uses this Studio package, not the entire compiler checkout.
When launching through Codex tools on macOS, use
`env -u CODEX_TOOL_CALL_ID tsrust dev --verbose`: a fresh invocation ID otherwise
changes Cargo's effective environment and correctly invalidates native caches.
Ordinary terminal launches can use the standard command above.

Files open in closeable tabs. Edit text and press Cmd+S, or use the title-bar save
icon. A dot marks unsaved changes. Saves check the original disk bytes and reject
conflicting external changes. Unsaved drafts are journaled under
`~/.tsrust/studio/recovery` and restored on restart. The journal also retains
open files and the active tab; clean buffers read current disk contents. Missing
clean files are skipped, while a missing dirty file preserves the journal and
reports recovery failure. Native drag order, splits, cursor/scroll positions and
sidebar expansion are not restored yet. Dirty tabs refuse to close;
explicit discard/close prompts and recovery of deleted files remain unfinished.
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
image formats still need decoder acceptance. Cmd+=, Cmd+- and Cmd+0 control the
shared tree/editor text size. Settings opens a separate window backed by
`~/.tsrust/studio/tsrust_studio.config.json`. Use its searchable theme selector
or JSON editor, then Save. The catalog contains two Studio themes and 36 upstream
community variants; source revision, authors, URLs and hashes are in
`src/themes/sources.json`, with upstream license files alongside them.

Layout settings use `navigationSide` and `activityBarSide` (`left` or `right`),
plus `panelPosition` and `chatPosition` (`left`, `right` or `bottom`). Save in
Settings to apply them to the current workspace. Older theme/font configuration
files retain their meaning: omitted layout fields default to left navigation and
activity bar, bottom task/terminal panel and right AI Chat. The chat toggle opens
an independent region, so chat and terminal can remain visible together.

The Language selector previews English or Brazilian Portuguese. Save persists
`locale` (`en` or `pt-BR`); Reload restores the saved language/theme and discards
an unsaved preview. Older settings default to English. Catalogs in `src/locales`
cover app controls, accessible names/tooltips, search summaries and owned errors.
File names, file contents, theme names, configuration keys and external technical
diagnostics retain their original text. Catalog keys use English message IDs as
fallback; `node ../../tools/check-studio-locales.mjs` checks coverage and matching
interpolation placeholders.

The bottom status bar reports the selected file type, UTF-8/unsaved state and the
current unconnected language-service state. Cmd+J and terminal icons toggle the
task/terminal panel at its configured location. Terminal, Problems, Output,
Debug, Source Control, AI Chat and Extensions regions explicitly describe their
unfinished integrations.

Native document tabs support reordering and split controls. Named Open tabs menus
reveal clipped files. Required work includes tree file moves, full pointer and
manual-scroll acceptance, pane/view persistence, JSON key highlighting, LSP and
terminal integration, broader locale/RTL coverage, full accessibility/IME acceptance and remaining editor/agent
workflows. This is an active development checkpoint, not complete editor acceptance.
See `docs/plans/tsrust-studio.md` at the repository root for full scope and evidence.
