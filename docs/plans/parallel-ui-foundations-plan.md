# Parallel UI foundations plan

Status: planning only, 2026-10-11. No UI feature is implemented or accepted by this document. Product decisions are in [UI_VISION.md](../../UI_VISION.md); routing remains a proposal pending its listed decisions. This plan does not dispatch a worker or authorize simultaneous writers in the main checkout.

Read the [updated feasibility/security review](ui-feasibility-review.md#follow-up-review-after-app-directory-and-loader-decisions) before assignment. The only first assignment is the bounded nonexecuting route manifest. State/lifecycle and real paired input proofs remain higher-value feasibility work and require separate assignments. The table below is an opportunity map, not a queue to implement or authorization to begin every package.

## Objective and bounded deliverable

Develop shared component APIs through paired web and native desktop evidence alongside continuing compiler/standard-library work. Start with contracts and isolated adapters, then integrate one generated static screen, a counter and an editable list. Keep familiar JSX/className authoring, consistent component behavior, platform-appropriate input/accessibility, and semantic HTML on the web.

Do not wait for the entire standard library. Do not assume async, HTTP, arbitrary module initialization, shared signal state, JavaScript generation or JSX support already exists. Renderer dependencies, JS/WASM selection, routing metadata and ownership changes require explicit reviewed contracts before implementation.

Application policy is separate from framework acceptance: use [the vision's responsibility boundary](../../UI_VISION.md#application-policy-and-framework-guarantees). Auth/CSRF, business authorization and data publication are app choices. Runtime witnesses test that configured helpers execute correctly and that the framework does not introduce implicit exposure; no universal policy engine or compulsory security bundle is on this workstream.

## Existing foundations to reuse

| Foundation | Source anchors | Implication |
|---|---|---|
| Source analysis and checked ownership | `crates/tsrust-compiler/src/lib.rs`, `syntax/mod.rs`, `analysis/mod.rs` | One language frontend; UI expressions use existing type/ownership checks |
| Rust emission | `crates/tsrust-compiler/src/emit/mod.rs` and emission helpers | Reuse checked application behavior; add a narrow UI lowering boundary |
| Project loading and aliases | `project/mod.rs`, `project/resolution.rs`, `project/session.rs` | UI route discovery is an additional entry graph, not a second module resolver |
| Typed immutable data imports | `project/data/module.rs`, `project/data/shape.rs`, `project/data/lowering.rs`, `project/data_project_tests.rs` | Theme configuration can reuse typed data; semantic token validation is new |
| Native signatures/build | `native/bindings.rs`, `native/signatures.rs`, `native/types.rs`, `cli/runtime_build.rs` | UI libraries need declared ownership/callback/features, selected dependency linking and offline builds |
| Editor | `crates/tsrust-lsp/src`, including `builtins.json` | Shared facts, complete help and source diagnostics; no independent editor-only API |

The checked program is internal and ownership-rich, not a frozen public renderer interface. Do not expose it wholesale or start a general compiler IR rewrite. Latest standard-library ledger: S01–S09 accepted within documented limits, S10 JSON in progress, threads/async pending. Reconcile current commits at actual workstream start.

## What can start in parallel

Suggested paths below are future ownership boundaries, not a requirement to create every directory or crate now.

| Work package | Can start | Isolated scope and output | Exit evidence |
|---|---|---|---|
| Component and style contracts | Now | `docs/ui/`, `tests/ui-contracts/`: View/Text/Button, small utility vocabulary, token units, states and target capability matrix | Examples specify accepted/rejected inputs, layout/text/accessibility behavior and cross-target assertions |
| Token/semantic-description prototype | After the small fixture contract is agreed | Initially one isolated UI prototype package; typed token validation and manually constructed semantic nodes with stable IDs | Missing tokens, invalid units, unsupported utilities and deterministic output tests; no claims of compiled JSX |
| Paired web/native adapters | Alongside token work once the fixture schema is shared | `experiments/ui-foundations/`: HTML/CSS output and one desktop platform, independent of active compiler internals | Same fixture yields matching semantic state/layout intent; real DOM/native window checks, keyboard activation, text and basic DPI/accessibility evidence |
| Route discovery/matcher prototype | Now for reference fixtures; behavior after routing decisions | `tests/ui-routing/` and isolated manifest/matcher module | Literal/parameter paths, conflicts, source locations, typed parameter model and builder/matcher round trips; no server dependency |
| Loader/static-generation contracts | Now as fixtures with supplied data | Enumerated params, typed page-data results and output manifest, separate from HTTP | Complete finite path set, missing-data/duplicate failures, request-vs-build capabilities and matching native adapter requirements |
| Persistent layout/partial navigation experiment | After route boundary fixtures are shared | Isolated web adapter comparing htmx with a small transport; local native outlet counterpart | Header identity/state preserved, page swapped, history/direct entry, stale-response control, region disposal and dirty-input behavior; mock transport does not count as production server support |
| State/lifecycle research | Now as direct Rust contracts and witnesses | Component owner, binding access, retained events, scheduling and teardown models | Two independent counters, stale-handler rejection, cleanup and conditional dependencies; missing compiler behavior becomes focused dependency witnesses |
| Constrained-display adapter | After semantic description is useful; not on critical path | Headless software adapter to the same fixture contract | Fixed dimensions, input replay, limited color conversion and redraw output; hardware/memory claims explicitly excluded |

Begin with one isolated UI worker, not separate workers for every row. Paired targets are two acceptance surfaces; they do not require multiple agents. Add workers only with explicit authorization and disjoint ownership. No shared compiler source changes are necessary for the initial manually constructed backend fixtures.

## Compiler integration conditions

| Integration | Required decision/proof | Shared surfaces requiring compiler-owner coordination |
|---|---|---|
| `.tsrx` static templates | Grammar, props/children types, source spans, portable vs target-specific nodes | Parser, syntax/type checking, source walkers, formatter, module discovery, LSP and editor grammar |
| Checked UI description | Ownership of props/children and expression/binding references; narrow versioned contract | Checked facts and Rust/UI emission; do not duplicate type checking |
| Counter/state/events | Owner lifetime, shared reads/writes, callback retention, event batching/reentrancy, teardown/escape rules | Callable/native signatures, analyzer/lifetime rules, generated runtime and catalog |
| Routing integration | First convention subset, path normalization, links and layout identity/lifetimes | Project graph, entry generation, source diagnostics, incremental invalidation and editor navigation |
| Per-route static metadata | Literal metadata vs general const exports; checked keys, precedence, build hooks/effects | Module declaration handling, checker, formatter/editor, build driver |
| Interactive browser execution | Bounded JS or WASM approach, state/event semantics, dependency exclusion | A new browser output path and build integration; Rust emission alone is insufficient |
| Request rendering/deployment | Route execution contract and serving/runtime support | Synchronous HTML rendering can precede async; Axum/Tokio hosting depends on relevant async/HTTP gates |

UI workers must not solve blockers with hidden clones, unchecked casts, lifetime erasure, general shared capture cells, a second parser/type checker, or compiler-only exceptions unknown to the editor. Give the compiler owner the smallest failing witness and required behavior; continue independent adapter work.

## Paired acceptance milestones

### Shared static screen

One token-themed component containing layout, text and a button generates semantic HTML/CSS and a standalone native window. Verify escaping, token errors, source mapping, text/layout behavior and capability rejection. Manually constructed fixtures prepare this gate but do not close it. Do not require pixel-perfect font rasterization across targets.

### Independent interactive counters

Mount two component instances; update one by pointer and keyboard and prove the other is unchanged. Verify disabled activation, accessible name/role/focus, event/update ordering, targeted binding evaluation and deterministic disposal. Report binding work separately from layout/painting. Reject or safely invalidate callbacks after owner destruction. Browser and native behavior must both execute from generated application artifacts before acceptance.

### Editable list and navigation

Add, select, edit, validate and remove records using stable IDs. Include empty/invalid input, focus after insertion/deletion, keyboard navigation, text selection and platform input/IME checks. Unmount/reorder must preserve the intended identity and dispose removed state. When routing is agreed, exercise `/items` and `/items/[id]`, direct entry, back navigation, missing/deleted IDs, checked links and layout lifetimes on both targets. Keep data local so HTTP/databases do not block the core interface proof.

### Explicit static and dynamic web output

After rendering decisions, test a literal static route, enumerated parameter route and runtime route using the same manifest. Verify effective config, generated path list, escaped content, unsupported static dependencies, unknown parameter behavior and static-only deployment rejection. Preserve interaction on static pages. Request isolation, server/client dependency boundaries and explicit serialization precede acceptance of the dynamic adapter. Applications select their access/cache/CSRF policies; representative protected fixtures validate installed-helper propagation without requiring those helpers on every app. No implicit page cache or on-demand generation is accepted by this gate.

The owner prefers request rendering with partial UI updates. Verify a complete document on direct entry and a changed outlet on enhanced navigation while the shared header remains mounted. Check back/forward, title, focus, late responses, region cleanup and explicit layout-data refresh. Static builds enumerate every declared path and run its loader; missing required data fails generation. Static partial navigation must run against published artifacts with no render server. Typed-data updates and persistence are separately scoped; an HTML swap experiment does not prove them.

## Worktree, integration and evidence rules

- Use a committed baseline in a separate managed worktree when implementation is authorized. Inspect current status and handoffs first; one writer per tree. Do not copy the main checkout's uncommitted work or modify its current compiler/stdlib audit.
- The UI worker owns isolated contracts, prototypes and fixtures. The compiler owner owns active shared analyzer/parser/emitter changes until an explicit integration handoff. Shared catalogs, workspace membership, dependency locks and benchmark registrations must be coordinated.
- Keep prototype dependencies isolated and pinned. Do not add every proposed renderer/runtime crate to the production workspace. Review dependency provenance, licensing and target support before selection.
- Reuse the documented Cargo target arrangement with `CARGO_INCREMENTAL=0`; serialize conflicting builds/consumer refreshes. Do not create a target per UI gate or overwrite another workstream's executables.
- Follow [development-workflow.md](../development-workflow.md): focused failing tests, smallest meaningful compiler/native/editor checks, full-project formatting before source checkpoints, token preservation/idempotence and formatted execution. Include complete IntelliSense and examples for each accepted API.
- Prepare matching reference workloads, independent expected results, source fingerprints and backend drivers as behavior lands. Browser rendering, native rendering and compiler timings have different scopes; do not manufacture a cross-target speed ratio from unlike work.
- Run full regressions and performance review at meaningful integrated milestones, not after each adapter edit. Keep accepted implementation distinct from pending measurements and actual platform coverage.
- SQLite is canonical for closed evidence. A worker must not concurrently write the main store; integrate immutable records serially with byte verification. Keep only concise active logs and prune obsolete outputs before handoff.
- Hand off commits, exact acceptance evidence, source/compiler identity, remaining dependency witnesses and measured/unmeasured status. Integrate and refresh shared consumers only under exclusive ownership.

## Immediate next deliverable and stopping boundary

The owner has clarified request rendering with persistent layouts/partial updates, complete loader-backed build-time generation, shared desktop navigation and explicit application-owned not-found handling. For build data containing IDs 1, 2, 4, generate all three; IDs 3 and 5 are absent. Static output reflects a build snapshot, not live database state. Visible CLI generation and typed dependency/parameter injection are desired; exact syntax and lifetimes remain open.

Optional first task: [nonexecuting UI route manifest](ui-route-contracts-first-task.md). It discovers special files, binds loaders to colocated pages, records middleware ancestry and validates supplied method declarations without parsing or executing application source. No scaffolding, catalog installer, static generation, loader execution or fake DI is included. One test command and one demo close the task. If capacity is limited, continue compiler work instead; the route prototype is not a prerequisite for UI feasibility research.

Stop the first assignment at its manifest/tests/demo and compiler handoff. Token work and paired adapters are not part of it. Local UI state/input experiments can precede async, but actual Tsrust async loaders and Axum middleware should wait for accepted relevant S13 async, S14 task/runtime and S15 HTTP capabilities and their transitive prerequisites. Async alone does not provide JSX, loader result inference, typed middleware contexts, browser execution or native editing. Proceed to generated `.tsrx` acceptance only after the compiler integration contract is agreed.
