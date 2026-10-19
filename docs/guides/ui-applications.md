# UI application ownership and offline components

## Live mode, retained visibility and native headers

`withTheme(node, config.ui.theme, mode)` accepts `light`, `dark`, or a
component-local `Signal<string>`. A live mode updates the palette after handlers,
preserving child state, node identities and field values. Nested explicit theme
scopes remain independent. Invalid modes throw before replacing the checked
palette. Theme token data is still read during construction; this is not yet a
live token-record editor.

`<View hidden={showCode.get()}>…</View>` removes the subtree from layout, paint,
interaction and accessibility while retaining its state and callbacks. Showing it
restores its prior values. Hidden descendants defer reactive refresh until shown.
Use a conditional JSX branch when removal should dispose and later reconstruct
its children. `aria-hidden` instead keeps noninteractive content visible while
excluding it from accessibility.

`run(node, title, options?)` accepts creation-time `WindowOptions`. Defaults are
640×480 logical pixels and native titlebars. Width/minWidth accept 240–16384;
height/minHeight accept 160–16384. Minima default to 240×160 and cannot exceed
initial dimensions. Invalid options return `Err` before opening a window.

On macOS, `titlebar: 'custom'` extends content into a transparent titlebar while
preserving real traffic-light controls. Reserve space for these controls and mark
custom drag surfaces with `<View windowDragRegion>`. Text and blank space drag;
interactive descendants, including disabled controls and labels, exclude dragging.
Other desktop platforms currently return `Err` for custom titlebars. Browsers
validate options and retain their host window. Live chrome switching and custom
chrome on other native platforms remain unsupported.

See [the paired application example](../../examples/ui-application-state/README.md) for
an edited draft and counter that survive theme and preview/source switching.

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

- `tsrust ui list --bundled [--json]` lists one current payload per component and
  source variant. The terminal browser uses component names, with version details.
- `tsrust ui init [--path src/local/controls]` prepares missing UI settings.
  Existing settings remain; repeated initialization is safe.
- `tsrust ui add button card --target native [--cwd project] [--path src/local/controls]`
  resolves the entire embedded dependency closure before writing.
- `tsrust ui add` in a terminal selects multiple components with Space and installs
  them together with Enter. Cancellation writes nothing. Without rich terminal
  support, enter comma-separated numbers. Target selection still precedes writes.
- An explicit `@version` must name the current bundled version. New installations
  do not select historical versions; compatible existing installed source stays owned.
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

The public catalog offers the current component payload and exact token dependency.
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
the lock cannot reproduce your edits. A first installation uses `card.tsrx`.
An upgrade preserves it and adds the current version under `card-v3.tsrx`.
Minor/patch releases use names such as `card-v3-1-2.tsrx`, avoiding dots that would
look like source extensions. Already occupied versioned files without a matching
installation record cause the entire batch to fail before publication. Re-adding
an installed version preserves its actual filename and edits. A missing recorded
file must be restored from version control; re-add never silently repairs it.

Only newly generated bundled source has its parsed relative imports updated to
the dependency's recorded filename. Comments, ordinary strings, existing local
components and application imports stay intact. The lock records both immutable
templates and generated file hashes/paths. Historical payloads retained internally
validate existing installation records; they are not additional choices or a
promise of future version support. `--path` remains available for alternate trees.

LSP uses compiler component types for props/variants, the shared utility catalog
for class completion/hover, and theme data for definition links. JSON theme help
and compiler diagnostics follow unsaved project buffers. Unsupported static JSX
classes are compiler errors; dynamic class strings are validated on construction.

Native artifacts have no implicit size/count quota. `tsrust cache info` reports
project cache bytes; `tsrust cache clean` explicitly removes reusable artifacts
under namespace locks. Running private executable copies remain valid. A later
build recreates removed support. `TSRUST_CACHE_DIR` selects the same shared cache
for build, run, dev and cache info/clean. Source/backend/profile identities and
workspace locks still apply when projects share that directory.

A runtime cache miss also reuses a persistent, locked dependency workspace under
`target/tsrust-native/runtime/build-v1`. Runtime source revisions rebuild the support
library while unchanged Cargo dependencies retain their artifacts. Toolchain,
bundle and profile changes use separate workspaces. Dev never waits on a busy
workspace; that build uses private temporary storage. Resolved Rust binding
packages currently retain their existing private build protocol. Cache info and
clean include the persistent dependency workspaces; cleaning waits for active
build/link readers and preserves lock identities.

