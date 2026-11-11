# Tsrust Studio

Native developer-tool application under active implementation. Created with
`tsrust new apps/studio --ui`; application source remains `.tsr`/`.tsrx`.

Run `tsrust dev --verbose` in this directory. Studio opens this package as its
initial workspace. The custom macOS title bar holds sidebar, file-search and
settings actions with tooltip icons. The activity rail switches Files/Find file;
the sidebar collapses and uses Kit's draggable Resizable divider. No permanent
status/action bar or app-name banner consumes editor space. Errors appear only
when an operation fails.

Expand the native Tree and open real files in closable tabs. Cmd+F searches the
active editor; Cmd+P opens filename/path search. Cmd+= and Cmd+- resize editor text;
the settings menu also provides text reset, appearance and tab-move actions.
Tree row text scaling/ellipsis and state retention across tab moves are implemented
in the shared renderer and awaiting the updated live-runtime acceptance. Drag tab
reordering and complete keyboard/accessibility acceptance remain open.

The package uses separate chrome, navigation and editor components and a shared
theme asset. File extensions go directly to the Kit syntax resolver; Studio no
longer maintains an extension/language table. Its syntax palette distinguishes
import/preprocessor keywords from identifiers. Generated target/.git/node_modules
are omitted. UTF-8 preview reads remain bounded to4MiB; binary/read errors preserve
the active document. Filename search currently builds this package's listing once;
worker-backed content search and file watching remain required.

Each open file now uses a revisioned document session, with a saved-text baseline.
Reopening gets a fresh session; old proposals reject even when the path and bytes
match. The document model refuses dirty reload/close without explicit discard.
Those guards are tested; native close prompts and disk saving are not yet connected.

This stage is read-only: no editor writes or save controls are enabled. Safe
editing, saves, external-change handling, recovery and dirty-close prompts remain required.
Native tab switching retains scroll, selection and each file’s Find query,
verified in the running macOS app. Undo/redo with editable buffers remains unverified. Initial search typing through CUA sometimes loses
its first character; raising the window and selecting/retyping the query worked,
but focus/AX behavior is not fully accepted. Search controls also need AX names.

On macOS, dev builds use a temporary native app identity without signing or release
packaging. Run with desktop access: the restricted process sandbox can deny macOS
pasteboard/XPC access and abort startup. Failed builds preserve the last working
window. Successful source rebuilds replace the app and currently reset open tabs.
Keep the owner-requested current dev supervisor running between changes.

Run `tsrust test tests --profile` for document-proposal and file-opening acceptance.
The separate native-AI admission lives in `experiments/studio-ai`; it is not a
Studio runtime dependency or a qualified model distribution.

Full required scope and current evidence:
[plan](../../docs/plans/tsrust-studio.md),
[capability audit](../../docs/plans/tsrust-studio-audit.md).
Signing, packaging and publication follow full acceptance and owner authorization.

Required configuration work includes persisted locale/translations, theme/fonts,
navigation/activity-bar side, and terminal/AI-chat docking. The current settings
menu is not the finished settings service. Terminal defaults to a resizable bottom
panel under the editor and AI chat to the right; neither panel is implemented yet.
