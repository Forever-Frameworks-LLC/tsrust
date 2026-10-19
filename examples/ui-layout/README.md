# Shared layout witness

Run `tsrust run examples/ui-layout/main.tsr`, or build its WebAssembly browser
entry with tools/build-ui-web.mjs. `inspect.tsr` verifies generated CSS through
std:testing without opening a window.

The inline project theme sets spacing.md to 20 logical pixels. Two equal grid
columns share the remaining width after 20px padding/gap. Each left cell is 40px
high; the right cell spans both rows and their gap. Explicit placement preserves
source/accessibility order. The empty boxes are geometry witnesses, not new
components or a complete application design.
