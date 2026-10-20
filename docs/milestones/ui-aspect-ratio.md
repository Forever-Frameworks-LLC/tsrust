# Shared AspectRatio layout and editable component

Implementation checkpoints: `c06a6a3b` and `438444bf`. Focused compiler, installer,
editor, real-browser and actual native documentation-route acceptance passes.

The pinned Base UI/Nova source requires `ratio`; there is no default. The editable
local component forwards ordinary View metadata and owned children, fills the
available width and accepts caller class overrides. Shared utilities include
`relative`, `aspect-auto`, `aspect-square`, `aspect-video`, and positive finite
arbitrary ratios. Ratios normalize to the native f32 layout domain on both targets;
zero, negative, nonfinite and underflow/overflow inputs reject explicitly.

Native preferred dimensions resolve before ordinary clamping. This avoids a Taffy
leaf aspect-ratio floor overriding explicit/max heights and avoids transferring a
max-height limit into an otherwise unconstrained definite width. Empty and populated
boxes pass seven geometry cases in root, flex and grid contexts at scales 1, 1.5 and
2. An independent Node-after-tsc oracle agrees with the seven recorded dimensions.
These are untimed correctness observations.

The portable component remains partial: ratio is a construction-time prop, with no
reactive update contract yet. Intrinsic-width cross-axis sizing, media loading and
fitting, absolute/sticky offsets and container queries are unsupported. These are
remaining library requirements rather than hidden fallbacks or parity claims.

`std:testing.throws` consumes one synchronous, zero-argument callback returning void.
It accepts any thrown value, discards the value without requiring its error type,
and fails outside the catch when the callback returns normally. It does not await
async work or match error contents. Callback captures retain checked ownership and
ordinary exception cleanup. Its source example, signature, hover and parameter
help are maintained together.

The optional browser JavaScript bridge remains deferred. Its separate plan requires
per-export compiler/LSP availability across every configured target, including
unsaved implementation and target changes. No Node execution, network request or
Cargo build is allowed in those editor queries. Core UI components remain shared
compiled code; native and embedded do not acquire a JavaScript runtime.

## Focused acceptance

Five new compiler/editor witnesses pass: formatted documented assertion execution,
consuming callback ownership and independent resource release, native component
construction and invalid dynamic ratios, offline installation/idempotence, required
prop diagnostics and utility help, and throwing-assertion hover/signature help.
An initial import-bearing fixture incorrectly used the standalone checker; the
project CLI witnesses replace it. The assertion catches without binding its
intentionally discarded unknown failure value. Original failures remain retained.

The actual live documentation route passes seven geometry comparisons against the
independent Node-after-tsc oracle, exact highlighted/copyable source, retained
preview switching, no extra focus/role, 480/768/1440px overflow checks, theme changes
and disposal. Native editing/theme/accessibility and the six existing component
galleries reuse their accepted evidence. These checks add no benchmark timing,
OS gesture or native pixel claims.

Installed CLI/LSP consumers are refreshed from `438444bf`. The active editor maps
the new inode and settings are restored byte-for-byte with their original mtime.
The real dev supervisor retains the accepted publication during repair and now
serves the expanded app without an error; its observed initial build is 16,864 ms
with warm dependencies. This is an operational observation, not a controlled
performance measurement or proof of persistent frontend caching.

The actual native AspectRatio documentation route builds, remains alive for five
seconds, emits no output/errors and is reaped. Its generated executable is
18,307,432 bytes with SHA-256
`fd88be2e7958949bac02cd4cd2dea979d253bac9a19f2f7f3eb6cbcb3bf771a0`.
The native geometry witness covers 126 scenes (seven cases, three container
contexts, empty/populated children and three scales). No extra component role or
focus stop is introduced.

Canonical evidence uses `tsrust-aspect-*` paths in `benchmarks/records.sqlite`,
including failed attempts and repairs. Closed records are independently restored,
matched by exact bytes, size and SHA-256, then removed from loose storage. Current
compiler/editor consumers, live dev preview and warm dependency caches remain.
No full compiler/LSP matrix, SQL suite or timed benchmark run is due for this section.
