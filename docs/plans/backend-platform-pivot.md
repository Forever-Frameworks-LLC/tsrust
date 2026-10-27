# Tsrust backend application platform pivot

Current direction: the owner's 2026-10-27 cleanup/production reset is defined in
[the production baseline](production-application-baseline.md). Implement real CLI
scaffolding before further task-demo expansion. Default apps are minimal greeting
landing pages; verify fresh generated output, visible dev mode and editor errors.
Accepted backend/SDK foundations and all required acceptance scope are retained.

Date: 2026-10-26. Status: owner-approved pivot; roadmap reconciliation and the
first backend dependency-isolation section are in progress. The working prototype
is not implemented. Follow [the executable prototype goal](../agent-goal.md) and
[template/package decisions](backend-platform-packages.md).

## Product decision

Follow-up: [native targets and secure updates](native-targets-and-secure-updates.md)
records the owner's later Slint/LVGL and optional Slint desktop direction, plus
automatic update setup through a Tsrust-hosted service. Its priority/questions
register governs those future decisions; the current Electron/Expo goal remains
active, and the old universal UI implementation stays frozen.

Keep Tsrust as the backend language and native compiler. Application authors use
TypeScript-style syntax with Tsrust's documented ownership, numeric and async
semantics. React supplies web presentation, Electron supplies the desktop shell,
and Expo supplies mobile presentation. The compiler generates checked client
packages for explicitly exposed backend operations.

The product promise is: author native services in Tsrust, consume them from normal
TypeScript applications, and use established frontend tools. Native compilation
does not guarantee every workload is faster than Node; retain matched measurements
and report backend execution separately from serialization and transport overhead.

Shared contracts and clients are required. Shared web/desktop presentation is the
default. Identical mobile components are not required. Embedded UI and universal
rendering leave the active roadmap until a concrete device/product justifies them.

The owner approved replacing the old universal UI completion goal. Active goals,
roadmap and governing instructions now point here. Preserve implementations and
historical evidence in place; unfinished UI acceptance is frozen, never passed.
React web/Electron uses shadcn; Expo uses React Native Reusables. Electron Forge's
Vite plugin is explicitly accepted despite its documented experimental status.

## Keep, freeze, and replace

| Area | Decision | Reason and boundary |
| --- | --- | --- |
| Parser, checker, ownership, Rust emission | Keep and prioritize | Foundation of native backend authoring; close gaps required by real services. |
| LSP, formatter, diagnostics, VS Code | Keep | Backend authoring must remain reliable; preserve compiler/editor agreement. |
| Modules, Rust bindings, native resources | Keep | Reuse existing checked lifetimes and native integration. |
| Async/tasks, channels, cancellation | Keep | Essential service and sidecar behavior; retain accepted limits. |
| Axum/Hono-style HTTP and middleware | Keep | Reuse accepted service foundation; preserve application-owned auth policy. |
| JSON/schema validation, SQL, migrations, dates, decimal, UUID, bytes | Keep | Backend building blocks and sources of checked boundary contracts. |
| CLI project discovery, build cache, watcher, process supervision | Keep/reuse | Extend to frontend coordination instead of writing another build system. |
| SQLite benchmark storage and original records | Keep | Preserve measured evidence and public export privacy rules. |
| Broad language/Node compatibility inventories | Reprioritize | Required backend features first; no arbitrary npm compatibility promise. |
| `std:ui`, `.tsrx` rendering, `tsrust-ui` | Freeze, then isolate | Preserve source and evidence; stop adding custom rendering features. |
| Table and remaining custom controls, Nova/Base UI ports, GPU/IME/focus/layout work | Stop active expansion | Adopt upstream frontend components instead. Pending checks stay pending or become explicitly retired scope. |
| Tsrust browser WASM UI and reactive framework | Freeze | React owns client rendering/state; backend WASM is not a pivot requirement. |
| Universal page/layout/loader routing, SSR and hydration | Stop custom framework work | Use frontend tools when needed; retain HTTP service routing. |
| Documentation application built with Tsrust UI | Replace later | Preserve authored reference content/examples; move presentation to an established docs tool. |
| Offline component installer, versions, locks, owner-edit guards | Reuse concepts selectively | Apply to template provenance and upgrades; do not retain renderer dependencies merely for installer reuse. |
| Universal embedded/mobile UI abstraction | Defer | No current product witness justifies its cost. |

