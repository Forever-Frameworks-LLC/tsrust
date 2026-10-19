# UI application ownership and offline components

Create a project with `tsrust new counter --ui --target native`. `build.entry` selects the visible
`src/main.tsr` bootstrap; `src/app/page.tsrx` is the root screen. The bootstrap imports
`tsrust.config.json` and applies `config.ui.theme` with `withTheme`, then calls `run`. Application handlers
and state live in ordinary project source. A CLI update does not rewrite them.

The `ui` object in `tsrust.config.json` owns `theme` data, the `components`
directory (default `src/components/ui`), explicit `target` choices and `lock` path (default
`ui-components.lock.json`). Source stays under `src/`: `src/main.tsr` is the
executable bootstrap and `src/app/page.tsrx` is the screen convention. Imports are
ordinary relative paths or configured aliases. For example, `src/main.tsr` imports
config from `../tsrust.config.json` and Page from `./app/page`, then calls
`withTheme(Page(), config.ui.theme)`. Custom destinations do not rewrite imports.

Legacy projects can still use a relative JSON file path for `ui.theme`; existing
source and installation records keep working. New projects embed the data.
Changing an existing project's layout requires moving its source and updating
ordinary imports; installation never moves or overwrites application files.

The inline theme requires schemaVersion 1; light/dark objects; spacing.sm/md/lg;
radius.sm/md/lg; typography.base/xl. Light and dark each require background,
foreground, card/cardForeground, primary/primaryForeground,
secondary/secondaryForeground, muted/mutedForeground, accent/accentForeground,
destructive/destructiveForeground, border, input and ring. Colors accept `#RRGGBB`, `#RRGGBBAA`, checked in-gamut `oklch()`
and aliases such as `@foreground` or `@card-foreground` within the same mode.
Unknown fields, missing tokens/references, cycles and unsupported gamut are errors.
OKLCH lightness/alpha accept 0–1 or 0–100%; chroma is nonnegative and unitless;
hue is a unitless 0–360 degree value. Other functional syntaxes, hue/chroma units,
missing components and out-of-range values are unsupported. The resolved background
must be opaque: transparent native windows are not implemented. Functional colors
use renderer float precision; native paint encodes premultiplied 8-bit sRGB, so
small pixel rounding differences remain possible. Source theme strings stay unchanged. Spacing accepts 0–512 logical pixels; radius 0–256; text sizes 8–128. Dimensions are range-checked before conversion to the shared single-precision rendering format; fine fractions can round during layout.

`withTheme(Page(), config.ui.theme, 'dark')` defaults to light when
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
Handlers are optional, including on reactively disabled buttons. Predicate
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
- `tsrust ui init [--path src/local/controls]` prepares missing UI settings.
  Existing settings remain; repeated initialization is safe.
- `tsrust ui add button@1.0.0 card@1.0.0 --target native [--cwd project] [--path src/local/controls]`
  resolves the entire embedded dependency closure before writing.
- `--dry-run [--json]` previews without writes. `--offline` and `--yes` are
  accepted for scripts. Installation is always offline; missing target choices
  prompt only in a terminal without --yes or --dry-run.
- `--variant default` selects the published source variant. Button's runtime
  visual variants are independent component props.

The first `ui add` needs no preceding `init`. It creates missing configuration,
inline tokens, `src/app`, the configured component directory and an exact lock.
In an empty directory it also creates a visible bootstrap and root page. Existing
entries, output paths, aliases, complete themes and application source stay intact.
Missing UI fields are filled individually; malformed values are errors. The CLI
prints setup locations and explains theme application and component imports.
A nested working directory uses the nearest project root. `--dry-run` writes nothing.

Imports never create setup. Build/run/dev detect missing UI configuration or
recognized bundled component paths and show the exact `tsrust ui add` command.
They do not install files. `check`, `emit`, formatting and LSP also remain
read-only. Custom imports are not guessed or downloaded. Theme application stays
explicit in the bootstrap; adding UI does not rewrite an existing main function.

`ui.target` contains booleans, for example:

```json
"target": { "native": false, "web": true, "embedded": false }
```

