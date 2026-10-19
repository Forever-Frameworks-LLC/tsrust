# Rich documentation text and web development server

Source checkpoints: `0323d1fb`, `74f66d45`, `aac7c4af`, `2fb36412`,
`328e9de6`, `6bbacdbe`, `19d1d634`, `86033af4`, and `a8f63f4c`.

## Observable behavior

`tsrust dev --target web --port 4317` serves the configured UI project, watches
its source and declared documentation inputs, and reloads after successful
publication. Cargo dependencies and the exported package workspace persist.
Compilation errors retain the previous successful app. JavaScript and Wasm use
the same publication revision; stale asset requests fail explicitly.

The documentation app has section navigation, styled inline Markdown, highlighted
source and an exact-source Copy control. Theme syntax colors are configurable in
both modes. Copy reports success only after the platform write completes; denial
allows retry and completion after disposal cannot retain or revive a mount.
Nested Text and local Link elements form one wrapping paragraph on both targets.

The landing page uses the actual interactive example. Search instructions,
Back/Next controls, per-page development status and the placeholder benchmark
section are removed. Inventory destinations still state when unavailable.

## Focused evidence

The live browser observer passes section navigation, inline semantics, compiler
lexeme colors, selection across tokens, exact platform clipboard bytes, permission
failure/retry, pending completion, reentrant disposal, both modes and 480px layout.
Initial observer failures were browser final-empty-line selection semantics,
headless focus, and an empty clipboard item; the repaired observer distinguishes
visual selection from exact DOM ranges and verifies the unmodified copied bytes.
It restores materialized clipboard items and owns/reaps its Chrome process.

Native inline layout tests pass wrapping, styled glyph runs, link hit/accessibility
geometry, syntax colors and focus-ring tokens. The final accessibility repair
excludes decorative aria-hidden content from link names while preserving painted
text. Existing text-editor, clipboard-state, theme fallback/alias/cycle and source
lexeme checks are reused. The focused LSP protocol case passes JSX help and
standalone, custom-named and inline configuration theme completion/hover.

CLI checks pass nested configuration discovery, lock release, frozen web frontend
inputs, revision rejection and a complete 2 MB Wasm response from a nonblocking
listener. macOS accepted sockets now explicitly use blocking writes with a timeout.
The actual live server serves complete current JavaScript and Wasm responses.

Source-only callback proofs are reused across ownership inference passes and the
ordinary project retry into recursive access inference, within one input. Isolation,
malformed-next-input and existing capture/origin witnesses pass. The final native
project witness uses formatted modules and std:testing assertions for generic calls,
recursive mutation, array length and values. No ownership inference pass is omitted.
The new proof-count witness initially omitted a generic contract, then its malformed
input replacement missed formatter spacing; both fixture mistakes are corrected.
Passed cases were retained and only the affected witness was rerun. The real docs build still took
297,033 ms in the debug compiler: startup speed remains unresolved. This is an
operational observation, not a performance benchmark.

## Accepted scope and limits

Formatted generated native inline execution passes. The complete shared native docs
app builds, remains alive for a bounded five-second launch, emits no output/errors
and is reaped. This adds no OS gesture, native clipboard gesture or pixel claim.
Installed CLI/LSP consumers are refreshed; editor settings are restored byte-for-byte
with their timestamps and the active editor maps the installed inode.

Closed original logs, failed attempts, screenshots and receipts are independently
restored and verified from SQLite before loose records are removed. Two obsolete
runtime workspaces (1,941,569,860 bytes) are removed after checking their exact
identities, unused locks and accepted replacements. Both current warm workspaces
remain. Obsolete executable variants (1,412,375,680 bytes), the reaped native
smoke executable and eight byte-verified historical preview files are removed.
All five consumers are current through `a8f63f4c`; the active preview keeps its
accepted assets in memory. No complete compiler/LSP matrix, SQL suite or timed
benchmark was run.

Markdown images remain labeled references and external links display destination
text. Native selection is limited to a paragraph/code block; cross-paragraph
selection, text-range accessibility and selection autoscroll remain incomplete.
Dynamic keyed routing, overlay/focus families and most pinned components are still
required. This section does not claim complete shadcn/Base UI parity.

Key SQLite records include `tsrust-docs-inline-browser-clipboard.log`,
`tsrust-docs-inline-lsp-repair.log`, `tsrust-docs-inline-native-hidden.log`,
`tsrust-docs-inline-native-execution.log`, `tsrust-docs-inline-publication.json`,
`tsrust-docs-inline-republication.json`, `tsrust-docs-inline-native-launch.json`,
`tsrust-docs-fallback-final.log` and `tsrust-docs-fallback-native-assertions.log`.
The intentional live dev log remains open outside SQLite.
