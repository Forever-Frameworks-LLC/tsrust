---
title: "Build your first service"
description: "A guided track from a new backend project to a deployed service with generated clients."
head:
  - tag: script
    attrs:
      src: /tsrust/learn.js
      defer: true
---

This track builds a native HTTP service step by step: create the project, add routes and request bodies, serve files, authorize requests, call other services, generate clients, evolve the API safely and deploy it. Each step links to the [language](/tsrust/language/declarations/) and [standard library](/tsrust/library/overview/) reference for the exact rules.

**Before you start:** [install Tsrust](/tsrust/start/installation/), [prepare native build tools](/tsrust/tools/rust-setup/) and run the [quickstart](/tsrust/start/quickstart/). Use [`tsrust test`](/tsrust/tools/testing/) to check each step as you go.

<div data-learn-track="first-service"></div>

## Create the service

Create a service with a greeting operation, then generate clients independently
for the applications that call it. You need the CLI and
[prepared native build tools](/tsrust/tools/rust-setup/).

### Create and run the service

```sh
tsrust new hello --app
cd hello
tsrust dev
```

The starter listens on `127.0.0.1:4318`. Creation refuses an existing destination
and does not install dependencies. `--target backend` is optional; frontend
targets are rejected.

| File | Purpose |
| --- | --- |
| `app/main.tsr` | HTTP routes, server limits and listening address. |
| `app/api/operations.tsr` | Exported operation implementations. |
| `app/api/context.tsr` | Application request context. |
| `app/api/access.tsr` | Authorization policy. |
| `app/lib/greeting.tsr` | Reusable greeting logic. |
| `app/tests/greet.test.tsr` | Native assertions. |
| `tsrust.config.json` | Build entry and selected API operations. |

Use `tsrust new hello` without `--app` for a console project.

### Review access before deployment

The starter authorization function returns `true`: its greeting API is public.
Replace `app/api/access.tsr` before adding protected operations. Authentication
and deployment policy remain your responsibility.

The `api` configuration explicitly selects operations, context and authorization.
Ordinary function exports do not automatically become API operations. The server
imports `dispatch`, `describe` and `openapi` from `api:server`; the compiler checks
and supplies those handlers in memory. Checking and running the service need no
generated client or checked-in dispatcher.

The starter mounts these routes:

| Route | Purpose |
| --- | --- |
| `POST /api` | Dispatch a selected operation. |
| `GET /api/description` | Return the checked description used by SDK generation. |
| `GET /api/openapi.json` | Return the OpenAPI document. |