Enable native, web or both; at least one must be enabled. Embedded must remain
false because its backend and hardware profiles are not implemented. Unknown
fields, nonbooleans and an all-disabled target object are errors. Legacy configs
without the object remain readable, but UI deployment requests explicit setup.
New projects and first additions select targets interactively in a terminal.
Pass repeatable `--target native` / `--target web` flags for scripts or previews;
flags enable targets without disabling existing selections. Edit booleans in the
config to disable a target. Existing selections do not prompt again. Missing
choices in scripts, `--yes` or `--dry-run` fail before writes.

Native build/run/dev reject a window entry when native is disabled. Web output
uses the existing bounded Wasm builder, which requires web=true. A general web
dev server and embedded deployment are not implemented; selecting web never
launches a native window as a fallback. HTML serialization remains usable in
project tests independently of the selected deployment backend.

Writes preflight source/config/lock collisions, traversal, symlinks, versions and
compatibility. All bytes stage before publication. Configuration and the lock
publish with source under one installation lock; failures restore unchanged
published bytes and remove owned staging files. Concurrent owner edits are retained
and reported rather than overwritten. This is a filesystem transaction for normal
errors, not a crash-proof journal or a defense against hostile concurrent directory
replacement.

The binary embeds v1 and v2 component payloads and the exact v1 token dependency.
There are no network calls, npm steps or hooks. Missing versions, incompatible
runtime/compiler API/Rust requirements, dependency conflicts, occupied paths and
unsafe destinations fail preflight. The pinned Rust toolchain must be installed;
installation never downloads it. `ui add` is distinct from Rust-package `add`.

For embedded tokens, a lock file entry names `tsrust.config.json` and JSON pointer
`/ui/theme`. Hashes describe the initially installed data; later edits stay owned.
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

## Semantic local components

Editable components use `std:ui` primitives on both backends. Exact JSX attribute
names are preserved, including `aria-label`, `aria-hidden` and `data-slot`:

```tsrust
<View as="section" role="region" aria-label="Account">
  <Text as="h2" className="text-xl font-bold">Profile</Text>
  <Text role="status">Saved</Text>
  <View role="separator" aria-orientation="horizontal" />
</View>
```

Use quoted keys in ordinary props records, for example
`{'aria-label': 'Account'}` and `props['aria-label']`. Heading tags carry accessible
levels without introducing browser-only font sizes or margins. Text accepts span,
div, p, h1–h6, code and kbd; View accepts div or section. A named region requires a
nonblank accessible label. Alert is assertive; status is polite. A separator must
be empty, is not focusable, and defaults to horizontal orientation. Class names
still determine its dimensions. Hidden decoration remains visible but is excluded
from accessibility; enabled controls inside a hidden subtree are rejected, even
when a reactive update would enable them later.

Only supported semantic roles are accepted. This API does not expose arbitrary
HTML, editable controls, ARIA relationships or simulated interactive roles.
See `examples/ui-semantics` for native and browser entries. The selected component
baseline is current Base UI + Nova, pinned in
[the coverage inventory](../plans/ui-component-coverage.json). Existing offline
Button/Card v1/v2 are retained; they do not yet claim Nova parity. Pending inventory
entries are not installable placeholders.

