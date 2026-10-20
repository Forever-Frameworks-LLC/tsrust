# Reactive keycaps and bounded Markdown frames

Kbd 2 keeps `Kbd`, `KbdGroup`, their keyboard tags and default slots from the pinned
shadcn Base UI + Nova source. The baseline is commit
`daaf156ee00ed91cc7e6bdd937c1b4973152d141`, Kbd blob
`1fc8d9c188183e3df985d9a1f454ca692186790f`. Caller slot overrides and reactive
classes forward through both exports. Unspecified SVG sizes receive `size-3`;
explicit sizes keep precedence. Offline upgrades preserve existing source/imports
and install `kbd-v2.tsrx`; shared attribution remains outside editable component
bodies. The implementation checkpoint is `dae8cb59`.

The paired reference has shortcut composition, live size/palette/gap overrides,
explicit/default SVG sizes, scoped numeric spacing/radius/font utilities and an
independent theme toggle. API metadata now describes the current installed source.
Keycaps do not install shortcut handlers. Paragraph-inline layout boxes, arbitrary
DOM events/refs, full Base UI composition, a keycap body-font theme field and an
implemented Tooltip remain unsupported. Naming requires a compatible role; an
icon-only SVG can have its own accessible name.

## Markdown repair

Opening the expanded property table exposed a Wasm memory trap. The recursive
Markdown dispatcher retained a 173,216-byte frame for every nesting level. JSX
construction now belongs to nonrecursive helpers; recursive block and inline
traversal retain only their dispatch state. The block dispatch frame is now
17,040 bytes. Existing stack limits, semantic validation, source order, example
placement, child transfer and cleanup remain intact.

The actual Kbd route now mounts its nested table and three examples. A generated
native witness renders the same Markdown and all three examples with assertions
for table headings/cells, content, semantics and example presence. Its registration
also verifies formatting and idempotence for the affected source files.

## Focused acceptance

- Offline current-version selection, upgrade preservation, attribution and
  idempotent formatted installation pass.
- The generated native keycap witness verifies caller slots, an explicitly named
  group, SVG child preservation and independent capture release exactly once.
  Native layout/style evidence for the shared utilities is reused. CSS text is
  not treated as a concrete SVG geometry oracle.
- Installed-LSP completion/hover accepts reactive props, rejects unknown props and
  retains zero stale/member counters.
- Actual browser clicks preserve keycap/group identities while changing height
  from 20 to 32 pixels and group gap from 4 to 8 pixels. Source/preview switching
  preserves that state. The copied 2,742 bytes match the project source exactly
  on the macOS clipboard; the browser automation clipboard reader returned empty.
- Scoped light/dark switching preserves identities and stays independent of the
  outer theme. Custom spacing produces 15-pixel default icons and 30-pixel explicit
  icons; radius is 8 pixels and the caller's generic mono family applies.
- Measured keycap text contrast is at least 5.81:1 in the checked combinations,
  including composited ancestor backgrounds. At 390 and 1280 pixels there is no
  document overflow or clipped keycap. Existing navigation breakpoint evidence is
  reused. No console errors occur after the repaired publication.

Original diagnostics and failed witnesses remain retained with the successful
repairs. The real dev watcher stays at port 4317; current public component source
is read by the refreshed LSP without another server binary change. Existing native
OS-window startup evidence is reused; no new OS drag/clipboard gesture acceptance,
full regression matrix, SQL suite or runtime benchmark pass is implied.

SQLite stores the pinned references, focused logs, geometry/contrast observations,
clipboard hashes, measured Wasm frames and closure receipts. Closed loose records
are removed after independent byte/hash and database integrity verification.
