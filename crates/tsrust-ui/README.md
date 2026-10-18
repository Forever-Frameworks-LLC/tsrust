# Static UI renderer foundation

The owner selected GPU-first Winit, Taffy and Vello on 2026-10-18. This crate
implements a small owned semantic tree, semantic HTML output, native flex layout,
Parley text shaping, Vello GPU rendering and AccessKit accessibility. Native
resources belong to the event loop. Closing or suspending the window releases its
surface, renderer and adapter. There is no global window registry or worker API.

The Rust example remains a renderer witness. Separately, `examples/ui-static` now
constructs the same tree through checked `.tsrx` and `std:ui`: generated semantic
HTML and a generated native GPU window pass focused acceptance. Interactive state,
events and browser execution remain open. See the current compiler handoff.

## Current contract

- `View` owns child nodes. `Text` and `DisabledButton` own text and reject child
  nodes. View text must be wrapped in Text. Maximum depth is 128 nodes.
- The utility vocabulary is `flex-row`, `flex-col`, `gap-2`, `gap-4`, `p-2`, `p-4`,
  `text-base`, `text-xl`, `font-bold`, `bg-background`, `bg-primary`,
  `text-foreground`, `text-primary-foreground`, and `rounded`.
- Unknown utilities and conflicting utilities of one category reject. Utility
  whitespace is normalized. Units are logical pixels; native presentation scales
  them by the window DPI. Font size and color inherit through containers.
- HTML text, titles and class attributes are escaped. Buttons use semantic disabled
  button elements. Native accessibility exposes labels, disabled roles and bounds.
- Native text uses system fonts and Parley shaping, wrapping and bidi handling.
  Pixel-identical fonts across platforms are not promised. Editing/IME is absent.
- Native rendering uses the GPU without a silent CPU fallback. Surface loss or
  validation failure returns an error. Zero-size and occluded frames are skipped.
- The static scene has no executable callbacks, request authority or asynchronous
  state. Existing checked owner/handle contracts remain the future state boundary.

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
Paired browser/native visual comparison and interactive accessibility remain open.
