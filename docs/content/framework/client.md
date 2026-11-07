<!-- tsrust-docs
{"id":"reference-framework-client","group":"framework","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"HttpClient","name":"send"},{"receiver":"HttpBody","name":"next"},{"receiver":"HttpBody","name":"text"},{"receiver":"HttpClientOptions","name":"ca"},{"receiver":"HttpClientOptions","name":"bodyLimit"}]}
-->
# HTTP client and bodies

Use the client and body APIs with their explicit ownership and cancellation rules. Streaming support depends on the concrete operation and target; it is not implied by a body type.

A reusable `Client` inherits the selected HTTP backend's ordinary request, redirect
and proxy defaults. Construction does not install a total request/body timeout or
disable automatic redirect following and proxy discovery. `ClientOptions.ca` controls CA trust: supplying it replaces public roots for that client
without disabling certificate verification. Proxy discovery depends on the backend
features compiled into the toolchain; this is not a promise to read every platform's
system proxy settings. Redirect loops fail through `HttpError`. Cookie storage and
automatic decompression remain disabled by the current backend feature selection.

This changes the v0.1 client defaults, which forced a 30-second deadline and disabled
redirects and proxies. The existing constructor has no explicit timeout, redirect or proxy controls.
An optional `bodyLimit` controls cumulative response bytes. Omitting it adds no
response-byte cap.

```tsrust
import {Client} from 'std:http';
function main(){
  const client=new Client({bodyLimit:4096});
  console.log('bounded client');
}
// prints: bounded client
```

`bodyLimit: 0` accepts empty bodies only. Negative or target-nonrepresentable
limits throw `HttpError` code `options` during construction. The budget is
cumulative across chunks; excess data throws code `limit`. It does not truncate
text or disable TLS verification. Omitting `ca` preserves public roots.

`request.form(maxFields?)` and `request.multipart(maxParts?)` accept optional
nonnegative cardinality budgets. The same arguments work on `c.req`. Zero allows
only an empty collection; omitted or `undefined` adds no cardinality cap. Invalid bounds fail with code `options` before a contextual body is taken,
so a later valid contextual call can still read it. Exceeding a valid bound fails
with code `limit`. These controls do not replace the configured request byte budget.

The multipart collector keeps the selected extractor's own 2097152-byte default
when there is no application byte policy. An explicit server `bodyLimit` replaces
that extractor budget, including framing. A cardinality option does not disable
the extractor's byte constraint.

In v0.2, ordinary request/response reads and form cardinality no longer inherit
the former one-MiB/128-item Tsrust ceilings. Explicit `bodyLimit`, `maxFields` and
`maxParts` preserve bounded operations. Backend protocol and allocation constraints
still apply; omission is not a promise that every input can be allocated.
