# HTTP

## Router and request context

Use `Router` for the Hono-style API over Axum. `App` below retains the older
owned-request interface. The server lifecycle is shared: `listen` consumes the
router, `wait` keeps it serving, and `shutdown` drains cleanup.

```tsrust
import { Router } from 'std:http';

type State = { greeting: string; };
const app = new Router<State>({ greeting: 'Hello' });
app.use(async (c, next) => {
  c.header('x-before', 'entry');
  await next();
  c.header('x-after', 'exit');
});
app.get('/', c => c.text(clone(c.env.greeting)));
```

Matching callbacks run in registration order and unwind in reverse order around
`await next()`. Route registration accepts multiple callbacks. A returned response
short-circuits downstream work. Literal paths, `:name` segments and terminal `/*`
scopes are supported; `*` matches every path. GET also handles HEAD. Unmatched
paths and methods return 404. Optional/regex segments, nested router mounting,
route groups and automatic typed route parameters are not supported.

`c.req` exposes request headers, parameters, method and URL. `await c.req.text()`,
`form()` and `multipart()` consume the body once while preserving metadata;
repeated body reads throw. The context's body cannot be moved into detached work.
`c.text`, `html` and `json` create responses to return; `c.header` changes response
headers before or after downstream work. `c.res` is read-only: after `await next()`
it exposes the current response status and headers. Before a response returns,
its status is 404. Unhandled handler errors become a generic 500 before middleware
unwinds. There is no custom onError callback or response-body replacement API yet.

`Router<State, Variables>` owns application state once. `c.env` exposes it read-only;
use supported concurrent handles for pool/cache/channel operations. `Variables`
is a record declaring literal keys for `c.set` and `c.get`. A new request starts
empty. Setting consumes a value and releases any replaced value. Getting returns
undefined when absent; scalar values copy and other values borrow Context for the
containing scope. End that scope before mutating Context. Arbitrary string keys,
caller loans, local resources and detached context work are rejected.

`getCookie(c, name)` returns the first valid matching cookie, percent-decoded with
literal plus signs. Malformed escapes remain literal. `setCookie(c, name, value,
options?)` appends a separate Set-Cookie header, with Path=/ by default and encoded
UTF-8 values. Options include domain, maxAge (0..34560000 seconds), secure, httpOnly,
sameSite and partitioned. `deleteCookie` writes an empty value with Max-Age=0;
match the original path/domain. Invalid attributes throw. Secure/Host cookie
prefix constraints and Partitioned's Secure requirement are checked. Expires,
Priority and signed-cookie helpers are unsupported. Authentication, session expiry
and duplicate-cookie policy belong to application code.

See [context middleware](../../examples/http-context/main.tsr),
[application/request state](../../examples/http-state/router.tsr) and
[service routes](../../examples/service/main.tsr) for executable examples.

## Legacy App and shared transport

`std:http` uses Axum for routing and Tokio for asynchronous I/O. Create an `App`,
register named async handlers, then await `app.listen('127.0.0.1:3000')`.
Listening consumes the app; registration cannot change while it serves.
For a standalone service, await `server.wait()` to keep its scope alive; see
[server](../../examples/http/server.tsr). Cancelling that waiting task requests
shutdown and its async owner drains cleanup. Process termination is not a promise
of graceful cleanup; use explicit shutdown or task cancellation when coordinating
a controlled stop.

See [loopback](../../examples/http/loopback.tsr) for middleware, route parameters,
JSON, body streaming and shutdown; see [validated bodies](../../examples/http/validated-body.tsr)
for schema validation and explicit 400 responses.

Handlers take an owned `Request` and return `Task<Response>`. Middleware takes
`Request` and `Next`. It can return early or await `next.run(request)` once, then
change response headers. Middleware runs in registration order and unwinds in
reverse order, including missing routes and wrong methods. All `use` calls apply
to every route regardless of registration order. Handlers may own read-only retained configuration and supported concurrent
pools/clients/senders. Each request retains that owner across await. Mutable or
consuming captures, caller loans, program state, local caches and request resources
are rejected. See [retained state](../../examples/http-state/README.md).

