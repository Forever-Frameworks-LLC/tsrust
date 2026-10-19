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


The first bounded standalone preparation timed out after 240 seconds with one
Cargo job. Resuming its persistent workspace completed in 98,987 ms; an unchanged
restart completed in 5,778 ms with runtime reuse but an unexpected executable-cache
miss. These are diagnostic observations, not a statistical benchmark.

Exact cache identity comparison isolates that miss to generated Rust: source graph,
toolchain, features and flags were identical. Callback error-layout sharing compared
raw failure-discovery order instead of canonical native payload alternatives. Layout
keys now use the existing checked canonical payloads, preserving error contracts,
injection, ownership and cleanup. A focused witness reverses discovery order and
requires identical Rust, then executes both thrown payload cases. Four fresh UI
compiler processes also emit byte-identical Rust. Both focused witnesses pass;
no full compiler/LSP or service/SQL suite was rerun. Installed refresh and repeated
standalone startup observations follow this source checkpoint.