Description and OpenAPI handlers use the same application access policy as
dispatch. See [authorization](/tsrust/learn/first-service/#authorize-requests) for protected services.

### Check changes

From the project root:

```sh
tsrust check
tsrust test
tsrust format --check
```

For rolling deployments, retain prior checked contracts and list them in
`api.compatible`. Update `api.package.version` for SDK releases; change
`api.apiVersion` for an intentionally breaking API major. Review
[compatibility rules](/tsrust/learn/first-service/#evolve-the-api) before changing a public contract.

For release builds and destination checks, see [deployment](/tsrust/learn/first-service/#deploy).

### Generate a client

Keep the service running and use a second terminal:

```sh
tsrust api generate --url http://127.0.0.1:4318/api/description --out js-client
tsrust api generate --url http://127.0.0.1:4318/api/description --language rust --out rust-client
```

Both commands require an explicit output directory. Generation writes client
files without changing the server or installing packages. Backend development
does not require Node or npm. The TypeScript client uses named asynchronous calls;
the Rust client uses awaitable calls and an optional async HTTP transport.

A generated `http-api.json` can be used later without the running service or its
source:

```sh
tsrust api generate --description js-client/http-api.json --out another-client
```

Remote description URLs require HTTPS; loopback HTTP is allowed. Redirects are
rejected and descriptions are limited to 1 MiB. For protected descriptions and
consumer setup, see [independent API clients](/tsrust/learn/first-service/#generate-api-clients).

### Understand OpenAPI scope

OpenAPI describes the service's single `POST /api` RPC endpoint. If you mount the
dispatcher elsewhere, update the deployment specification's path/server URL and
client endpoint. This export does not import arbitrary REST/OpenAPI services.

## Routes and request bodies

### Router and request context

Use `Router` for the Hono-style API over Axum. [`App`](/tsrust/learn/first-service/#call-other-services-and-stream) retains the older
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

### Match routes and run middleware

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

### Read request bodies

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

#### Validate a JSON body

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
See [listener limits](/tsrust/learn/first-service/#listener-limits) to configure the request
budget. Handle errors in your route when clients need a specific error response;
unhandled errors become a generic 500.

### Return a response

Return `c.text(value)`, `c.html(value)` or `c.json(value)` from a handler.
`c.json(value)` serializes a response; it is separate from `c.req.json()`, which
reads request data. Use `c.header` to change response headers before or after
`await next()`.

After `await next()`, read-only `c.res` exposes the response status and headers.
Before a response returns its status is 404. Unhandled handler errors become a
500 before middleware unwinds. Custom `onError` callbacks and replacing the
response body through `c.res` are not supported.

### Application and request state

#### Share application state

`Router<State, Variables>` owns application state once. `c.env` exposes it read-only;
use supported concurrent handles for pool/cache/channel operations.

#### Store request-local values

`Variables` is a record declaring literal keys for `c.set` and `c.get`. Each request
starts with no stored values.

- `c.set` consumes a value and releases any previous value for that key.
- `c.get` returns `undefined` when the key is absent.
- Scalar values copy; other values borrow Context for the containing scope.
  End that scope before mutating Context.

Arbitrary string keys, caller loans, local resources and detached context work are
rejected.

### Cookies

Import `getCookie`, `setCookie` and `deleteCookie` from `std:http` as needed.

| Helper | Behavior |
| --- | --- |
| `getCookie(c, name)` | Returns the first valid matching cookie, or `undefined` when absent. Percent-decodes the value; plus signs stay literal and malformed escapes remain literal. |
| `setCookie(c, name, value, options?)` | Appends a separate Set-Cookie response header and percent-encodes the UTF-8 value. |
| `deleteCookie(c, name, options?)` | Writes an empty value with Max-Age=0. Match the original path and domain. |

#### Cookie options

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

Continue with [static files](/tsrust/learn/first-service/#serve-static-files) or [clients, streaming and HTTPS](/tsrust/learn/first-service/#call-other-services-and-stream).

## Serve static files

Serve a built frontend alongside your API using `Router.serveStatic`. Mount only
public deployment assets; the directory must exist before the server starts.

### Mount a frontend

Build your frontend into `web/dist`, configured for the URL prefix `/web/`.
Then register the mount before starting the listener:

```tsrust
import { Router } from 'std:http';

async function main(): Task<void> {
  const app = new Router();
  app.serveStatic('/web', 'web/dist', 'index.html');
  const server = await app.listen('127.0.0.1:3000');
  await server.wait();
}
```

Open `http://127.0.0.1:3000/web/`. The example requires `web/dist/index.html`.
Relative filesystem paths resolve when the mount is registered; use an absolute
path if the process may start from a different working directory.

| Argument | Meaning and constraints |
| --- | --- |
| `prefix` | A non-root URL path such as `/web` or `/admin`, without a trailing slash. Each segment permits ASCII letters, digits, hyphens and underscores. |
| `directory` | An existing directory containing public files. |
| `fallback` | Optional existing relative `.html` file inside that directory. Omit it or pass an empty string to disable fallback. |

Add another mount with `app.serveStatic('/admin', 'admin/dist', 'index.html')`.
Prefixes cannot overlap: `/web` and `/web/admin` cannot be mounted separately.

### Requests and fallback behavior

Files use streamed GET/HEAD responses with MIME types, range requests and
conditional dates. Earlier matching routes retain registration priority; other
HTTP methods continue to later application routes.

| Request | Result |
| --- | --- |
| `/web` | 307 redirect to `/web/`, preserving the query string. |
| `/web/` or an existing directory | Serve its `index.html` when present; no directory listing. |
| Existing file under `/web/` | Serve the file. |
| Missing extensionless path with `Accept: text/html` | Serve the configured fallback, if enabled and HTML is acceptable. |
| Missing path with a file extension, such as `.js`, `.woff2` or `.png` | 404; never the HTML fallback. |
| `/api`, `/admin` or `/webish` | Outside this mount. |

An absent Accept header or only `*/*` does not request the HTML fallback.
`text/html;q=0` also disables it. The extension check keeps typical missing asset
requests from receiving an HTML page they cannot use. An extensionless asset path
can receive the fallback if the request explicitly accepts HTML.

### Invalid paths and deployment safety

Invalid configuration throws HttpError code `static` before registration.
Malformed path escapes, hidden/dot segments, encoded separators, controls,
double encoding and unsafe Windows path forms return 400. Resolved paths outside
the root, including outside-root symlinks, return 404.

Roots are trusted read-only deployment assets: concurrent
local filesystem writers are outside this containment contract. Never mount
uploads, source, credentials or a directory with adversarial writers. Public
selection, cache/CSP headers and any access policy remain application-owned.
Configuration strings are borrowed/copied; the router retains no caller loans.

Use [Router](/tsrust/learn/first-service/#routes-and-request-bodies) to register API handlers alongside these public mounts.

## Authorize requests

Select the operations clients may call and supply an application-owned access
policy. Exporting a function alone does not expose it over HTTP. Start with a
[backend application](/tsrust/learn/first-service/#create-the-service) for a complete working configuration.

### Configure the server

In `tsrust.config.json`, the `api.server` object identifies the context type and
authorization function. This fragment shows the starter's paths; merge it into
its existing `api` settings rather than replacing the whole configuration:

```json
{
  "server": {
    "context": {
      "module": "app/api/context.tsr",
      "export": "AppContext"
    },
    "authorize": {
      "module": "app/api/access.tsr",
      "export": "authorize"
    },
    "maxRequestBytes": 65536,
    "maxResponseBytes": 65536
  }
}
```

Paths are relative to the configuration directory. Use forward slashes, without
`.` or `..` path segments. Both byte limits default to 65536 and accept integers
from 1 through 16777216.

Import `dispatch`, `describe` and `openapi` from `api:server` and mount only the
handlers you need on your Router. The starter mounts all three. They share the
same authorization hook, including requests for the public API description.
Compilation checks these handlers in memory; it needs no generated dispatcher
file or client output directory.

### Make an access decision

The configured authorization function receives the native HTTP context and
returns `Task<boolean>`. It runs before dispatch reads or decodes the request
body or calls an operation:

- Return `true` to allow the request.
- Return `false` to send HTTP 403 with `api-denied`.
- Throwing sends HTTP 500 with `api-internal`, without exposing the exception.

The starter returns `true`, so its greeting API is public. Replace that policy
before exposing protected operations. Application credentials, state and policy
stay in backend modules; the generated SDK does not implement authentication
for you. See the repository's
[application-owned bearer policy](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/backend-tasks/access.tsr)
for a larger example with origin checks and explicit CORS preflight handling.

### Pass private context to an operation

Add `"context": true` to an entry in `api.operations` when its first parameter
is `ReadOnly<Context<...>>` or `Mutable<Context<...>>`, including a named alias.
The compiler supplies that parameter on the server. It is absent from the wire
input and does not change the public contract identity.

The remaining input is zero or one supported plain-data argument. An arbitrary
pool/resource or an owned `Context` cannot be hidden as the context parameter.
Keep resources in application state and pass only supported data to clients.

### Bound requests and responses

`maxRequestBytes` limits the UTF-8 request text's byte length after it has
been read. `maxResponseBytes` bounds serialization of operation results and their
success envelopes. These guards do not replace the Router's admission limits.
Configure its body limit, concurrency and timeouts too; see
[listener limits](/tsrust/learn/first-service/#listener-limits).

Dispatch requires POST and `application/json`. It admits the selected operation
only when its contract identity or versioned per-operation fingerprint matches a
supported profile. Use a generated client to supply this protocol metadata.

| Condition | HTTP status | API error code |
| --- | --- | --- |
| Policy denies access | 403 | `api-denied` |
| Method is not POST | 405 | `api-method` |
| Missing or unsupported content type | 415 | `api-media-type` |
| Body exceeds the API request limit | 413 | `api-request-limit` |
| Invalid request or operation arguments | 400 | `api-validation` |
| Operation is undeclared | 404 | `api-operation` |
| Incompatible operation profile | 409 | `api-incompatible` |
| Result fails its checked schema | 500 | `api-result` |
| Policy, operation or result serialization throws | 500 | `api-internal` |

Router admission failures can occur before these handlers run. The table covers
ordinary dispatcher failures, not every malformed protocol-metadata case.
Public errors use bounded messages rather than exposing backend exception text.
Typed domain-result unions remain successful response data. Failure or
cancellation does not imply rollback: an operation may already have changed state.

### Retain older contracts

List reviewed baseline files in `api.compatible`; `api.server.compatible` remains
supported for older configurations. Keep each baseline immutable.

Compatibility checks require old inputs to fit the current operation and current
results to project into the old client's strict shape. SDK history also checks
whether old replica results satisfy a new client. Unknown or incompatible shapes
are rejected before invocation. Schema compatibility cannot prove unchanged
business behavior; see [versioning and rollout](/tsrust/learn/first-service/#evolve-the-api).

### Export a dispatcher for an existing project

Normal `api:server` compilation needs no physical dispatcher. For an existing
project that requires one, set `api.server.output` to a private directory such
as `app/generated-api`, then explicitly run `tsrust api generate`.

Keep that private directory separate from the public client package. Generation
checks the handler and output ownership before replacing files. Generated hash
markers and locks protect both destinations; edited files are preserved.
Replacements are individually atomic and retryable, not one transaction across
the filesystem.

`--check` writes nothing. `--out` overrides the client destination while the
configured private server keeps its own destination. Omit `server` for
client-only generation from source.

## Call other services and stream

Use `Client` to call HTTP services and `Router` to expose routes. This page covers
transport behavior shared by Router and the older owned-request `App` API.
Start with [HTTP routing](/tsrust/learn/first-service/#routes-and-request-bodies) for request handlers.

### HTTP clients

A `Client` reuses connections. Await `client.send(request(method, url, text))`;
HTTP error statuses remain responses, while I/O failures throw `HttpError`.

The client requires an absolute HTTP or HTTPS URL without embedded credentials.
Automatic redirects, environment proxies, cookie storage and decompression are
disabled. You can still set a `Cookie` request header explicitly.

The total request deadline is 30 seconds; use the existing task timeout for a shorter one.
A timeout while sending throws code `timeout`; other transport failures use `request`.
Invalid URLs throw code `url` before a request is sent.
Reading the response body can also fail after `send` succeeds.

The task retains a client borrow until settled. The response owns its body.

Save this complete loopback example as `main.tsr`, then run `tsrust run main.tsr`.
It starts a local server on a free port, prints `200` and `Hello`, then shuts down:

```tsrust
import { Router, Client, request } from 'std:http';

async function main(): Task<void> {
  const app = new Router();
  app.get('/', c => c.text('Hello'));
  const server = await app.listen('127.0.0.1:0');
  try {
    const client = new Client();
    const response = await client.send(request('GET', 'http://' + server.address() + '/', ''));
    console.log(response.status());
    console.log(await response.text());
  } finally {
    await server.shutdown();
  }
}
```

### Responses and body streams

#### Construct a response

`text(value, status = 200)`, `json(value, status = 200)` and
`html(value, status = 200)` construct responses. JSON uses the existing checked
serialization rules; give record values a type before serialization. HTML is an
explicit raw-content escape hatch, without escaping. Use text for untrusted plain
text. Status accepts 200 through 599. Transport framing headers cannot be set by
application code.

#### Read or forward a body

Reading `request.text()` or `response.text()` consumes its receiver. `body()`
transfers the single body capability. Await `body.next()` for `Bytes | undefined`,
or `body.text()` for strict UTF-8. `stream(body)` forwards frames on demand; it
does not collect the entire body first. Body reads and forwarding enforce a
cumulative byte limit, including chunked messages. The default is 1 MiB;
listener request bodies use the configured `bodyLimit`. A limit error does not return
truncated successful data. Errors after response headers cannot change the status.

#### Keep the request context usable

Inside a route, `const body = c.req.body(); const value = await body.text();`
leaves `c` usable for `return c.text(value)`. The stream inherits the configured
request byte limit. See the executable
[context streaming test](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-context/body-stream.test.tsr).

### Listener lifecycle

Listening requires an explicit IP and port; port zero selects a free port. The
server supports HTTP/1.1 and cleartext HTTP/2, with a default maximum of 128 connections.
`server.address()` reports the bound endpoint. Await `server.shutdown()` to stop
accepting connections, drain active requests for up to five seconds by default, then cancel
remaining I/O and drain child tasks. Dropping the server requests this cleanup;
its async owner waits for completion. Client disconnects cancel pending request
work and dispose owned children. CPU work that never suspends remains cooperative.
Cancellation does not promise to execute source `finally` blocks.

### Listener limits

Both `Router.configure(options)` and `App.configure(options)` set limits before
`listen` consumes the application. If you do not call `configure`, the defaults below apply. When you do call it,
all seven integer fields are required:

| Field | Default and accepted values |
| --- | --- |
| `bodyLimit` | Default: 1048576 bytes. Range: 1 to 67108864 bytes. |
| `maxConnections` | Default: 128. Range: 1 to 65536. |
| `maxRequests` | Default: 128. Range: 1 to 65536. |
| `requestTimeoutMs` | Default: 30000 ms. Range: 1 to 3600000 ms. |
| `shutdownTimeoutMs` | Default: 5000 ms. Range: 1 to 3600000 ms. |
| `handshakeTimeoutMs` | Default: 5000 ms. Range: 1 to 3600000 ms. |
| `requestsPerSecond` | Default: 0 (disabled). Range: 0 to 1000000. |

Invalid fields throw `HttpError` code `options` without changing configuration.

#### What happens when a limit is reached

| Limit | Observable behavior |
| --- | --- |
| Connection capacity | Extra connections remain in the OS backlog. |
| Concurrent handlers | Overload returns 503. |
| Requests per second | Exhausting the global fixed one-second arrival window returns 429 with `Retry-After: 1`. |
| Handler deadline | Returns 504 and requests cooperative cancellation. |
| Request body bytes | Oversized Content-Length returns 413; streaming reads reject when their cumulative byte budget is exhausted. |

The arrival limit is not per-user or distributed; up to twice the rate can cross
a window boundary. Response streaming after handler completion is outside the
handler deadline.

### Application-produced streams

`streamChannel(chunks)` consumes a `Channel<Bytes>` and returns a response without
collecting its body. Sends suspend when the item capacity is full. This bounds
queued chunks, not total bytes; keep individual chunks bounded too. No Content-Type
is added. Dropping all external senders ends the body after queued chunks drain.
Dropping the response closes its receiver, and pending sends fail with code `closed`.

The bridge does not start or detach a producer task. Its owner must keep it alive
until consumption finishes. A task started inside a request is still canceled when
that request scope ends. Producer failure is not encoded as a stream error; choose
an application error protocol. See the [complete producer example](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http/producer-stream.tsr).

### HTTPS

Use the same listener and client APIs. `app.listen(address, { cert, key })` enables
TLS; omitting the second argument keeps cleartext HTTP. The certificate string is
a PEM chain, leaf first. The private key must match and be unencrypted PEM
(PKCS#1, PKCS#8 or SEC1). Invalid credentials throw `HttpError` with code `tls`
before the socket binds. TLS negotiates HTTP/1.1 or HTTP/2.

`new Client()` verifies public HTTPS certificates using default trust roots.
`new Client({ ca: pem })` trusts only the supplied nonempty PEM CA bundle for that
client. Hostname, certificate chain and validity checks remain mandatory. A certificate-verification
failure makes `send` throw `HttpError` code `request`; configuration errors do not
include certificate or private-key contents in messages.

The following is a fragment inside an async function. `app` is a configured
Router or App; `certificatePem`, `privateKeyPem` and `privateCaPem` are strings
loaded from your certificate files, and `Client` is imported from `std:http`.

```tsrust
const server = await app.listen('127.0.0.1:8443', {
  cert: certificatePem, key: privateKeyPem
});
const client = new Client({ ca: privateCaPem });
```

Creating the listener task consumes the app and TLS options immediately; awaiting
it starts the server. Dropping an unawaited task releases those owned settings. Client construction borrows options
only during construction. Handshakes count against the configured connection limit and
have a five-second default deadline. Shutdown interrupts incomplete handshakes and drains
established connections using the configured shutdown budget (five seconds by default).

Run `tsrust run examples/http/https.tsr` from the repository root for a complete
local example. Its checked-in credentials are public test fixtures, never deployment
credentials. Mutual TLS, certificate reload, ACME, and disabling certificate verification
are not supported.

### Use the owned-request App API

`std:http` uses Axum for routing and Tokio for asynchronous I/O. Create an `App`,
register named async handlers, then await `app.listen('127.0.0.1:3000')`.
Listening consumes the app; registration cannot change while it serves.
For a standalone service, await `server.wait()` to keep its scope alive; see
[server](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http/server.tsr). Cancelling that waiting task requests
shutdown and its async owner drains cleanup. Process termination is not a promise
of graceful cleanup; use explicit shutdown or task cancellation when coordinating
a controlled stop.

See [loopback](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http/loopback.tsr) for middleware, route parameters,
JSON, body streaming and shutdown; see [validated bodies](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http/validated-body.tsr)
for schema validation and explicit 400 responses.

#### Handlers and middleware

Handlers take an owned `Request` and return `Task<Response>`. Middleware takes
`Request` and `Next`. It can return early or await `next.run(request)` once, then
change response headers. Middleware runs in registration order and unwinds in
reverse order, including missing routes and wrong methods. All `use` calls apply
to every route regardless of registration order.

Handlers may own read-only retained configuration and supported concurrent
pools/clients/senders. Each request retains that owner across await. Mutable or
consuming captures, caller loans, program state, local caches and request resources
are rejected. See [retained state](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-state/README.md).

#### Route matching

`get`, `post`, `put`, `patch`, `delete`, `head`, `options` and `route` register
literal paths with optional `:name` segments. Literal paths outrank parameters.
`input.param('name')` returns `string | undefined`. Duplicate routes and conflicting
parameter names throw `HttpError`. GET reserves HEAD. Catch-all paths and route
groups are not implemented. Missing routes return 404; wrong methods return 405.
Unhandled application errors become a generic 500 without exposing their payload.

## Generate API clients

The backend is the API source of truth. New applications select their public
operations and authorization in `tsrust.config.json` and import compiler-owned
handlers from `api:server`; checking and running never require generated clients
or dispatcher files. See [backend applications](/tsrust/learn/first-service/#create-the-service).

Generate independent TypeScript or asynchronous Rust SDKs from a live service:

```sh
tsrust api generate --url http://127.0.0.1:4318/api/description --out js-client
tsrust api generate --url http://127.0.0.1:4318/api/description --language rust --out rust-client
```

Use `--description http-api.json` instead of `--url` for a saved description, without
backend source. The same checked description exports OpenAPI.

SDK generation does
not change application source, register operations or start a server.

URL and saved-description generation always require `--out`. For source-based
generation, `api.output` can supply the default destination.
Optional `api.server.output` retains explicit legacy dispatcher export, which new
applications do not use. Keep deployment compatibility baselines application-owned
rather than depending on a generated client directory.

### Generate from source configuration

Use the existing `api` settings in `tsrust.config.json`, or create a separate
`tsrust.api.json` for a client export. The following standalone configuration uses
the greeting function from the [backend starter](/tsrust/learn/first-service/#create-the-service). Save it at
the starter project root and select it explicitly with `--config`:

```json
{
  "schemaVersion": 1,
  "apiVersion": 1,
  "entry": "app/api/operations.tsr",
  "output": "packages/api",
  "package": { "name": "@example/api", "version": "0.1.0" },
  "operations": [
    {
      "name": "greet",
      "module": "app/api/operations.tsr",
      "export": "greet",
      "kind": "query"
    }
  ]
}
```

The entry is checked through the normal module/compiler path and can be a library
module without an executable main function. Selected modules
must belong to that checked graph and selected names must be actual named function
exports. An ordinary export is never automatically an operation. Query/mutation
kind is explicit metadata, not proof that a query is side-effect-free. Application
access policy remains a server responsibility when dispatch is added.

```sh
tsrust api generate --config tsrust.api.json
tsrust api generate --config tsrust.api.json --check
tsrust api generate --config tsrust.api.json --out ./client-package
```

#### Paths and configuration selection

Without `--config`, the CLI searches upward for API configuration. If the same
directory has both inline `api` settings and `tsrust.api.json`, it requires an
explicit selection. Configured
entry/module/output paths are relative to that file and exclude `.`/`..` and
backslashes; use portable forward slashes. An explicit output flag is relative to
the current directory and can select a standalone destination. Package
versions require `major.minor.patch`; prerelease labels and build metadata are
not supported. See [version rules](/tsrust/learn/first-service/#choose-version-numbers).

### Understand generated output

The TypeScript output contains JavaScript ESM, TypeScript declarations, a manifest, package
metadata and the generated runtime's MIT notice. It has no install scripts or
framework dependencies. No backend bodies, source paths, credentials, database
dependencies or source-tree imports are copied into it. The package is private
by default. Review the generated files, package metadata and your own licensing
before enabling publication; generation/check/build never publishes anything.

#### Regenerate safely

Existing nonempty application directories are not adopted as generated output.
Only previously generated files whose recorded hashes still match, or already
match the intended replacement, are replaced. Edited generated files produce a
diagnostic and remain untouched. Unrelated files remain user-owned. `--check`
does not write files and fails for stale outputs or markers. A lock excludes
concurrent generators, and files are individually replaced atomically. A failed
partial replacement can be retried using the old/planned hash checks.

### Calls and wire values

The TypeScript package exposes `createClient(transport)`, `ApiError`, a frozen
manifest and a contract identity. Named calls follow your selected operation
names: `greet` becomes `api.greet(...)`, while `tasks.list` would become
`api.tasks.list(...)`.

To try the generated starter client without a running server, save this as
`try-client.mjs` beside `js-client`, then run `node try-client.mjs`:

```js
import { createClient } from './js-client/index.js';

const api = createClient({
  async request(operation, input) {
    if (operation !== 'greet') throw new Error('Unexpected operation');
    return { ok: true, value: `Hello, ${input}!` };
  },
});
console.log(await api.greet('Ada')); // Hello, Ada!
```

This transport is a local demonstration. Use the HTTP adapter to call the service.

A transport implements
`request(operation, checkedInput, options)` and returns either
`{ok:true,value}` or `{ok:false,error:{code,message}}`. Input is validated and copied
before transport. Result validation rejects malformed values and unexpected
fields. Calls return promises; they add no caching, retries or implicit mutation
invalidation. Optional cancellation signals are forwarded to the transport and do
not imply rollback or backend cleanup completion.

Operations currently accept no arguments or one required data argument. Put
multiple inputs in a request record. Rest/default/optional top-level parameters
and unresolved generic functions require concrete wrapper functions. Async
operation signatures describe eventual data, not transportable tasks.

#### Supported data

| Value | Contract |
| --- | --- |
| Strings | Well-formed Unicode; lone UTF-16 surrogates reject before transport. |
| Booleans and null | Preserved; null is distinct from a missing field. |
| Numbers | Finite values only. JSON encodes negative zero as zero. |
| Native integers | Must fit ±9,007,199,254,740,991; larger values reject instead of rounding. |
| Arrays and tuples | Dense arrays; tuple length and each element are checked. |
| Plain records | Declared data properties only; accessors and unknown keys reject. |
| Optional fields | Missing or undefined properties are omitted. Required fields must exist. |
| Literals and unions | Values must satisfy a declared alternative. |
| Void results | Encoded as null, returned as undefined by the TypeScript client. |

Special JSON keys remain data without changing object prototypes. Recursive or
deeper-than-128-level schemas, functions and native resources are rejected.
Collections and domain objects need explicit conversion to supported data;
there is no implicit serialization for them.

### Call the service over HTTP

The separate `@example/api/http` export uses the host Fetch API. The plain package
entry still requires no DOM types or Fetch globals. TypeScript HTTP consumers need their
host's Fetch type declarations, whether supplied by browser, Node or React Native
tooling. Pass an absolute endpoint URL:

With the starter listening on port 4318, save this as `try-http.mjs` beside
`js-client` and run it with a Node version that provides Fetch:

```js
import { createClient } from './js-client/index.js';
import { createHttpTransport } from './js-client/http.js';

const api = createClient(createHttpTransport({
  url: 'http://127.0.0.1:4318/api',
}));
console.log(await api.greet('Ada')); // Hello, Ada!
```

For an installed package, use its configured name, such as `@example/api` and
`@example/api/http`, instead of relative file imports. Protected services can
supply `headers` as an object or an async function returning headers. For example,
`headers: async () => ({ authorization: 'Bearer ' + await currentToken() })`
requires your own `currentToken` function. Pass `{ signal: controller.signal }`
as the second call argument when using an application-owned `AbortController`.

#### Request headers and credentials

Each call posts `{operation,input}` as JSON to the configured endpoint and sends
`x-tsrust-contract` with a selected checked wire-profile identity. JSON Content-Type
parameters carry the requested API major, current operation fingerprint and current checked manifest identity. This adds no new CORS header names. The server retains
application authorization responsibility. Headers are resolved
for each call so credential changes take effect; the adapter sets its protocol
headers after application headers. Cookies are omitted and redirects reject.
Use HTTPS for remote credentials. Loopback HTTP is available for the desktop bridge.
URLs containing embedded credentials or fragments reject during construction.

#### Errors and cancellation

Non-2xx responses can carry the same structured public failure envelope. A success
envelope on a failing HTTP status rejects. HTML, invalid JSON and invalid output
data reject with protocol/HTTP/validation errors. Host network exception details
are replaced with an `api-network` error; cancellation uses `api-cancelled`.
There are no automatic retries, including mutations. Cancellation requests abort
through the host Fetch API; it does not prove rollback or backend cleanup.

Supply `fetch` explicitly for a host-specific adapter or testing.

#### Response size

Response text is read before parsing; this client
does not impose a streaming memory limit. Backend request/response bounds and
mobile Fetch behavior remain service/platform acceptance requirements.

For host Fetch usage, see
[the Fetch API documentation](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch).

### Ordinary local package adoption

Generate into an explicit folder, then use the application's package manager:

```sh
tsrust api generate --config tsrust.api.json --out ./dist/client
cd dist/client
npm pack --ignore-scripts
cd /path/to/existing-app
npm install /path/to/project/dist/client/example-api-0.1.0.tgz --ignore-scripts
```

Replace the absolute paths above with your project and consumer paths.
The tarball filename follows the configured npm package name/version. Inspect
`npm pack --json --ignore-scripts` before sharing. Its allowlist includes runtime,
declarations, contract, README and license; generated ownership markers and
unrelated files are excluded. Generated packages are private by default; local
packing works without publication. Consumers need no compiler, backend source or
generated-directory symlink. Generated plain SDKs have no framework or cache peers.

#### Workspace consumers

For an npm workspace, include the generated package directory in the root's
workspaces and declare its configured name/version in the consuming application's
normal dependencies. Users own their package manager, scripts and dependencies.
Other package managers use their ordinary package/workspace workflows; SDK
export never installs packages or rewrites the consuming application's files.

### Use the Rust client

The Rust export contains `Cargo.toml`, owned DTO types and asynchronous named
calls. Add it as a normal Cargo path dependency in your consumer. Its default
`http` feature supplies `HttpTransport`; disable default features when providing
a custom transport. Run HTTP calls inside your application's Tokio runtime.
Generation does not fetch Cargo dependencies.

Read the generated `README.md` and `src/lib.rs` for the package name and exact
operation types. Optional fields use `Option`; nullable unions preserve explicit
null separately. The Rust generator currently rejects tuples longer than 16
elements. Do not assume the TypeScript SDK's Fetch behavior describes the Rust
transport's limits.

### Next steps

- [Configure authorization and dispatch](/tsrust/learn/first-service/#authorize-requests) on the server.
- [Retain compatible SDK contracts](/tsrust/learn/first-service/#evolve-the-api) for rolling deployments.

## Evolve the API

Keep old clients working while deploying new server replicas. Retain the checked
contracts you have released, and let generation reject incompatible changes
before it replaces output files.

### Choose version numbers

In the project's `api` settings:

| Setting | Meaning |
| --- | --- |
| `package.version` | SDK version in `major.minor.patch` form, such as `0.2.0`. |
| `apiVersion` | Positive API compatibility major; defaults to `1`. |

SDK version components must be nonnegative integers without leading zeroes.
Prerelease labels and build metadata are not accepted. Versions cannot move
backwards. Raising the API major requires raising the SDK major too.

The generated `contractId` identifies the checked manifest. `wireContractId`
also includes the API major, while preserving legacy v1 identities. Identical
schemas in different API majors therefore have different wire identities.

### Decide whether a change is compatible

Generated `compatibility.json` retains deduplicated checked schemas and immutable
SDK release references. Keep it with generated output.

For source-based generation, the compiler reads canonical history even when using
`--out` and merges destination history before replacing output files. Generation
from an HTTP description uses the history carried by that description. It rejects
a description that omits history already retained by the destination SDK.

| Change | Requirement |
| --- | --- |
| Change a checked schema | Choose a new SDK version. |
| Keep the same API major | The server must still accept released inputs, and older replica results must satisfy the new SDK. |
| Add an optional result field | Can remain compatible when older replicas omit it. |
| Require a new result field | Use a new API major, or keep the field optional during rollout. |
| Raise the API major | Raise the SDK major as well. |

Business semantics still need application review; schema containment cannot prove
them.

### Publish a changed SDK

These checks preserve release-to-contract identity, not an npm registry's release
immutability or every generated runtime byte. Regeneration with a newer Tsrust
compiler can update runtime code without changing DTOs. Select a new package
version before publishing any changed package, including runtime-only changes.
Generation does not publish or silently change package versions for the owner.

### Retain application-owned baselines

The server retains its own baseline files under application-owned paths, referenced
by `api.compatible`. It never needs the generated client's `compatibility.json` directory
to start. Copy a reviewed release baseline into the app when retaining it for a
rollout; the served description includes the configured history.

Known published baselines can be imported with `api.compatible`, a list of
relative `manifest.json` or `compatibility.json` paths. Legacy `api.server.compatible`
remains supported. A fresh selected client can omit methods from foreign server
baselines, but subsequent releases cannot silently remove its own published calls.
Existing owned pre-metadata packages bootstrap from their manifest/package version.

### Roll out across mixed replicas

Both generated clients select a retained profile before I/O. Its input codec must
accept the actual supplied arguments, and its output must be accepted by the
current SDK. Modern dispatchers also recognize retained per-operation fingerprints,
independently of unrelated selected methods.

Strict response codecs and old-client projections remain in force. The clients
never strip supplied arguments or retry requests to find support. Common calls
can survive mixed replicas; newly supplied arguments or new operations can require
newer servers.

#### Handle a compatibility failure

- **TypeScript/JavaScript:** `ApiCompatibilityError` extends `ApiError` and
  includes the requested `apiVersion`.
- **Rust:** `ApiError` includes `api_version`; `is_compatibility()` identifies
  compatibility failures.

Both retain the `api-incompatible` and `api-operation` codes. Do not infer fleet-wide
capabilities from one replica or retry ambiguous writes. Retain older handlers or
migrations for as long as older API majors must work.

## Deploy

Build your service on the OS and architecture where it will run. Prepare the
[native build tools](/tsrust/tools/rust-setup/) first, then run these commands from
the configured [backend project](/tsrust/learn/first-service/#create-the-service) root.

### Check and build

Check the source and run its discovered tests before producing a release build:

```sh
tsrust check
tsrust test
tsrust format --check
```

On macOS or Linux:

```sh
tsrust build --release -o target/server
./target/server
```

In Windows PowerShell, choose an executable filename explicitly:

```powershell
tsrust build --release -o target/server.exe
.\target\server.exe
```

`--release` selects optimized native compilation. `-o` selects the output path;
it does not select a target platform. These build commands have no cross-compilation
switch. The server stays running until stopped; use your deployment environment's
process supervisor to manage its lifetime.

### Choose the listening address and access policy

The starter listens on `127.0.0.1:4318`, which accepts connections from the same
host. Set the address passed to `listen` in `app/main.tsr` for your deployment's
network arrangement. A reverse proxy on the same host can forward to loopback;
remote clients need an appropriately reachable listener or proxy.

The starter greeting is deliberately public. Review
[authorization](/tsrust/learn/first-service/#authorize-requests) before adding protected operations. Its
`/api/description` and `/api/openapi.json` handlers use application policy too;
decide whether those routes should be reachable by your clients.

Configure TLS termination, request limits and process supervision for the actual
service. See [HTTP transport](/tsrust/learn/first-service/#call-other-services-and-stream) for the native TLS and server
configuration APIs. Keep deployment secrets out of source and generated client
packages.

### Copy the executable and application files

The compiled program runs without the Tsrust compiler, Cargo, Node or a JavaScript
runtime. Copy the executable plus any configuration, assets and other files your
application reads at runtime. For example, a static-file directory must remain
available at the path your application uses; a relative path depends on the
process working directory.

A native executable is not necessarily statically linked. The destination may
still need OS libraries, external native dependencies and TLS trust material.
Building for the same OS and architecture alone does not establish compatibility
with every OS release or container image.

### Verify the destination

Run the actual artifact in the intended destination image or machine. Verify that
it starts, can read its required files, accepts the intended requests, rejects
unauthorized requests and can reach its external services. Check the process's
working directory and environment as part of that deployment.

`tsrust doctor --native` validates the build host. It does not prove that a copied
executable will run on another machine. A successful source test run also does not
exercise your production network, certificates or access configuration.

### Roll out API changes

Generate independent clients from the checked contract. Keep compatible prior
contracts available while older clients are deployed, and follow the
[API compatibility guide](/tsrust/learn/first-service/#evolve-the-api) when changing operation shapes or
versions. Updating the server executable does not update client applications.
