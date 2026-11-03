---
title: "Build your first service"
description: "Define HTTP routes and middleware in source, generate clients, and deploy a backend."
head:
  - tag: script
    attrs:
      src: /tsrust/learn.js
      defer: true
---

Build an HTTP service with source-defined routes, shared state and middleware.
Generate clients from the same checked source. API endpoints and authorization do
not need settings in `tsrust.config.json`.

**Before you start:** [install Tsrust](/tsrust/start/installation/) and
[prepare native build tools](/tsrust/tools/rust-setup/).

<div data-learn-track="first-service"></div>

## Create the service

```sh
tsrust new hello --app
cd hello
tsrust dev
```

Open `http://127.0.0.1:4318/` or try:

```sh
curl http://127.0.0.1:4318/api/greet/Ada
```

The greeting route returns `{"greeting":"Hello, Ada!"}`. Routes and middleware
live in `app/main.tsr`; reusable logic lives in `app/lib`, and tests in `app/tests`.
The project config contains ordinary build paths. You do not add an endpoint list,
context type, authorization policy or client-generation settings to it.

The starter uses this pattern:

```tsrust
import { Router } from 'std:http';

type State = { greeting: string; };
async function main(): Task<void> {
  const app = new Router<State>({ greeting: 'Hello' });
  app.use(async (c, next) => {
    c.header('x-content-type-options', 'nosniff');
    await next();
  });
  app.get('/', c => c.json({ greeting: clone(c.env.greeting) }));
  const server = await app.listen('127.0.0.1:4318');
  await server.wait();
}
```

Check changes with `tsrust check`, `tsrust test` and `tsrust format --check`.
Only registered HTTP routes are served. Exporting a helper function does not
expose it. Client generation never registers routes or starts a server.

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

Response objects can be written inline; their shape is inferred:

```tsrust
app.get('/', c => {
  return c.json({ greeting: 'Hello', nested: { ready: true } });
});
```

When copying an owned field from shared application state into a response object,
use `clone`, for example `c.json({ greeting: clone(c.env.greeting) })`.

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

Register access checks with `app.use` before protected routes. Middleware receives
the request context and `next`; returning a response stops downstream handlers.
Awaiting `next()` runs them and then resumes middleware.

```tsrust
app.use('/private/*', async (c, next) => {
  const authorization = c.req.header('authorization');
  if (authorization !== 'Bearer development-only') {
    return c.json({ error: 'Unauthorized' }, 401);
  }
  await next();
});
app.get('/private/profile', c => c.json({ name: 'Ada' }));
```

The fixed token illustrates control flow. Replace it with your application's
credential validation before protecting real data. Keep credentials out of
source and generated clients. Shared application state uses `c.env`; request-local
values use `c.set` and `c.get`. See [application state](#application-and-request-state).
No authorization function or middleware list belongs in JSON configuration.

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
serialization rules; object literals infer their field types. HTML is an
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

Generate from the project's checked build entry. The service need not be running:

```sh
tsrust api generate --out js-client
tsrust api generate --language rust --out rust-client
```

An explicit entry works without a project config:

```sh
tsrust api generate app/main.tsr --out js-client
```

The compiler reads registered Router methods, literal paths and checked JSON
response shapes. It follows handler functions; helpers are not exposed as
endpoints. Dynamic methods or paths cannot form a static contract. A response
whose JSON shape cannot be inferred stays an ordinary HTTP response in the client.
Request bodies remain application-defined; the client does not infer request
validation from arbitrary parsing code.

### TypeScript client

Use the generated package from your own application. A relative import also works
while developing locally:

```typescript
import { createClient } from './js-client/index.js';

const client = createClient('http://127.0.0.1:4318');
const result = await client.get('/api/greet/:name', {
  param: { name: 'Ada' }
});
console.log(result.greeting); // Hello, Ada!
```

Method names and path literals are checked by TypeScript. JSON routes return
parsed, validated response data. TypeScript clients reject integer responses
outside JavaScript's safe integer range; use strings when JavaScript consumers
need wider integers. Tsrust values and generated Rust models retain `i64`
precision. Other routes return a `Response`, so you can
choose `.text()`, `.arrayBuffer()` or streaming. Options accept ordinary fetch
settings, plus `param`, `query` and `json`. Use `headers` for application credentials
and `signal` for cancellation. Non-2xx responses throw `HttpError`; its `response`
contains the original status, headers and body. The optional `fetch` constructor
option accepts your application's fetch implementation.

### Rust client

Add the generated crate as a normal path dependency. Its README lists the method
for each route. Inside an async application:

```rust
use tsrust_service_client::{Client, RequestOptions};

let client = Client::new("http://127.0.0.1:4318")?;
let mut options = RequestOptions::default();
options.param.insert("name".into(), "Ada".into());
let result = client.get_api_greet_name(options).await?;
println!("{}", result.greeting);
```

JSON responses use generated Rust models. Other routes return
`reqwest::Response`. `Client::with_http` accepts an existing Reqwest client for
application transport settings.

### Share and regenerate

Each client includes `http-api.json` and `openapi.json`. Share the contract instead
of backend source; consumers can generate either language without an API config:

```sh
tsrust api generate --description http-api.json --out client
tsrust api generate --description http-api.json --language rust --out rust-client
```

`--url` can fetch an application-hosted contract instead. Publishing it is your
choice; generation adds no description endpoint to your service. Source, saved
contract and URL selection are mutually exclusive. `--out` is required.

Regenerate after route or response changes. Add `--check` to verify an existing
client is current. Generated directories carry ownership markers; regeneration
preserves unrelated application files and rejects edited generated files.
Generated packages are private local packages by default. Bring your own frontend,
package manager and release process.

## Evolve the API

Treat method, path, request and response behavior as your public HTTP contract.
Adding an unrelated route leaves existing clients unchanged. Removing a route,
changing its method or removing a required response field can break deployed
clients. Keep old routes while migrating consumers, then regenerate and test
clients against the destination service. Updating the server executable does not
update client applications.

Existing applications using `api:server` retain their configured RPC protocol and
versioned compatibility tools. See [configured RPC compatibility](/tsrust/tools/configured-rpc/).
Those settings are not part of the Router workflow.

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
[authorization](/tsrust/learn/first-service/#authorize-requests) before adding protected routes. Contract files are local generation outputs;
the starter does not serve a description or OpenAPI endpoint.

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

Generate independent clients from checked routes and follow the
[HTTP rollout guidance](#evolve-the-api) while older clients remain deployed. Updating the server executable does not update client applications.
