# Rust/GPUI and Tsrust, one app built both ways

The same small application written twice: `rust/` with GPUI Kit directly and
`tsrust/` with `std:ui`. Both use `gpui-kit` 0.9.0 with Tsrust's patched
`gpui-component`/`gpui-base`, the same locked dependency versions and Tsrust's
GPUI build profile. Each has three source files (main, row, list model).

The app: a header with the visible count and a frame number, a search Input, an
"Add item" Button, and a scroll area with 2,000 rows (label plus a muted
"N chars" column). Neither version virtualizes the list.

`BENCH_MODE` selects an automated run in both:

- `startup`: quit on the first 16 ms tick after the window opens.
- `idle`: quit after 10 seconds.
- `animate`: for 600 ticks every 16 ms, set the query to the next digit (the
  list refilters) and the frame counter.

## Running

```bash
(cd rust && cargo build --release --offline)
(cd tsrust && tsrust build)
python3 measure.py rust/target/release/gpui-compare tsrust/target/gpui-compare 5
python3 edit_loop.py "$(command -v tsrust)" 5
```

`measure.py` alternates engines, activates each window the same way (neither
app activates itself) and reports median wall time, CPU time and peak RSS.
`edit_loop.py` changes the row's muted label to a new string per edit: Rust with
`cargo build`/`cargo check` (dev profile, incremental), Tsrust with one
`tsrust dev --verbose` session and `tsrust check`.

## Results (2026-11-17, MacBook Air M3, 8 GB)

| | Rust/GPUI | Tsrust | Ratio |
|---|---|---|---|
| Release binary | 23.8 MiB | 45.0 MiB | 1.9x |
| Startup wall (n=5) | 0.33 s | 0.53 s | 1.6x |
| Startup CPU | 0.24 s | 0.44 s | 1.8x |
| Idle 10 s, CPU (includes startup) | 0.29 s | 0.51 s | |
| Peak RSS | 159-161 MiB | 203-204 MiB | +44 MiB |
| Animate 600 ticks, wall (ideal 9.6 s) | 14.0 s | 21.5 s | 1.5x |
| Animate 600 ticks, CPU | 10.1 s | 20.5 s | 2.0x |
| Rebuild after an edit (dev, n=5) | 0.81 s | 1.20 s | 1.5x |
| Check after an edit (n=5) | 0.35 s | 0.40 s | 1.1x |
| Clean release build | about 107 s | 163 s | |
| Clean dev build | 143 s | (runtime cache was warm) | |

A Tsrust rebuild after an edit: check 0.37 s, emit 0.06 s, rustc for the two
changed crates 0.13 s, program crate and link 0.45 s. The generated program is
1.4 MB of Rust in six crates (mostly the `std:ui` module and the types crate)
for 70 lines of source.

## What the profiles show

Animate mode, 5-second `sample` of the main thread:

- **Both**: most of each frame is GPUI text layout. Every changed line goes
  through `CTLineCreateWithAttributedString`, and CoreText rebuilds the font's
  ASCII advance cache (`TFont::InitASCIIDataCache`, variable-font deltas) for
  each line, which suggests GPUI creates a new `CTFont` per shaped line
  (`CTFontCreateCopyWithAttributes`). A fix in the patched GPUI would speed up
  both versions.
- **Tsrust only, about 26% of samples**: the tick handler's refresh. The `each`
  binding over the filtered list rebuilds every row's Node when the list
  changes (`Row` runs again for all visible rows) instead of keeping rows whose
  item is unchanged.
- **Tsrust only, about 19%**: `Root::render`, mostly `collect_styles`, which
  resolves every node's utility classes again on every frame, plus building
  elements from Nodes.
- `computed` holds only scalars, so the filtered list cannot be shared between
  the count and the rows; the Tsrust version filters twice per change.

## Defects found

- `quit()` and other UI commands ran only during a render. A window that was
  covered never rendered, so it never quit, and each pending tick kept
  refreshing the tree. Fixed: commands now also run right after the event that
  queued them.
- Tsrust applications never activate their window, so a program launched from
  a terminal opens behind other windows. Left as is: `tsrust dev` relaunches
  would otherwise take focus from the editor on every save.

## Takeaways

- For a small app the Tsrust edit loop is close to Rust's (+0.4 s per rebuild).
  Studio's longer loop comes from its size: whole-program check passes and
  large generated crates.
- Runtime cost is about 2x CPU per update in this list-heavy case, coming from
  rebuilding rows and recomputing styles, not from GPUI. Keyed row reuse in
  `each` and caching resolved styles per node until its classes change are the
  two largest levers.
- The binary is about 2x larger and memory about 44 MiB higher; startup about
  0.2 s slower.
