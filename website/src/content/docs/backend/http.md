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

## Match routes and run middleware

Matching callbacks run in registration order and unwind in reverse order around
`await next()`. Route registration accepts multiple callbacks. A returned response
short-circuits downstream work.

| Pattern or request | Behavior |
| --- | --- |
| Literal path | Matches that path. |
| `:name` segment | Matches a named path segment. |
| Terminal `/*` | Matches a path scope. |
| `*` | Matches every path. |
| HEAD request | Uses the matching GET route. |
| Unmatched path or method | Returns 404. |

Optional/regex segments, nested router mounting, route groups and automatic typed
route parameters are not supported.

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

Register this route **before** calling `app.listen`, which consumes the router.
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

### Share application state

`Router<State, Variables>` owns application state once. `c.env` exposes it read-only;
use supported concurrent handles for pool/cache/channel operations.

### Store request-local values

`Variables` is a record declaring literal keys for `c.set` and `c.get`. Each request
starts with no stored values.

- `c.set` consumes a value and releases any previous value for that key.
- `c.get` returns `undefined` when the key is absent.
- Scalar values copy; other values borrow Context for the containing scope.
  End that scope before mutating Context.

Arbitrary string keys, caller loans, local resources and detached context work are
rejected.

## Cookies

Import `getCookie`, `setCookie` and `deleteCookie` from `std:http` as needed.

| Helper | Behavior |
| --- | --- |
| `getCookie(c, name)` | Returns the first valid matching cookie, or `undefined` when absent. Percent-decodes the value; plus signs stay literal and malformed escapes remain literal. |
| `setCookie(c, name, value, options?)` | Appends a separate Set-Cookie response header and percent-encodes the UTF-8 value. |
| `deleteCookie(c, name, options?)` | Writes an empty value with Max-Age=0. Match the original path and domain. |

### Cookie options

| Option | Accepted values and default |
| --- | --- |
| `path` | An absolute cookie path; defaults to `/`. |
| `domain` | Optional domain; omitted by default. |
| `maxAge` | Integer seconds from 0 through 34560000. Omit for a session cookie; 0 requests deletion. |
| `secure` | Boolean, defaults to false. |
| `httpOnly` | Boolean, defaults to false. |
| `sameSite` | `Strict`, `Lax` or `None` (lowercase also accepted); omitted by default. |
| `partitioned` | Boolean, defaults to false; requires `secure: true`. |

Invalid attributes throw `HttpError` with code `cookie`. Names prefixed with
`__Secure-` require `secure: true`. Names prefixed with `__Host-` additionally require
`path: '/'` and no domain. Expires, Priority and signed-cookie helpers are unsupported.

Authentication, session expiry and duplicate-cookie policy belong to application
code.

See [context middleware](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-context/main.tsr),
[application/request state](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-state/router.tsr) and
[service routes](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/service/main.tsr) for executable examples.

Continue with [static files](../static-files/) or [clients, streaming and HTTPS](../http-transport/).
