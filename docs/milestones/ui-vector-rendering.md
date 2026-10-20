# Shared vector rendering

Implementation source is `b79807d3`; editor recovery and example repairs are
`a3525f16`. The installed CLI uses the former compiler source; the installed and
active LSP include the latter repair. Paired focused acceptance and canonical evidence cleanup pass.

## Delivered behavior

`std:ui` exports Svg, Path, SvgProps and PathProps. Owned Path children render
through browser SVG namespaces and native Vello geometry. The same checked path
model preserves source strings, resolves viewport alignment/aspect policy, clips
by default, inherits theme paint and supports explicit caps, joins and fill rules.
Live `d` expressions update retained geometry without replacing path identity.
Invalid updates retain the last valid data; disposal releases their evaluators.

Unlabelled icons default to decorative presentation. Meaningful images require a
label; descriptions can refer to current text data. Native paths add no child
accessibility stops. The docs page includes local button icons, a six-value live
chart with a text alternative and four viewport policies. This is a drawing
foundation, not a completed chart component or interaction system.

Strict parsing uses svgtypes 0.16.1 with existing Kurbo 0.13.1. The initial Kurbo
parser accepted invalid trailing input; the original failed witnesses are retained.
Limits and unsupported cases are explicit in the public reference and authored
[guide](../content/ui/foundations/vector-graphics.md): auto/fit sizing, arbitrary SVG,
nested SVG, gradients, filters, masks, path interaction and non-scaling strokes are
not implemented. Nonzero arc radii at or below 0.00001 units reject instead of
silently becoming lines. Zero radii follow SVG's line behavior.

## Focused acceptance

- Six runtime/native vector cases and the added strict-parser edge case pass.
  They cover malformed/nonfinite input, curves, relative/implicit commands,
  alignment, native scene encoding, image descriptions, identity and release.
- Five actual native viewport results match independent Node-after-tsc results.
  These are untimed correctness checks.
- The formatted generated-native contract fixture and documented Svg example
  pass. The explicit unlabelled-image case requests role=img; default decorative
  presentation is a valid distinct case.
- Source-derived editor completion/hover, two receiver agreement rows, five
  vector scenarios in four typing modes, consistency counters and the installed
  LSP witness pass. Existing condition and semantic-attribute rows also pass.
- Incomplete JSX now materializes imported generic parameter fields. Inherited
  generic aliases retain complete arguments and substitute renamed parameters;
  quoted metadata fields no longer disappear while another function is invalid.
- Documentation generation reports 163 destinations, 24 checked examples and
  970 API declarations. The updated application checks without diagnostics.
- The actual native docs route builds and remains quiet for five seconds before
  termination and reaping. This proves bounded startup, not OS gestures or
  cross-platform GPU pixel equivalence.
- Global/editor LSP images are byte-identical. Editor settings bytes and
  modification time are restored exactly. Cold whole-repository scheduling is
  still an open concern.

The actual port-4317 dev route passes independent three-step chart and four-policy
viewport checks, stable path identity, button focus, image/button accessibility,
390/768/1280px bounds, theme stroke changes and disposal. The observer records exact
source hashes. The preview initially reports 158461 ms while warming the shared
Wasm target; this is an observation, not a controlled performance result.
The native startup process and browser are reaped; the actual dev watcher remains.
No full compiler/LSP regression, SQL suite or timed benchmark is implied.

## Remaining library work

Context/lazy children, event/ref composition and focus/disclosure/overlays remain
required foundations. Continue the pinned component inventory, charts, live theme
generation and blocks. Existing Badge source still lacks automatic icon sizing
and padding, transitions and complete event/ref composition. Build latency,
HTML-first startup, executable splitting and cold editor scheduling remain open.

SQLite independently verifies 42 closed section records (284018 original bytes),
plus receipts and cleanup helpers. Exact decompression, bytes, sizes, SHA-256,
integrity and foreign keys pass. The closed earlier preview log is included;
only the active vector preview log remains loose. Cleanup removes 156 disposable
outputs containing 28130 files and 2778752550 bytes, including obsolete test
objects, superseded compiler artifacts and the replaced web workspace/duplicate
Cargo target. Current consumers, warm runtime caches and owner files remain.
