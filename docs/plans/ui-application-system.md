# UI application system: source ownership, themes and offline distribution

Status: researched implementation contract, 2026-10-18. This reconciles the owner's
current direction with UI_VISION.md sections 6, 18 and 19. It does not claim that
the described catalog or theme system is implemented. Continue from this contract
without another planning approval. Existing ownership, cancellation and resource
acceptance remains authoritative.

## Product experience

A UI project owns its root page, reusable component source, theme and installation
record. Editing those files changes the application. The compiler supplies checked
primitives and target adapters; updating the CLI never replaces application files.
The same authored component and tokens produce semantic browser DOM and native GPU
output. Local interaction remains independent of networking and authentication.

The intended project is:

```text
tsrust.config.json            executable entry, aliases and UI source locations
ui.theme.json                 semantic colors, spacing, typography and radius
ui-components.lock.json       immutable installation identities and compatibility
app/page.tsrx                 root UI screen
components/ui/button.tsrx     editable styled Button
components/ui/card.tsrx       editable composed Card
src/main.tsr                  inspectable startup and root composition
tests/                        application-owned tests
```

The `components/ui` destination is a configurable ordinary source directory. The
page imports its Button from that directory. The local Button can import the
semantic Button primitive from `std:ui` under an alias. No second renderer, hidden
application arithmetic or public lifetime attachment is needed.

`app/page.tsrx` is the agreed screen convention. `build.entry` currently selects the
executable bootstrap. Root setup must make that distinction visible; naming files
alone does not implement nested routing or layout retention.

## Established patterns and repository fit

| Concern | Established pattern | Tsrust implementation boundary |
|---|---|---|
| Component ownership | shadcn distributes editable source rather than hiding the styled layer in a package | Install ordinary checked `.tsrx` files; retain runtime primitives in `std:ui` |
| Theme customization | shadcn uses paired semantic tokens and light/dark overrides | A project-owned, platform-neutral JSON theme feeds both target adapters |
| Utility styling | Tailwind connects token namespaces to utility classes | One checked style model resolves supported utilities and variants for both backends |
| File distribution | shadcn registry items describe files and dependencies, with configured destinations | Embed immutable registry metadata, payloads and licenses in the CLI; resolve everything locally |
| Reactivity | Fine-grained state changes preserve component instances | Preserve the accepted weak captures and tree-owned state; extend ordinary state types for controls |
| Native drawing | The selected Winit/Taffy/Vello/Parley stack already exists | Finish its semantic and input contracts; do not replace it during theme/catalog work |