Frozen code receives only fixes needed to preserve active backend behavior or carry
out isolation. No recurring renderer benchmarks or cross-platform GPU acceptance
are required for the new product. Historical acceptance remains intact.

Current source has a direct `tsrust-ui` dependency in
`crates/tsrust-compiler/Cargo.toml`, and the workspace includes compiler, LSP and UI
crates. Removing the renderer is therefore a traced dependency migration, not a
directory deletion. Keep shared checker, resource, formatting and backend witnesses
when retiring UI-specific suites.

## Application structure

Proposed generated project layout; names are conventions, not implemented syntax:

```text
app/                       # Tsrust backend: operations, service code, startup
web/                       # React application, reusable by Electron
native/                    # Electron main/preload, packaging, desktop capabilities
mobile/                    # Expo application
packages/
  api/                     # Generated framework-independent TypeScript client
  query/                   # Thin generated query options, if separately useful
tsrust.config.json
package.json               # Standard JavaScript workspace and scripts
```

Keep frontend source in ordinary `.ts`/`.tsx`, checked by TypeScript and upstream
tools. Tsrust compiles backend `.tsr` sources. Do not teach the compiler React,
Electron or Expo internals. `packages/api` has no backend executable code,
credentials, database dependencies or accidental imports from server modules.

Desktop uses the web application by default. Mobile shares API calls and suitable
pure TypeScript logic; it uses Expo/React Native presentation. Optional targets
must not require installing every target's toolchain.

## Checked API boundary

Expose operations explicitly. A public module export alone must not publish an
endpoint. Choose the smallest declaration/configuration mechanism after tracing
existing checked HTTP and schema facilities; API spellings remain undecided.

Use one compiler-derived operation manifest for server dispatch, input/output
validation, generated client types and compatibility metadata. It records stable
operation identity, query/mutation kind, serializable arguments/results, public
errors and explicit context requirements. Avoid independently maintained server
and client interfaces.

Start with bounded JSON request/response operations. Define the wire rules before
code generation: optional versus null, tagged unions, finite floats, integer range,
decimal, dates/instants, UUID, bytes, unknown fields and errors. Use safe-range JSON
numbers for initial integers; reject wider exported ranges until an explicit
lossless codec exists. Encode exact decimal/date domain values explicitly rather
than silently coercing them. Ownership wrappers, borrowed references, functions,
SQL pools and opaque native handles cannot cross the boundary. Recursive or generic
exports need a supported finite concrete schema or a useful compiler rejection.

Validate requests in the backend and responses at the generated client boundary.
Return structured public failures; retain internal diagnostic causes server-side.
Enforce operation policy and validation regardless of transport. Auth, tenant and
record access remain application policy; desktop locality does not grant arbitrary
native capabilities. Never automatically serialize request context or service state.

Generate deterministic packages with declaration documentation, runtime codecs,
client calls and an API identity. Handle stale generation in check/build. Specify
rolling compatibility for deployed mobile/web clients; a hash alone is not a
compatibility policy. Test additive and breaking contract changes.

Use TanStack Query for caching/retries/invalidation. The owner requires a wrapper
with named function calls and generated hooks: `api.tasks.list(args)`,
`queries.tasks.list.useQuery(args)` and `queries.tasks.create.useMutation()` are
proposed spellings, not implemented APIs. Also expose typed query/mutation options
for prefetching, loaders and direct TanStack use. Generate query keys/options
and mutation options around the plain client, rather than writing a query engine.
Include operation, canonical arguments and relevant identity scope in keys. Clear
or separate authenticated caches. Do not automatically retry mutations. Cancellation
must distinguish stopping client delivery from acknowledged backend cancellation;
it cannot imply rollback of a committed mutation.

## Runtime modes

| Mode | Proposed behavior |
| --- | --- |
| Hosted web | React assets served by the native service or a static host; API uses HTTP. |
| Hosted mobile | Expo calls the deployed HTTP API; no bundled Tsrust mobile runtime in the initial scope. |
| Desktop local | Electron main starts a bundled native executable; renderer calls allowlisted preload methods. |
| Desktop remote | Same generated API targets a configured hosted service. |

Implement HTTP first. Prove desktop initially with Electron main calling that
same HTTP adapter over an authenticated ephemeral loopback listener. The renderer
does not receive the local service credential. Restrict binding, validate origin
and host as applicable, and avoid a fixed discoverable unauthenticated port.
This provides a small initial integration with existing HTTP support.