Native windows scroll content that exceeds their viewport, including horizontal
overflow. Wheel and trackpad input scroll the page; text fields consume their own
scroll range first and pass remaining movement to the page. PageUp/PageDown,
Home/End and vertical arrows scroll outside text editing. Tab reveals the newly
focused control. Painting, hit tests, accessibility and IME positions share the
same scroll offset. Nested component scroll areas and scrollbar controls are
separate library work; this window behavior does not claim those components.
Native text layout retains fractional coordinates so rounded widths do not cause
an extra painted line after measurement.

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
div, p, h1–h6, code and kbd; View accepts div, section or label. A named region requires a
nonblank accessible label. Alert is assertive; status is polite. A separator must
be empty, is not focusable, and defaults to horizontal orientation. Class names
still determine its dimensions. Hidden decoration remains visible but is excluded
from accessibility; enabled controls inside a hidden subtree are rejected, even
when a reactive update would enable them later.

Only supported semantic roles are accepted. This API does not expose arbitrary
HTML or simulated interactive roles. Text controls and checked label/description
relationships are covered below.
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

Scalar state uses `signal(initial)` for int, number, boolean or string.
`Signal<T = int>` defaults its type only; the initial value remains required.
String reads and writes are independent snapshots. Numeric storage preserves full
int and IEEE-754 values; number text uses ordinary language formatting (-0 is 0).
Arrays, records, nullable values, resources and callbacks are not scalar state.
See examples/ui-state for a paired component and native assertions.

Direct Signal children and supported scalar JSX expressions bind updates.
Numeric children preserve their exact scalar types; null/undefined are omitted.
Bare booleans remain errors. Local-state JSX ternaries lazily mount selected node branches. See examples/ui-composition and the computed-text
section below.

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

### Owned JSX forwarding and metadata conditions

A local interface can extend a primitive's props. `<View {...props} />` moves the
source once, forwards only fields declared by ViewProps, and destroys excess
spread fields once. Omitted optional fields become undefined; missing required
fields and explicitly written unknown attributes fail. Later attributes/spreads
win, including an explicitly present undefined. Read/clone ordinary string fields
before moving props if they contribute to a later override. Ordinary `{...props}`
and record rest retain their checked-copy semantics and reject resource copying.
See `examples/ui-forwarding` for the executable ownership witness.

`data-slot` and `data-size` carry checked ASCII letters/digits/hyphens; empty means
absent. `data-size` does not assign dimensions or accessibility roles. Shared
class conditions support `data-[slot=VALUE]`, `data-[size=VALUE]`,
`has-data-[slot=VALUE]`, `has-data-[size=VALUE]`, and the named-ancestor forms
`group-data-[slot=VALUE]/NAME` and `group-data-[size=VALUE]/NAME`. Put `group/NAME`
on an ancestor to identify it. `has-data` searches descendants, excluding self;
group conditions match any named ancestor, excluding self. `[.CLASS]` tests an
exact class on the same node. Names and values use ASCII letters/digits/hyphens.
Conditions can compose with each other and one hover/focus/active/disabled state.
All conditions must match. Arbitrary CSS selectors remain unsupported.

Later classes override earlier properties within each state. Matching interaction
states override base properties, retaining the existing portable contract.
Predicates do not increase browser selector specificity. Native layout uses the
same predicates against borrowed tree context without cloning descendants.
Browser selectors stay live when DOM metadata changes; this does not introduce
a public DOM mutation API or dynamic tree reconciliation.

### Card content, scoped spacing and rounded clipping

`tsrust ui add card@3.0.0 --offline` installs the seven editable Base UI + Nova
content parts with the original MIT license and pinned source attribution. The
working example is `examples/ui-card`. This remains a portable content port:
image primitives/edge variants, inline-size containment/container queries and
named heading-font assets are unsupported, so full Nova Card parity is pending.
CardHeader uses ordinary grid intrinsic sizing. Existing v1/v2 payloads remain
immutable and compatible; API 2 accepts API 1 source locks.

Declare numeric spacing with `[--NAME:--spacing(N)]`; N uses the same checked scale
as p/gap. Use p/px/py/pt/pr/pb/pl/gap/gap-x/gap-y-(--NAME) to consume it. Names use
ASCII letters/digits/hyphens. Values resolve in the declaring theme scope and
inherit as logical pixels; nested theme changes do not recompute inherited
values. Later matching declarations win. Metadata conditions are supported;
interaction-state declarations, expressions, aliases and variable cycles fail.
A missing variable resolves to zero, replacing earlier padding/gap values. Card
sets its own --card-spacing, so a nested Card starts a new scope.

