# Alert composition and parent positioning

Implementation in progress; not accepted. Continue after the accepted reading
section at `cae9256b`. No additional owner approval is needed.

- [x] Shared relative/absolute positioning and bounded independent insets.
- [x] Each UI box is a containing block on both renderers; inline text rejects box utilities.
- [x] Checked `data-variant` metadata and data/group selectors, including editor documentation.
- [x] New immutable Alert item: Alert, AlertTitle, AlertDescription and AlertAction.
- [x] Compiler API 3 prerequisite; older installed items remain supported.
- [x] Native geometry, metadata/merge, offline installer and editor witnesses are written.
- [x] Source examples for variants, retry/hide state, responsive actions, validation,
  scoped palettes and parent positioning are written.
- [ ] Build/refresh SDK consumers, install Alert into docs through the actual offline CLI.
- [ ] Register examples/API source, author the Alert and positioning pages, browser oracle.
- [ ] Batch targeted compiler/native/editor/browser acceptance after the section is complete.
- [ ] Refresh actual preview/editor, inspect open diagnostics, import/verify SQLite evidence.

The pinned Base UI/Nova source and CSS remain canonical evidence in SQLite.
SVG/icon handling, arbitrary descendant CSS, generic reactive variant props,
fixed/sticky positioning, transforms and z-index are outside this bounded section.
Keep Alert partial until its remaining clone capabilities are implemented. The
new component must not hide application retry, networking or authorization policy.

The docs component file is intentionally installed by the new bundled CLI rather
than manually fabricating an installation lock. That SDK bootstrap is an
implementation prerequisite, not an acceptance run for an incomplete section.