Sources: [shadcn's source ownership](https://ui.shadcn.com/docs),
[project destinations](https://ui.shadcn.com/docs/components-json),
[semantic theming](https://ui.shadcn.com/docs/theming),
[registry file/dependency records](https://ui.shadcn.com/docs/registry/registry-item-json),
[CLI previews](https://ui.shadcn.com/docs/cli), and
[Tailwind token namespaces](https://tailwindcss.com/docs/theme).
These establish the pattern, not compatibility with React or all Tailwind syntax.
Tsrust's offline embedded catalog is the owner's explicit requirement; it is not
an assumption that shadcn's ordinary network workflow already guarantees offline use.

The installed renderer dependency closure already contains `color` 0.3.3.
Its `src/parse.rs` provides `parse_color` and `DynamicColor::from_str`; Peniko
re-exports that crate. Evaluate this parser and its conversions before writing
another color parser. Publish supported color spaces and device conversion rules.
The [Design Tokens format](https://www.designtokens.org/tr/2025.10/format/) provides
useful typing and alias rules, but adopting those ideas does not require replacing
the vision's readable JSON with a complete interchange-standard implementation.

## Current implementation gaps

The current utility parser contains literal RGB colors, spacing and radius values.
Names such as `bg-primary` are not yet customizable theme tokens. `new --ui` creates
a page but does not install a local styled component or an immutable catalog.
There is no production `ui add`, configurable component destination, installation
lock or compiler/runtime compatibility contract for installed component versions.
The existing route-manifest experiment is declaration tooling, not a production
UI routing runtime or installer.

JSX currently lowers named components through ordinary checked function calls.
It supports explicit attributes, children and expressions; its parser does not yet
provide JSX prop spread. That is a concrete prerequisite for convenient component
prop forwarding, not permission to clone callback environments or generate a
component-specific compiler escape hatch. Accepted generic interface inheritance,
project aliases, typed JSON imports and retained callbacks should be reused.
Signal storage is currently `i64`; string/boolean state and editable controls remain
real gaps. Do not introduce separate public signal-text or signal-button components.

A new Chromium presentation check also found that `button.ui` overrides the
single-class `rounded` utility through CSS specificity. The native focus/layout
checks passing does not make browser styling complete. Fix stylesheet ordering and
specificity as part of the shared style contract, not another isolated palette patch.

## One theme and style contract

The JSON theme is the authoring source for semantic foreground/background pairs,
card, primary, secondary, muted, accent, destructive, border, input and focus-ring
colors, plus spacing, radius and typography scales. Light and dark modes use the
same names. Project configuration identifies the theme file and component paths.
Unknown keys, invalid values, missing references and alias cycles produce source
errors. Avoid coercion, implicit defaults that conceal missing tokens, and silent
fallback to a hardcoded palette.

A checked style representation sits between classes and target rendering. It
contains semantic references, dimensions, typography, states and layout properties.
Web lowering emits CSS variables and ordered utility rules. Native lowering uses
the same resolved values for layout, paint, focus geometry and hit testing. Project
theme values belong to application data, not the reusable support crate's source;
a color edit must not trigger a new renderer dependency compilation.

Support the useful Button/Card contract as a complete subset: semantic colors,
spacing, text alignment, radius, border/focus ring, disabled and interaction states,
and typed variants/sizes. Identify unsupported CSS capabilities explicitly. Normal
customization must not require modifying the Rust utility parser. Component defaults
and caller overrides must resolve deterministically by semantic property before
CSS generation; simply concatenating classes and rejecting the resulting duplicate
color/padding category does not supply shadcn-like customization. Reuse the shared
style model for this supported merge policy rather than duplicate a full Tailwind
class engine in each backend. Root theme scope
belongs to the mounted application tree, not a process-global singleton. Theme mode
changes should preserve component state; this requires the appropriate checked
state support before claiming runtime theme switching.

## Offline installation and version ownership

The production CLI namespace is `tsrust ui`; existing `tsrust add` manages Rust
packages and keeps that meaning. Implement `ui list --bundled`, `ui add`, dry-run,
explicit destination selection and reproducible noninteractive operation together.
`new --ui` must reuse that installer rather than maintain a second template system.
Exact command spellings should follow section 19's examples where practical.

Before writing, resolve the complete local dependency graph, exact versions,
variants, licenses, paths and compiler/runtime/toolchain requirements. Preview all
changes. An unavailable version, dependency conflict, unsafe destination or
unsupported prerequisite fails before files are changed. No downloads, package
manager invocation or registry hooks occur during component installation.

Each published identity has immutable payload bytes and hashes. Bundle source in
versioned registry records so a future source formatter cannot rewrite a published
payload accidentally. Record template hashes and initially generated file hashes
separately. Re-adding an installed version preserves edits and leaves its lock
record unchanged. A differing current hash means customized source, not corruption.
Initially refuse upgrades into occupied paths; do not invent an automatic merge.
Catalog/CLI, component, runtime ABI, compiler build and Rust toolchain identities
remain distinct. A moving `0.1.0` compiler string alone is not a compatibility pin.

The first catalog must retain its published first version and dependencies.
Dropping a supported payload requires an explicit future retention policy; do not
promise every historical version forever. Existing installed source remains usable
with its recorded supported toolchain. New catalog versions do not rewrite it.

## Coherent implementation and acceptance

The next complete section connects configuration, theme validation, shared styling,
editable Button/Card source, embedded catalog installation, compatibility records,
compiler forwarding support, LSP and the standalone project. It is complete only
when a user can install offline, edit a component, change the theme, and see the
result on both backends while preserving state ownership and local edits.

Implement witnesses alongside the section, then run targeted acceptance once:

- Create a UI project and install exact bundled versions without network access.
- Select a nondefault component destination and resolve its ordinary imports.
- Change semantic colors, radius and typography; compare actual browser styles
  with native resolved geometry/colors and focus behavior.
- Customize installed source; run a newer catalog; prove source and lock bytes stay
  unchanged. Reject absent versions, conflicts and overwrites before any write.
- Check JSX forwarding, callback ownership and failure behavior through generated
  execution; do not substitute emitted-Rust patches for missing language support.
- Verify theme/class/variant diagnostics, completion, hover and navigation using
  compiler metadata, including invalidation after an unsaved theme edit.
- Verify app-only/theme-only edits reuse native support. Reuse accepted lifetime,
  browser interaction and SQL evidence rather than rerunning unaffected families.

Then complete the editable-list family: string/boolean state, real text editing and
IME, selection, focus, stable keyed insertion/removal and deterministic disposal.
Routing follows the existing app-tree contract: persistent layouts, typed paths,
loader association, missing/error source and application-installed middleware.
The existing Hono-style API over Axum remains the server boundary; authentication
policy stays in application code. Neither phase should be replaced by more counter
micro-optimizations.

## Cache policy reconciliation

The historical dev-loop plan explicitly required 32-entry/256 MiB caches. That
explains the inherited implementation; it is not a language or renderer restriction.
The observed optimized GPU dependency bundle is 305,877,497 bytes. Silently refusing
it caused repeated support builds. The interim 512 MiB runtime budget demonstrates
reuse but does not establish the desired final product policy.

Separate artifact correctness from disk housekeeping. Recommend retaining valid
runtime artifacts without a fixed per-artifact rejection threshold, reporting
cache usage, and providing explicit cleanup. Any automatic quota should be an
explicit user policy with visible eviction, not an unexplained renderer cutoff.
Keep atomic publication, identity checks, corruption detection, private executable
copies and process-group cleanup. Evaluate storage layout once against real UI
artifacts instead of repeatedly raising constants.