`text-sm` uses 7/8 of typography.base for font size and 5/4 for default line height
(14px/20px at base=16). `leading-snug`/`leading-normal` use inherited ratios 1.375
and 1.5; explicit leading overrides the text-sm default regardless of class order.
`font-medium`/`font-semibold` use weights 500/600. `font-sans`/`font-serif`/`font-mono`
select system UI, serif or monospace generic families; actual fonts and glyph
rasterization depend on the platform. Arbitrary named/font-asset loading is not
implemented. Card titles are ordinary content; supply a semantic heading child
when needed rather than assigning every title an invented heading level.

`radius.xl` is optional, finite 0–256, and falls back to lg. `rounded-xl` and
rounded-t/r/b/l/tl/tr/br/bl-xl use it; other corner variants use none/sm/md/lg.
Radii scale together to fit the box. Border sides use border-t/r/b/l, with -0 or
-2 selecting zero or two logical pixels; unqualified border uses one pixel.
Later classes override each side/corner independently.

`overflow-hidden` clips descendant paint and pointer targets to the rounded
padding box, including asymmetric inner border curves. `overflow-visible`
restores overflow. The node's own outer ring/focus outline remains outside its
content clip. Descendant outlines are clipped by ancestors. Native hit geometry
uses physical coordinates at the current display scale. Clipping does not disable
controls or remove them from source, keyboard or accessibility order. Container scrolling
and automatic container keyboard-focus scrolling remain unsupported on native; do not place
necessary controls entirely outside a clipped region. Native curves use cubic
quarter-ellipse approximation; pixel-identical rasterization is not claimed.

## Editable text controls

`tsrust ui add input@3.0.0 textarea@3.0.0 label@3.0.0 --offline` adds editable local
Base UI + Nova text-control adaptations. Use their project imports. `std:ui`
supplies the portable Input/Textarea primitives and View's label tag; there is no
separate browser module or public lifetime handle. See `examples/ui-inputs`.

```tsrust
const name = signal('Ada');
return (
  <View>
    <Label htmlFor="name">Name</Label>
    <Input id="name" value={name.get()} onChange={(event) => {
      name.set(event.target.value);
    }}/>
  </View>
);
```

`value` selects controlled editing; application state accepts or rejects each
committed edit. Direct signals and state-reading JSX value, disabled and readOnly
expressions refresh after handlers. An unchanged controlled echo preserves cursor
selection. Omit value to use uncontrolled editing initialized by defaultValue.
Empty string and zero controlled values take precedence. Values accept strings,
exact ints and numbers with ordinary formatting; edits always produce strings.
Input removes CR/LF, while Textarea normalizes CRLF/CR to LF. Browser values with
unpaired UTF-16 surrogates fail explicitly instead of silently becoming replacement
characters; portable strings contain Unicode scalar values.

`onChange` receives borrowed target/currentTarget snapshots with value, name and
id. Composition preedit remains internal until commit. Native editing delegates
Unicode shaping and selection to Parley and clipboard operations to the OS. Cut
only deletes after successful copying. Clipboard failures leave values unchanged
and report a message. Linux clipboard persistence after exit needs a clipboard
manager; no helper process stays alive to retain it. Browser autocomplete is a
hint; native autocomplete is not implemented. The native accessibility tree
exposes focus, values, labels/descriptions, required/invalid/read-only/disabled
state and SetValue. Accessibility text-range navigation remains pending.

Use htmlFor/id or wrap one control in a label. Nested labels and multiple enclosed
controls fail. Keep explicit IDs unique within the document; duplicate IDs are not
rejected. aria-describedby accepts ordered ID references; missing references
are ignored. aria-invalid is an explicit boolean, with no validation policy.
Field labels and descriptions do not introduce framework-owned application state.

Nova spacing supports half steps through 3.5 and size 5. Viewport sm/md/lg/xl/2xl
variants use 640/768/1024/1280/1536 logical pixels. dark matches the enclosing mode;
placeholder:text-COLOR styles only the hint, and aria-invalid matches true.
outline-none removes the fallback focus outline: retain a visible focus ring.
field-sizing-content grows Textarea within min/max bounds. Editor scrolling is
supported; general native container scrolling is still pending.

