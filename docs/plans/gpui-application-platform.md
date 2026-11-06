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

## Execution checklist after v0.2.0

- [ ] Reconcile retained UI/compiler/runtime behavior and freeze the first bridge
  contracts; admit a pinned GPUI Kit graph and desktop/WASM toolchain requirements.
- [ ] Prove a shared counter + controlled-input + conditional-child slice on
  desktop and WASM, including disposal, accessibility and direct-GPUI comparison.
- [ ] Add `.tsr` client generation with native/browser transport and the existing
  contract/compatibility acceptance; complete CLI/editor/docs examples together.
- [ ] Add shared-root setup/build/watch/codegen integration and migration tests;
  prove stale-contract errors, atomic output and clean offline release builds.
- [ ] Wrap the complete pinned component catalog and reproduce every inventoried
  upstream story on desktop/web, preserving explicit unsupported-platform gaps.
- [ ] Validate the basic applications and packaging on macOS/Windows/Linux and
  web; review correctness, accessibility, resources and measured bridge overhead.
- [ ] Review remaining experimental web limitations and the concrete release
  candidate with the owner before any publication.

The first executable step is admission plus the smallest shared reactive slice,
not installing every component before validating the bridge. The complete
catalog remains required scope. Design questions discovered by that slice should
be brought to the owner with concrete alternatives; routine integration choices
do not need repeated approval.
