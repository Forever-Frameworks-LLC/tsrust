---
title: HTTP clients, streams and HTTPS
description: Configure transport limits, manage server lifetime and use verified TLS.
---

Use `Client` to call HTTP services and `Router` to expose routes. This page covers
transport behavior shared by Router and the older owned-request `App` API.
Start with [HTTP routing](../http/) for request handlers.

## HTTP clients

A `Client` reuses connections. Await `client.send(request(method, url, text))`;
HTTP error statuses remain responses, while I/O failures throw `HttpError`.

The client requires an absolute HTTP or HTTPS URL without embedded credentials.
Automatic redirects, environment proxies, cookie storage and decompression are
disabled. You can still set a `Cookie` request header explicitly.

The total
request deadline is 30 seconds; use the existing task timeout for a shorter one.
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

## Responses and body streams

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
cumulative byte limit, including chunked messages. The default is 1 MiB;
listener request bodies use the configured `bodyLimit`. A limit error does not return
truncated successful data. Errors after response headers cannot change the status.

Inside a route, `const body = c.req.body(); const value = await body.text();`
leaves `c` usable for `return c.text(value)`. The stream inherits the configured
request byte limit. See the executable
[context streaming test](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-context/body-stream.test.tsr).

## Listener lifecycle

Listening requires an explicit IP and port; port zero selects a free port. The
server supports HTTP/1.1 and cleartext HTTP/2, with a default maximum of 128 connections.
`server.address()` reports the bound endpoint. Await `server.shutdown()` to stop
accepting connections, drain active requests for up to five seconds by default, then cancel
remaining I/O and drain child tasks. Dropping the server requests this cleanup;
its async owner waits for completion. Client disconnects cancel pending request
work and dispose owned children. CPU work that never suspends remains cooperative.
Cancellation does not promise to execute source `finally` blocks.

## Listener limits

Both `Router.configure(options)` and `App.configure(options)` set limits before
`listen` consumes the application. If you do not call `configure`, the defaults below apply. When you do call it,
all seven integer fields are required:

| Field | Default | Accepted range |
| --- | --- | --- |
| `bodyLimit` | 1048576 bytes | 1..67108864 |
| `maxConnections` | 128 | 1..65536 |
| `maxRequests` | 128 | 1..65536 |
| `requestTimeoutMs` | 30000 | 1..3600000 |
| `shutdownTimeoutMs` | 5000 | 1..3600000 |
| `handshakeTimeoutMs` | 5000 | 1..3600000 |
| `requestsPerSecond` | 0 (disabled) | 0..1000000 |

Invalid fields throw `HttpError` code `options` without changing configuration.
### What happens when a limit is reached

Connections beyond capacity remain in the OS backlog. Concurrent handler overload
returns 503. The global fixed one-second arrival window returns 429 with
`Retry-After: 1` when exhausted. It is not a per-user or distributed limiter;
up to twice the rate can cross a window boundary. Handler deadlines return 504
and request cooperative cancellation; response streaming after handler completion
is outside that deadline. Oversized Content-Length returns 413; streaming reads
reject once their cumulative byte budget is exhausted.

## Application-produced streams

`streamChannel(chunks)` consumes a `Channel<Bytes>` and returns a response without
collecting its body. Sends suspend when the item capacity is full. This bounds
queued chunks, not total bytes; keep individual chunks bounded too. No Content-Type
is added. Dropping all external senders ends the body after queued chunks drain.
Dropping the response closes its receiver, and pending sends fail with code `closed`.

The bridge does not start or detach a producer task. Its owner must keep it alive
until consumption finishes. A task started inside a request is still canceled when
that request scope ends. Producer failure is not encoded as a stream error; choose
an application error protocol. See the [complete producer example](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http/producer-stream.tsr).

## HTTPS

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

## Use the owned-request App API

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

### Handlers and middleware

Handlers take an owned `Request` and return `Task<Response>`. Middleware takes
`Request` and `Next`. It can return early or await `next.run(request)` once, then
change response headers. Middleware runs in registration order and unwinds in
reverse order, including missing routes and wrong methods. All `use` calls apply
to every route regardless of registration order.

Handlers may own read-only retained configuration and supported concurrent
pools/clients/senders. Each request retains that owner across await. Mutable or
consuming captures, caller loans, program state, local caches and request resources
are rejected. See [retained state](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/http-state/README.md).

### Route matching

`get`, `post`, `put`, `patch`, `delete`, `head`, `options` and `route` register
literal paths with optional `:name` segments. Literal paths outrank parameters.
`input.param('name')` returns `string | undefined`. Duplicate routes and conflicting
parameter names throw `HttpError`. GET reserves HEAD. Catch-all paths and route
groups are not implemented. Missing routes return 404; wrong methods return 405.
Unhandled application errors become a generic 500 without exposing their payload.