These adaptations are a text-control slice, not complete Input parity. Password,
file, number/date and other specialized types are rejected. DOM event methods,
refs, Base UI value-change details, form submission, composition callbacks, resize handles, animated transitions
and Label peer/group disabled styling remain pending. Generic callback-result
inference from an unannotated closure alone also remains a language gap; generated
input getters use an expected checked scalar union without precision erasure.

## Nullable children and component results

`Child` and `TextChild` accept null and undefined. They produce no node or text,
preserve sibling order and leave adjacent text in one run. A component can return
`Node | null | undefined`, including a component with local scalar state. Present
results retain their state automatically; absent results release local callbacks
before their state owners. No public lifetime helper is needed. See
`examples/ui-nullable-children` for independent counters and resource observations.

```tsrust
function Message(props: { visible: boolean; }): Node | null {
  return props.visible ? <Text>Ready</Text> : null;
}
```

Ordinary scalar props select initial content. JSX ternaries reading directly
declared component state mount lazily, as described below. Keyed lists remain
separate work. Signal payloads still require nonnullable scalar
values. Bare boolean literals remain unsupported; use a ternary with
null/undefined for absence and strings or numeric snapshots for visible text.
Nullable arrays containing owning Nodes or Signals still cannot be cloned,
borrowed into owning props or sent to workers.


## Computed scalar text

Keep using Text, View and Button. Supported JSX expressions reading directly
declared component state become live text bindings: `{count.get() + 1}`, string
operations/templates, scalar conditional branches and top-level/imported helpers
with declared scalar return types. Application formatting belongs in `.tsrx`, `.tsr` or
portable Rust functions; no framework increment helper or public binding factory
is needed. See `examples/ui-computed-text`.

Precompute outside JSX for a snapshot. Unknown/inferred helper results, generic
child contracts and state reached through arbitrary prop records remain snapshots. Getter expressions must be pure; construction and refresh
may evaluate them. Null/undefined getter values contribute no characters; this
changes text. Node branches use the conditional mounting contract below; keyed
lists remain separate work. Captured owned data follows ordinary callback rules without
implicit clones.

Numeric literal and owning numeric-array children preserve their exact types.
Integers use all 64 bits; numbers keep ordinary NaN/infinity formatting and -0 as
0. Bare boolean literals remain unsupported; explicit boolean state bindings
produce true/false text. Getter errors stop the affected mount, preserve their
message and release the callback environment independently of other mounts.


## Conditional mounting

Use ordinary JSX ternaries, including nested conditions. Only the selected branch
is constructed. Conditions reading directly declared component state refresh after
handlers; selecting the same branch preserves its local state and controls.
Removing a branch releases its callbacks, resources and local owners once.
Selecting it again constructs fresh state. Transparent internal slots add no
layout gap, wrapper element or accessibility node.

```tsrust
function Page(): Node {
  const visible = signal(true);
  return (
    <View>
      <Button onPress={() => { visible.set(!visible.get()); }}>Toggle</Button>
      {visible.get() ? <Counter/> : null}
      <Input aria-label="Persistent name" defaultValue="Ada"/>
    </View>
  );
}
```

The neighboring input keeps its identity, selection, focus and active composition
when the counter appears or disappears. Removed controls cannot receive later
framework events, and their IDs are not reused. Native text editors cancel removed
composition and release editing state. Browser listeners belong to the mount;
new branches participate through delegated events without retaining dead handlers.
New nodes receive the project theme and scoped styles.

A component can also return a local-state ternary directly. Its returned Node owns
the dynamic slot, even when the selected branch currently contributes no visible
node. Local-state conditional results therefore describe a live subtree rather
than exposing a null snapshot. Ordinary scalar conditions and precomputed values
retain their normal snapshot behavior. Expressions must obey normal callback
ownership: repeatedly moving a prebuilt Node from a capture is an error. Construct
it inside the selected JSX branch instead. Getter conditions must be pure.

Button phrasing, label nesting/control counts, separator content and aria-hidden
rules are checked again when branches change. Failure stops the affected mount
and releases its owned tree; other mounts keep working. This does not add keyed
lists, generic reactive prop forwarding or persistent routing. See
`examples/ui-conditional-mounting` for the paired interactive example.

## Keyboard content and separators

`tsrust ui add kbd separator --offline` adds editable local `Kbd`, `KbdGroup` and
`Separator` source from the pinned Base UI + Nova baseline. Kbd displays text;
it does not register keyboard shortcuts. Both keyboard parts use semantic kbd
containers and accept owned composed children, including live text. Interactive
children still reject when nested inside a Button. Caller classes override
component properties and inherit the configured theme.

