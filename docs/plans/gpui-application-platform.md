# GPUI application platform after v0.2.0

Owner direction recorded 2026-11-05. Status: researched implementation plan;
no GPUI integration or platform acceptance is claimed here.

## Priority and product intent

Finish the full [v0.2.0 standard-library checklist](stdlib-0.2.0.md) first.
Then pursue the owner's strong preference for GPUI and GPUI Kit as Tsrust's
actual UI framework, preserving the previously discussed React/Solid-like
`.tsrx` syntax and signal-driven binding/update model. The owner accepts an
experimental browser target; production browser maturity is not a prerequisite
for beginning this work. Desktop means macOS, Windows and Linux.

The requested scope includes the complete upstream component library, its
examples on desktop and web, basic applications, generated `.tsr` API clients,
and a CLI-managed shared root with separated backend/frontend projects and
automatic client generation. This supersedes the backend-only UI freeze for
post-v0.2.0 planning. Backend-only workflows and dependency isolation remain
requirements. The earlier React/Electron/Expo direction is historical.

This is not permission to publish v0.2.0 or a later version. No new version
number or release promise is assigned to this plan.

## Owner decisions, 2026-11-09

- Remove both earlier UI attempts: the ground-up renderer (`crates/tsrust-ui`,
  `std:ui`/`ui-runtime`, vello/winit/accesskit dependencies, the `legacy-ui`
  feature, UI tests, examples and docs) and every Electron/React/Expo/Vite/shadcn
  template or pin. Git history keeps them. Keep only the `.tsrx` parser, JSX
  syntax and signal semantics, retargeted to GPUI Kit.
- Components are added shadcn-style: `tsrust ui add <component>` copies an
  editable `.tsrx` wrapper into the project's `components/`, backed by a thin
  compiler-owned binding to the GPUI Kit component.
- A CLI-generated starter, `examples/gpui-starter`, imports every GPUI Kit
  component with runnable examples in a shadcn-like gallery. It is tracked in
  git and regenerated on major changes; it is the acceptance project for the
  LSP, startup time and the `ui add` flow, opened in VS Code.
- Desktop (macOS/Windows/Linux) first; WASM follows, because GPUI Kit's web path
  is showcase-only and needs nightly.
- Rust toolchain: GPUI Kit 0.9.0 fails on 1.96.1 (`Option::map_or_default`) and
  builds on 1.99.1. Users get a minimum version, Rust 1.99 or newer from their
  installed stable toolchain, checked by setup/doctor; native caches already key
  on the exact `rustc` version. The repository pins 1.99.1 only for reproducible CI.

## Research and boundaries

Primary sources checked 2026-11-05:

