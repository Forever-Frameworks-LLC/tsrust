# Editable text controls: focused acceptance

Status: portable text-control section accepted, 2026-10-19. Base UI + Nova remains the
pinned component baseline; these are portable text-control adaptations, not full
Input, Textarea or Label parity. The coverage inventory keeps each partial.

The editable local `input@3.0.0`, `textarea@3.0.0` and `label@3.0.0` sources install
offline with immutable identities, dependency/license records and caller-owned
files. Existing versions remain unchanged. `examples/ui-inputs` supplies a complete
paired project, explicit entry/targets and an application-owned page. Its controls
use the same `std:ui` facade on both targets.

Controlled values follow application state; uncontrolled values start from
`defaultValue`. Empty and zero controlled values win. Derived value/readOnly
expressions and direct signal values retain weak bindings. String, exact int and
number display preserve their ordinary language conversions. Editing produces
strings, with HTML-compatible line-ending normalization. Unsupported input types
and unpaired browser UTF-16 surrogates fail explicitly. Unrelated component props
keep their declared scalar types. General callback-result inference remains a
language gap; generated input getters use an explicit checked scalar union.

`onChange` receives borrowed target/currentTarget snapshots containing value,
name and id. It retains no DOM object. The native bridge constructs the checked
record layout and stores only actual handlers. No public handle, lifetime
attachment, adapter or dummy callback appears in application code. Failed getters
and handlers release the model and its callback environments; independent mounts
remain usable. Explicit browser disposal also removes listeners, styles and DOM.

Native text editing reuses Parley for Unicode shaping, selection and IME. Preedit
never updates application state; cancel restores replaced text and selection.
Navigation, selection, undo/redo, editor scrolling and app-owned OS text clipboard
are implemented. Clipboard copy/cut failures preserve text, and Linux clipboard
persistence after exit requires a clipboard manager. No helper daemon is spawned.
Arboard 3.7.0 is pinned and included in offline packages. Platform clipboard
interaction and an independent screen-reader session were not exercised.

Labels, descriptions, required/invalid/read-only/disabled state and native SetValue
are exposed through real HTML controls and AccessKit. Native accessibility
text-range navigation remains pending. Nested labels and multiple enclosed
controls fail. Explicit IDs must remain unique within the browser document;
duplicate IDs are not rejected. Autocomplete is a browser hint, not a native
completion service.

Nova input styling adds half spacing, viewport breakpoints, placeholder colors,
invalid/dark/focus-visible conditions and content-sized Textarea. Existing
source-order override rules remain intact. Native scene assertions cover scoped
dark themes and responsive text. Chromium checks custom radius/spacing, responsive
text and nested dark disabled/invalid alpha colors. Color channels are compared
independently of CSS serialization.

Focused checks reuse completed evidence: 63 distinct native/shared runtime
witnesses, six compiler witnesses, event documentation/consistency, all eight
new/affected receiver agreements and two typing-speed rows. A public-name
regression covers imported native aliases and preserved generic arguments in
completion/hover; diagnostics retain those arguments too. The final inferred-event
witness borrows the field through console.log; its original standalone expression
consumed a borrowed string and correctly failed the callback access contract.
Earlier failures and repairs remain immutable evidence.

Fresh Wasm Chromium observations match the independent Node-after-tsc oracle for
controlled/uncontrolled editing, caret preservation, composition commit, derived
state, exact integers, labels, descriptions, readonly/disabled state and disposal.
A separate fresh entry proves throwing getters, throwing handlers, malformed UTF-16
and independent mount cleanup. A fresh generated package builds offline/locked
and passes native GPU presentation with three resize transitions.
Whole-project formatting verifies token preservation and idempotence. Exact
original logs and source identities use `tsrust-ui-input-*` in canonical SQLite.
No SQL suite, full compiler/LSP matrix or new performance timing is claimed.
The affected generic Signal alias/member checks also pass after the editor fix.

Remaining parity includes specialized input types, Base UI value-change details,
form integration, DOM event methods/refs, composition callbacks, native text-range
accessibility, resize handles, transitions and Label peer/group disabled styling.
Field composition, nullable/dynamic children, keyed lists, overlays, all remaining
components and blocks remain required work. This section does not replace that
roadmap with conditional follow-ups.
