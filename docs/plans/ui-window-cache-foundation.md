# Native viewport and persistent dependency foundation

Owner priority: repair the standalone counter's startup, missing scrolling and
resized multiword labels, then develop it into reusable component documentation.
The documentation application needs navigation, preview/code cards, detailed API
examples, light/dark mode and configurable native window chrome with working
controls and drag regions. These remain required implementation, not conditional
application work. The component library remains incomplete.

A typography/resize witness reproduces a one-line measured label painting as two
lines after Taffy rounds its width down. Vello/Parley layout now keeps fractional
logical coordinates. Native window scrolling shares offsets across paint, hit
regions, accessibility and text input/IME. Wheel/trackpad motion first scrolls a
text editor and then the page. Keyboard focus reveals its control. Native scroll
accessibility exposes bounds, offsets and actions. Nested scroll components and
scrollbar controls remain pending.

A packed runtime cache already existed, but cache misses materialized a new Cargo
target and source/vendor directory in temporary storage. A stable locked workspace
now preserves Cargo fingerprints and dependency artifacts across runtime source
revisions. Exact backend/bundle/profile identity and Cargo's own feature checks
separate incompatible builds. Dev falls back to private storage when the shared
workspace is busy. Existing resolved binding overlays retain their private build
protocol. Explicit cache info/clean includes workspaces and keeps lock inodes.

Focused native renderer acceptance passes 31 cases. The dependency witness changes
the runtime implementation, verifies unchanged serde artifact path/modification
time, and executes both newly linked results. Workspace serialization/identity and
explicit clean with an unrelated symlink target pass. Standalone startup/GPU,
consumer refresh, remaining content/editor checks and final acceptance remain
pending. No full regression, SQL suite or throughput benchmark was repeated.