Accessible names require a button, heading, named section or supported naming role.
A nonempty `aria-label` on generic text/containers, paragraphs, code or presentation
throws instead of producing different native and browser semantics. Native paragraph,
code and live-role containers preserve their text as accessible content children.
See [ARIA naming rules](https://www.w3.org/TR/wai-aria-1.2/#namefromprohibited).

## Composed children

View and Button accept ordered strings, direct signals and owned component Nodes.
Adjacent literal/bound text shares one text node. Empty literals are omitted;
explicit Nodes keep their position and ownership. Text remains a text leaf accepting
strings/signals. View as=span creates a layout box with a span DOM tag; class names
still select its layout. Font size, color and text alignment inherit.

```tsrust
<Button onPress={() => { count.set(count.get() + 1); }}>
  <View as="span" className="flex-row gap-2">
    <Text as="kbd" aria-hidden>+</Text>Count: {count}
  </View>
</Button>
```

Buttons require noninteractive phrasing descendants: spans, code, kbd and text.
Flow elements, nested controls and nonpresentational roles fail instead of changing
meaning. Hidden decoration does not contribute to the button's accessible name.
An omitted onPress creates a normal focusable button with no application action;
it does not require a dummy callback. Reactive disabled predicates still refresh
after other handlers. Disabled controls remain excluded from activation/focus.

Nodes cannot be reused in a second parent, cloned or sent to workers.
Existing owned `string[]` and `Node[]` values can move into `Child[]`, including
optional child arrays forwarded by older components. Each element moves once;
resources cannot be cloned implicitly. Borrowed array widening is rejected: form
the wider owned array first, then borrow it if needed.

Only direct Signal children bind updates. Computed strings/trees are construction-time
snapshots; numeric/boolean/null children and dynamic conditional child trees are
not provided as reactive rendering. See examples/ui-composition.

## Shared layout

Container display/track/wrap utilities require View or a composed Button; Text
remains a text leaf and rejects them. Item sizing, padding and alignment still
apply to text leaves. `View` defaults to a flex column. `flex`, `grid`, `flex-row`, `flex-col`,
`flex-wrap` and `flex-nowrap` select layout. Grid columns/rows accept counts 1–12,
`none`, `grid-cols-[1fr_auto]` and `grid-rows-[auto_auto]`. Numeric tracks divide
space equally with zero minimum. `auto-rows-min` sizes implicit rows to minimum
content. Child `col-start`/`row-start` use one-based lines 1–13; spans accept 1–12.
Source, keyboard and accessibility order remain unchanged by visual placement.

`p`, `px`, `py`, `pt`, `pr`, `pb`, `pl`, `gap`, `gap-x` and `gap-y` accept
0, px, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64 and 96. Sizes 2/4/6 preserve
spacing.sm/md/lg. Size 1 is sm/2; size 3 averages sm/md. Other numeric sizes use
spacing.md/4 per unit. px means exactly one logical pixel. Combined utilities
expand into independent sides/axes: `pl-6 p-4 pr-2` finishes with md left/top/bottom
and sm right. Interaction states have their own overrides.

`w`, `h`, `min-w`, `min-h`, `max-w`, `max-h` and `size` accept that scale, full and
fractions 1/2, 1/3, 2/3, 1/4 and 3/4. size changes both preferred dimensions.
Auto is supported except for maxima, where none removes the maximum. Min/max
constraints still apply. Percentages use the parent content box; percentage height
needs a definite parent height and otherwise behaves as auto. Root preferred and
maximum widths are honored on native and web.

`self-*` and `justify-self-*` accept start/center/end/stretch/auto. self controls the
flex cross/grid block axis; justify-self controls grid inline alignment. grow/shrink
use factor 1; grow-0/shrink-0 use 0. Wrapping and factors respect min/max sizes.
Defaults match flex column containers with no explicit tracks/dimensions.

Compiler checks, completion, hover, theme navigation, native Taffy and browser CSS
share these rules. Unknown utilities fail explicitly. Arbitrary CSS, responsive
breakpoints, named grid lines, subgrid, scrolling and general track expressions
are unsupported. No layout utility duplicates or retains a child's owner. See
examples/ui-layout for the paired custom-theme grid witness.


## Alpha and surface rings

`bg-`, `text-`, `border-` and `ring-` semantic colors accept `/N` to multiply the
source alpha by an integer percentage 0–100. For example an 80% primary color
with `bg-primary/50` paints at 40% alpha. Invalid/fractional percentages fail;
common percentages are suggested after slash. transparent is zero-alpha black.
Color alpha does not remove content from accessibility.

`ring-0`, `ring-1`, `ring-2` and `ring-3` set an outer surface ring width in logical
pixels. `ring-foreground/10` selects its color; color alone does not create a ring.
The default ring color uses the enclosing theme's ring token. Each node resets
ring variables to its own defaults; a parent's surface ring color does not leak
into a child's ring. Surface rings leave layout size unchanged. The legacy `ring`
utility remains a 2px outline with 2px offset, separate from surface rings and
applicable whenever its state matches. Native buttons keep their default focus
outline. See examples/ui-colors for both targets and unchanged imported theme data.

Default body, primary button, muted text and focus color pairs have focused contrast
checks in light/dark mode. Arbitrary custom palettes still require an accessibility
review; accepting a valid color does not guarantee sufficient contrast.