| Source | Finding relevant to admission |
|---|---|
| [GPUI Kit overview](https://gpui-kit.com/docs/) | Upstream labels desktop Stable across macOS, Windows and Linux. Kit reexports its matched GPUI snapshot and combines behavior, styled components and assets. Its JavaScript extension shell is separate. |
| [Component catalog](https://gpui-kit.com/component/) | The surface spans basic controls, forms, overlays, data display, editing and application layout. Inventory the actual release catalog and stories rather than treating the advertised component count as completeness. |
| [WebAssembly guide](https://gpui-kit.com/docs/webassembly/) | Browser support is labeled Showcase only. Shared Rust views need a WASM entry point and loader. Current limitations include one canvas/top-level window. Graphics initialization is asynchronous; module loading alone proves no rendered frame. The upstream gallery uses nightly and a matching wasm-bindgen CLI. |
| [Installation](https://gpui-kit.com/docs/installation/) | Validate OS/toolchain requirements: macOS 15+, Windows MSVC/SDK and Linux graphical/Vulkan dependencies. Keep GPUI dependencies matched to Kit. |

These are upstream claims, not our platform test results. The bridge and project
design below are Tsrust proposals inferred from those capabilities and the local
compiler contracts. Admit an exact compatible upstream release/commit and locked
dependency graph during implementation; do not independently upgrade GPUI crates
or change the user's default Rust toolchain. Use Kit's supported facade first.
No JavaScript extension runtime is required for Tsrust-generated Rust views.

The [upstream licensing section](https://gpui-kit.com/docs/#license) distinguishes
Apache-2.0 software/examples from CC BY 4.0 documentation prose/illustrations.
Preserve applicable notices and asset licenses. Attribute adapted documentation
to GPUI Kit, link its source and [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/),
and identify changes. Prefer adapting runnable example code rather than copying
documentation prose.

## Reuse the language and state contracts

Start from retained [UI vision](../../UI_VISION.md), actual `.tsrx` fixtures and
[reactive-prop contracts](../content/ui/foundations/reactive-props.md). Historical
illustrations are not all implemented APIs. In particular, accepted examples use
`signal(...).get()`/`.set(...)`; an older `count.value` sketch does not override
them.

- Preserve function components, JSX children/props, typed callbacks, signals,
  derived values, live props and composition. Components construct once; signal
  updates do not rerun arbitrary component bodies as React renders.
- Keep ordinary props' construction-time semantics distinct from live props.
  Preserve ownership, checked errors and disposal rather than adding implicit
  cloning, public lifetime handles or an unchecked dynamic value layer.
- Compile to a thin safe Rust facade over GPUI/Kit. Map signal dependencies to
  GPUI entity updates/invalidation. GPUI may invoke its render path repeatedly;
  this must not repeat Tsrust component construction or reset signal state.
- Prove stable component identity, keyed insertion/removal/reordering, conditional
  children, event ordering, controlled inputs and feedback-loop prevention.
  Removing a subtree releases handlers/subscriptions once; async completion must
  not update disposed views. Keep UI-thread work and background work explicit.
- Preserve familiar styling/theme patterns where supported. Inventory existing
  utilities and theme tokens against Kit styling before promising compatibility;
  provide diagnostics for unsupported operations rather than silently ignoring
  them. Do not recreate an independent widget/layout renderer.
- Browser and desktop share component source and state semantics. Use explicit
  target capability diagnostics for filesystem, process, windows and other
  platform APIs. WASM UI work must also admit the necessary compiler/runtime
  subset; a native compiler's success does not establish WASM compatibility.

Every public wrapper needs compiler typing, ownership analysis, native emission,
editor signatures/completion/hover/parameter help, docs and executable examples.
Compare bridge overhead and update allocations with equivalent direct GPUI views
before expanding the catalog. Use existing measurement ingestion for actual runs.

## Complete catalog and example acceptance

At admission, pin a component/story inventory to the selected upstream revision.
Track every public component, its documented example variants and required
behavioral primitives. Each row records upstream source, Tsrust wrapper/props/
events, example, desktop OS results, browser results and remaining limitations.
No component disappears because wrapping it is inconvenient. An unsupported
target remains an explicit gap; a platform limitation is not a passing test.

Port upstream website examples into one Tsrust gallery with shared `.tsrx`
stories, a desktop runner and a WASM host. Cover all inventoried examples, not
only an attractive subset. Preserve their behaviors and accessible semantics.
Implement in coherent dependency families: theme/root/layout; basic controls;
forms/selection; overlays/focus; collections/virtualization; editor/dock/rich
content and remaining platform-dependent components. This orders work without
reducing the requested full catalog.

For each family, verify real interaction, keyboard navigation, focus restoration,
text/IME input where applicable, sizing, scrolling, themes, disabled/error states,
accessibility and teardown. Use upstream interaction/testing facilities where
possible plus actual visible platform checks. Browser acceptance requires painted
content and successful interaction, fonts/assets, resize/DPI, GPU fallback and
browser permission behavior. Record browser accessibility and startup/download
size explicitly. Desktop-only multiwindow features get target-specific stories;
in-window overlays are tested on both. Do not claim production web support from
gallery success or treat experimental status as permission to skip the gallery.

After catalog coverage, build small real applications with shared desktop/web
sources: a local task/notes app for forms, persistence and navigation; and an
API-backed data browser for tables, filtering, async loading, authentication,
cancellation and errors. These are proposed acceptance applications, not new
application-framework abstractions. Exercise packaging and clean-machine launch
on all three desktop OSes and a served browser build.

## Generate native Tsrust API clients

The current `tsrust api generate` implementation supports TypeScript and Rust.
Add a third language producing ordinary typed `.tsr` modules. Proposed spelling:
`--language tsrust`; freeze CLI help/config spelling with implementation.

Reuse the existing HTTP description, named operation/model generation, codecs,
validation, structured errors, authentication configuration, compatibility
manifest and `--check` staleness behavior. Do not duplicate the contract model.
Inputs remain source/config, a saved description or a live description URL; a
running backend is not required for source/offline generation. The server owns
its handlers and schema and never imports its generated client to start.

Use the native HTTP transport on desktop/backend consumers and an admitted
browser fetch transport on WASM. Preserve contract errors and supported wire
types across languages. Explicitly handle browser CORS, credentials/cookies,
header restrictions and cancellation. Never copy server secrets or server-only
imports into frontend output. Unsupported contract types/operations must fail
generation with actionable diagnostics rather than degrade to untyped values.

Acceptance includes an actual `.tsr` consumer calling the existing server,
standalone generation from a saved description, auth and malformed-wire failures,
old/new client-server compatibility, editor diagnostics, offline export and
desktop/browser application integration. Existing TypeScript/Rust clients remain
supported and independently usable.

## Shared application root and generation graph

Support one directory containing separated backend and frontend projects through
the CLI. Proposed layout, subject to reuse of existing config/path handling:

```text
my-app/
  tsrust.config.json
  backend/src/
  frontend/src/
  frontend/generated/api/
```

One frontend may target desktop, web or both. Named additional frontends may use
separate paths. The root maps project entries, targets, contract source and client
output; exact config keys and CLI commands are not implemented by this document.
Keep generated files outside handwritten source ownership. Keep backend/frontend
build units separate, with explicit shared model modules when suitable; shared
folders must not make frontend imports of backend handlers legal.

Preserve backend-only, frontend-only, existing layouts, custom output paths and
external API consumers. Setup must inspect existing files, avoid overwrites and
offer a concrete migration for existing projects. Normal check/editor operations
stay read-only and do not install dependencies or rewrite projects implicitly.

The dependency order is:

1. Check backend contract source and extract the validated description without
   running its service or executing handler side effects.
2. Generate/check deterministic client modules and their contract identity.
3. Type-check frontend against that exact client, then compile/package targets.

Development watch mode regenerates on contract changes before frontend checking,
debounces repeated events and prevents generated-output watch loops. Publish
complete outputs atomically so editors/builds never consume partial files.
Backend implementation changes that preserve the contract need no client churn.

A release build performs generation before frontend checking from local checked
source or a pinned saved description. It must not depend on a running service or
fetch a mutable remote schema. Reuse existing stale-output checks; include
contract/generator identity so a stale client cannot pass the build. Define
whether generated source is committed per project, while making both policies
reproducible. Keep locked/offline builds, portable vendored exports and shared
Cargo caches. UI dependencies must remain absent from backend-only builds.

### Workspace design (2026-11-09 proposal)

A workspace is a root `tsrust.workspace.json` beside member projects that each
keep their ordinary `tsrust.config.json` (the existing schema, including
`build` and `api`). It works like a Cargo workspace or Turborepo: members stay
independent, and a backend never gains UI dependencies.

```json
{
  "schemaVersion": 1,
  "projects": {"api": "backend", "app": "frontend"},
  "clients": [
    {"project": "app", "from": "api", "language": "tsrust", "output": "src/generated/api"}
  ]
}
```

- `projects` maps names to relative member directories. Paths must stay inside
  the workspace; no member may contain another.
- `clients` is the generation graph: `from` names a member whose `api`
  configuration describes the contract, and `output` is relative to `project`.
  Languages are `typescript`, `rust` and, with the client step below,
  `tsrust`. Generated output is written atomically and checked for staleness by
  the existing `tsrust api generate --check`.
- At the root, `check`, `build`, `test` and `format` run every member in
  dependency order (contract members first, then clients regenerated or checked,
  then consumers); `--project NAME` selects one. Inside a member the CLI behaves
  exactly as today.
- `tsrust new NAME --workspace` scaffolds `backend/` (service) and `frontend/`
  (UI app), plus one client edge. `tsrust workspace add PATH` adopts an existing
  project without rewriting it.
- External consumers keep using `tsrust api generate` directly; the workspace
  only records edges and order.

## Bridge design (2026-11-09)

The bridge keeps today's JSX lowering (`<Tag {...}/>` is a call `Tag({...})`) and
adds one bundled module, `std:gpui`, backed by a small Rust runtime over GPUI Kit.

- Components run once. A function component's body executes at construction and
  returns a `Node`. Nothing re-runs the body; GPUI's repeated `render` only
  re-evaluates the node tree's live parts.
- `Node` is a retained description owned by its view: a Kit/intrinsic element
  kind, static props, children and live slots. The view entity stores the root
  node and builds GPUI elements from it in `Render::render`.
- `signal<T>(initial)` returns a native `Signal<T>` handle (`get`/`set`/
  `update`), `computed(() => ...)` a read-only `Computed<T>`. A signal records the
  views that read it during render and `set` calls `cx.notify()` on them, the
  GPUI-native invalidation path. Signals are shared handles (`Rc`), not cloned
  values; ownership rules stay explicit in the type surface.
- Live props: an attribute or child expression that reads a signal is checked as
  an ordinary expression and lowered to a closure evaluated at render time; other
  props stay construction-time values. The analysis marks the closure boundary,
  so `{count.get()}` updates while `{label}` does not.
- Events (`onClick`, `onChange`, ...) are typed Tsrust callbacks invoked through
  GPUI listeners on the UI thread. Controlled inputs pair `value` with `onChange`
  over Kit `InputState`, matching the admission slice.
- Conditional children (`{cond ? <A/> : <B/>}`, `&&`) and keyed lists are live
  slots with stable identity; removing a slot drops its subscriptions once.
- Kit components are compiler-owned wrappers in `std:gpui` with typed props
  records. `tsrust ui add` copies an editable `.tsrx` composition (variants,
  layout and defaults) into the project, shadcn-style; the bundled wrapper stays
  the stable boundary to GPUI Kit.
- `run(App)` opens the window (`application().with_assets(...).run`) with the
  Kit theme initialized; dependencies come from a separate locked GPUI
  dependency set, never the backend set.

First slice: counter, controlled input and conditional greeting, compared
side by side with `experiments/gpui-kit-admission` for behavior, binary size,
build time and startup time.

Owner decision, 2026-11-09: restore the earlier checked reactive lowering
(`signal`, `computed`, text bindings, branches, controlled inputs and their
closure adapters) instead of adding generic typed callbacks to the native
binding framework first. Its renderer is replaced, not revived:

- `std:ui` stays the public module name (backend-neutral, so the later WASM
  target keeps the same imports); GPUI is its desktop renderer.
- `crates/tsrust-ui` again holds the retained Node model, utility classes and
  theme, without vello/winit/accesskit or a browser renderer.
- `crates/tsrust-compiler/src/stdlib/ui/gpui.rs` renders a Node tree with one
  GPUI root view. Events update the tree, refresh its live bindings and call
  `cx.notify()`. Unclassed buttons and text inputs use GPUI Kit `Button` and
  `Input`; classed views and buttons resolve the utility classes into GPUI
  styles.
- `gpui-dependencies/` is the backend manifest plus GPUI Kit behind `ui`/`gpui`
  features, with its own lock seeded from the backend lock (every backend pin
  unchanged). Backend-only programs never resolve it.
- Programs that call `run` keep the OS main thread (GPUI requires it) instead
  of the 8 MiB worker thread other entrypoints use.

First slice result (2026-11-09, macOS arm64, debug):
`experiments/gpui-kit-admission/tsrust/main.tsrx` reproduces the Rust reference
(counter to 2, controlled input "Ada", conditional "Hello, Ada!"), driven by
synthesized clicks and keystrokes. Window-visible startup over 5 launches:
244 ms median for Tsrust and 253 ms for the reference. Binary size: 34 MB and
77 MB (the GPUI set builds dependencies without debug info). A cold first
build compiles GPUI Kit in about 3 minutes with 4 jobs; an application rebuild
reusing the cached GPUI support takes 12 s.

## Execution checklist after v0.2.0

Branch `codex/gpui-ui`. Push once, to merge, when the section is complete.

- [x] Workspace: `tsrust.workspace.json` members and client edges; root
  check/build/test/format with `--project`. Scaffolding (`new --workspace`,
  `workspace add`) follows with the UI CLI.
- [x] Admission: pinned `gpui-kit =0.9.0` direct-Rust reference slice (counter,
  controlled input, conditional child) in `experiments/gpui-kit-admission`
  builds with Rust 1.99.1 (1.96.1 lacks stable `Option::map_or_default`).
- [x] Removal: delete the ground-up renderer (`crates/tsrust-ui`, `std:ui`,
  `ui-runtime`, UI lowering in analysis/emit, `ui_project`, UI CLI, dev web/
  frontend, `legacy-ui`, vello/winit/accesskit dependencies), UI tests, examples
  and docs, and every Electron/React/Expo/Vite/shadcn template and pin. Keep the
  `.tsrx` parser, formatter, JSX syntax and editor parsing. Backend layers pass.
  JSX still lowers to plain component calls (`<Tag {...}/>` is `Tag({...})`);
  native support now always builds from the shared backend dependency set, whose
  UI-free native manifest dropped from 571 to 326 locked packages.
- [ ] Bridge (first slice done; remaining: Textarea, focus/active states,
  theme mode, Kit theme sync): `.tsrx` function components lower to GPUI views (signals become
  entity state plus `cx.notify`; events use `cx.listener`; intrinsic elements map
  to GPUI elements; Kit components bind through compiler-owned wrappers). Match
  the admission slice and compare against it.
- [ ] CLI: `tsrust new NAME --ui` (and `--workspace`), `tsrust ui add <component>`
  and `tsrust ui list`, with editor completion/hover for wrapper props.
- [ ] Catalog: every GPUI Kit 0.9.0 component wrapped, with an example each.
  62 components (display, controls, Kit-state inputs, overlays, layout, table)
  generated from `stdlib/ui/kit.json` and rendered in the gallery; remaining:
  List/Tree/VirtualList (row delegates), Command, Combobox, Calendar, TimeField,
  Editor, Chart, Notification, Carousel and Dock.
- [ ] Starter: `examples/gpui-starter`, generated by the CLI, a shadcn-like gallery
  of every component; verified in VS Code (LSP diagnostics, completion, hover),
  with measured startup time; regenerated on major changes.
- [ ] Desktop validation on macOS, Windows and Linux; then WASM.

The first executable step is admission plus the smallest shared reactive slice,
not installing every component before validating the bridge. The complete
catalog remains required scope. Design questions discovered by that slice should
be brought to the owner with concrete alternatives; routine integration choices
do not need repeated approval.
