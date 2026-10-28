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

Request metadata stays available after reading a context request body. Choose one
body reader; calling another reader afterward throws `HttpError` code `body`,
even if the first read failed or its stream was dropped.

| Operation | Use it for |
| --- | --- |
| `await c.req.text()` | Read the body as UTF-8 text. |
| `await c.req.json()` | Parse checked, owned `JsonValue` data. |
| `await c.req.form()` | Read URL-encoded form fields. |
| `await c.req.multipart()` | Read multipart fields and file parts. |
| `c.req.body()` | Transfer the unread stream without performing I/O. |

### Validate a JSON body

`json()` parses JSON syntax; a schema checks the application's expected shape.
The following route accepts `{"name":"Ada"}` and responds with `Ada` and status 202:

```tsrust
app.post('/echo', async (c) => {
  const value = await c.req.json();
  const schema = s.object({ name: s.string() });
  const decoded = schema.parse(value);
  return c.text(decoded.name, 202);
});
```

Add `import { s } from 'std:schema';` alongside the Router import in the server
example above. The [complete request test](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-context/json.test.tsr)
also demonstrates malformed data, schema rejection and repeated-read failures.

`c.req.json()` accepts no arguments or type arguments and parses regardless of
Content-Type. It does not cast unchecked input to a DTO.

| Failure | Error |
| --- | --- |
| Empty, malformed or trailing JSON | `JsonError` with code, message and UTF-8 byte offset |
| Body transport or invalid UTF-8 | `HttpError` |
| A second body read | `HttpError` code `body` |
| Valid JSON with the wrong schema shape | Schema validation error |

The request byte limit applies first. JSON additionally enforces a 16 MiB input
limit, depth 64, 1,000,000 entries per collection and 1,000,000 total values.
See [listener limits](../http-transport/#listener-limits) to configure the request
budget. Handle errors in your route when clients need a specific error response;
unhandled errors become a generic 500.

## Return a response

Return `c.text(value)`, `c.html(value)` or `c.json(value)` from a handler.
`c.json(value)` serializes a response; it is separate from `c.req.json()`, which
reads request data. Use `c.header` to change response headers before or after
`await next()`.

After `await next()`, read-only `c.res` exposes the response status and headers.
Before a response returns its status is 404. Unhandled handler errors become a
500 before middleware unwinds. Custom `onError` callbacks and replacing the
response body through `c.res` are not supported.

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
UTF-8 values. Options include `domain`, `maxAge` (0..34560000 seconds), `secure`, `httpOnly`,
`sameSite` and `partitioned`. `deleteCookie` writes an empty value with Max-Age=0;
match the original path/domain. Invalid attributes throw. Secure/Host cookie
prefix constraints and Partitioned's Secure requirement are checked. Expires,
Priority and signed-cookie helpers are unsupported. Authentication, session expiry
and duplicate-cookie policy belong to application code.

See [context middleware](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-context/main.tsr),
[application/request state](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-state/router.tsr) and
[service routes](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/service/main.tsr) for executable examples.


Continue with [static files](../static-files/) or [clients, streaming and HTTPS](../http-transport/).