```tsrust
import { View, Text, Node } from 'std:ui';
import { Kbd, KbdGroup } from './components/ui/kbd';
import { Separator } from './components/ui/separator';

function Toolbar(): Node {
  return (
    <View className="gap-4">
      <KbdGroup><Kbd>Ctrl</Kbd>+<Kbd>K</Kbd></KbdGroup>
      <Separator />
      <View className="flex-row items-center h-12 gap-4">
        <Text>First section</Text>
        <Separator orientation="vertical" />
        <Text>Second section</Text>
      </View>
    </View>
  );
}
```

Separator defaults to horizontal, is nonfocusable, and exposes orientation through
accessibility and data metadata. Horizontal lines have one logical pixel of height
and fill available width. Vertical lines have one logical pixel of width and
stretch in their parent's cross axis. They accept no children; arbitrary render
callbacks and DOM props remain outside the declared portable interface.

`inline-flex`, preferred `w-fit`/`h-fit`/`size-fit` and `text-xs` share compiler,
native and editor behavior. Fit-content uses the available size bounded by
min-content and max-content; unbreakable text can overflow. text-xs uses 3/4 of
`typography.base` with a default line height of one base unit. `in-data-[slot=VALUE]`
matches any ancestor, excluding self; separator `data-horizontal`/`data-vertical`
variants match orientation. SVG icons and Tooltip itself remain pending; styling
under an explicit tooltip-content context does not install tooltip behavior.

## Shared component attributions

New offline installations place full notices in `attributions/` at the project
root. Adapted shadcn source shares `attributions/shadcn-MIT.txt`; Tsrust component
source shares `attributions/tsrust-MIT.txt`. Additional distinct notices get
content-addressed filenames. Each generated component has one short relative
license reference, which also works with a custom component directory.

The lock records notice paths and SHA-256 hashes instead of repeating full notice
text in each new record. Immutable catalog templates and their original hashes
stay unchanged. Re-adding components preserves existing source and legacy records.
Missing, edited or symlinked notices fail installation with an explicit diagnostic;
installation does not overwrite them.

Move legacy full headers explicitly, preserving all component bodies and edits:

```sh
tsrust ui attributions --dry-run
tsrust ui attributions
```

This command moves recognized leading license blocks into shared files, retains
other provenance comments, and changes no application imports or component logic.
Existing template/generated hashes remain original installation provenance.
Repeated migration is idempotent. The same transactional publication/rollback
covers source comments, notices and lock changes. No network or Rust compilation
is needed. Package exports and the bounded web builder include recorded notices.
Keep `attributions/` with distributed source and carry the applicable notices
with packaged applications. UI attribution collection covers installed component
sources; vendored Rust dependencies retain their own notices under `vendor/`.

### Scrollable panes and viewport sizing

`overflow-auto` creates a bounded scroll container on both targets. Constrain its
height/width, and set `min-h-0` or `min-w-0` when a flex child must shrink. Utilities
such as `h-screen`, `w-screen` and `max-h-screen` use the logical native window or
browser viewport. Percentage heights still need a definite parent height.

```tsrx
<View role="region" aria-label="Article" tabIndex={0}
  className="h-screen overflow-auto p-4">
  <Text>Article content</Text>
</View>
```

A container is not a Tab stop by default. `tabIndex={0}` adds a source-order stop;
`tabIndex={-1}` permits pointer/accessibility focus without a sequential stop.
Positive values and values below -1 reject at construction. Use a named region
for a pane users need to reach by keyboard. It has no activation callback.

Wheel/trackpad input first scrolls the innermost pane under the pointer, then
passes unused deltas to scrollable ancestors and the window. Keyboard PageUp,
PageDown, Home, End and arrows target the focused pane or its nearest scrollable
ancestor. Space scrolls an explicitly focused pane; buttons and text fields keep
their normal behavior. Tab reveals the next control through nested clips.
`overflow-hidden` clips and supports focus reveal, but does not consume wheel
input. Rounded clipping, pointer targeting, editor positions and accessibility
bounds use the same offsets. Hiding a pane with `hidden` retains its scroll
position; removing its subtree retires that state.

The runnable documentation foundation now lives in `docs/`, with editable controls
installed by the normal offline CLI. It is an initial shared shell; routing,
Markdown, full-text search and complete component reference pages remain pending.
