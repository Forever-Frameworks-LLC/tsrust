# Badge and shared text flow

The portable Badge section passes focused acceptance. Source implementation is
`650feede`, with packaging and decoration repairs in `61a0d5bf`. The installed
CLI/LSP use `61a0d5bf`; the later Link-predicate addition is test-only.

## Delivered behavior

Editable offline Badge source provides six Nova palettes, live classes and
nullable variants, a static class helper and once-only replacement rendering.
Undefined selects default. Null removes the palette and variant attribute.
Replacement Link/Button examples keep handlers and labels in application code.
The installer retains the project's configuration and shared attribution.
Compiler API 5 guards the new utilities; prior APIs remain supported.

Shared whitespace modes, local underlines/offsets, larger theme-derived corners
and the bounded `[a]` selector work on both renderers. Parley performs whitespace
processing. Native inline hit ranges and selection use displayed byte offsets;
source strings and editable values remain unchanged. Ancestor decorations keep
their color/offset, and stop at absolute/atomic boxes. This follows the relevant
[CSS decoration propagation rules](https://www.w3.org/TR/css-text-decor-3/#line-decoration).
Font rasterization, glyph metrics and decoration ink skipping are not pixel parity.

The docs app contains 162 destinations, 21 checked examples and 966 API declarations.
The authored Badge gallery includes all six palettes, live/null transitions,
replacement controls, custom corners and a wrapping long label. It uses editable
installed source and source-derived API reference data.

## Focused acceptance

- Four native flow/decoration witnesses pass, including Unicode links/selection,
  all whitespace modes, editable values and independent decoration boundaries.
- Three shared utility witnesses pass, including native Link-only matching,
  browser selectors, class documentation, theme paths and editing exclusions.
- The formatted compiler/native witness passes offline installation, all six
  palettes, null/default behavior, replacement composition and repeat installation.
- The focused LSP witness passes against both the built and installed executable:
  local/inherited props, literal completion, nullable/live values and utility help.
- The actual dev preview matches six independent Node-after-tsc transitions.
  It checks stable nodes, nullable attribute removal, caller padding, replacement
  semantics, keyboard focus rings, 390/768/1280px bounds, accessibility and disposal.
- Both theme modes pass text contrast checks; the minimum is 4.6695268158:1.
  The current-page Link already has the muted background, so its hover assertion
  checks the resolved muted token rather than requiring a color change.
- The actual formatted native docs route builds and stays quiet for five seconds;
  the test process is terminated and reaped. This proves startup, not OS gestures
  or GPU pixel equivalence. Existing accepted layout/resource evidence is reused.
- Whole-project formatting reports zero remaining changes and zero fingerprint
  updates. Native execution checks the formatter round trip. Editor settings bytes
  and modification time are restored exactly; the active editor uses the new image.

The original fixture, packaging and observer failures are retained with their
repairs. No full compiler/LSP matrix, database suite or timed benchmark was run.
The browser and Node witnesses are untimed correctness evidence.

## Remaining work

Badge remains partial relative to full Base UI/Nova: SVG icons and icon-dependent
padding, animated transitions and full event/ref composition are not implemented.
Context, focus/disclosure/overlay foundations, the remaining component families,
charts, theme generation and blocks remain required. Build latency, cold editor
scheduling, HTML-first startup and executable splitting remain open.

The actual port-4317 dev watcher serves the updated gallery at
`/docs/ui/components/badge`. Its initial build reports 135410 ms; this is an
observation, not a controlled performance result. Canonical evidence is stored in
SQLite under `tsrust-badge-*` and the closed earlier preview log. The active preview
log remains loose; closed verified records and disposable executables are removed.

SQLite independently verifies 27 section records (203134 original bytes) plus the
cleanup receipt. Exact bytes, sizes, SHA-256, decompression, integrity and foreign
keys pass. Cleanup removes 32 disposable test outputs totaling 247753204 bytes.
Only the actual dev watcher and active workspace LSP remain from this work. Shared
dependencies and warm native/runtime caches remain available.
