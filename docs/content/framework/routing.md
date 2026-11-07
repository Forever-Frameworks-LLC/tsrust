<!-- tsrust-docs
{"id":"reference-framework-routing","group":"framework","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:http","name":"Router"},{"receiver":"std:http","name":"Context"},{"receiver":"std:http","name":"ContextNext"},{"receiver":"std:http","name":"App"},{"receiver":"std:http","name":"Client"},{"receiver":"std:http","name":"text"},{"receiver":"std:http","name":"html"},{"receiver":"std:http","name":"json"},{"receiver":"std:http","name":"request"},{"receiver":"std:http","name":"stream"},{"receiver":"std:http","name":"Request"},{"receiver":"std:http","name":"Response"},{"receiver":"std:http","name":"Body"},{"receiver":"std:http","name":"Server"},{"receiver":"std:http","name":"Next"},{"receiver":"std:http","name":"HttpError"},{"receiver":"std:http","name":"TlsOptions"},{"receiver":"std:http","name":"ClientOptions"},{"receiver":"std:http","name":"ServerOptions"},{"receiver":"std:http","name":"FormField"},{"receiver":"std:http","name":"MultipartPart"},{"receiver":"std:http","name":"getCookie"},{"receiver":"std:http","name":"setCookie"},{"receiver":"std:http","name":"deleteCookie"},{"receiver":"std:http","name":"CookieOptions"},{"receiver":"std:http","name":"streamChannel"},{"receiver":"HttpRouter","name":"get"},{"receiver":"HttpRouter","name":"post"},{"receiver":"HttpRouter","name":"put"},{"receiver":"HttpRouter","name":"patch"},{"receiver":"HttpRouter","name":"delete"},{"receiver":"HttpRouter","name":"head"},{"receiver":"HttpRouter","name":"options"},{"receiver":"HttpRouter","name":"route"},{"receiver":"HttpRouter","name":"use"},{"receiver":"HttpRouter","name":"listen"},{"receiver":"HttpRouter","name":"configure"},{"receiver":"HttpApp","name":"get"},{"receiver":"HttpApp","name":"post"},{"receiver":"HttpApp","name":"put"},{"receiver":"HttpApp","name":"patch"},{"receiver":"HttpApp","name":"delete"},{"receiver":"HttpApp","name":"head"},{"receiver":"HttpApp","name":"options"},{"receiver":"HttpApp","name":"route"},{"receiver":"HttpApp","name":"use"},{"receiver":"HttpApp","name":"fallback"},{"receiver":"HttpApp","name":"listen"},{"receiver":"HttpApp","name":"configure"},{"receiver":"HttpServerOptions","name":"bodyLimit"},{"receiver":"HttpServerOptions","name":"maxConnections"},{"receiver":"HttpServerOptions","name":"maxRequests"},{"receiver":"HttpServerOptions","name":"requestTimeoutMs"},{"receiver":"HttpServerOptions","name":"shutdownTimeoutMs"},{"receiver":"HttpServerOptions","name":"handshakeTimeoutMs"},{"receiver":"HttpServerOptions","name":"requestsPerSecond"},{"receiver":"HttpServer","name":"address"},{"receiver":"HttpServer","name":"shutdown"},{"receiver":"HttpServer","name":"wait"},{"receiver":"HttpRouter","name":"serveStatic"},{"receiver":"std:http","name":"requestBytes"},{"receiver":"std:http","name":"requestStream"},{"receiver":"std:http","name":"requestChannel"},{"receiver":"std:http","name":"requestForm"},{"receiver":"std:http","name":"requestMultipart"},{"receiver":"std:http","name":"Middleware"},{"receiver":"std:http","name":"CorsOptions"},{"receiver":"std:http","name":"cors"},{"receiver":"std:http","name":"secureHeaders"},{"receiver":"std:http","name":"compress"},{"receiver":"HttpCorsOptions","name":"origins"},{"receiver":"HttpCorsOptions","name":"methods"},{"receiver":"HttpCorsOptions","name":"allowedHeaders"},{"receiver":"HttpCorsOptions","name":"exposeHeaders"},{"receiver":"HttpCorsOptions","name":"credentials"},{"receiver":"HttpCorsOptions","name":"maxAgeSeconds"}],"examples":["native-service","http-middleware"]}
-->
# Router and middleware

