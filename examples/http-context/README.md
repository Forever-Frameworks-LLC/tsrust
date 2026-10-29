# Hono-style HTTP context

This section implements request contexts and ordered middleware over the existing
Axum transport. `Router` uses the same `listen`, `Server.address`, `Server.wait`
and `Server.shutdown` lifecycle as `App`. The older explicit Request/Response
adapter remains available; it is not the Hono compatibility contract.

`main.tsr` demonstrates owned configuration captured by a synchronous handler,
exclusive context borrowing, middleware entry and reverse unwind, response header
changes after `await next()`, short-circuiting, and registration order. No
framework authentication or session policy is included.

Focused Router acceptance covers typed variables, cookie helpers, body helpers
and generic error responses before middleware unwinds. See
[application/request state](../http-state/router.tsr) and the
[HTTP guide](../../website/src/content/docs/learn/first-service.md) for supported behavior and explicit limits.
This is a scoped Hono-style API, not a claim of full package compatibility.
Context callbacks currently use `Mutable<Context>` and `ContextNext` annotations
when written as named functions; inline parameter types are inferred. `next()`
consumes its continuation and borrows the sole visible context until completion.
The compiler must reject a second invocation or context mutation during that loan.

Reference contracts: [Hono middleware](https://hono.dev/docs/guides/middleware)
and [Hono Context](https://hono.dev/docs/api/context). Registration and unwind
order are executable assertions, not timing evidence.

The ordered middleware witness also covers scoped paths, multiple callbacks per route, inline async arrows, conditional responses, parameter-route registration order and request-body reads. Server lifecycle methods remain unchanged.