`get`, `post`, `put`, `patch`, `delete`, `head`, `options` and `route` register
literal paths with optional `:name` segments. Literal paths outrank parameters.
`input.param('name')` returns `string | undefined`. Duplicate routes and conflicting
parameter names throw `HttpError`. GET reserves HEAD. Catch-all paths and route
groups are not implemented. Missing routes return 404; wrong methods return 405.
Unhandled application errors become a generic 500 without exposing their payload.

`text(value, status = 200)`, `json(value, status = 200)` and
`html(value, status = 200)` construct responses. JSON uses the existing checked
serialization rules; give record values a type before serialization. HTML is an
explicit raw-content escape hatch, without escaping. Use text for untrusted plain
text. Status accepts 200 through 599. Transport framing headers cannot be set by
application code.

Reading `request.text()` or `response.text()` consumes its receiver. `body()`
transfers the single body capability. Await `body.next()` for `Bytes | undefined`,
or `body.text()` for strict UTF-8. `stream(body)` forwards frames on demand; it
does not collect the entire body first. Body reads and forwarding enforce a
cumulative 1 MiB limit, including chunked messages. A limit error does not return
truncated successful data. Errors after response headers cannot change the status.

A `Client` reuses connections. Await `client.send(request(method, url, text))`;
HTTP error statuses remain responses, while I/O failures throw `HttpError`.
The client requires an absolute HTTP or HTTPS URL without embedded credentials.
Redirects, environment proxies, cookies and decompression are disabled. The total
request deadline is 30 seconds; use the existing task timeout for a shorter one.
The task retains a client borrow until settled. The response owns its body.

Listening requires an explicit IP and port; port zero selects a free port. The
server supports HTTP/1.1 and cleartext HTTP/2, with at most 128 connections.
`server.address()` reports the bound endpoint. Await `server.shutdown()` to stop
accepting connections, drain active requests for up to five seconds, then cancel
remaining I/O and drain child tasks. Dropping the server requests this cleanup;
its async owner waits for completion. Client disconnects cancel pending request
work and dispose owned children. CPU work that never suspends remains cooperative.
Cancellation does not promise to execute source `finally` blocks.

TLS listener configuration and certificate fixtures belong to S16. This slice
has no WebSocket/SSE protocol, producer-stream API, request dependency container,
authentication policy, file router or UI renderer. Fixed resource limits are
intentional; no configuration API is implied.

## HTTPS

Use the same listener and client APIs. `app.listen(address, { cert, key })` enables
TLS; omitting the second argument keeps cleartext HTTP. The certificate string is
a PEM chain, leaf first. The private key must match and be unencrypted PEM
(PKCS#1, PKCS#8 or SEC1). Invalid credentials throw `HttpError` with code `tls`
before the socket binds. TLS negotiates HTTP/1.1 or HTTP/2.

`new Client()` verifies public HTTPS certificates using default trust roots.
`new Client({ ca: pem })` trusts only the supplied nonempty PEM CA bundle for that
client. Hostname, certificate chain and validity checks remain mandatory. A failed
handshake makes `send` throw `HttpError` code `request`; configuration errors do not
include certificate or private-key contents in messages.

```tsrust
const server = await app.listen('127.0.0.1:8443', {
  cert: certificatePem, key: privateKeyPem
});
const client = new Client({ ca: privateCaPem });
```

Creating the listener task consumes the app and TLS options immediately; awaiting
it starts the server. Dropping an unawaited task releases those owned settings. Client construction borrows options
only during construction. Handshakes count against the 128-connection limit and
have a five-second deadline. Shutdown interrupts incomplete handshakes and drains
established connections using the existing five-second shutdown budget.

Run `tsrust run examples/http/https.tsr` from the repository root for a complete
local example. Its checked-in credentials are public test fixtures, never deployment
credentials. This slice does not add mutual TLS, certificate reload, ACME, or an
option to disable verification.
