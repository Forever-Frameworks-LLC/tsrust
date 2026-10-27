# UI vision: feasibility and security critique

Reviewed 2026-10-11 against UI_VISION.md sections 1–19, both UI plans, compiler/native/module source, current standard-library ledger, older desktop/web proposals and linked primary references. Observed source HEAD: `4cfadf0bf02f2d2d17fa0d4ac4fca63a4032fc41`. UI planning files were uncommitted. This is a preimplementation architecture review, not a repository vulnerability scan, exploit validation or performance measurement. Risks below are prospective unless explicitly described as observed source facts. No UI implementation exists to certify. Review recommendations do not silently replace owner decisions.


## Responsibility clarification applied to this review

Owner clarification: applications choose authentication, authorization, CSRF, confidentiality, caching and deployment policy. The framework provides composable tools and honors their configured scope; it does not require every app to adopt the protected editable-list example's policies. [The vision](../../UI_VISION.md#application-policy-and-framework-guarantees) is authoritative. The original analysis is retained with responsibility corrections below; earlier blanket statements that these application policies were framework release blockers are superseded.

Framework blockers concern its own behavior: bypassing installed middleware, leaking request state, implicit loader serialization, unsafe ordinary template output, ambiguous dispatch or broken lifecycle guarantees. Optional helpers need their own correctness evidence when implemented. Choosing business rules, identifying confidential fields and deciding which helpers to install are not prerequisites for the route prototype or core UI work. This is an audit of design documents, not a security certification of an application.

## Framework vulnerability lessons from React and Next.js

Owner clarification: the security review is primarily about vulnerabilities introduced by framework machinery, as illustrated by React/Next.js CVEs, rather than applications choosing the wrong business policies. The responsibility split above remains valid; it must not downplay defects below the application boundary. These are selected primary-source case studies checked on 2026-10-11, not an exhaustive/current vulnerability inventory or a claim that Tsrust is affected.

