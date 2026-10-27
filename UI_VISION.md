# Universal UI vision

> Active scope update, 2026-10-26: the owner approved the
> [backend platform pivot](docs/plans/backend-platform-pivot.md). This document
> preserves the universal UI design and historical decisions. Its unfinished
> renderer/component/embedded obligations are frozen, not active prerequisites.
> Presentation now uses React/Vite + shadcn on web/Electron and Expo + React
> Native Reusables on mobile. Follow [the prototype goal](docs/agent-goal.md).

Status: product direction refined with the owner on 2026-10-11; the checked `.tsrx` component and paired interactive counter foundation is implemented. The current compiler handoff records accepted behavior. Broader API examples below remain design sketches unless covered by that acceptance.

The [feasibility review](docs/plans/ui-feasibility-review.md#follow-up-review-after-app-directory-and-loader-decisions) preserves the original research and unresolved contracts. Its historical first-task sequencing is superseded by the current compiler handoff. State ownership and bounded browser/native counters now have focused acceptance; text input, dynamic trees, routing and navigation remain unfinished. Review recommendations remain distinct from owner-approved decisions.

Owner update (2026-10-18): `tsrust.config.json` owns inline `ui.theme` tokens.
Application source belongs under `src/`: the route tree is `src/app/`, reusable
controls are `src/components/`, and bundled controls default to
`src/components/ui/`. References to `app/` and `components/` in earlier examples
below are relative to `src/`. First `ui add` provisions missing settings and directories. `ui.target` explicitly
selects native/web booleans; embedded stays false until supported. Target selection
is interactive without flags in a terminal. Imports and compiler/editor analysis
remain read-only and report setup commands. The current
application guide records implemented behavior and remaining routing work.

## Agreed direction and open decisions

- Develop the shared API through paired web and desktop prototypes from the beginning. Neither target is merely a later port.
- Keep authoring familiar: JSX, `className`, token-based utilities and composable components. This does not promise React, arbitrary TypeScript, npm or full Tailwind compatibility.
- Aim for consistent component appearance and behavior while preserving platform shortcuts, text input, accessibility and OS dialogs. Pixel identity is not required for every font rasterizer or display.
- Custom-drawn desktop controls are the leading approach for consistency. Native windows and OS services remain appropriate; no desktop WebView dependency. The owner selected a GPU-first Winit + Taffy + Vello native foundation on 2026-10-18. This selects the rendering direction; generated paired UI acceptance remains open.
- Follow a paired counter with an editable list: add, select, edit, validate and remove items without requiring networking or a database.
- Prioritize core APIs and target interfaces. Embedded hardware is not required for initial acceptance. Headless software rendering can test constrained-display behavior but does not prove MCU timing, memory, drivers or `no_std` support.
- Reuse the existing compiler's language, types, ownership and module contracts. A separate UI crate or driver may be useful; a separate language implementation is not the intended direction.
- File-based routing inspired by the Next.js app directory is a priority. The owner prefers server rendering per request with partial navigation/data updates and persistent shared layouts. Static routes obtain their required data and generate all declared pages at build time. Desktop shares route paths and typed links for local navigation; web rendering settings apply only to web builds. Detailed loader, update and fallback contracts remain proposals in section 18.
- Not-found behavior must be explicit, editable application source. A generator may provide a default handler, but the framework must not silently substitute a hidden component. CLI generation should preview the destination relative to the current directory and confirm artifact kind. Dependency and parameter injection are desired ergonomics; their typed ownership/lifetime contracts remain to be designed.
- No separate `views/` convention is required. `src/app/` owns routes and route-local UI; `src/components/` owns reusable, editable `.tsrx` components. The owner wants shadcn-like CLI source installation from a catalog bundled offline in the binary, with immutable versions from the start and no automatic replacement of installed components when the CLI updates.
- The owner wants an Axum-backed Tsrust HTTP library with Hono-like ergonomics to make middleware, security and UI templating concrete. Use the [HTTP/UI contract proposal](docs/plans/axum-hono-http-ui-contract.md) to explore that boundary; exact APIs and implementation order are not yet accepted. Keep this within the existing HTTP/async roadmap rather than creating a competing server runtime.
- Use a dedicated `src/app/` route tree. Each segment, including the root, may define special layout, template, page, loader, not-found, error, route and middleware files. A colocated `loader.tsr` supplies type-safe data to its page by convention, without a page import or re-export. It runs only for that page's execution, never merely because a directory or API route is visited. Middleware is a sequential ancestor chain: every preceding segment's middleware runs before the selected handler unless an earlier middleware returns a response. Descendants cannot replace or bypass inherited middleware. Details below distinguish this owner direction from proposed lifecycle/dispatch rules.

### Application policy and framework guarantees

The owner's security concern primarily targets framework-origin vulnerabilities such as server-protocol code execution, middleware bypass, decoder denial of service and unintended source/data exposure. Use the [React/Next.js disclosure lessons](docs/plans/ui-feasibility-review.md#framework-vulnerability-lessons-from-react-and-nextjs) as concrete runtime design and acceptance inputs. Familiar app-directory conventions do not imply adopting RSC or a remote-function protocol.

The application author chooses authentication, record/tenant authorization, CSRF protection, session behavior, data disclosure, caching and deployment access policies. The framework supplies composable helpers and typed integration points; these policies are not mandatory middleware or global prerequisites for building the UI framework. A public app or a different authentication model can use different helpers or none. Generated starter choices must be visible and editable.

Framework guarantees are mechanical: run the installed middleware consistently across full-page, fragment, enabled data and mutation requests; preserve request/provider lifetimes; escape ordinary template text in its output context; keep server-only dependencies out of browser code; and make serialization and response selection explicit. A typed identity proves that a declared provider/middleware supplied a value, not that the application's authorization logic is correct. Missing a required injected capability is a composition/type error; not installing optional CSRF middleware is not itself a framework error.

For example, an application may install a CSRF helper that provides a typed token, pass that token through page/component props into a form, and have the helper validate submitted tokens on configured mutations. Token provision and validation are distinct. The application owns installing/configuring the helper and using the token; the helper owns correctness of its documented validation, and routing must not bypass it for enhanced requests.

Data sent to an authenticated browser is recipient-visible, not necessarily publicly accessible. Server-side loader-to-page injection does not automatically serialize the loader result. Rendering selected fields must not embed unused fields, context, providers or credentials. An explicit JSON response or client-data projection sends exactly its declared value under the documented serializer; the application chooses the fields and recipients. Type checking verifies representability/capabilities, not business confidentiality. The framework does not infer which ordinary strings or records are secret.

Likewise, static generation selects when data is evaluated, not who may read it. The application chooses build inputs, rendered data and host access controls. The framework documents snapshot behavior, does not invent request/session context at build time, and does not emit additional data beyond the selected output contract. A public static-host example may use public data; that is not a universal ban on access-controlled static deployments or trusted build credentials.

These decisions supersede the older native-controls-first preference and web-then-desktop prototype sequence in the historical desktop documents. Target-specific HTML/native capabilities remain typed; portable primitives must not erase their differences. See [the parallel UI plan](docs/plans/parallel-ui-foundations-plan.md) for work boundaries and acceptance conditions. Existing compiler and standard-library priorities remain in their own ledgers.

Your Role
Act as a principal compiler engineer, Rust systems architect, UI framework designer, and technical cofounder.
I am building a TypeScript-to-Rust compiler with the long-term goal of creating a TypeScript-first development platform capable of generating native Rust applications, web applications, server infrastructure, and embedded firmware.
I want you to help evaluate, architect, and progressively implement this vision.
Be technically rigorous and skeptical. Challenge architectural choices, identify hidden complexity, research established implementations, and favor proven abstractions over unnecessary reinvention.
Do not assume the existing compiler supports features it has not implemented. Inspect the repository before recommending changes or writing code.
Your job is to help build a viable system, not merely validate my ideas.

⸻

1. Current Compiler State
The compiler translates TypeScript into Rust.
Current development includes:
* Accepted project modules/editor support, generics and substantial ownership-aware closure and retained-callback support within documented boundaries.
* Accepted native binding/build foundations, Result, bytes/resources, collections and synchronous host APIs within their recorded platform scope.
* In-progress S10 JSON; type-safe immutable JSON/JSONC imports and aliases have focused compiler/native/editor and installed-consumer evidence, while JSON family acceptance remains open.
* Async/await has not yet been implemented.
* Named-component JSX and `.tsrx` tooling are accepted. Generated static `std:ui` trees now produce semantic HTML and native GPU output. The bounded paired counter now has component-owned integer state, retained handlers and reactive Text children. State is referenced directly in JSX and arrow callbacks; get/set are state primitives, while arithmetic belongs to application handlers. No public lifetime-attachment or handle helpers are required. Browser input/disposal and native presentation/event-path witnesses pass. General reactive properties, dynamic trees, input/IME and async UI remain open.
* The owner selected WebAssembly for the paired counter on 2026-10-18. This bounded choice reuses checked Rust handlers; it is not a permanent browser-backend commitment.
Use the [standard-library roadmap](docs/plans/standard-library-roadmap.md), latest acceptance records and actual source rather than historical progress paragraphs to establish current support. Existing closure support is not proof of a shared reactive-state contract.
Do not assume JSX, closures, generics, reactive state, complex module resolution, or other language features are working until you verify their implementations.
The TypeScript-to-Rust compiler is the foundation of this project. The UI framework must be designed to work with its architecture, not require a completely separate language implementation.
Before proposing implementation work, inspect:
1. Parsing and AST representation.
2. Type checking and type information.
3. Intermediate representations and lowering passes.
4. Rust code generation.
5. Ownership and borrowing strategies.
6. Module resolution and imports.
7. Existing standard-library implementations.
8. JSON import handling.
9. Compiler testing and diagnostics.
Identify which existing components can support the proposed UI architecture.

⸻

2. Overall Product Vision
The goal is a unified development experience in which developers write TypeScript and TSX and produce:
Backend Applications
* Native Rust executables.
* Command-line applications.
* HTTP servers and APIs.
* Microservices.
* Server-side rendering infrastructure.
Web Applications
* Real HTML and CSS.
* Static site generation (SSG).
* Server-side rendering (SSR).
* Partial hydration and interactive islands.
* Reactive client-side interactions.
* Potential Rust/WASM or generated JavaScript execution.
Web rendering should primarily use native browser capabilities rather than drawing an entire web application onto Canvas.
Native Desktop Applications
* Windows, macOS, and Linux.
* Actual native executables.
* Native OS windows and input handling.
* Consistent user interfaces, with custom drawing as the leading renderer strategy.
* No HTML or WebView dependency.
* Shared TSX components and design tokens.
* GPU-accelerated rendering where appropriate.
Embedded Applications
* Embedded Linux devices.
* ARM systems and industrial touch panels.
* Raspberry Pi-class hardware.
* Eventually microcontrollers such as ESP32 and STM32.
* Custom-drawn displays.
* Software rendering and framebuffer output.
* Restricted-memory and potentially no_std execution.
* Hardware-specific optimizations and capability checks.
The defining experience should be:
Write TypeScript/TSX once, reuse components and behavior where practical, and compile to the appropriate native representation for each target.
Cross-platform consistency should not require identical binaries, rendering engines, or hardware capabilities.

⸻

3. Core Architectural Philosophy
Build a compiler-first UI system inspired by React’s developer ergonomics, Solid’s fine-grained reactivity, Tailwind’s utility styling, and shadcn’s customizable components.
However, do not simply recreate React in Rust.
The compiler should understand UI components, templates, styling, static content, dynamic bindings, and target capabilities.
Prefer:
Statically compiled UI templates with explicitly identified reactive regions.
The compiler should attempt to identify:
* Static component structures.
* Dynamic properties and text.
* State dependencies.
* Conditional rendering.
* Dynamic lists.
* Event handlers.
* Static style definitions.
* Dynamic style variants.
* Layout constraints.
* Target-specific capabilities.
* Compile-time optimizations.
A general-purpose virtual DOM and runtime reconciler should not be required by default.
Evaluate where dynamic reconciliation genuinely becomes necessary, especially for arbitrary runtime component creation.
A component’s structure may be compiled at build time, instantiated at application startup, and selectively updated at runtime.
Do not assume every layout can be fully computed at build time.

⸻

4. Compiler and UI Intermediate Representations
Investigate an architecture involving the following stages:
TypeScript/TSX source
→ Parsing and type checking
→ Typed compiler IR
→ UI semantic IR
→ Target-specific lowering
→ Rust, HTML/CSS, or other generated artifacts
The semantic UI representation should describe:
* Component identity.
* Component hierarchy.
* Props and types.
* Accessibility semantics.
* Layout constraints.
* Style references.
* Tokens and variants.
* Events.
* Reactive bindings.
* Static and dynamic regions.
For native and embedded targets, introduce a separate scene representation describing:
* Resolved geometry.
* Text and glyph layouts.
* Images.
* Rectangles and paths.
* Clipping.
* Transforms.
* Draw order.
* Dirty regions.
The rendering backend should not need to understand TypeScript components.
Avoid treating HTML, CSS, or physical pixel coordinates as the universal UI representation.
Investigate whether UI-specific lowering should extend an existing typed IR or introduce a distinct representation.
Define ownership and lifecycle contracts between the generated application, UI runtime, layout system, and renderers.

⸻

5. Universal UI Package
Design a developer-facing UI package with familiar TSX syntax.
An illustrative API:
import {
  View,
  Text,
  Button,
  signal,
} from "@platform/ui";

export function Counter() {
  const count = signal(0);
  return (
    <View className="flex-col gap-4 p-4 bg-background">
      <Text className="text-xl font-bold">
        Counter
      </Text>

      <Text>{count.value}</Text>

      <Button
        variant="primary"
        onPress={() => count.value++}
      >
        Increment
      </Button>
    </View>
  );
}
The exact API is not finalized. This sketches state owned by each component instance, not an executable module-level singleton. Before implementation, specify how the text binding and retained handler access that state, when subscriptions run, and how both are disposed. Application-wide and request-scoped state require separate explicit scopes; no implicit shared capture cells or clones are authorized.
Design reusable primitive components such as:
* View
* Text
* Image
* Button
* Stack
* Grid
* ScrollView
* TextInput
* Switch
Higher-level components should be composable, similar to shadcn/ui.
Keep component semantics separate from renderer implementations.
Consider how accessibility, keyboard focus, pointer events, touchscreen interactions, and input devices map across platforms.
Identify features that cannot be represented consistently across targets and establish explicit capability rules.

⸻

6. Universal Design Tokens and Styling
I want a styling system inspired by Tailwind and shadcn.
It must be token-customizable and capable of targeting web, native desktop, and embedded displays.
Design tokens should be platform-independent rather than fundamentally CSS variables.
Given the compiler’s existing support for type-safe JSON imports, investigate whether JSON can be the initial source of truth for theme configuration.
Example:
{
  "colors": {
    "background": "#FFFFFF",
    "foreground": "#111827",
    "primary": "#2563EB",
    "primaryForeground": "#FFFFFF"
  },
  "spacing": {
    "sm": 4,
    "md": 8,
    "lg": 16
  },
  "radius": {
    "sm": 4,
    "md": 8,
    "lg": 12
  }
}
Support semantic token references and a practical subset of Tailwind-like utilities.
For example:
<Card className="bg-card p-4 rounded-lg">
  <Text className="text-primary font-semibold">
    Hello
  </Text>
</Card>
Investigate generating:
Web: CSS variables, optimized stylesheets, reusable CSS classes.
Desktop: Typed Rust style structures and runtime-switchable theme values.
Embedded: Compile-time-resolved style tables, static constants, limited dynamic theme capabilities, and device-appropriate color formats.
Distinguish semantic design units from device pixels.
Evaluate:
* Compile-time class extraction.
* Static style deduplication.
* Dynamic variant support.
* Dark/light themes.
* Responsive layouts.
* Density scaling.
* Font scales.
* Color conversion.
* Unsupported styling diagnostics.
* Dead-style elimination.
Do not assume all CSS capabilities can or should exist on embedded systems.

⸻

7. Web Compilation and Server-Side Rendering
The preferred web target generates actual semantic HTML and CSS.
Support three eventual modes:
Static Site Generation
Evaluate static TSX components at build time and produce HTML/CSS.
Server-Side Rendering
Generate Rust rendering functions capable of producing safely escaped HTML for request-specific data.
The architecture should integrate naturally with Rust servers such as Axum.
Interactive Hydration
Identify interactive regions and generate the minimum necessary browser-side behavior.
Investigate both:
* Rust/WASM hydration.
* Generated JavaScript for client interactions.
Do not commit to one before comparing bundle sizes, startup latency, runtime complexity, and interoperability.
The system should support server-only code boundaries so credentials and server dependencies cannot accidentally enter client bundles.
Explore islands-style hydration to minimize browser-side runtime costs.
Preserve:
* Native DOM accessibility.
* Browser text selection.
* Standard forms.
* Keyboard navigation.
* SEO.
* Browser navigation.
* Progressive enhancement where practical.

Rendering time and interactivity are independent. Build-generated HTML may contain interactive controls; request-rendered HTML may contain none. Early paired prototypes need minimal browser interaction without requiring the complete SSR/hydration stack. Section 18 proposes explicit route rendering modes without silently changing them based on compiler optimization.

⸻

8. Native Desktop Rendering
The native desktop target must not rely on HTML or WebViews.
Investigate:
* Winit for window management and input.
* Wgpu for GPU abstraction.
* Taffy for layout.
* Custom or existing Rust 2D renderers.
* Text shaping and font rasterization libraries.
Separate:
1. Window and surface management.
2. Input and platform events.
3. Layout.
4. Scene generation.
5. Rasterization and GPU drawing.
6. Accessibility integration.
Avoid building an entire graphics engine before evaluating reusable Rust rendering libraries.
The output should be a standalone native Rust executable generated from TypeScript/TSX.
Include basic high-DPI behavior, keyboard focus, accessible control semantics and deterministic window teardown in early acceptance. Advanced multi-window support, packaging and broader native integration follow later.

⸻

9. Embedded Rendering
Embedded Linux and bare-metal/RTOS microcontrollers must be treated as distinct deployment profiles.
For embedded Linux, investigate reusing the desktop UI runtime with software or GPU rendering.
For constrained microcontrollers, investigate:
* no_std execution.
* Minimal runtime dependencies.
* Fixed-capacity arenas and collections.
* Static component structures.
* Compile-time asset conversion.
* Font subsetting and rasterization.
* RGB565 output.
* Partial redraws.
* Dirty rectangle tracking.
* Scanline and tiled rendering.
* Direct display-driver integration.
* Predictable memory requirements.
Study Slint’s software-rendering implementation and embedded-graphics.
Do not assume all shared runtime crates can use heap allocation or the Rust standard library.
Explore how UI templates and assets can be stored in read-only memory while dynamic state remains small and bounded.
A realistic first hardware target should be selected only after comparing its RAM, flash, display resolution, graphics hardware, and Rust tooling.
The compiler should reject unsupported features with meaningful diagnostics.

Initially exercise the interface through a headless software target with fixed viewport, supplied input events, restricted colors and inspectable redraw output. This is a portability experiment, not a hardware emulator or embedded acceptance claim. Real hardware selection and `no_std`/allocation constraints follow a separate feasibility decision; they do not block the shared API prototypes.

⸻

10. Reactivity and Runtime Model
Investigate fine-grained, signal-based reactivity rather than defaulting to whole-component re-rendering.
The compiler should map reactive expressions to dependency information where feasible.
Do not assume all dependencies are statically knowable. Specify dynamic subscription changes for conditional reads, or diagnose unsupported forms. Component-owned state and handlers must have an explicit lifetime contract before generated interaction is accepted.
When state changes, invalidate affected semantic nodes, layout regions, and/or drawing regions as appropriate.
Consider:
* Stable node identity.
* Conditional insertion/removal.
* Dynamic lists and keys.
* Nested state dependencies.
* Event handlers.
* Component lifetimes.
* Cleanup and disposal.
* Reactive scheduling.
* Layout invalidation.
* Graphics invalidation.
* Cross-thread safety.
* Embedded memory constraints.
Do not conflate reactive updates with asynchronous I/O.
Basic interactive UI should not require async/await.
Separate synchronous UI event processing from future asynchronous networking and background tasks.

⸻

11. Rust and JavaScript Library Integration
The broader compiler ecosystem should eventually offer TypeScript-friendly APIs backed by established Rust crates.
Potential integrations include:
TypeScript ecosystem	Rust ecosystem
Zod	Serde + validation
Hono	Axum + Tower
Promise/async APIs	Tokio
Fetch	Reqwest
React-style UI	Custom compiled UI engine
Database APIs	SQLx
Investigate a declarative library-mapping system rather than building every integration as a custom compiler special case.
Differentiate:
* Full API compatibility.
* Partial compatibility.
* Native alternative APIs.
* Compile-time transformations.
Do not claim behavioral equivalence where semantics differ.
Especially investigate JavaScript/Rust mismatches around ownership, object semantics, async execution, exceptions, mutation, and runtime validation.

⸻

12. Existing Projects to Research
Use official documentation and source repositories as primary references.
Study design and implementation techniques rather than blindly copying architectures.
Compiler-Driven and Reactive UI
SolidJS — Fine-grained reactivityhttps://docs.solidjs.com/advanced-concepts/fine-grained-reactivity
Study dependency tracking, targeted DOM updates, signals, and component execution behavior.
Leptos — Rust SSR and hydrationhttps://leptos.dev/
Study Rust server rendering, hydration, reactive state, and islands.
Dioxus — Rust cross-platform UIhttps://dioxuslabs.com/
Study cross-platform component architecture, rendering backends, SSR, and tradeoffs involving virtual DOMs.
Layout and Graphics
Taffy — Rust layout enginehttps://github.com/DioxusLabs/taffy
Study Flexbox/Grid computation without browser DOM dependencies.
Wgpu — Cross-platform GPU abstractionhttps://wgpu.rs/
Study graphics portability across native environments.
Winit — Window and event managementhttps://github.com/rust-windowing/winit
Study lifecycle management, input events, and cross-platform window surfaces.
Embedded UI
Slint — Compiled native and embedded UIhttps://slint.dev/
Slint software rendererhttps://docs.slint.dev/latest/docs/rust/slint/platform/software_renderer/struct.SoftwareRenderer
Study compiled UI components, embedded execution, framebuffer rendering, dirty regions, and line-by-line software rendering.
Embedded Graphicshttps://github.com/embedded-graphics/embedded-graphics
Study allocator-free drawing primitives and display-driver abstractions.
Design System
Tailwind CSShttps://tailwindcss.com/docs/theme/
Study utility classes, theme namespaces, and design token generation.
shadcn/uihttps://ui.shadcn.com/docs/theming
Study semantic tokens, composable components, customization, and developer-owned component source.
Additional Architectural References
Sveltehttps://svelte.dev/docs/svelte/overview
Study compiler-generated UI updates and the division between compile-time and runtime behavior.
Astrohttps://docs.astro.build/en/concepts/islands/
Study static generation, component islands, and selective client-side hydration.
For each project, document:
1. The architectural technique worth adopting.
2. The underlying implementation details.
3. Relevant source files or modules.
4. Constraints and limitations.
5. Licensing considerations.
6. Whether the approach is appropriate for web, native, embedded, or all three.
7. Whether it is feasible within the current compiler architecture.
Verify current APIs and project status before making implementation recommendations.

⸻

13. Proposed Repository Architecture
Evaluate a structure conceptually similar to:
packages/
  compiler/
  ui/
    core/
    tokens/
    signals/
    components/

crates/
  ui-core/
  ui-style/
  ui-layout/
  ui-reactivity/
  ui-scene/
  ui-text/

  render-wgpu/
  render-software/
  render-embedded/

  platform-web/
  platform-desktop/
  platform-linux/
  platform-mcu/

examples/
  static-page/
  counter/
  dashboard/
  desktop-app/
  embedded-screen/
Do not automatically impose this structure.
Review the current repository organization and propose changes only when justified.

⸻

14. Implementation Strategy

Favor small, independently testable vertical slices with paired web/desktop evidence.

1. Freeze a small provisional contract for primitives, tokens, state lifetime, target capabilities and routing. Establish source-backed compiler dependency witnesses.
2. In an isolated worktree, develop token validation, a minimal semantic UI description, route-manifest fixtures and paired backend adapters. Manually constructed descriptions are allowed for backend experiments; they do not count as compiled Tsrust UI acceptance.
3. Coordinate `.tsrx` parsing, shared checking, UI lowering, source provenance, formatting and editor work with the compiler owner. Generate one token-themed static component into semantic HTML/CSS and a native window from the same source.
4. Prove a paired interactive counter: independent instances, matching state transitions, pointer/keyboard activation, disabled behavior, targeted binding updates and cleanup. Settle a bounded browser-execution approach for this slice; do not silently turn a spike into the final JS/WASM decision.
5. Build the shared editable list with stable item identity, validation, focus, text editing and disposal. Add `/items` and `/items/[id]` navigation when routing semantics are agreed. Initial data is local; persistence/networking is separate.
6. Add explicit static route generation and request-rendering adapters as their contracts and dependencies land. Synchronous HTML rendering can precede HTTP/async; a deployed dynamic web server cannot be claimed before its serving dependencies work.
7. Continue constrained-display tests independently. Select real embedded hardware later and measure its actual limits. Expand SSR, hydration, async services and platform support through separate milestones.

The [parallel UI foundations plan](docs/plans/parallel-ui-foundations-plan.md) defines what can start now, shared-file ownership, stopping boundaries and proof obligations. Do not begin framework implementation merely because this planning document exists.

⸻

15. Non-Negotiable Design Goals
* TypeScript-first developer experience.
* Idiomatic and inspectable generated Rust wherever practical.
* Strong static type safety.
* Predictable runtime behavior.
* Minimal unnecessary runtime overhead.
* Reusable, token-driven UI components.
* Native HTML/CSS web output.
* Consistent native desktop UI, evaluated first with custom-drawn controls.
* Embedded-friendly rendering.
* Explicit target capability profiles.
* Clear compiler diagnostics.
* Testable cross-target semantics.
* Progressive adoption of advanced language/runtime features.
* No WebView dependency for native UI.
* No requirement for a React runtime.
* No dependency on async for basic UI interaction.
Treat these as design goals to test, not as claims that every target can achieve perfect feature parity.

⸻

16. What I Want From You First
Do not start by building a complete UI framework.
First, inspect the existing repository and produce:
A. Current Compiler Capability Assessment
Identify what exists, what is incomplete, and which assumptions need verification.
B. Recommended Architecture
Propose concrete compiler passes, IR schemas, runtime boundaries, and backend contracts.
Include diagrams and Rust/TypeScript interface sketches.
C. Technical Decision Record
Compare major architectural choices and explain recommendations.
In particular:
* Static templates vs virtual DOM.
* Signals vs component re-execution.
* Shared rendering vs target-specific rendering.
* Rust/WASM vs generated JavaScript hydration.
* Taffy vs alternatives.
* Custom rendering vs integration with existing UI engines.
* std vs no_std runtime separation.
D. Dependency and Risk Analysis
Identify blockers involving language semantics, ownership, rendering, layout consistency, hydration, accessibility, and embedded constraints.
E. Smallest Viable Prototype
Recommend the smallest end-to-end demonstration that proves the architecture.
Prefer a simple token-themed TSX component compiled into both HTML/CSS and a native Rust application.
F. Implementation Plan
Break the prototype into small tasks with:
* Files/modules affected.
* Required compiler functionality.
* Dependencies.
* Acceptance criteria.
* Tests.
* Expected generated output.
G. Research Evidence
Cite authoritative documentation and specific repository implementations for all important architecture recommendations.
Distinguish demonstrated implementations from speculative techniques.

⸻

17. Collaboration Rules
Work incrementally.
Before large changes:
1. Explain the problem being solved.
2. Identify existing code that could be reused.
3. Propose the smallest reasonable implementation.
4. Explain tradeoffs.
5. Specify tests.
6. Identify possible regressions.
Do not silently introduce major dependencies or rewrite compiler architecture.
When uncertain, present alternatives with evidence.
Prioritize correctness, simplicity, observability, and maintainability over ambitious feature counts.
Challenge me when I propose something technically unsound or unnecessarily complex.
The ultimate objective is a TypeScript compiler and application ecosystem where a developer can build web applications, native desktop applications, embedded interfaces, and Rust-backed services from one coherent language and component model—without sacrificing the native strengths of each target.

## 18. File-based routing and rendering: proposal for discussion

The owner wants app-directory conventions, path/link discovery, bracketed parameters and dynamic-by-default behavior with explicit per-file configuration for static/generated output. Follow-up direction: prefer server rendering per request while retaining shared layouts during partial navigation/data updates; use loader-like data access to generate all declared static pages at build time; share routes and local navigation with desktop. An htmx-like approach is acceptable to investigate, not a selected dependency. Exact APIs and the remaining rules below are proposals, not a claim of Next.js compatibility.

### Paths and composition

```text
app/
  layout.tsrx                 shared application shell
  page.tsrx                   /
  items/
    page.tsrx                 /items
    [id]/
      page.tsrx               /items/:id
  docs/
    [...slug]/
      page.tsrx               /docs plus one or more segments
```

Proposed first conventions:

- `page.tsrx` creates a routable screen; ordinary colocated files do not.
- `layout.tsrx` composes the shell around descendant pages. Shared layouts should stay mounted during child navigation, preserving header state while the main outlet changes. Remount keys, changed layout parameters and state disposal must be specified before interactive navigation is accepted.
- `[id]` supplies one decoded string parameter. `[...slug]` supplies a nonempty sequence; `[[...slug]]` would allow zero segments. Numeric/schema conversion must be explicit. Optional catch-all support can follow the first matcher slice.
- `(group)` organizes files without contributing to the URL. Duplicate resulting routes are errors, including collisions across groups. Groups can be deferred without changing basic paths.
- Static segments take precedence over parameters, then catch-alls. Structurally equivalent parameter routes with different names are an error, not filesystem-order selection.
- Path matching and link generation share a canonical manifest. Decide case, trailing slash, percent encoding, encoded separators, malformed input and query repetition rules before freezing it; test path building/matching round trips and ambiguity diagnostics.
- Generated route identities and parameter types support checked internal links. A familiar `<Link href="/items">` is a candidate surface; parameterized links need a checked builder or structured `href`, with the exact spelling still open. External links remain explicit and preserve browser anchor behavior.
- Query/search values are separate from route parameters. A query change must not silently enumerate a new static output file or imply trusted/validated input.
- The special-file vocabulary includes HTTP handlers, middleware and error boundaries as described below. Implementation can stage these capabilities; an unsupported special file must receive a diagnostic rather than silently becoming an ordinary file. Parallel/intercepted routes and loading conventions remain separate increments. File routing does not imply React Server Components, server actions or Next.js caching semantics.

### Segment special files and sequential middleware

Every segment, including `app/` itself, can supply the following files. The vocabulary and sequential middleware direction are owner-requested; lifecycle details and export signatures below are recommendations pending their acceptance fixtures. Only root not-found is currently proposed as required; do not force empty boilerplate for every file in every directory.

| File | Proposed responsibility |
|---|---|
| `layout.tsrx` | Persistent wrapper for the segment and descendants; retain mounted state while its identity remains unchanged |
| `template.tsrx` | Optional wrapper intentionally remounted when its declared navigation identity changes; state/effects reset, unlike layout |
| `page.tsrx` | UI screen at the exact path; supplies the page data/render contract |
| `loader.tsr` | Optional conventional data loader for the colocated page; inject its checked successful result into page props without imports/re-exports |
| `not-found.tsrx` | Visible application-owned missing-resource UI; nearest applicable boundary, falling back to the required root file |
| `error.tsrx` | Visible failure boundary with sanitized context and explicit recovery; no recursive handling of its own failure |
| `route.tsr` | Explicit HTTP method handlers at the path; HTTP-only capability, not automatically a native navigation screen |
| `middleware.tsr` | Ordered middleware for this segment and descendants, wrapping all request representations |

`.tsrx` marks proposed UI syntax; `.tsr` remains ordinary language syntax. Names are reserved only within the configured app convention, not globally across all source folders. A directory with wrappers/middleware but no page/route still contributes to descendants without creating an endpoint.

Loader association is exact and local: `app/items/loader.tsr` supplies `app/items/page.tsrx`, not every descendant page. A page without a loader is valid and has no implicit `data` value. A loader without a page never runs or creates an endpoint; tooling should report it as unpaired. A colocated API method does not invoke the page loader. Successful mutation navigation may subsequently perform a page GET, which runs its loader through the normal middleware chain.

The compiler derives the page's injected `data` type from the loader's checked successful result, including awaited results. Unknown property access or incompatible page annotations are diagnostics; no `any`, unsafe cast, runtime object guessing or evaluation of the loader during type checking. Not-found, redirect and failure outcomes bypass ordinary page rendering rather than being passed as page data. Loader edits must invalidate the dependent page's checked facts and editor help. Middleware-provided params/identity/dependencies likewise require checked context contracts.

For HTTP, run the loader after inherited middleware succeeds on page document/fragment/explicit page-refresh requests. For static pages, use the agreed build-time execution context and enumerated inputs; discovery/editor analysis never executes it. Native page loading uses declared native-capable providers. There is no implicit inheritance of parent page loaders for layout data and no public JSON endpoint just because a loader file exists. Loader function export spelling and runtime cancellation/revalidation remain implementation contracts to settle; automatic discovery and typed result injection are the owner-approved direction.

Example chain for `/items/4`:

```text
app/middleware.tsr
  → app/items/middleware.tsr
    → app/items/[id]/middleware.tsr
      → selected page/loader or HTTP method handler
```

Run middleware in ancestry order, root to leaf, never concurrently. If one file exports an ordered collection, run it in declared order. Each middleware can return an early response or await `next`; successful response processing unwinds leaf to root. `next` may run at most once and may not escape. Later middleware and the handler do not run after an early response. Ancestor checks are mandatory: child declarations only add behavior. This includes direct HTML, fragment, enabled data and mutation requests; generated endpoint plumbing must not introduce an alternate unprotected path.

Middleware runs for requests even if a browser layout stays mounted; layout persistence never implies an authorization cache. Typed context gains declared capabilities only after the relevant middleware succeeds. A region/request header cannot choose which middleware runs. Static artifact access is the application/deployment owner's policy; build generation does not run session middleware with an invented request. HTTP middleware is not silently executed as desktop navigation logic: shared application guards need an explicit target-independent contract.

Before implementing: define matched ancestry for unknown paths, fallbacks, method-not-allowed and preflight requests. Root middleware must cover fallback handling; do not use a raw string prefix (`/items` matching `/items-private`) as route ancestry. Protected-segment not-found responses must not become a bypass around applicable policies.

Owner-approved dispatch direction: `page.tsrx` and `route.tsr` are independently optional. A segment may contain neither, either, or both. A page owns GET at its segment path, so a colocated route cannot register `get("/")`; it may register nonconflicting methods such as `post("/")`. With no page, a route may register `get("/")`. Neither file is required at the root. Proposed HTTP detail: reserve HEAD alongside page GET and diagnose explicit HEAD conflicts. Duplicate method/path ownership, including generated data/fragment endpoints, fails at build time. Full HTML versus fragments is one page handler with representation selection after the common policy chain. Do not silently choose between a page and an API GET based on filesystem order or a client-controlled header. Relative route subpaths, if supported later, must also enter the common manifest and collision checks.

See the [concrete editable-list app example](docs/plans/editable-list-app-example.md) for proposed middleware/auth, server loaders, pages, method routes and full/partial request flows. Its code is an API design sketch, not accepted compiler syntax.

Define template navigation keys (params/query/refresh), layout identity and boundary scope before implementation. A data refresh should not reset templates by accident. UI fallbacks must not wrap JSON API responses unless explicitly configured. A failing boundary escalates outward; root-boundary failure uses a minimal safe server failure response, not a hidden replacement component. Layout/middleware failures require a boundary outside the failed dependency; an error file cannot be assumed to catch its own initialization.

### Separate three concepts

1. **Path shape:** literal or parameterized, such as `/items/[id]`.
2. **Rendering time:** runtime or build time.
3. **Interactivity:** browser/native event behavior after the initial view exists.

A parameterized route can be generated at build time. A build-generated page can remain interactive. An optimizer can hoist static markup inside a dynamic page without changing when route data is evaluated or cached.

Direction agreed; exact configuration and fallback rules remain provisional:

| Mode | Meaning | Unknown parameter values |
|---|---|---|
| Dynamic (default) | Server-render a full document on direct entry, or a requested region on enhanced navigation/data refresh; no implicit shared rendered-page cache | Match and run the route; the application may return not found |
| Static | Enumerate route inputs, load their required data and produce all declared pages at build time | Use the application's explicit not-found handler for values absent from the build's complete data/path set; no implicit runtime fallback |
| Generated on demand / revalidated | Future mode with explicit cache key, invalidation, concurrency, failure and deployment semantics | Deferred; never implied by `static` |

Here, “generated” initially means static generation, not a separate ambiguous execution mode. Client-only web applications are a possible explicit deployment mode, not another meaning silently assigned to `dynamic`.

### Partial navigation and data updates

Direct entry and refresh must produce a complete document. Enhanced internal navigation preserves the common layout instances and replaces the changed page/outlet. Browser back/forward, canonical URLs, title/head updates, focus/scroll restoration, loading/error behavior and stale-response suppression belong to the navigation contract. A normal link still supports direct entry and opening in a new tab.

There are two distinct update paths to evaluate:

- **Server-rendered regions:** run the relevant loader/render operation and return HTML for a declared boundary. This is the leading initial web transport experiment, inspired by htmx's targeted swaps. It does not require compiling arbitrary application logic to JavaScript.
- **Typed data refresh:** return checked serializable loader data and update bindings in a client-owned region. This requires a client execution/serialization contract; it is not free merely because partial HTML updates work. It can follow the first fragment experiment.

Keep these as web adapters behind shared route, loader and state interfaces. Desktop updates native nodes/state and never consumes HTML fragments. Local counters, unsaved form edits and immediate selection should remain local where possible; server-first rendering does not require a request for every keystroke or event. The paired local counter still requires a bounded browser runtime decision independently of fragment navigation.

Assign each region one update owner. Replacing a region disposes its handlers/subscriptions before mounting the replacement; preserve unaffected layout state. Do not let fragment replacement and a client reactive runtime concurrently own the same nodes. Test that late responses cannot overwrite a newer route and that data refresh does not silently erase dirty input. Define explicit layout invalidation when header data really changes; persistent layout does not mean permanently stale data.

Full-document and fragment responses need distinct request/response contracts and correct cache separation. Server loaders and credentials stay server-side; only application-selected HTML or explicitly selected serializable data crosses to the browser under the chosen access policy. Recipient-visible does not mean public to everyone; loader injection does not imply whole-result serialization. Request rendering must not retain one user's mutable state for another request. These requirements also apply if htmx is selected; it supplies transport/DOM behavior, not Tsrust's ownership or security boundaries.

### Loaders and complete static generation

A parameter enumerator answers **which pages exist**; a loader answers **what data a particular page needs**. Keep these contracts distinct even if both use the same data source. A loader for `/items/[id]` cannot by itself establish every possible `id`.

Proposed flow:

```text
Dynamic direct entry: params → server loader → layout + page → full HTML
Dynamic navigation:   params → server loader → changed outlet → HTML region
Static build:         enumerate params → loader for each → complete page artifacts
Desktop navigation:   params → compatible local loader/adapter → native outlet update
```

Enumerators must supply a finite, validated set of parameter values, including full nested parameter combinations. Reject duplicate output paths and mismatched parameters. Load every declared page and its required layouts; fail the build on missing required data or failed generation rather than publishing an apparently complete partial site. An explicit skip/not-found policy can be designed later. A successful build emits the generated-path manifest. Values not enumerated are outside that static build; no runtime fallback or on-demand generation is silently assumed.

**Owner clarification: actual data determines existence.** If the declared build dataset contains IDs `1`, `2`, `4`, generate those three pages. IDs `3`, `5` are absent; never infer a contiguous range, fill gaps or impose an arbitrary generation cap. A failed loader for existing ID `4` is a build failure, not a missing page. Enumeration must finish successfully before the output is accepted. Completeness relative to a live database requires the data adapter's snapshot/pagination contract; the framework cannot prove that a user-written enumerator did not omit a row. Intentionally filtered datasets must be explicit.

A static deployment reflects its last build snapshot. Rows added later cannot appear until a new build, and rows deleted later may retain their generated page until then. Live existence checks require an explicit runtime service and are a different deployment capability. This limitation must be visible in build/deployment documentation rather than claiming current database knowledge.

### Explicit application-owned not-found behavior

Require a root not-found handler for routable apps. Proposed convention: `app/not-found.tsrx`, with an ordinary exported component/function that the developer can inspect and edit. Project initialization may generate a useful default in the confirmed destination; route validation reports the missing file if it is absent. No invisible fallback UI. The full app-directory design permits nested not-found boundaries; the initial prototype may support only root handling if it explicitly rejects unsupported nested declarations. Nearest-applicable selection and middleware/error scope must be tested before nested acceptance.

Invoke it for unmatched routes, absent generated paths and an explicit loader not-found outcome. Keep malformed parameters, unauthorized access and infrastructure/loader failures distinct; they are not automatically 404s. A declared static path whose required data disappears during generation fails the build under the initial contract.

The handler controls presentation and recovery links; the web adapter preserves HTTP 404 status for direct requests. Enhanced navigation must display the not-found region rather than discard it as a generic transport error. Desktop invokes the same semantic handler with a local navigation outcome. Static exports emit a 404 artifact and hosting requirements; verify host configuration actually returns status 404, not merely a successful response containing an error page. The handler must not depend on the missing page's data/provider succeeding.

The owner requires loader-style data access and complete build-time output, not a particular function name. `generateStaticParams` below is a candidate name; `loader` and its typed context/result remain to be designed. Prefer the same page-data contract across request and build execution, with execution-specific capabilities: static loaders cannot read a user's request/session. Server-only loaders need explicit native counterparts or target diagnostics; sharing routes does not make database credentials portable.

Static enhanced navigation must work from published artifacts without a rendering server. Evaluate prebuilt fragments or extracting the declared outlet from a full generated document; choose one before accepting static partial navigation. Local edits do not rewrite those build artifacts. Live persistence or refresh needs an explicit service/data source and is separate from static page generation.

### Candidate configuration surface

```typescript
// Proposed metadata in app/items/[id]/page.tsrx; not supported syntax today.
export const config = {
  web: { render: "static" }
};

// Proposed separate build hook; exact result annotation/API remains open.
export function generateStaticParams() {
  return [{ id: "first" }, { id: "second" }];
}
```

Omitting metadata would retain the dynamic default. Nesting web settings makes the target scope visible if desktop shares the route tree. Configuration should be a checked literal metadata declaration: no arbitrary side-effectful module execution just to discover routes. The current compiler does not generally support exported executable top-level constants; either a narrow metadata form or a broader approved language extension is required. Do not special-case it without compiler/editor agreement.

Recommended resolution rule: project web default, overridden explicitly by a page. Initially, layout files do not implicitly change descendant rendering mode. Build-time rendering still checks the entire selected layout/page dependency graph. Report effective settings in diagnostics/build output; reject unknown keys and conflicting or unsupported options.

Static mode must fail with source diagnostics if it requires request-specific cookies, headers, session state or undeclared runtime data. Never silently make it dynamic or expose request data in a generated artifact. Start with explicit local build inputs; remote build data needs an explicit reproducibility and credential policy. Build hooks must not run during editor analysis. Static-only deployment rejects unresolved dynamic routes rather than silently changing their semantics.

Rendering metadata does not authorize bundling server dependencies into browser code. Define server/client capability boundaries and serialization independently. Routes produce an inspectable output manifest with their effective rendering mode, generated paths, assets and required runtime capabilities.

### Shared native navigation direction

Reuse route identities, typed parameters, layouts and links for local screen navigation. Desktop dynamic evaluation means evaluation on navigation inside the application, not an HTTP request. Web prerender settings affect the web artifact only; they do not freeze native application state. Persistent app state and route-owned state have separate lifetimes. Deep-link OS registration is later work. Unsupported server-only dependencies on native routes must receive diagnostics, not be ignored.

Before accepting shared navigation, specify browser history/back/forward, native back navigation, direct entry, missing routes, focus restoration, route replacement and layout/page disposal. The editable-list witness should use stable item IDs and define what happens when the selected item is deleted.

### Decisions still required

The [follow-up review](docs/plans/ui-feasibility-review.md#follow-up-review-after-app-directory-and-loader-decisions) is the consolidated decision register, including core state/browser/native blockers and the application/framework responsibility boundary. The items below summarize the routing portion.

1. Define typed loader outcomes for found/not-found/failure and the explicit handler's context. Static data existence and the required visible not-found handler are now owner decisions, not open choices.
2. Compare an existing htmx adapter with a small purpose-built transport using the same persistent-layout/fragment fixtures. Decide only after checking lifecycle integration, accessible navigation, history and dirty-input behavior. Raw-data updates may follow; neither htmx nor a JS/WASM application runtime is selected yet.
3. Specify the typed loader context/result, parameter-enumerator surface, mutation/revalidation contract and target-specific data adapters. Keep local interaction independent of server round trips.
4. Before implementation: confirm metadata spelling, project/page precedence, layout state lifetime, path normalization, typed-link shape and the first convention subset. First design witnesses should make these concrete rather than expanding to all Next.js features.

### Reference boundary

Official sources reviewed on 2026-10-11: Next.js [dynamic segments](https://nextjs.org/docs/app/api-reference/file-conventions/dynamic-routes), [layouts](https://nextjs.org/docs/app/api-reference/file-conventions/layout), [route groups](https://nextjs.org/docs/app/api-reference/file-conventions/route-groups), [static parameter generation](https://nextjs.org/docs/app/api-reference/functions/generate-static-params), and [route segment configuration](https://nextjs.org/docs/app/api-reference/file-conventions/route-segment-config). These establish the reference conventions. The explicit modes and configuration contract above are Tsrust proposals; they do not reproduce or promise Next.js's execution, cache or component model.

Follow-up references: [htmx documentation](https://htmx.org/docs/) demonstrates targeted HTML swaps and history behavior; [React Router pre-rendering](https://reactrouter.com/how-to/pre-rendering) describes declared pre-rendered paths and build-time loaders. They motivate the separate route/data/transport contracts here without committing to either framework.

## 19. Visible scaffolding and typed injection

The owner wants NestJS/shadcn-like generation ergonomics: select what to create, confirm where it will be written, then own the resulting source. Candidate commands such as `tsrust ui generate page items` are sketches; do not add them to the production CLI before the isolated prototype and compiler integration contract are accepted.

The generator should show the detected project root, invocation directory, artifact kind, resolved destination and planned file changes. Support explicit destination/kind flags, dry-run and reproducible noninteractive use. Interactive use confirms the destination and kind before writing. Never infer a destructive overwrite from confirmation; existing files require a separate explicit replacement decision. Generated components, layouts, pages, not-found handlers, loaders and provider wiring remain readable/editable source. Never regenerate over customized components silently. Unknown kinds and ambiguous project roots receive actionable errors.

### Offline, versioned component source

Use `src/components/` for reusable UI, with `src/components/ui/` as a suggested destination for catalog components. This is a configurable ordinary source directory, not a second routing root. Route-local types/components can remain under `app/` without becoming endpoints. No `views/` layer is required. Installed files are owned by the developer and imported locally; native/compiler primitives remain a separately versioned underlying capability.

Candidate commands (not implemented):

```text
tsrust ui add button@1.0.0 --cwd ./my-app --path src/components/ui
tsrust ui add text-input@1.0.0 --cwd ./my-app --dry-run
tsrust ui list --bundled
```

The CLI binary contains the catalog metadata and source/template payloads, including required component dependencies, styles, assets and license notices for its supported entries. Installation does not contact a registry, run remote hooks or silently install external packages. Any runtime/toolchain prerequisite must already be available and compatible; otherwise fail with a precise offline diagnostic before writing. “Offline component installation” must not be presented as proof that every native build dependency is bundled in this binary.

Component identity includes name, immutable version and any explicit variant. Published bytes for that identity never change. CLI version, catalog format version, component source version and required UI-runtime/toolchain versions are distinct. Resolve optional omitted versions visibly to an exact bundled version before confirmation; never reinterpret an installed version as whatever is newest.

Keep a project-owned machine-readable manifest, provisionally `ui-components.lock.json`, recording exact identities, dependency versions, selected variants, destination paths, original template hashes, hashes of initially written source after generation, and runtime/toolchain compatibility. Do not put credentials in it. Customized source remains authoritative; a differing hash means edited, not corrupt. Source, lock data and pinned project toolchain/runtime belong in version control; the lock file alone cannot recreate user edits.

Updating the CLI does not modify installed source, lock entries, styles or provider wiring. New versions are available for explicit installation/migration only. Re-adding an installed component must preserve local edits; upgrades show a diff and require a separate migration decision. Initial implementation can refuse upgrades into occupied paths rather than implement a fragile automatic merge. Missing dependencies and conflicting component/runtime versions fail before any writes; adding one new component must not silently upgrade a dependency used by an old component.

The bundled catalog must retain explicitly supported older variants and their exact dependency closure. If a requested version is absent, report it rather than downloading or substituting another version. The historical-retention support window and binary-size budget need definition; do not promise every historical payload in every future binary without a distribution policy. Existing installed components remain usable without reinstallation.

Versioned templates alone cannot guarantee behavior under a changed compiler or UI runtime. Pin those project dependencies too, maintain supported compatibility tests, and retain the required local toolchain for offline builds. CLI updates must not silently switch an existing project's pinned runtime/compiler; an incompatible project upgrade is a separate explicit migration. This is the mechanism required to honor the owner's no-silent-breakage goal, not merely a version field on copied files.

Initial acceptance witnesses: install exact v1 offline; customize it; update to a CLI catalog containing v1 and v2; prove the source and lock remain byte-identical; add v2 explicitly to a separate destination if compatible; reject unavailable versions and incompatible dependency graphs; reject overwriting customized files; verify the older installed version still builds against its pinned supported toolchain/runtime once actual components are implemented.

Distinguish dependency injection from request/route parameter injection:

- **Dependencies:** typed, explicit provider identities and composition/wiring, including fake providers for tests and target-specific adapters. Prefer generated/checkable wiring initially. Specify application, request, navigation and component scopes, disposal order, borrowing/mutation/transfer and any thread restrictions. A longer-lived provider must not retain a shorter-lived dependency. Do not silently create global singletons, clone resources, erase lifetimes or introduce shared ownership.
- **Parameters:** bind declared route/search/request inputs to typed loader or handler arguments. Route shape supplies raw string/segment values; numeric parsing, defaults and validation are explicit. A missing dependency, invalid parameter, absent record and backend error are distinct outcomes. Build-time execution has no ambient request/cookies, and desktop navigation has no automatic server credentials.

Decorators, runtime reflection and a Nest-compatible container are not selected. Start with an explicit typed context or generated argument wiring; compare decorator sugar only after the compiler can express the underlying contracts. Final syntax, provider scopes and lifecycle behavior require narrow examples and compiler review. The first task records source associations and supplied handler declarations only. Scaffolding, component installation and dependency execution are separate follow-ups; none is included in the first prototype.

References reviewed 2026-10-11: [Nest CLI](https://docs.nestjs.com/cli/usages), [Nest custom providers](https://docs.nestjs.com/fundamentals/custom-providers), [Nest parameter decorators](https://docs.nestjs.com/custom-decorators) and [shadcn CLI](https://ui.shadcn.com/docs/cli). These inform ergonomics; their runtime/type semantics are not imported into Tsrust.

Next bounded assignment: [nonexecuting UI route manifest](docs/plans/ui-route-contracts-first-task.md).
