<!-- tsrust-docs
{"id":"reference-framework-realtime","group":"framework","order":35,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:http","name":"sse"},{"receiver":"std:http","name":"sseWithProducer"},{"receiver":"std:http","name":"connectWebSocket"},{"receiver":"HttpRouter","name":"webSocket"},{"receiver":"HttpWebSocket","name":"receive"},{"receiver":"HttpWebSocket","name":"send"},{"receiver":"HttpWebSocket","name":"split"},{"receiver":"HttpWebSocket","name":"close"},{"receiver":"HttpWebSocketRead","name":"receive"},{"receiver":"HttpWebSocketWrite","name":"send"},{"receiver":"HttpWebSocketWrite","name":"close"},{"receiver":"HttpSseEvent","name":"data"},{"receiver":"HttpSseEvent","name":"event"},{"receiver":"HttpSseEvent","name":"id"},{"receiver":"HttpSseEvent","name":"retryMs"},{"receiver":"HttpSseOptions","name":"keepAliveMs"},{"receiver":"HttpSseOptions","name":"keepAliveText"},{"receiver":"HttpWebSocketHalves","name":"read"},{"receiver":"HttpWebSocketHalves","name":"write"},{"receiver":"HttpWebSocketOptions","name":"readBufferSize"},{"receiver":"HttpWebSocketOptions","name":"writeBufferSize"},{"receiver":"HttpWebSocketOptions","name":"maxWriteBufferSize"},{"receiver":"HttpWebSocketOptions","name":"maxMessageSize"},{"receiver":"HttpWebSocketOptions","name":"maxFrameSize"},{"receiver":"HttpWebSocketOptions","name":"acceptUnmaskedFrames"},{"receiver":"HttpWebSocketClientOptions","name":"ca"},{"receiver":"HttpWebSocketClientOptions","name":"headers"},{"receiver":"HttpWebSocketClientOptions","name":"socket"},{"receiver":"HttpWebSocketHeader","name":"name"},{"receiver":"HttpWebSocketHeader","name":"value"},{"receiver":"HttpWebSocketText","name":"kind"},{"receiver":"HttpWebSocketText","name":"data"},{"receiver":"HttpWebSocketBinary","name":"kind"},{"receiver":"HttpWebSocketBinary","name":"data"},{"receiver":"HttpWebSocketClose","name":"kind"},{"receiver":"HttpWebSocketClose","name":"code"},{"receiver":"HttpWebSocketClose","name":"reason"}]}
-->
# Server-sent events and WebSockets

Import realtime operations and their owned transport types from `std:http`.
SSE uses the ordinary HTTP dependency family. WebSocket operations select the
optional WebSocket dependency family; ordinary HTTP and SSE do not need it.

## Server-sent events

`sse(events, options?)` consumes a `Channel<SseEvent>` and returns a streaming
`Response`. Each event has required string `data` and optional string `event`,
string `id`, and nonnegative integer `retryMs`. Event names reject CR/LF; IDs
also reject NUL. Invalid queued events fail the response body when read.

```tsrust
import {sse, SseEvent} from 'std:http';
import {Channel} from 'std:futures';
async function main():Task<void>{
  const events=new Channel<SseEvent>(1);
  unwrap(events.sender().trySend({data:'hello',id:'1'}));
  events.close();
  console.log((await sse(events).text()).replaceAll('\n','|'));
}
```

Expected output: `data: hello|id: 1||`.

Formatting preserves the selected SSE backend exactly. Empty data produces a
single newline and dispatches no browser event. Newlines split data fields;
CRLF is preserved by the backend's line processing and can produce an extra
browser-visible line break. Do not assume byte equality with a JavaScript SSE
encoder that normalizes line endings.

`SseOptions.keepAliveMs` explicitly enables comment keepalives with a
nonnegative interval. `keepAliveText` supplies their text and rejects CR/LF.
Text without an interval uses a 15-second interval. Omitting both disables
keepalives. There is no implicit channel capacity or event-byte limit: callers
construct the channel and choose any explicit application policy.

For a producer that belongs to one response, use
`sseWithProducer<E>(events, producer: Owned<Job<void,E>>, options?)`. It consumes
both owners. Dropping the unpolled response or partially read body closes its
receiver, cancels the producer and joins actual settlement through the current
scope's cleanup. Producer failure becomes an HTTP body error after queued data;
it cannot replace already sent headers. The original typed cause is retained
until settlement and then dropped exactly once. Catch inside the producer when
its detailed cause must be observed. A client body read does not expose `E`.

Channel-only `sse` does not acquire an external producer's Job. Its caller must
retain and await that owner explicitly, including on cancellation.

## WebSockets

Register `router.webSocket(path, handler, options?)` with an owned async socket
handler. Existing router middleware runs before upgrade, so authentication and
Origin policy remain explicit application choices. There is no automatic
reconnect, origin allowlist or authentication policy.

`connectWebSocket(url, options?)` returns a lazy `Task<WebSocket>`. The URL must
use `ws:` or `wss:`. URL userinfo does not create an Authorization header;
provide authentication explicitly through headers. Client options provide
explicit headers, socket options and optional PEM `ca`. A supplied CA replaces
public roots. Certificate validity and hostname verification remain enabled;
omitting `ca` uses public roots.

A socket receives tagged owned `WebSocketMessage` values: text has string
`data`, binary has owned `Bytes` data, and close has optional `code` and `reason`.
`receive()` returns `undefined` at EOF. Backend ping/pong handling remains
active. `send(message)` flushes the message; `close(frame?: Owned<WebSocketClose>)` completes
the close exchange. Close reasons are limited to 123 UTF-8 bytes by the protocol,
and invalid/reserved wire close codes fail before sending.

`split()` consumes the socket and returns independently owned `read` and `write`
halves. Destructure the result into separate bindings before keeping an operation
pending: `const { read, write } = socket.split()`. The read half offers `receive`; the write half offers `send` and `close`.
Pending operations retain their receiver loans until settlement. Socket owners
and halves cannot be cloned.

Socket options preserve backend defaults: read and write buffer sizes are
128 KiB, maximum message size is 64 MiB, maximum frame size is 16 MiB,
maximum write buffer size is unlimited, and unmasked client frames are rejected.
Optional maximum-size fields accept `null` for unlimited and omission for the
backend default. Sizes are nonnegative target-representable integers; maximum
write buffer size must exceed write buffer size. No extra ordinary cap is added.

Sending binary data transfers its owned vector into the backend byte owner.
Receiving consumes the backend byte owner: unique storage can transfer its
allocation; shared storage requires a backend conversion copy. Text and close
reasons become owned strings through safe UTF-8 conversion, which may reuse the
backend allocation. These conversions do not promise universal zero-copy.
Text and binary tags remain distinct.

Server shutdown preserves the HTTP server's existing graceful policy. Explicit
`ServerOptions.shutdownTimeoutMs` bounds application-selected stalled-client
shutdown; omission does not invent a deadline. Upgraded handlers remain owned by
the listener scope, including disconnect and cancellation cleanup.

The stable `examples/realtime/realtime.test.tsr` suite groups public behavior and
cleanup assertions. Its runtime duration and native build duration are separate;
source examples and backend admission evidence do not substitute for executing
that suite and its relocated offline export.