| Disclosure | Documented failure | Design implication for this project |
|---|---|---|
| React CVE-2025-55182 | Unauthenticated code execution through Server Function request payload decoding; RSC support could expose an app even without app-defined Server Function endpoints | Avoid a general remote function/object reconstruction protocol. Inventory every generated endpoint and pre-handler decoder; loader discovery alone creates no network-callable function. [React advisory](https://react.dev/blog/2025/12/03/critical-security-vulnerability-in-react-server-components) |
| Next.js CVE-2025-29927 | Middleware authorization bypass; the advisory's workaround filters external `x-middleware-subrequest` requests | Never accept an external header as proof that middleware already ran. Keep dispatch/recursion state in trusted runtime context, with explicit trust boundaries for any internal transport. [Maintainer advisory](https://github.com/vercel/next.js/security/advisories/GHSA-f82v-jwr5-mffw) |
| React CVE-2025-55184, CVE-2025-67779, CVE-2026-23864; CVE-2025-55183 | Decoder denial of service, including incomplete fixes, plus Server Function source exposure under the disclosed stringification conditions | Bound parser work and regression-test malformed variants; do not deserialize function references into ordinary data or implicitly stringify runtime functions. [React follow-up](https://react.dev/blog/2025/12/11/denial-of-service-and-source-code-exposure-in-react-server-components) |

The implications are our engineering recommendations, not claims made by the advisories. SSR means rendering HTML on a server; it does not require React Server Components, remote Server Functions or their wire protocol. The proposed app-directory ergonomics can use ordinary HTTP, server-local loaders and HTML fragments without adopting those mechanisms. Static delivery removes the per-request renderer from that deployment, but build tooling and any attached APIs still have their own attack surface. Rust/Axum is not evidence that dispatch, parser, cache or dependency vulnerabilities cannot occur.

Recommended initial framework constraints and later acceptance witnesses:

1. **Data-only ingress.** Forms and explicitly schema-checked data select declared handlers; request values never name arbitrary imports, functions, providers or template files. Do not reconstruct closures, executable object graphs or server capabilities from client payloads. Define supported wire types separately from rich in-process page props.
2. **Trusted dispatch state.** Client representation headers can request a supported response format, not skip middleware or select a different policy chain. Test forged internal-looking headers, path encodings, direct/full/fragment/data requests and rewrites against the same installed-policy trace. Reject unsupported selectors consistently.
3. **Bounded parsing before handlers.** Limit body bytes, nesting, field counts, allocation and parsing work; reject malformed/unsupported encodings without panics or unbounded loops. Handler authentication cannot be assumed to protect a decoder that runs first. Async timeouts alone do not interrupt non-yielding CPU loops. Use fuzz/property tests at actual exposed parsers when implemented.
4. **Explicit emission and dispatch inventories.** Enumerate generated routes, enabled decoders and response projections. A loader is server-local unless a separate endpoint is declared. Verify that unused loader fields, server source and provider context do not appear in generated responses/assets. Explicit application-selected responses remain application policy.
5. **Representation and dependency discipline.** Test full/fragment cache separation and preserve chosen application headers. Track exact framework/runtime dependencies, maintain adversarial regression cases after fixes, and provide explicit patched releases. An immutable component catalog must not silently change installed source, but needs visible update guidance for affected components/runtime versions.

These obligations attach to the relevant runtime/transport feature, not to the initial nonexecuting manifest prototype. The prototype can record endpoint inventory and ancestry; it cannot certify decoder safety or middleware execution. No promise of CVE immunity is made.

## Follow-up review after app-directory and loader decisions

Review date: 2026-10-11. Re-read the vision, editable-list sketch, HTTP contract, parallel plan and standard-library ledger. Current HEAD at inspection was `a7a3338f2d33c0fdde393ece9ee9f30e346856d9`; the working tree has active compiler source reorganization. Current frontend anchors are `crates/tsrust-compiler/src/syntax/mod.rs`, `analysis/mod.rs` and `emit/mod.rs`, with shared integration in `lib.rs`. This is a document/source review, not a fresh compiler acceptance run. The earlier review below remains historical evidence; its HEAD describes that earlier inspection; moved source links below now point to their current module locations.

**Recommendation: proceed only with the narrowed nonexecuting route-manifest experiment, if a separate worker is useful. Wait for accepted async/task/HTTP capabilities before implementing the Tsrust Axum runtime.** If work capacity is limited, finishing compiler foundations is more valuable than this optional tooling. Do not wait for async to investigate synchronous component lifetime or native text input: async will not solve either problem.

The new conventions improve authoring coherence: one app tree, independently optional pages/API routes, sequential ancestor middleware, colocated automatically injected loaders, visible 404 source and offline owned components. They do not reduce the main runtime risks. The earlier first task had accumulated routing, static builds, provider execution, filesystem scaffolding and versioned installation. It is now limited to discovery/declaration validation; the remaining features require separate acceptance.

### Settled direction, not questions to reopen

Keep paired web/native interfaces, JSX/className, app-directory routing, server rendering by default, persistent-layout partial updates, page/route coexistence without conflicting GET, exact local loader pairing, explicit not-found source, real-row static enumeration and offline versioned component source. No separate views directory. Numeric gaps are absent rows, never pages to invent. These choices do not settle implementation signatures or lifecycle policy.

### Remaining decisions and when they block

| Unresolved contract | Recommended starting boundary | Required before |
|---|---|---|
| Component ownership, signals, retained callbacks and disposal | Single UI-thread owner; explicit handles/scopes; prove independent instances, reentrancy, removed/reused list keys and stale callbacks before choosing public signal API | Any generated interactive counter, regardless of async |
| Browser execution versus HTML replacement | One owner per region. Compare a bounded local browser runtime separately from server fragment transport. htmx-like swapping cannot by itself execute shared local Tsrust event logic | Paired counter and enhanced navigation respectively |
| Native controls/text/style capability subset | One desktop OS first; reuse a text/input subsystem where possible. Specify semantic primitives, units, class conflict rules, IME/focus/accessibility and actual supported behavior | Renderer selection and paired editable-list acceptance |
| Loader result and injected context types | Compiler synthesizes page input from successful loader result; explicit redirect/not-found/error control outcomes and target capabilities. Resolve page annotations, middleware context refinement and module cycles without executing source | Loader/page compiler and editor integration |
| Loader re-execution and layout data | Page loader stays local. Specify initial load, params/search change, refresh, cancellation and invalidation. Persistent layout data needs a separate explicit input/provider contract, not inherited page loaders | Navigation/data loading; a future user-specific header still needs an explicit data source; the example currently uses constant header content |
| Build/server/browser/native dependency boundary | Server-only identity/repositories stay server-side; inject target-capable adapters. Define supported serialization types, explicit recipient-visible projections and conversion rules; confidentiality selection belongs to the application. Provider scope/disposal and no retained request state must be checked | Shared loaders, DI or browser data delivery |
| HTTP dispatch and middleware failure behavior | One manifest, page-owned GET and recommended HEAD reservation. Settle route subpaths, 405/OPTIONS/CORS, exceptions versus early responses and matched ancestry on unknown URLs. Choose which policies run before protected 404s | Any exposed HTTP service; prefix matching or a fragment header must not bypass auth |
| Paths, links and configuration | Freeze a small normalized segment policy and structured typed-link surface. Decide config location/spelling/precedence and separate enumerator convention. No execution during discovery | Production routing/build integration; provisional policy is enough for the isolated experiment |
| Partial navigation identity and concurrency | Specify layout/template remount keys, parent-param changes, dirty forms, focus/history/title and auth-change invalidation. Version responses; discard stale navigation results; cancellation does not undo a mutation | First partial-update runtime |
| Static completeness and deployment | Finite snapshot dataset, duplicate/failure handling, bounded generation with explicit failure, atomic publication and old-path removal. Require build-capable inputs; applications select output data and host access policy | Static generation acceptance |
| 404/error boundary input and host delivery | Required visible root source; nested selection and sanitized contexts. Decide safe terminal failure response and static host status/asset mapping. API misses must not unexpectedly become UI pages | Error handling and static deployment |
| Offline component compatibility | Immutable source identities plus runtime/compiler pins. Define retained catalog versions, dependency conflict policy, binary size and explicit migration/security-fix policy | First catalog release, not route discovery |

These are contracts to prove with narrow examples, not a request for the owner to answer twelve broad questions before any work can happen. The proposed prototype policy is reversible; language/runtime decisions require reviewed compiler witnesses.

### Newly visible traps

- **Type inference across files can become circular.** A page importing a component that imports its page's inferred props can create a type dependency loop. Generate an internal checked association; do not execute imports or use `any` to break it. Define a cycle diagnostic or supported inference boundary.
- **Page-local loaders leave layout data intentionally unsolved.** A header showing a user or item count needs explicit data access and invalidation. Retaining its DOM does not authorize retaining request providers, and a child page refresh must not imply an invisible ancestor loader run.
- **A route-only directory is still a route.** Discovery must not depend on finding a page. Conversely, a layout/middleware-only directory is not an endpoint. The old task was page-centric and would have missed this distinction.
- **A route match is not a record existence check.** `/items/3` matches `[id]` even when row 3 is absent. Dynamic loaders determine resource existence; the static output manifest determines built-path membership. Conflating these makes authorization and 404 behavior unreliable.
- **Read-only loaders are a behavioral rule, not yet a proven effect system.** Document that navigation/build/prefetch may repeat them. Do not promise compile-time prevention of arbitrary external side effects without the corresponding capability/effect design.
- **An error boundary may depend on the failing layout/provider.** It cannot necessarily render within that same scope. Error selection needs dependency-aware fallback and a nonrecursive terminal failure path.
- **Offline and immutable does not mean permanently secure or compatible.** Existing component bytes remain owner-owned, but security fixes need a visible advisory/migration path; newer compilers cannot guarantee compatibility merely by preserving old template versions.

Framework release criteria remain consistent execution of installed middleware, request/provider isolation, ordinary output encoding, explicit serialization, canonical dispatch, full/fragment representation separation and documented resource bounds. Applications choose auth/CSRF, object permissions, cache policy and published data. Tests use explicit sample policies to prove composition, not to require a universal policy or claim that types detect confidential fields. No implementation vulnerability is claimed; the narrowed prototype validates declarations and paths only.

### Sequencing and honest acceptance claims

1. Optional now: [nonexecuting route manifest](ui-route-contracts-first-task.md), isolated Node built-ins, no shared compiler changes. Finish once; do not expand it while waiting.
2. Independently valuable before async: bounded state/lifetime feasibility and real web/native input experiments, each separately assigned. These test the highest UI risks. They are not included in the first route task.
3. Coordinate generated UI integration with the compiler owner once `.tsrx`, props/events, state ownership, lowering and editor/formatter contracts are reviewed. Async is not a prerequisite for a local counter/list.
4. Implement Tsrust async loaders and Axum middleware only after the relevant S13–S15 capabilities and transitive prerequisites are accepted. S13 needs suspend-safe captures/loans/drop, S14 cancellation/shutdown/task scopes, and S15 request/response/body/handler integration. A narrow HTTP slice must not be called full S15 acceptance. Keep schemas and serialization prerequisites visible.
5. Add static generation, production CLI/catalog, fragment navigation and broader targets as separately accepted families. No universal UI release commitment yet.


## Verdict

The narrow concept is feasible: one ownership-aware language, a bounded component vocabulary, HTML on the web, custom native UI, explicit target adapters. The complete document is not yet an implementable first-release plan. It combines a language extension, reactive runtime, UI toolkit, web framework, build/deployment system, CLI and dependency-injection system, then adds a second constrained runtime for microcontrollers. Every layer has its own compatibility and security obligations.

Do not commit to the full architecture now. Proceed only with reversible feasibility experiments and explicit exit criteria. Keep production implementation behind the state/lifetime, execution-boundary and text/input proofs below. A successful routing generator would not establish that the UI framework is viable.

My earlier first-task recommendation overweighted isolated work and underweighted uncertainty reduction. The routing/scaffolding task is useful tooling, but its tests can all pass while the central cross-target UI premise fails. It should remain optional supporting work, with no authority to freeze language or runtime APIs. Before a broad implementation assignment, prioritize one state/lifecycle experiment and one real paired input/navigation slice.

## What the repository actually establishes

- [`lib.rs`](../../crates/tsrust-compiler/src/lib.rs) exposes analysis and Rust generation; `Analysis` wraps an internal checked program. [`analysis/mod.rs`](../../crates/tsrust-compiler/src/analysis/mod.rs) contains detailed capture, callable and ownership facts. This is substantial reusable machinery, not a stable backend-neutral UI interface.
- [`syntax/mod.rs`](../../crates/tsrust-compiler/src/syntax/mod.rs) is the shared language parser/AST. JSX and UI metadata are new work. Adding an extension entails formatter, source walking, diagnostics, resolver, dev watcher, packaging and editor integration as well as parsing.
- [`project/resolution.rs`](../../crates/tsrust-compiler/src/project/resolution.rs) resolves source imports/aliases; it is not an HTTP URL normalizer or a safe output-path mapper. Reusing its contract does not mean reusing filesystem paths as route identity.
- [`native/bindings.rs`](../../crates/tsrust-compiler/src/native/bindings.rs) supplies declared argument access, return borrowing and feature selection. Existing retained testing callbacks and the private `SharedCallback` intrinsic do not prove a general signal/DI lifetime model.
- The [standard-library ledger](standard-library-roadmap.md) reports accepted module/resource/collection/host foundations and in-progress JSON; async and production HTTP remain future gates. Server-first prototype transport can use an isolated host harness, but that is not a completed Tsrust web server.
- [`runtime/mod.rs`](../../crates/tsrust-compiler/src/runtime/mod.rs) and [string semantics](../guides/language/string-semantics.md) use standard native storage and UTF-8 behavior. There is no evidence here of general allocator-free MCU support or a browser backend with matching language semantics.

No compiler regressions were rerun for this document review. Existing acceptance records were read as records, not independently re-certified.

## Architectural issues that can invalidate the plan

### 1. The central state model is still unspecified

The counter requires state that survives component construction and is reachable from a binding and a retained handler. The editable list adds multiple handlers, row reordering, removal and delayed work. Ordinary lexical borrowing does not automatically cover these lifetimes. Native compilation and Rust memory safety do not prove freedom from logical use-after-disposal, leaks, stale IDs, deadlocks or runtime borrow failures.

Choose and prove one model: statically constrained scopes, owner-managed generational handles, explicit reference-counted handles, or a documented combination. Each trades ergonomics, runtime checks, disposal timing and compiler complexity. “No tracing GC” does not prohibit reference counting, but “no implicit shared ownership” means the sharing policy cannot be hidden. Do not promise the compiler will infer every safe arrangement.

Required witness: two instances; two readers and a writer; nested conditional subscriptions; reorder/remove a row; callback queued before removal; callback invoked during another update; slot reused for a new row; owner destroyed. Specify which operations are rejected statically, return a runtime outcome, or are legal. Prove cleanup without increasing live allocations over repeated cycles. Leptos's documented signal ownership/disposal tradeoffs are evidence that this is a real framework problem, not merely a missing syntax feature. [Leptos lifecycle](https://book.leptos.dev/appendix_life_cycle.html)

### 2. Three execution models currently sit behind one component promise

Request rendering is ephemeral server work. Browser signals outlive requests. Native components run inside an OS event loop. An HTML fragment cannot transfer a closure, Rust borrow or database connection to either other environment. The same loader signature may express a common data contract, but it cannot make all dependencies available on all targets.

Define a capability matrix for build/server/browser/native code, explicitly selected serializable boundary values, and explicit state ownership. Keep browser event functions separate from server loaders; avoid automatic cross-process closure lifting. Distinguish persistent browser layout instances from server layout evaluation: a header can stay mounted while the server independently evaluates request data. There is no implied per-user server component object retained between HTTP requests.

JS output means another semantic backend: native integer overflow, ownership/moves, errors, UTF-8 operations and disposal must remain coherent. WASM avoids some duplicated computation semantics but still needs DOM bindings, browser I/O, event dispatch and dependency exclusion. Rust's generic WASM target does not make ordinary OS services available. [Rust WASM target](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html)

An htmx adapter addresses server-to-DOM updates, not arbitrary local reactive component execution or native rendering. The first browser counter still needs a bounded local execution choice. Keep that as an explicit experiment rather than leaving the question open while promising identical executable components.

The no-tracing-GC goal also needs a target boundary. Generated JavaScript or htmx runs in a garbage-collected browser runtime; using WASM does not remove the browser's own managed DOM/runtime. A credible promise is about the generated native application and chosen application-state runtime, not a blanket claim that every web dependency avoids GC. This must be a deliberate product clarification.

Native event-loop integration needs its own threading rule: initial state/layout work should stay on the UI thread, with explicit message/data transfer for later background work. Successful Tokio/thread implementation elsewhere will not automatically make UI handles transferable or safe to mutate during painting. Queued completion after window destruction is a required rejection/cleanup case.

### 3. The rendering stack is a toolkit project, not a graphics dependency choice

Winit, Taffy and a painter do not together supply a complete editable TextInput. Text requires shaping, font fallback, bidirectional handling, grapheme-aware editing, selection, caret movement, IME composition, clipboard, undo, password behavior, accessibility actions, focus and scrolling. The existing language's byte/scalar operations cannot be used as user-visible cursor positions indiscriminately. [Unicode segmentation](https://www.unicode.org/reports/tr29/), [Winit IME events](https://docs.rs/winit/latest/winit/event/enum.Ime.html)

Taffy solves layout computation; the application still supplies text measurements and integrates painting and input. AccessKit supplies accessibility infrastructure; the toolkit must supply correct semantics and actions. [Taffy](https://github.com/DioxusLabs/taffy), [AccessKit](https://accesskit.dev/)

The editable list is the right feasibility test precisely because editing exposes these costs. Keep it small, but do not count an ASCII-only mouse demo as acceptance. Evaluate an existing toolkit/text subsystem against the same test before assembling a bespoke one. Library availability alone does not establish acceptable licensing, target coverage, bundle cost or maintainability; review pinned candidates and their assets before adoption.

### 4. Visual consistency must have a bounded meaning

Browser CSS and native layout differ in intrinsic sizing, defaults, font metrics, subpixel rounding, scrolling and focus behavior. A Button that looks identical can still expose different semantics. “Same everywhere” is not an acceptance criterion until tolerances and deliberate differences are named.

Define portable units, box sizing, font/line-height policy, overflow, supported layout subset and component state semantics. Keep semantic HTML for links/buttons/forms; a generic View cannot replace every meaningful element. Prioritize accessibility and platform text input above screenshot equality. Web keyboard patterns are a useful reference, not a requirement to suppress OS conventions. [WAI keyboard interface](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/)

`className` is a syntax choice, not a CSS compatibility guarantee. Resolve conflicting utilities, variant precedence, arbitrary values, pseudo-states, responsive breakpoints and dynamic class construction explicitly. Reject unsupported rules instead of silently approximating them. Do not implement Tailwind's whole language to preserve familiar spelling. JSON shape inference likewise does not validate color syntax, token cycles, units, contrast or theme fallback.

### 5. Compile-time optimization has been asked to solve runtime problems

Conditionals and keyed lists need runtime insertion, removal, identity preservation and some form of local reconciliation. Avoiding a general virtual DOM does not remove this work. Dynamic signal reads need dependency tracking or a restricted supported subset. A button update can invalidate layout beyond its text node; targeted binding evaluation is not a guarantee of one-node paint.

Keep compile-time templates, runtime semantic instances and renderer scenes distinct. Do not create a giant universal IR that contains every target feature. Start with only the fields required by two actual adapters. If geometry, identity and semantic ownership are mixed, accessibility and server rendering become harder rather than easier.

### 6. “All static pages” is a data consistency and deployment contract

Rows 1, 2, 4 are easy in an immutable fixture. In a real database, pagination can overlap or skip rows while writes occur; rows can disappear between enumeration and loading. A transaction snapshot, frozen export, versioned dataset or explicitly weaker consistency model is needed. The framework cannot infer exhaustive enumeration from arbitrary code.

IDs need not be integers, sequential or globally unique. Tenant, locale and nested params multiply the output set. Enforce bounded work with clear build failure when limits are exceeded, never silently publish the first N pages. Bound memory and concurrency; provenance should record declared build identities rather than dumping loader inputs or provider context. The application controls any additional metadata it explicitly publishes.

Publish a complete generation atomically so pages, assets and route manifests cannot come from different builds. Remove obsolete paths in the next deployment. A row deleted after the previous build remains published until artifacts and caches are replaced; this can become a serious confidentiality issue. A plain static host cannot promise current authorization or revocation. The application/deployment owner decides which data to publish and whether to use access-controlled artifact delivery. The framework documents snapshot persistence and does not silently emit unused loader fields or build-provider context. It cannot infer confidentiality from business values or make a public static host enforce live access rules.

### 7. Partial updates need lifecycle and distributed-state rules

Navigate A → B → C; B completes last. Save twice; responses arrive backwards. Delete while editing. Log out while a request is in flight. A browser back/forward cache restores an old authenticated page. A new server deployment returns markup incompatible with the old browser runtime. Each needs defined behavior.

Use navigation/data generations, explicit cancellation semantics, response schema/build compatibility and dirty-edit policy. Aborting a request or dropping its response does not undo a server mutation. Mutations need their own retry/idempotency and conflict policy. Document that page loaders may run for navigation, prefetch, retries and builds; applications should use explicit mutation handlers for writes. The framework must not disguise writes as required loader behavior, but no automatic effect-system proof of arbitrary application I/O is promised.

One region must have one DOM/state owner. Fragment replacement must release events and subscriptions exactly once, preserve unaffected layout state and not erase active IME composition. Separate navigation focus behavior from background refresh; do not steal focus for every data update. Version skew should fall back to a full navigation or an explicit error, not unsafe patching.

### 8. DI expands ownership obligations and can erase useful boundaries

A generated parameter list is straightforward; a general Nest-like container is not. Request providers, application providers, native window providers and component providers have different lifetimes. Cycles, aliases, transient factories, failures during construction, cleanup order, thread affinity and async initialization all need rules. A singleton retaining a request/session is a security bug even if every pointer is memory-safe.

Use explicit typed provider wiring and test replacements first. Do not infer scope from the apparent lifetime of the first consumer. Parameter injection is parsing/validation plus provenance, not permission: `id: int` says nothing about whether the caller may access that record. Keep invalid syntax, missing record, denied access and infrastructure failure distinct, while allowing application policy to deliberately conceal resource existence.

### 9. Embedded scope is presently aspirational

A host software framebuffer proves drawing and input portability, not MCU viability. RAM/flash, allocation strategy, stack depth, panic/error handling, fonts/assets, DMA/display drivers and timing determine the design. `no_std` is a runtime/library/compiler capability profile, not a renderer toggle. Retain separate embedded Linux and MCU tracks, and make no MCU promise until a chosen device/profile has an actual resource budget and executable proof.

### 10. The product and maintenance surface is too wide for one initial release

Every accepted feature multiplies target, deployment, rendering-mode, ownership, editor, generated-source, security and performance checks. A single platform can establish an experiment, not Windows/macOS/Linux support. Inspectable Rust does not automatically make generated-program crashes debuggable; source maps, panic/error provenance and invalidation diagnostics need work. Hot reload with state preservation adds another lifecycle model; use explicit reset initially.

There is no trustworthy schedule estimate from this review. Novel lifetime/browser work must be measured first. Compiler speed, startup, idle memory, text editing latency, reactive fanout and native build cost need separate baselines; existing compiler/runtime benchmarks do not establish UI performance. A counter fast enough to animate says little about list editing or load. Stop expanding the primitive set until one real data-editing slice works.

## Security review: concrete prospective failure paths

Assets: user/session data; server credentials; generated public artifacts; developer files; native host capabilities; request/provider state. Untrusted inputs: paths/query/forms, stored content, remote data/assets, navigation responses and optional third-party templates. Build code and native application code are trusted executable code unless an explicit sandbox is built. Typed or Rust-backed code is not a sandbox.

The following separates prospective application risks from framework obligations. These are acceptance criteria for the relevant mechanism/helper when implemented, not a requirement for a universal security policy or a reason to delay unrelated UI work.

| Risk | Application-owned choice | Framework/helper guarantee |
|---|---|---|
| Authorization across endpoint variants | Choose identity, object/tenant permissions and where checks apply | Preserve installed middleware/guards across generated representations; no header-selected bypass. Test with explicit sample allow/deny policies |
| Static data disclosure | Select build credentials, output fields and artifact access controls | No fabricated request identity or implicit whole-loader/context serialization; document snapshot and deployment behavior |
| HTML/URL/state injection | Choose raw HTML and its sanitization policy; choose allowed application content | Context-aware encoding for ordinary output, documented URL/state handling and explicit raw-content escape hatch; raw trusted output is not automatically sanitized |
| Fragment execution and targets | Choose trusted content, client scripts and enhancement options | Do not interpret ordinary untrusted text as script/eval or arbitrary template selection; define region ownership and explicit executable-content contracts |
| CSRF and mutation policy | Install/configure protection appropriate to the app; pass tokens to forms; choose mutation authorization | Offered token/validation helpers work as documented when installed, with consistent enhanced-request coverage. Token injection alone is not validation; missing requested context is a type/composition error |
| Cache and request isolation | Select cache headers, keys and any intentional shared application state | Honor chosen headers, distinguish document/fragment representations, isolate request providers, and introduce no hidden shared user-response cache |
| Browser persistence | Choose sensitive-view retention/logout behavior | Document any framework-owned history persistence; provide lifecycle/invalidation controls and avoid unexpected persistent response storage. Host browser behavior is not fully controlled by the framework |
| Build execution and SSRF | Choose trusted build code, credentials, external destinations and egress policy | Discovery/editor do not execute app code; actual build execution is explicit. Supplied fetch helpers honor configured limits; the framework is not a sandbox for arbitrary application networking |
| Path/identity confusion | Configure base paths/proxy integration | One documented normalization contract for matching/building; keep URL and filesystem identity separate; diagnose unsupported or ambiguous cases |
| Scaffolder writes | Choose kind/destination and explicit replacement | Preview accurate writes, reject escapes/conflicts, use safe creation semantics and preserve customized source. Confirmation alone is not overwrite authorization |
| Native capabilities | Decide which file/process/network actions the app exposes | Typed explicit adapter boundaries; no framework-invented remote execution or claim to sandbox native application code |
| Unbounded work | Choose dataset sizes, policies and configured budgets | Define runtime/build limits and failure behavior; clean up owned resources and never silently truncate a complete static build |
| Errors and existence leakage | Choose concealment, error presentation and logging details | Minimal structured default context, ordinary escaping and nonrecursive terminal failure; no automatic stack/provider dumping. App-authored raw disclosures remain app responsibility |
| Component supply chain | Select versions, review custom code and approve migrations | Bundled immutable identities, explicit compatibility, no hidden hooks/downloads or overwrites; preserve installed source as specified |

For applications that require authorization, their policies should cover the relevant requests and object operations; typed IDs do not establish those permissions. [OWASP authorization](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html)

HTML, attribute, URL, CSS and JavaScript contexts need different handling; default text escaping does not cover all of them. CSP is an additional defense, not a replacement for correct sinks. [OWASP XSS prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)

Applications using cookie-backed authentication should select suitable CSRF defenses and safe mutation semantics; providing those helpers does not mandate their use in every application. [OWASP CSRF prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html) Applications accepting untrusted network destinations own their SSRF policy, supported by documented fetch-helper controls where offered. [OWASP SSRF prevention](https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html)

The htmx documentation exposes script/eval controls and warns that HTML history can be stored in localStorage. Evaluate it with those defaults deliberately configured, rather than treating a small library as a security-free adapter. [htmx security](https://htmx.org/docs/#security)

HTTP response variants and private data require explicit cache policy. A page being rendered dynamically does not itself tell browser/proxy/CDN caches how to behave. `Vary` distinguishes representations; it is not authorization. [RFC 9111](https://www.rfc-editor.org/rfc/rfc9111.html)

## Edge-case acceptance corpus missing from the current plan

| Area | Cases that need explicit outcomes |
|---|---|
| Routes/links | Empty ID; ID containing slash/percent/NUL; malformed UTF-8; repeated slash; duplicate query keys; `+` vs space; trailing slash; `.`/`..`; reserved Windows names; case-only and Unicode-normalization collisions; base path; fragments; route renamed during dev |
| Records | ID 0, string UUIDs, leading zeros, ID `new` colliding with a literal route, IDs unique only within a tenant, deleted selected row, stale revision, access revoked, provider unavailable |
| Serialization | 64-bit integer beyond JS safe-number range; negative zero/nonfinite number; null vs missing; timezone/date policy; UTF-8 byte positions vs DOM UTF-16 positions; nonserializable closure/resource/provider; untrusted extra fields |
| Layout/state | Parent param changes but layout remains; two windows share one route; two counters accidentally share state; duplicate/reused list keys; child removal while focused; conditional dependency stops being read; updates during disposal; observer cycles |
| Input/accessibility | Composed emoji, combining marks, RTL mixed text, CJK IME, undo/paste, keyboard-only use, screen-reader actions/announcements, zoom/text scaling, forced colors, reduced motion, touch vs hover, disabled/link semantics |
| Navigation/data | Slow B after fast C; mutation retry; back after logout; dirty input during refresh; offline failure; full refresh of deep link; incompatible deployment response; title/head update without body replacement; scroll/focus restoration |
| Platform/resources | Window closed with queued work; suspend/resume; GPU/device loss; DPI monitor change; missing/corrupt font or image; resource creation fails halfway; inaccessible clipboard; UI-thread callback invoked from worker |
| Static builds | Empty dataset; pagination overlap/gap; row deleted mid-build; duplicate output paths; million-row dataset; provider timeout; partial deployment; obsolete files/caches; private DTO fields accidentally included |
| 404/errors | Missing explicit handler; handler throws; missing layout dependency; direct 404 vs fragment 404; host returns 200 for error artifact; authorization deliberately masks existence; 500 must not be cached as a 404 |
| Scaffolding | Nested cwd/monorepo ambiguity; cancellation; no TTY; existing customized files; symlink ancestors; concurrent process creates destination; crash after first write; invalid source identifiers; non-ASCII names; root config with executable hooks |

For IDs, keep strings at URL boundaries and define numeric conversion explicitly. The current native language's integer and string behavior must be preserved or deliberately adapted in browser data bindings, not silently replaced by JS defaults. This is a backend contract, not just a serializer choice.

## Recommendations before implementation

1. **Freeze a small execution/state contract first.** Specify component construction/disposal, binding/handler access, server/browser/native boundaries, and explicit recipient-visible serialization. Choose one browser counter execution experiment. Do not finalize DI/container or universal loader syntax first.
2. **Run a lifecycle feasibility spike in isolation.** Test the owner/handle model and compiler-relevant rejection witnesses, then report what is already expressible and what needs a language change. Direct Rust success is necessary evidence but not proof of Tsrust acceptance.
3. **Run the paired input/navigation proof early.** One browser and one native OS, real text editing, persistent header, changed outlet, local dirty state and disposal. Compare reuse of a toolkit/text subsystem against custom assembly. Reject a solution that needs duplicate business logic or destroys accessibility to match visuals.
4. **Keep routing/scaffolding as optional bounded work.** It can proceed independently only as provisional tooling. Add collision, explicit 404, limits, nonexecution and safe-write tests. Do not let prototype filenames/metadata become irreversible public API by inertia.
5. **Validate framework mechanisms and explicitly selected sample policies.** Test output encoding, explicit serialization, response variants, provider isolation and execution of installed middleware at the adapter boundary. Protected demos can install auth/CSRF/no-store helpers; public demos remain valid without them. Application authors decide what is published and who may access it.
6. **Defer high-multiplication features.** General DI/decorators, client data reconciliation, shared caches, on-demand generation, full Tailwind/CSS, broad component catalog, state-preserving hot reload, multiwindow shared state and MCU execution remain separate families.

Suggested go/no-go conditions: no leaked/stale state in repeated mount/remove runs; no bypass of installed policies; no undeclared loader/context data in response/build artifacts; real IME/focus/accessibility proof; explainable browser/native data semantics; measured acceptable prototype interaction/build costs. Choose actual budgets before optimization. Failing a condition should narrow the product or change the implementation strategy, not be hidden behind a passing scaffolder demo.

## Changes to proposed wording, not new owner decisions

- Replace “same on every target” promises with a tested portable semantic subset and explicit target capabilities.
- Keep server-first web as the chosen direction, but state that local reactivity still needs a browser execution model.
- Scope the no-tracing-GC promise to the native/application runtime where it is actually enforceable; browser host behavior is not under the compiler's control.
- Treat immutable build snapshots and explicit emitted-output contracts as part of static generation; the application owns confidentiality and deployment access policy.
- Preserve visible application-owned 404 UI, while allowing a minimal non-UI failure response if that handler fails. The fallback handles failure of the error path; it is not a hidden replacement component.
- State that scaffolding supports developer ergonomics but proves neither type safety nor UI feasibility.
- Keep all recommendations here distinct from accepted language semantics. No implementation should start automatically from this review.
