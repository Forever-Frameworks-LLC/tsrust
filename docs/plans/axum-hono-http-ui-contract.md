# Axum-backed Hono-like HTTP and UI contract

Status: owner-requested direction, 2026-10-11; API sketches and sequencing recommendations, no implementation. Complements [HTTP/Hono research](../http-and-hono.md), the [standard-library roadmap](../standard-library-roadmap.md), and [UI feasibility review](ui-feasibility-review.md). Hono-like familiarity is the initial recommendation; source compatibility is not promised. Axum/Tower/Tokio are backend dependencies selected by HTTP use, not exposed Rust interop or a second runtime.

## Why establish this boundary

A small HTTP contract makes request lifetime, typed dependencies, middleware, authorization, loader outcomes and HTML/fragment responses concrete. It can resolve several UI server-boundary questions before choosing a native renderer. It does not resolve browser local reactivity, native text editing or UI state disposal; those feasibility proofs remain required.

```text
explicit API routes ─┐
file-based UI routes ├─ checked route/policy manifest
                    ┘          ↓
                 Tsrust HTTP contracts
                 request → middleware → loader/handler
                                   ↓
                     typed data / document / region response
                                   ↓
                       Axum + Tower + Tokio
```

File routing supplies UI route declarations; programmatic registration supplies API routes. Normalize both to a common route identity and reject conflicts. Desktop uses route/data contracts with native adapters; it does not acquire an Axum server merely to navigate locally. Static builds invoke declared build-safe loaders directly and never fabricate an authenticated HTTP request.

## Responsibility boundary

