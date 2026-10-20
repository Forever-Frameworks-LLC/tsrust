# Documentation index

The active [documentation application and component-library plan](plans/documentation-and-component-library.md)
covers the paired web/native docs project, organized Markdown, offline search,
all pinned components/blocks, themes, authoring APIs and curated benchmark pages.
It distinguishes planned work from accepted implementation and preserves the
developer evidence paths below.

## Runnable application foundation

This directory is also a standalone UI project. Its explicit entrypoint is
[src/main.tsr](src/main.tsr), shared page is [src/app/page.tsrx](src/app/page.tsrx),
and editable controls are under [src/components/ui/](src/components/ui/).
[tsrust.config.json](tsrust.config.json) enables web and native targets and owns
the light/dark theme. Run `tsrust dev` from this directory on macOS for the current
custom-titlebar example. Other native platforms require `titlebar: 'native'`.

The site has a landing page at `/` and documentation at `/docs`. Run
`tsrust dev --target web --port 4317` here for the live browser preview. The real
CLI watches application source and Markdown, retains the previous application on
build failure and reloads after successful publication. Article construction and
search data are lazy; this does not yet split executable downloads or render
HTML before Wasm startup.

[The experience audit](plans/documentation-experience-audit.md) lists required
navigation, theme, content and gallery work. The
[reference map](plans/documentation-reference-coverage.json) accounts for public
editor-catalog receivers. A reference destination or pending inventory entry is
not a completed component, tutorial or measured benchmark.

## Start here

- [Standard-library roadmap](standard-library-roadmap.md): current implementation status and remaining S08–S21 work.
- [Development workflow](development-workflow.md): implementation, verification, formatting, checkpoints and evidence retention.
- [Feature coverage](feature-coverage.md): supported operations and separately tracked measurement coverage.
- [Language and library guide](standard-library.md): public APIs and links to behavior guides.
- [Project vision](product-vision.md), [roadmap](roadmap.md) and [open decisions](open-decisions.md): broader direction and unresolved design choices.

## Current acceptance

S07 resources and bytes is accepted on macOS. [Family evidence](milestones/s07-resources-bytes.md)
records complete regressions, 17 matched workloads, direct Rust comparisons and installed-consumer checks.
S08 collections is in progress; later families remain pending in the standard-library roadmap.

Use [collections](guides/collections.md), [native bytes](native-bytes.md), [recursive records](recursive-records.md) and
[native files](native-files.md) for current behavior. Earlier handoffs and plans record
incremental work and may describe restrictions that have since been removed.

## Finding files

| Location | Purpose |
| --- | --- |
| [guides/](guides/) and existing topic guides | Current language, library and product behavior |
| [plans/](plans/) | Design and implementation plans; check the current roadmap before treating old status as current |
| [milestones/](milestones/) | Accepted gates and feature-family evidence |
| `handoff-*.md` | Historical continuation notes; use current roadmap and family evidence first |
| [experiments/](experiments/) | Research witnesses and experiments |
| [measurements/](measurements/) | Measurement documentation; original records live in SQLite |
| [Compiler fixtures](../crates/tsrust-compiler/tests/fixtures/) | Executable and rejection witnesses grouped by feature |
| [Runtime benchmarks](../benchmarks/runtime/) | Drivers, source contracts, independent oracles and registration tests |
| [Benchmark workloads](../benchmarks/runtime/workloads/) | Matched Tsrust and TypeScript sources registered in the manifest |
| [Analytics storage](../benchmarks/storage/) | Import, verification and exact restoration of canonical SQLite records |

Keep new current guides concise. Update the owning roadmap and family evidence rather
than adding another status summary to this index. Preserve retained historical evidence paths.

## Document authority

Current behavior guides and accepted family evidence describe implemented behavior.
Plans and original workstream proposals do not silently approve unresolved language decisions.
Historical handoffs are context, not the current progress ledger. The development workflow
governs acceptance: full regressions and measurements follow meaningful feature-family
milestones, not fixed gate counts.

The [section map](section-map.md) links original vision sections. The existing
[original vision](archive/vision-original.md) remains historical reference; do not maintain a duplicate.

## Workstreams