After that proof, measure whether private framed stdio IPC is worthwhile. If chosen,
reuse the same manifest/codecs/policy dispatcher with bounded frames, correlation,
backpressure and isolated logs. It must not become a second service implementation.
Do not build both desktop transports before evidence calls for the second.

Launch the native executable as a child process of Electron main. Its lifecycle
includes readiness/version handshake, startup timeout, graceful drain, forced-stop
fallback, crash reporting and no orphaned process on application exit. Avoid a
native Node addon ABI requirement. Desktop binaries must be packaged per supported
OS/architecture; frontend packaging does not cross-compile the Rust backend.

Web asset serving must preserve API routing and limit SPA fallback to eligible
document navigation. Use upstream development servers for HMR. Server rendering,
offline synchronization, automatic failover and deployment-provider automation are
separate future features, not implications of serving a React build.

## Implementation sections and acceptance

### 1. Reconcile scope and isolate backend dependencies

Update the owner goal, continuation checklist, roadmap and UI vision to point to
this direction. Explicitly retire universal UI obligations rather than marking
them passed. Record existing source identity and preserve pending evidence. Trace
UI dependencies through compiler/build scripts/CLI/LSP/native support and separate
the backend path in a coherent section. Preserve existing files until isolation
passes. Backend check/build/run/test must work without initializing a renderer.

Acceptance: representative CLI, HTTP, SQL, async/resource and editor checks on the
new dependency path; useful failures for frozen UI commands. No unrelated compiler
rewrite. Preserve current LSP retention work and unresolved platform limitations.

### 2. Generate one checked HTTP client

Implement the manifest/export boundary and initial codecs. Use a SQLite-backed
task-list service with list/create/update operations, application-owned access
checks and documented public failures. Generate and type-check the plain client.

Acceptance: fresh generation is deterministic; changed backend contracts cause
client type errors or stale-generation diagnostics; malformed inputs/outputs,
integer overflow, null/optional differences and unsupported resources fail safely.
Native execution and an independent TypeScript client agree. Verify policy is not
bypassed through generated dispatch. Focused IntelliSense/examples are complete.

### 3. Ship the web vertical slice

Create a pinned React template with TanStack Query and upstream components.
Implement the task list with loading/error/empty states, editing and validation.
Support normal frontend dev tooling plus production assets served by Tsrust.

Acceptance: clean scaffold/install/build, real browser interaction, generated
client usage, authenticated cache isolation, safe mutation retry behavior, API/SPA
routing and no server-only dependencies or secrets in browser output.

### 4. Bundle the same app with Electron

Reuse web presentation and the generated client through a narrow preload adapter.
Bundle the native backend using the HTTP-first local mode above. Add an explicit
remote mode without automatic local/remote switching of data.

Acceptance: packaged Windows installation on the current host; startup failure,
crash, relaunch, app exit and child cleanup; forbidden renderer operations reject;
installed assets/backend run without the repository or Rust toolchain. Later
macOS/Linux packaging checks require those actual platforms and signing paths.

### 5. Prove mobile and versioned compatibility

Add a minimal pinned Expo template consuming the same generated API over HTTP.
Prove list/edit behavior and a previous client contract against a compatible newer
backend. Diagnose incompatible changes clearly. Verify reconnect and app lifecycle
behavior without promising offline writes.

Acceptance: actual supported simulator/device evidence, no Electron/Node-only
imports in the client, auth/error behavior and compatible rolling upgrade witness.
Do not claim iOS acceptance from Windows-only tests.

### 6. Add cancellable work and the milestone review

Add one long task with explicit cancellation and cleanup semantics after ordinary
calls work. Subscriptions are a follow-up unless an actual prototype requirement
needs them. When added, select an established network event transport and require
unsubscribe cleanup, reconnect rules and bounded backpressure. Document whether
events replay and how clients resynchronize after a gap.

Run the meaningful backend-family regressions and matched performance review.
Measure representative native compute, HTTP round trips, serialization, concurrency
and packaged desktop startup separately. Compare matched Node-after-tsc workloads
with unchanged protocols where comparable; new end-to-end scenarios need independent
oracles and recorded protocols. Existing benchmark samples stay immutable.

Acceptance: all target slices use the same contract, teardown releases resources,
cancellation does not claim false rollback, and measured results distinguish native
work from bridge overhead. Then decide whether to optimize desktop transport.

### 7. Templates, upgrades and final UI retirement

