# HTTP

`std:http` uses Axum for routing and Tokio for asynchronous I/O. Create an `App`,
register named async handlers, then await `app.listen('127.0.0.1:3000')`.
Listening consumes the app; registration cannot change while it serves.

See [loopback](../../examples/http/loopback.tsr) for middleware, route parameters,
JSON, body streaming and shutdown; see [validated bodies](../../examples/http/validated-body.tsr)
for schema validation and explicit 400 responses.

Handlers take an owned `Request` and return `Task<Response>`. Middleware takes
`Request` and `Next`. It can return early or await `next.run(request)` once, then
change response headers. Middleware runs in registration order and unwinds in
reverse order, including missing routes and wrong methods. All `use` calls apply
to every route regardless of registration order. Captured handler state is
currently rejected; no hidden shared mutable state is introduced.

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
