# UI application ownership and offline components

Create a project with `tsrust new counter --ui`. `build.entry` selects the visible
`src/main.tsr` bootstrap; `app/page.tsrx` is the root screen. The bootstrap imports
project JSON and applies tokens with `withTheme`, then calls `run`. Application handlers
and state live in ordinary project source. A CLI update does not rewrite them.

The `ui` object in `tsrust.config.json` has three required relative paths:
`theme`, `components` and `lock`. Defaults are `ui.theme.json`, `components/ui` and
`ui-components.lock.json`. Paths cannot traverse parents or symlinks during
installation. Imports remain ordinary relative paths or configured `paths` aliases.
Changing an installation destination does not silently rewrite application imports.

`ui.theme.json` requires schemaVersion 1; light/dark objects; spacing.sm/md/lg;
radius.sm/md/lg; typography.base/xl. Light and dark each require background,
foreground, card/cardForeground, primary/primaryForeground,
secondary/secondaryForeground, muted/mutedForeground, accent/accentForeground,
destructive/destructiveForeground, border, input and ring. Colors use opaque
sRGB `#RRGGBB` or an alias such as `@foreground` within the same mode. Unknown
fields, missing tokens/references and cycles are errors. Alpha, wide-gamut colors
and other CSS color syntaxes are explicitly unsupported; nothing is silently
clipped. Spacing accepts 0–512 logical pixels; radius 0–256; text sizes 8–128. Dimensions are range-checked before conversion to the shared single-precision rendering format; fine fractions can round during layout.

`withTheme(Page(), theme, 'dark')` defaults to light when
mode is omitted. Each subtree owns its checked theme; mounted trees do not share
state. Both adapters resolve the same classes and values. Browser output includes
scoped token variables and per-node rules; native output uses logical pixels and
sRGB colors. Theme JSON is application data, so theme edits reuse renderer support.
Reactive mode changes without remounting require later string/boolean state work;
this section does not claim that feature.

The local Button exposes typed default/secondary/outline/destructive variants and
sm/default/lg sizes. Card is a styled View. Edit their source directly. Supported
utilities include semantic bg-/text-/border- colors, p-/px-/py-/gap-0/2/4/6,
text-base/xl, font-bold/normal, rounded/rounded-none/sm/lg, border/border-0,
text-left/center/right, items-start/center/stretch, justify-start/center/between and
ring. hover:, focus:, active: and disabled: scope individual rules. Later classes
override earlier classes by property within a state, including padding axes.
State priority is base, hover, focus, active, disabled. Buttons center text, keep
rounded focus outlines and exclude disabled handlers from activation.
`disabled={count.get() < 5}` captures a weak derived condition and reevaluates after
handlers. It becomes enabled at 5 without rerunning or remounting the component.
Reactive buttons require a retained handler even while initially disabled. Predicate
errors use normal mounted-tree failure and cleanup. Direct JSX conditions can use
multiple component signals and ordinary short-circuit operators. Scalar values
computed earlier into ordinary variables remain snapshots; this is not a general
reactive variable system. Generic conditional child insertion/removal remains
separate work.

Unsupported utilities fail; arbitrary CSS, breakpoints, transitions, shadows,
gradients, custom fonts and arbitrary utility values are not implemented.

JSX prop spread consumes its donor record, transferring fields in evaluation order. Ordinary object spread keeps its existing clone semantics.
Read props needed for custom classes before moving props into `<View {...props}
className={classes} />`. The component source uses normal checked calls, never a
special compiler path or implicit callback clone.

## Offline installation

- `tsrust ui list --bundled [--json]` lists embedded exact payloads.
- `tsrust ui init [--path local/controls]` configures an existing non-UI project
  and installs its initial theme.
- `tsrust ui add button@1.0.0 card@1.0.0 [--cwd project] [--path local/controls]`
  resolves the entire embedded dependency closure before writing.
- `--dry-run [--json]` previews without writes. `--offline` and `--yes` are
  accepted for scripts; every installation is offline and noninteractive.
- `--variant default` selects the published source variant. Button's runtime
  visual variants are independent component props.

The binary embeds v1 and v2 component payloads and the exact v1 token dependency.
There are no network calls, npm steps or hooks. Missing versions, incompatible
runtime/compiler API/Rust requirements, dependency conflicts, occupied paths and
unsafe destinations fail preflight. The pinned Rust toolchain must be installed;
installation never downloads it. `ui add` is distinct from Rust-package `add`.

The lock records catalog, immutable payload and template hashes, initially
formatted source hashes, compiler executable SHA-256, compiler API/version,
runtime ABI, pinned Rust version, license text, dependencies and destinations.
Checks and builds reject unsupported installed compiler API/runtime ABI/Rust pins. Compiler executable identity is provenance; compatible compiler builds can retain
installed source. Same-version re-adds preserve customized bytes and the lock
unchanged. They do not repair deleted files. Commit source, theme and lock together;
the lock cannot reproduce your edits. Upgrades into occupied paths are refused;
install into another path and migrate explicitly. Published v1 payloads and their
closure remain bundled. No unlimited future retention policy is implied.

LSP uses compiler component types for props/variants, the shared utility catalog
for class completion/hover, and theme data for definition links. JSON theme help
and compiler diagnostics follow unsaved project buffers. Unsupported static JSX
classes are compiler errors; dynamic class strings are validated on construction.

Native artifacts have no implicit size/count quota. `tsrust cache info` reports
project cache bytes; `tsrust cache clean` explicitly removes reusable artifacts
under namespace locks. Running private executable copies remain valid. A later
build recreates removed support. `TSRUST_CACHE_DIR` selects a shared cache.