Frontend projects belong to their users. Users can cd into them, install or upgrade
dependencies, edit scripts and manage them using normal upstream tools. Tsrust owns
initial setup/import, generated clients, integration adapters and optional
orchestration, such as serving web assets. Convenience commands must not become a
closed dependency environment. Respect existing toolchains, scripts and lockfiles.
Offer scoped migrations when Tsrust integration changes, including required
integration dependency upgrades; unrelated framework/package updates remain the
user's decision. See the package decision record for exact ownership boundaries.

Support backend/client-only configuration, generated workspace imports, standalone
client export/packing and explicit publication. Existing external projects need
not adopt a Tsrust layout or frontend toolchain. Configure separate static app
mounts and retain a source-level mounting API: a backend can serve several web
apps with isolated asset roots and fallbacks alongside its API.

Keep template versions, pinned lockfiles, provenance and licenses. Vendor editable
application/component source where useful; consume frameworks as normal dependencies
rather than owning framework forks. Never overwrite user edits on upgrade. Initially
offer an explicit scoped migration diff rather than a custom automatic merge engine.
Exercise a Tsrust integration upgrade with a user-edited project and an independently
added dependency, plus a local exported-package consumer and two web app mounts.

Move reference docs to an established presentation stack. After replacement examples
and backend coverage pass, remove unused renderer/build dependencies in a separate
checkpoint. Git history preserves retired source; create no archive trees. Keep
reusable backend regression witnesses extracted from UI tests.

## Execution constraints and first deliverable

Work one section at a time in the shared tree. Inspect status and preserve untracked
owner `.agents/` and `docs/agent-goal.md`. Use local commits, the whole-project
formatter before source checkpoints, one Cargo target/job and disabled incremental
compilation. Batch affected checks per section. Keep complete regressions and timing
for meaningful milestones. No push, release or remote CI is implied by this plan.

The first reviewable deliverable is a reconciled roadmap plus a traced backend-only
dependency plan, followed by the task-list HTTP client proof. Do not begin with a
mass deletion, all-target scaffolding or a new compiler architecture. Keep generated
syntax and package choices provisional until that proof settles the real needs.

## References and evidence boundaries

### SDK compatibility and hosted replicas (owner steering, 2026-10-27)

Separate SDK package semantic version, stable API compatibility version and exact
generated manifest identity. The current implementation admits selected manifest
hashes and proves old clients against a new server; it does not yet prove mixed
old/new replicas behind a hosted endpoint. A compatible regeneration must not
become a deployment failure merely because its exact hash differs.

Generation should compare retained compatibility baselines and reject accidental
breaking updates unless explicitly versioned. Existing calls must remain usable
across compatible SDK updates. Breaking API versions need an explicit retention
and rollout policy. Generated runtime checks should report typed incompatibility
or use an explicitly supported fallback. A new SDK cannot supply a missing server
feature. Support checks must apply to the serving replica, not infer fleet-wide
availability from a response by one random instance. Never retry ambiguous writes
as a version-discovery mechanism.

Add executable old/new client against old/new server combinations through a
mixed-replica endpoint, including compatible schema additions, truly unavailable
features, retained older SDK behavior and bounded useful errors. Preserve current
wire validation, application-owned authorization and old-shape projection while
revising admission. This is required follow-up work, not acceptance of the current
exact-hash gate as a complete hosted deployment protocol.

- Existing backend acceptance: [standard-library roadmap](../standard-library-roadmap.md),
  [service readiness](service-readiness-plan.md), [continuation state](../agent-progress.md).
- Existing UI commitments being replaced: [UI vision](../../UI_VISION.md),
  [documentation/component plan](documentation-and-component-library.md).
- [Electron IPC](https://www.electronjs.org/docs/latest/tutorial/ipc) and
  [security guidance](https://www.electronjs.org/docs/latest/tutorial/security):
  use context isolation and narrow preload capabilities.
- [Expo monorepos](https://docs.expo.dev/guides/monorepos/): use normal workspace
  packages and Expo tooling rather than a custom mobile resolver.
- [TanStack Query](https://tanstack.com/query/latest/docs/framework/react/overview):
  reuse the query library; generated adapters are new Tsrust work.

Repository review supports reuse of substantial backend foundations, not readiness
of this new API generation or packaging product. Current dependency-isolation
checks are recorded separately in the continuation state. No new prototype or
cross-target packaging acceptance is implied by the plan or package research.
