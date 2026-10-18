# HTTP and Hono compatibility

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Partial native implementation. `Router` over Axum, ordered middleware, typed context and cookie helpers have focused acceptance. Hono parity and UI integration remain active requirements. See [the current readiness inventory](plans/language-framework-readiness.md). The historical vision below is not an implementation ledger.

**Proposed phase:** Backend proof.

**Scope:** Define HTTP primitives, routing, middleware, compatibility levels, and independent implementation constraints.

**Planning deliverable:** A Hono feasibility study and a minimal native HTTP server before broad compatibility claims.

**Dependencies to coordinate:** [async-and-runtime](./async-and-runtime.md), [ownership-and-memory](./ownership-and-memory.md), [metadata-and-serialization](./metadata-and-serialization.md), [error-handling](./error-handling.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-28"></a>

## 28. HTTP Framework

Eventually create a language-native framework inspired by Hono’s simplicity.
Target experience:
const app = new App();

app.get("/users/:id", async ctx => {
    const user = await users.find(ctx.params.id);

    if (!user) {
        return ctx.json({ error: "Not found" }, 404);
    }

    return ctx.json(user);
});

app.listen(3000);
Desired characteristics:
* tiny core
* low allocation
* async
* middleware
* typed context
* typed routes
* request validation
* WebSocket support
* streaming
* HTTP/1.1
* HTTP/2
* eventually HTTP/3
Research compile-time route generation.

<a id="web-14"></a>

## 14. Hono as a Core Design Target

Investigate making a Hono-compatible HTTP programming model part of the language’s standard backend experience.
This should be considered much earlier than an ordinary library port.
Hono is attractive because its developer API is:
* small
* composable
* TypeScript-friendly
* middleware-oriented
* standards-oriented
* familiar
* portable
* JSX-capable
The target experience might intentionally resemble:
import { Hono } from "http";

const app = new Hono();

app.get("/", ctx => {
    return ctx.text("Hello");
});

app.get("/users/:id", async ctx => {
    const id = ctx.req.param("id");
    const user = await users.find(id);

    if (!user) {
        return ctx.json({ error: "Not found" }, 404);
    }

    return ctx.json(user);
});

app.listen(3000);
Research whether the public API can be made substantially source-compatible with common Hono applications.

<a id="web-15"></a>

## 15. Hono Compatibility Goal

Investigate three possible compatibility levels.
Level 1 — Familiarity
The API feels like Hono but is not source-compatible.
Level 2 — API compatibility
Common Hono source code can be moved with minimal modifications.
Level 3 — Near-source compatibility
A useful subset of ordinary Hono server applications can compile after changing imports/package metadata.
Determine which level is technically and legally appropriate.
Do not require JavaScript runtime behavior merely to preserve compatibility.

<a id="web-16"></a>

## 16. Independent Hono-Compatible Implementation

Do not make the language runtime dependent on executing Hono’s JavaScript implementation.
Instead investigate implementing equivalent concepts directly using native language/runtime primitives.
Potential internal architecture:
Hono-like API
    ↓
compile-time route graph
    ↓
native middleware pipeline
    ↓
typed request/response primitives
    ↓
native HTTP runtime
Potential benefits:
* static route construction
* lower allocation
* no dynamic middleware lookup where unnecessary
* compile-time parameter extraction
* optimized middleware composition
* typed context state
* route-specific validation
* dead-code elimination
The surface can remain familiar even when the implementation differs completely.

<a id="web-17"></a>

## 17. Licensing and Naming

Before claiming literal Hono compatibility or using Hono branding:
* review Hono’s current license
* review trademark usage
* determine requirements for attribution
* distinguish API compatibility from official affiliation
* avoid implying endorsement
API compatibility and independent implementation should be treated separately from copying implementation or documentation.
If Hono’s license permits reuse, still decide whether clean independent implementation provides a healthier compiler/runtime architecture.

<a id="web-18"></a>

## 18. HTTP as a Standard Capability

Consider whether HTTP should be part of the standard library rather than requiring an ecosystem package.
Potential built-ins:
Request
Response
Headers
URL
URLSearchParams
Cookie
FormData
WebSocket
fetch
Using Web-standard APIs may make migration easier.
Then the Hono-compatible router can be extremely thin.
Example:
const app = new App();

app.get("/", ctx => ctx.text("Hello"));
could compile directly to the native HTTP server.

<a id="web-24"></a>

## 24. Hono as a Compiler Stress Test

Use a Hono-style framework as one of the earliest serious compiler workloads.
It exercises:
* generics
* closures
* async
* middleware
* function composition
* route parameters
* string handling
* maps
* context propagation
* shared application state
* error handling
* JSON serialization
* HTML generation
* traits
* lifetime inference
* concurrency
If these patterns become awkward, that is useful evidence that the language design needs revision.

<a id="web-27"></a>

## 27. Potential Compile-Time Route Type System

Investigate whether:
app.get("/users/:id", ctx => {
    const id = ctx.req.param("id");
});
can infer:
id: string
from the route literal.
Potentially:
app.get("/users/:id<int>", ...)
could infer:
id: int
Research whether such syntax helps enough to justify diverging from Hono compatibility.
Compatibility should not prevent optional native enhancements.

<a id="web-28"></a>

## 28. Native Web Standard Library Vision

Potential standard modules:
http
html
json
url
websocket
crypto
fs
process
stream
test
Hono-style routing may sit directly in:
import { App } from "http";
rather than being a third-party dependency.
Alternatively:
import { Hono } from "hono";
could remain available as a compatibility package layered over the same primitives.
Research both approaches.


## Current ordered middleware boundary

The native Hono adapter keeps the accepted `listen`, `wait` and `shutdown`
lifecycle. Callbacks run in source registration order, including overlapping
literal and parameter routes. `use(path, ...callbacks)` accepts literal paths,
`:name` segments and terminal `/*`; global middleware may omit its path.
Method registrations accept multiple callbacks. Async arrows use the same checked
Task ownership as named async functions. `await next()` unwinds in reverse order;
returning a response stops downstream execution. A continuation is consumed once
and holds an exclusive context loan while pending.

Regex paths, optional path segments, route groups and arbitrary wildcard positions
are not supported in this section. Named path values are decoded as UTF-8;
malformed encoded values produce 400. Typed request-local variables and cookie helpers are implemented. Application
error-handler integration remains open. Authentication, sessions
and authorization policy remain application responsibilities.