Follow [the vision's application/framework split](../../UI_VISION.md#application-policy-and-framework-guarantees). Auth, CSRF, object permissions, data projection, session and cache policies belong to the application. Helpers are optional, composable capabilities; installed helpers must honor their documented contracts. Middleware propagation, request isolation, ordinary output encoding and explicit serialization belong to the framework. A public route needs no invented identity or mandatory security bundle.

## Candidate authoring surface

See the [editable-list app example](editable-list-app-example.md) for concrete special files and document/fragment flows. The owner confirms page and route are independently optional. A page forbids colocated `get("/")`; a route-only segment may register it. Neither is required at the app root. Other method/path collisions are diagnosed in the shared manifest.

The following illustrates the desired experience only. Names, result types, generic context propagation, async syntax and registration are not implemented by this proposal. A function-based builder avoids requiring classes; these spellings are not yet the final public API.

```typescript
function main(): void {
  const app = createApp();
  app.use(requestId());
  app.use(errorResponses());

  app.get("/api/items/:id", requireSession(), async ctx => {
    const id = ctx.params.id;
    const item = await ctx.services.items.findForUser(ctx.user, id);
    return ctx.json(item);
  });

  app.mountPages("./app");
  // Serving/bootstrap syntax follows the accepted async-main/runtime contract.
}
```

`findForUser` is an application policy/repository operation, not a proposed global built-in; it must distinguish missing, denied and failed outcomes. `requireSession` only authenticates; it does not authorize arbitrary object access. Prefer typed route contracts and explicit provider declarations over a string-keyed context bag. The checker must prove required middleware/providers exist before allowing `ctx.user` or `ctx.services.items`. That proof is new integration work, not something a nominally typed context alone guarantees.

For UI routes, use one checked loader/policy for every representation:

```text
app/items/[id]/page.tsrx
  declared params + provider requirements + access policy
  loader(context) → page input ItemView / NotFound / Denied / Failure
  page(injected data) → typed UI template

direct request       → document renderer (layout + page)
enhanced navigation  → region renderer (changed outlet)
explicit data refresh→ selected recipient-visible data, only when enabled
```

Do not automatically expose every loader as a callable public endpoint. Generated endpoints must retain the same access policy, limits and output projection. Data passed to a template is not automatically safe to serialize wholesale. `Html`/template output is distinct from arbitrary strings; plain `ctx.text` and typed `ctx.json` remain explicit response paths. A raw HTML escape hatch, if any, needs a visibly trusted-content contract.

## Middleware rules to settle before code

Owner clarification: file middleware is a real sequential ancestor chain. `app/middleware.tsr` precedes `app/items/middleware.tsr`, which precedes `app/items/[id]/middleware.tsr` for `/items/4`. Run root-to-leaf on entry and leaf-to-root on response return. A middleware can short-circuit; otherwise it delegates at most once. Child declarations cannot remove ancestors. This rule applies to every page/API/fragment/data/mutation representation. Effective route manifests must show the complete chain. File registration must translate this order deliberately into Tower layers.

Each app segment may also define `layout.tsrx`, `template.tsrx`, `page.tsrx`, `loader.tsr`, `not-found.tsrx`, `error.tsrx` and `route.tsr`. A loader is associated only with its colocated page and injects its checked successful data without page imports/re-exports. It does not run for API methods, create an endpoint or inherit into descendant pages. See [UI_VISION section 18](../../UI_VISION.md) for proposed persistence, dispatch and error-boundary semantics. Special-file presence does not authorize skipping middleware, running server middleware in desktop navigation, or pretending static hosting executes the chain.

- Freeze route registration before serving. Define groups/prefixes and whether each `use` applies globally or only within a declared group; avoid silent registration-order gaps. Export an inspectable effective policy chain per route, including fallback/404 and method-not-allowed behavior.
- When combining file middleware with programmatic `use`, require explicit mount/group placement and reject ambiguous ordering. No implicit second policy pipeline. Test root fallback coverage, protected-segment misses, 405, preflight, early auth rejection and sanitization of middleware failures. A data loader reached through a generated endpoint must receive the same inherited chain as its page.
- Proposed order: registration order on entry, reverse order on return. Test exact traces for normal, early-return and failed requests. Axum `Router::layer` and Tower `ServiceBuilder` compose differently; normalize behind the wrapper rather than leaking that distinction.
- Recommend middleware returns a response explicitly. A conceptual `response = await next(ctx)` transfers the downstream request capability and returns the response for header/status changes. Capture allowed immutable metadata before forwarding; do not promise unrestricted context mutation before/after an await. Exact ownership signatures require a compiler witness.
- `next` is at most once and cannot escape or be invoked concurrently. Existing synchronous Once support is not proof of suspend-safe middleware. Omitting next is an explicit early response. Calling it twice must not duplicate handlers or mutations.
- Specify error conversion independently of Hono compatibility: decide the typed failure channel and central response mapping; never silently copy Hono's promise/error behavior. Error rendering must cover extractor rejection, missing provider, handler failure, middleware failure and renderer failure without dumping secrets.
- The body is consumed once. Authentication/body-signature validation, decoding, logging and uploads cannot each independently consume it. Explicit bounded buffering can support declared reuse; streaming remains a separate contract. No implicit unbounded body cache.
- Post-handler middleware can modify response metadata only before headers are committed. Streaming errors/disconnects need cleanup, not a retroactive status rewrite. Defer streaming/SSE/WebSockets until response lifetimes and backpressure are explicit.

## Security and provider boundaries

Apply the [framework CVE lessons](ui-feasibility-review.md#framework-vulnerability-lessons-from-react-and-nextjs): data-only wire formats, explicit endpoint inventory, trusted internal dispatch state and bounded pre-handler parsing. External headers cannot assert that middleware already ran. Initial HTML/fragment rendering does not introduce a general remote-function or executable-object deserializer. These constraints protect framework machinery independently of the application's chosen auth/CSRF policy.

Application dependencies (for example an explicitly shared pool), request identity, loader inputs and component state have separate owners. Define sharing and thread transfer for each; no implicit Arc/Mutex policy. Provider injection initially means generated typed wiring/extractors, not a reflection container. Missing providers must be diagnosed at composition where possible; construction/runtime failures remain typed outcomes.

A protected application might compose the following path; optional stages are selected by its author:

```text
transport limits / trusted proxy policy / request identity
→ bounded routing and extraction
→ authentication where required
→ object/tenant authorization and mutation protections
→ loader or action
→ application-selected response data
→ document, region or JSON response
→ final headers / safe error mapping / completion logging
```

This diagram is an application example, not required middleware or a universal order: parsing required for authorization, CORS preflight, body-signature authentication and global overload control need explicit placement and tests. Do not label one `secure()` middleware as application authorization. CORS is not authentication or CSRF protection. Trusted forwarded headers must come only from configured proxies.

Framework acceptance requires ordinary HTML/URL/state encoding contracts, full/region representation separation, request isolation, documented resource limits and cancellation/cleanup. Honor application-selected response/cache headers; do not introduce hidden shared page caches or log entire request/provider values by default. Error/404 rendering must not require the dependency that failed. A local development-listener default is a recommendation, not an application authorization policy.

Applications choose auth, CSRF, session, caching, log contents and data publication policies. Supplied helpers need focused correctness tests when implemented; their use is optional. Once installed, the route adapter must preserve their declared scope for fragments, data and mutations. Static builds have no implicit end-user identity; applications may supply trusted build providers and decide what to publish and how to protect the artifacts. No automatic confidentiality classification or mandatory public-data label is required.

## Feasibility work and dependencies

Design work can start independently now: example API/HTML/fragment requests, middleware traces, policy manifest, loader outcomes, provider scopes, explicit error pages and adversarial fixtures. This is more informative about the web architecture than a component generator alone.

An isolated direct-Rust Axum adapter experiment could later execute those fixtures without modifying the compiler, with pinned dependencies and separate integration ownership. It would validate backend behavior only. Do not build it until assigned; do not introduce Cargo dependencies to the active compiler tree during planning.

Actual Tsrust acceptance follows compiler async/await, suspend-safe ownership, retained handlers, thread-transfer policy, runtime bootstrap and HTTP native bindings. Reuse S13–S15 planning; do not mark the full S15 typed-validation/streaming scope complete after a small server. Plain synchronous HTML rendering and protocol fixtures can precede those gates. No hidden async or new custom scheduler to bypass them.

The first server contract witness should use explicit sample application policies and contain one public text endpoint, one protected JSON endpoint, one loader rendered as full HTML and a region, one mutation, and visible application-owned not-found/error handlers. Test exact middleware order, short circuiting, repeated-next rejection, body consumption, two-user isolation, authorization parity across representations, encoding, failed providers and cancellation. Also verify a route with no auth/CSRF helpers remains valid, and that unused loader fields are not automatically serialized. These fixtures prove composition and helper behavior, not arbitrary business-policy correctness. Transport-only Rust tests do not replace eventual compiler/native/editor/generated-source evidence. Streaming/performance work belongs to later bounded milestones.

## Primary references

Axum supplies handlers, extractors and response integration and uses Tower middleware; its runtime direction fits the existing Tokio plan. [Axum overview](https://docs.rs/axum/latest/axum/)

Layer order, middleware error conversion and backpressure placement are observable contracts that the wrapper must define. [Axum middleware](https://docs.rs/axum/latest/axum/middleware/index.html)

Request-body extraction consumes a stream and is distinct from extracting request parts. [Axum extractors](https://docs.rs/axum/latest/axum/extract/index.html)

Hono is the authoring reference for composition and before/after middleware. Its documented error behavior and mutable context must not be assumed equivalent to Tsrust ownership. [Hono middleware](https://hono.dev/docs/guides/middleware)