Compose routes and middleware with Router over Axum. Middleware order and next() unwinding are application-visible behavior. Keep credentials, authorization and session policy in application code.

## Explicit application resource policies

`App` and `Router` construction does not install application body, connection,
concurrent-request, rate or deadline limits. The selected transport's own defaults
remain in force. Call `configure` before listening to supply all seven existing
`ServerOptions` fields. Byte and connection counts must be positive and fit the
target; concurrent requests must also fit Tokio's semaphore. Deadlines must be
positive and fit the monotonic clock. Rate is nonnegative, with zero disabling it.
The former 64-MiB, 65536, one-hour and million-per-second ceilings are removed.

Configured overload responses, cumulative byte checks and deadline cleanup remain
unchanged. Invalid configuration leaves the preceding settings intact. Without an
explicit shutdown deadline, graceful shutdown can wait for stalled requests; set
`shutdownTimeoutMs` when the application requires bounded draining. This changes
the v0.1 implicit one-MiB/128-request/30-second policy.

`c.json(value)` uses the checked serializer without an implicit one-MiB output
ceiling. Existing `JSON.stringify(value,maxBytes)` and
`JSON.tryStringify(value,maxBytes)` retain explicit byte bounds and typed errors.

```tsrust
import {Router,Client,request} from 'std:http';
async function main():Task<void>{
  const app=new Router();
  app.get('/large',c=>c.json({payload:'x'.repeat(1048577)}));
  const server=await app.listen('127.0.0.1:0');
  const client=new Client();
  const response=await client.send(request('GET','http://'+server.address()+'/large',''));
  console.log((await response.text()).length); // 1048591, including JSON syntax
  match(JSON.tryStringify({payload:'x'.repeat(1048577)},1048576)){
    Ok(_)=>{throw 'bound ignored';},
    Err(error)=>{console.log(error.code);} // output-limit
  }
  await server.shutdown();
}
```

Owned binary, body and channel client upload factories are described in the [client reference](/framework/client).

## Source-installed policies

`cors(options?)`, `secureHeaders()` and `compress()` construct owned `Middleware` values. Install them with existing `App.use` or `Router.use`; `Router.use('/api/*', policy, callback)` preserves the same path and source-order rules as authored middleware. Construction alone changes no server. Installation consumes each value once, and aliases/stored factory calls keep that ownership rule.

CORS defaults are closed. Omitted/empty `origins`, `methods`, `allowedHeaders` and `exposeHeaders` allow nothing. `credentials` defaults false; a sole `'*'` is explicit permission and cannot be combined with credentials in any list. Header/method parsing, mixed wildcard lists and negative `maxAgeSeconds` fail with `HttpError` code `options` before installation changes a server. Seconds are nonnegative (zero is valid), without a Tsrust time/cardinality ceiling. Method/header lists advertise preflight permissions; they do not disable routes or change browser rules for simple requests. CORS affects browser visibility, not application authentication.

`secureHeaders` fills absent `X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN` and `Referrer-Policy: no-referrer`, preserving authored values. HSTS, CSP, COOP and COEP remain explicit deployment/application policies.

`compress` streams gzip/x-gzip or HTTP deflate using backend negotiation, level, Vary and framing. Known bodies below 32 bytes, image/gRPC/SSE responses and existing Content-Encoding are skipped. Missing/unsupported encodings and q=0 use identity; the selected backend also treats a bare `Accept-Encoding: *` as identity. Brotli/zstd are not enabled. Compression removes obsolete Content-Length without collecting the body, and propagates source errors. Request bodies keep their exclusive owners; response drop/cancellation settles the active source through existing cleanup. Ordinary HTTP does not select the optional middleware dependency feature.
