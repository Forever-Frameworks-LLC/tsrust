---
title: HTTP routing and request bodies
description: Handle requests with Router, middleware, checked JSON and explicit body ownership.
---

## Router and request context

Use `Router` for the Hono-style API over Axum. [`App`](../http-transport/) retains the older
owned-request interface. The server lifecycle is shared: `listen` consumes the
router, `wait` keeps it serving, and `shutdown` drains cleanup.

```tsrust
import { Router } from 'std:http';

type State = { greeting: string; };
async function main(): Task<void> {
  const app = new Router<State>({ greeting: 'Hello' });
  app.use(async (c, next) => {
    c.header('x-before', 'entry');
    await next();
    c.header('x-after', 'exit');
  });
  app.get('/', c => c.text(clone(c.env.greeting)));
  const server = await app.listen('127.0.0.1:3000');
  await server.wait();
}
```

Matching callbacks run in registration order and unwind in reverse order around
`await next()`. Route registration accepts multiple callbacks. A returned response
short-circuits downstream work. Literal paths, `:name` segments and terminal `/*`
scopes are supported; `*` matches every path. GET also handles HEAD. Unmatched
paths and methods return 404. Optional/regex segments, nested router mounting,
route groups and automatic typed route parameters are not supported.

## Read request bodies

`c.req` exposes request headers, parameters, method and URL. `await c.req.text()`,
`form()` and `multipart()` consume the body once while preserving metadata;
repeated body reads throw. `c.req.body()` transfers the unread stream without
performing I/O and preserves the context for metadata and response helpers.
The stream owns its body; acquiring it counts as the single read, even if the
stream is dropped or later fails. A subsequent `body()`, `text()`, `form()` or
`multipart()` throws `HttpError` code `body`.
`await c.req.json()` parses checked, owned `JsonValue` data and also consumes once,
including on failure. Use `schema.parse(await c.req.json())` for a typed DTO;
`json()` accepts no arguments or type arguments and does not make unchecked casts.
It parses regardless of Content-Type. Empty, malformed or trailing JSON throws
`JsonError` with a code, message and byte offset. Body and UTF-8 failures retain
their `HttpError` type. In addition to the request byte limit, JSON enforces its
16 MiB input, depth 64, and 1000000 entry/node limits. The
[JSON request test](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-context/json.test.tsr) demonstrates schema
validation and a response after reading. `c.json(value)` remains the response helper.
`c.text`, `html` and `json` create responses to return; `c.header` changes response
headers before or after downstream work. `c.res` is read-only: after `await next()`
it exposes the current response status and headers. Before a response returns,
its status is 404. Unhandled handler errors become a generic 500 before middleware
unwinds. There is no custom onError callback or response-body replacement API yet.

## Application and request state

`Router<State, Variables>` owns application state once. `c.env` exposes it read-only;
use supported concurrent handles for pool/cache/channel operations. `Variables`
is a record declaring literal keys for `c.set` and `c.get`. A new request starts
empty. Setting consumes a value and releases any replaced value. Getting returns
undefined when absent; scalar values copy and other values borrow Context for the
containing scope. End that scope before mutating Context. Arbitrary string keys,
caller loans, local resources and detached context work are rejected.

## Cookies

`getCookie(c, name)` returns the first valid matching cookie, percent-decoded with
literal plus signs. Malformed escapes remain literal. `setCookie(c, name, value,
options?)` appends a separate Set-Cookie header, with Path=/ by default and encoded
UTF-8 values. Options include domain, maxAge (0..34560000 seconds), secure, httpOnly,
sameSite and partitioned. `deleteCookie` writes an empty value with Max-Age=0;
match the original path/domain. Invalid attributes throw. Secure/Host cookie
prefix constraints and Partitioned's Secure requirement are checked. Expires,
Priority and signed-cookie helpers are unsupported. Authentication, session expiry
and duplicate-cookie policy belong to application code.

See [context middleware](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-context/main.tsr),
[application/request state](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-state/router.tsr) and
[service routes](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/service/main.tsr) for executable examples.


Continue with [static files](../static-files/) or [clients, streaming and HTTPS](../http-transport/).