| Document | Proposed phase | Scope |
| --- | --- | --- |
| [Product vision](./product-vision.md) | Groundwork | Define the intended developer experience and differentiated product thesis. |
| [Principles and boundaries](./principles-and-boundaries.md) | Groundwork | Define compatibility boundaries, early non-goals, and decision priorities. |
| [Research program](./research-program.md) | Groundwork | Track the required design studies and unresolved cross-cutting questions. |
| [Ownership and memory](./ownership-and-memory.md) | Core proof | Specify moves, inferred access, copying, sharing, captures, allocation, cycles, and global state. |
| [Type system and data shapes](./type-system.md) | Core proof | Specify static types, object shapes, capabilities, and generics. |
| [Primitives and collections](./primitives-and-collections.md) | Core proof | Specify numeric, string, array, and iterator behavior. |
| [Async and runtime](./async-and-runtime.md) | Backend proof | Define execution, task lifetimes, cancellation, concurrency, and runtime initialization. |
| [Error handling](./error-handling.md) | Core proof | Choose recoverable-error propagation and fatal-failure behavior. |
| [Metadata and serialization](./metadata-and-serialization.md) | Backend proof | Scope reflection, annotations, code generation, and checked serialization. |
| [Compiler, modules, and diagnostics](./compiler-and-modules.md) | Core proof | Define the frontend, analyses, intermediate forms, Rust generation, modules, and source diagnostics. |
| [Interoperability and secondary targets](./interop-and-targets.md) | Backend proof; later targets | Scope safe Rust wrappers, foreign interfaces, and eventual WebAssembly support. |
| [Standard library](./standard-library.md) | Core proof; later expansion | Define library boundaries and compatibility policy; implement only the subset needed by each prototype. |
| [Toolchain and development experience](./toolchain-and-development.md) | Core proof; ongoing | Scope CLI commands, editor/debugger integration, toolchain pinning, and iteration speed. |
| [Security and supply chain](./security-and-supply-chain.md) | Groundwork; ongoing | Define unsafe boundaries, build/install permissions, and dependency visibility. |
| [Packages and registry](./packages-and-registry.md) | Ecosystem proof | Scope content-addressed storage, version identity, registry requirements, and the package-manager prototype. |
| [npm analysis and porting](./npm-analysis-and-porting.md) | Ecosystem proof; later conversion | Separate compatibility inspection from automatic conversion and upstream maintenance. |
| [Backend ecosystem](./backend-ecosystem.md) | Backend proof; later packages | Define native framework ergonomics and the first ecosystem packages. |
| [Database integration](./databases.md) | Backend proof | Scope native drivers, typed queries, schema validation, and database ergonomics. |
| [HTTP and Hono compatibility](./http-and-hono.md) | Backend proof | Define HTTP primitives, routing, middleware, compatibility levels, and independent implementation constraints. |
| [Validation](./validation.md) | Backend proof | Compare schema APIs and type-derived validation, including HTTP integration. |
| [Server HTML and JSX](./server-html.md) | Web proof | Scope typed HTML, JSX components, escaping, streaming, and the native web showcase. |
| [Source migration and compatibility tests](./source-migration.md) | Backend proof | Measure application migration and independently test documented public HTTP behavior. |
| [Identity and naming](./identity.md) | Groundwork | Research the language name, extension, CLI, and ecosystem namespaces. |
| [Competitive landscape](./competitive-landscape.md) | Groundwork | Compare neighboring projects through public evidence without incorporating their implementation code. |
| [Benchmark methodology](./benchmarks.md) | Groundwork; ongoing | Define independent equivalent workloads, runtime metrics, and fair comparisons. |
| [Core and backend prototypes](./core-and-backend-prototypes.md) | Core proof; Backend proof | Preserve the first compiler, ownership, and native HTTP demonstration requirements. |
| [Native deployment](./deployment.md) | Backend proof | Define executable deployment and runtime resource requirements. |
| [Desktop architecture and rendering](./desktop-architecture.md) | Desktop research; later implementation | Compare rendering strategies and define distinct UI/HTML targets without committing to a renderer. |
| [Desktop reactivity and ownership](./desktop-reactivity.md) | Desktop research; later implementation | Scope fine-grained updates, component lifetimes, signals, conditional trees, identity, and state. |
| [Desktop controls, layout, and accessibility](./desktop-controls.md) | Desktop research; later implementation | Scope windows, controls, styling, layout, events, bindings, menus, accessibility, and the UI module. |
| [Desktop async and platform integration](./desktop-async-and-platform.md) | Desktop research; later implementation | Scope UI-thread execution, component-owned tasks, system APIs, target checks, packaging, and assets. |
| [Desktop tooling and benchmarks](./desktop-tooling-and-benchmarks.md) | Desktop research; later implementation | Scope compile iteration, reactive inspection, ownership inspection, and UI/resource benchmarks. |
| [Desktop prototypes and shared code](./desktop-prototypes-and-shared-code.md) | Desktop research; later implementation | Preserve open desktop questions and the counter, data application, and shared server/client experiments. |
