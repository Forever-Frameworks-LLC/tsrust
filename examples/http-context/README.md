# Hono-style HTTP context

This section implements request contexts and ordered middleware over the existing
Axum transport. `Hono` uses the same `listen`, `Server.address`, `Server.wait`
and `Server.shutdown` lifecycle as `App`. The older explicit Request/Response
adapter remains available; it is not the Hono compatibility contract.

`main.tsr` demonstrates owned configuration captured by a synchronous handler,
exclusive context borrowing, middleware entry and reverse unwind, response header
changes after `await next()`, short-circuiting, and registration order. No
framework authentication or session policy is included.

The current implementation is in progress, not a Hono compatibility claim.
Request-local typed variables, cookies, path-scoped middleware, body helper
coverage and error-handler integration remain required before this section closes.
Context callbacks currently use `Mutable<Context>` and `ContextNext` annotations
when written as named functions; inline parameter types are inferred. `next()`
consumes its continuation and borrows the sole visible context until completion.
The compiler must reject a second invocation or context mutation during that loan.

Reference contracts: [Hono middleware](https://hono.dev/docs/guides/middleware)
and [Hono Context](https://hono.dev/docs/api/context). Registration and unwind
order are executable assertions, not timing evidence.
