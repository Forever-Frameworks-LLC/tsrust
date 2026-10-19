# UI renderer foundation

The owner selected GPU-first Winit, Taffy and Vello on 2026-10-18. This crate
implements a small owned semantic tree, semantic HTML output, native flex layout,
Parley text shaping, Vello GPU rendering and AccessKit accessibility. Native
resources belong to the event loop. Closing or suspending the window releases its
surface, renderer and adapter. There is no global window registry or worker API.

The Rust example remains a renderer witness. Separately, `examples/ui-static` now
constructs the same tree through checked `.tsrx` and `std:ui`: generated semantic
HTML and a generated native GPU window pass focused acceptance. The paired counter adds integer state, retained synchronous handlers and a bounded
Wasm DOM backend. See the current compiler handoff for focused evidence and limits.

## Current contract

- View accepts owned Nodes and text parts. Text is a text leaf. Button accepts
  noninteractive phrasing children. The low-level Node::new constructor retains its
  strict shape rules; view/button_children normalize mixed content. Depth is at most 128.
- The utility vocabulary is `flex-row`, `flex-col`, `gap-2`, `gap-4`, `p-2`, `p-4`,
  `text-base`, `text-xl`, `font-bold`, `bg-background`, `bg-primary`,
  `text-foreground`, `text-primary-foreground`, and `rounded`.
- Unknown utilities reject; later utilities override earlier properties in each state. Utility
  whitespace is normalized. Units are logical pixels; native presentation scales
  them by the window DPI. Font size and color inherit through containers.
- HTML text, titles and class attributes are escaped. Buttons use semantic disabled
  button elements. Native accessibility exposes labels, disabled roles and bounds.
- Native text uses system fonts and Parley shaping, wrapping and bidi handling.
  Pixel-identical fonts across platforms are not promised. Editing/IME is absent.
- Native rendering uses the GPU without a silent CPU fallback. Surface loss or
  validation failure returns an error. Zero-size and occluded frames are skipped.
- Interactive nodes retain synchronous handlers; weak handles never retain state.
  Components own their integer state until disposal. Async UI and workers remain
  unsupported; ordinary Text supports literal and bound children.

## Development commands

Use the shared Cargo target, `CARGO_INCREMENTAL=0` and `CARGO_BUILD_JOBS=1`.

```sh
cargo run --offline --locked -p tsrust-ui --example static -- --html
cargo run --offline --locked -p tsrust-ui --features native --example static
cargo test --offline --locked -p tsrust-ui --features native
```

The acceptance-only feature exposes a bounded window witness:

```sh
cargo run --offline --locked -p tsrust-ui --features acceptance --example static -- --verify-window
```

It opens a real window, presents a GPU frame, drops the window resources and exits.
It is not a screenshot, pixel oracle, screen-reader session or input test. An
external process timeout is still required to bound driver initialization.

## Evidence

On macOS, the native dependency/API check, three focused semantic/layout tests and
real GPU window witness pass. The tests cover escaping, invalid shape/utilities,
depth, Unicode/multiline labels, disabled semantics and scaled accessibility bounds.
Original WGPU API mismatches and repaired results are retained in SQLite under
`tsrust-ui-*`. Existing locked package versions were retained. No SQL, complete
compiler/LSP regressions or performance timing was rerun. Generated static `.tsrx` HTML and native presentation are verified separately.
Browser pointer/keyboard/disposal and native keyboard/AccessKit action witnesses
pass for the counter. Pixel comparison, OS input automation and screen-reader
sessions remain unverified.

Buttons center text within their padding on both renderers. The native focus
outline follows the button's rounded geometry. Native layout uses the current
logical viewport width; resizing repaints from that size during the resize event,
without changing font sizes or accumulated scale factors.

## Element semantics

`std:ui` exposes exact JSX `id`, `aria-label`, `aria-hidden`, `data-slot`, `role`
and `aria-orientation` properties. Ordinary records declare quoted property names
and read them with bracket access. The compiler keeps those keys distinct from
camel-case names and dotted paths; editor completion inserts valid bracket access.

Text can select `span`, `div`, `p`, `h1`–`h6`, `code` or `kbd`; View selects `div`
or `section`. Headings retain their levels in AccessKit. Class names determine
appearance rather than browser heading defaults. Supported noninteractive roles
are alert, status, separator, group, note, region, none and presentation. Alert
and status expose assertive/polite live semantics. Separators must be empty and
are nonfocusable, with horizontal orientation by default. A region needs a
nonblank accessible name. Button retains its implicit interactive role.

Hidden accessible subtrees cannot contain enabled controls. A reactive change
that would enable a hidden control fails refresh and uses normal mounted-tree
cleanup. Semantic validation consumes failed subtrees and releases their captures.
Arbitrary HTML, event attributes, links, editable values, ARIA relationships and
interactive role emulation remain unsupported. These primitives are prerequisites
for the selected Base UI/Nova source port, not full component parity.

Accessible names require a button, heading, named section or supported naming role.
A nonempty `aria-label` on generic text/containers, paragraphs, code or presentation
throws instead of producing different native and browser semantics. Native paragraph,
code and live-role containers preserve their text as accessible content children.
See [ARIA naming rules](https://www.w3.org/TR/wai-aria-1.2/#namefromprohibited).

## Composed content

View and button factories also accept owned Child values (Nodes or text parts).
Adjacent text parts share a text node, preserving weak signal bindings; empty
literals do not create anonymous boxes. Button descendants must be noninteractive
phrasing content. Span layout containers support composed button content on both
backends. Native text alignment inherits into children; both renderers center
button content. A missing handler is a normal no-op control, with no browser click
listener allocated for it. This does not add dynamic child reconciliation, arbitrary
rich text or general DOM event objects.
