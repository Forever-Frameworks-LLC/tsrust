# HTTP loopback

Run with Node v24.11.1 first on PATH and TSRUST pointing at the verified compiler:

```
node benchmarks/http/run.mjs --verify
node benchmarks/http/run.mjs
```

Each batch makes 128 sequential keep-alive GET requests to its own loopback server,
checks status 200 and body `42`, and sums to the independent oracle 5376. Native
uses Axum/Reqwest; TypeScript uses Node HTTP/fetch after tsc. This measures the
combined client/server stack, not isolated server throughput or multicore scaling.
The timing protocol is three processes per engine, eight warmups, 31 samples,
alternating engine order. Verification checks one untimed sample per engine.

Both servers bind port zero and close after sampling. Records include source,
compiler and harness hashes. Import verified closed records into records.sqlite,
then remove the loose JSON. Keep original measurements when comparing revisions.

TLS: `node benchmarks/http/run.mjs --tls --verify` verifies the matched HTTPS
workload; omit `--verify` for the original 3-process, 8-warmup, 31-sample protocol.
Both implementations trust only the fixture CA and reuse connections. The Node TLS
counterpart uses `node:http2` with a persistent session and an explicit private CA, matching
the native client/server HTTP/2 negotiation.
Certificate files are read before timing. Run from the repository root.

`--service` measures 128 in-process authenticated form boundaries: SHA-256,
equal-length token comparison, explicit UTC second flooring, duplicate-preserving URL-form decoding and fixed
structured JSON output. Expected checksum is 128 × (32 + 7 + 2 + 1) = 5376.
Node uses its crypto primitives, Request and URLSearchParams after TypeScript
compilation. Native uses checked bytes, HTTP forms and JSON. Both use valid UTF-8;
Node's permissive malformed-percent behavior is not claimed as parity. This is
not network, database, session lookup or distributed-service throughput. Those
boundaries retain their existing matched workloads and integrated acceptance.
`--verify` builds debug and runs one untimed batch. Measurement retains the
three-process, eight-warmup, 31-sample release protocol; timing remains deferred.

The clock boundary floors a pre-epoch instant to -1000 milliseconds. Native input
retains nine fractional digits; Node Date input has three. Both lie in the same
second, so the independent expected floor is equal without claiming nanosecond
Date precision in Node.

`--context` selects the Hono-style context and middleware workload. It retains
configuration in Router application state, copies its body into a typed request
variable, reads that variable in the handler, and sets headers before and after
delegation. Every request checks both headers and the `42` response body.
The Node counterpart uses an independent explicit context/continuation over Node
HTTP; it is not a measurement of the Hono npm package. Both sides use the existing
3-process, 8-warmup, 31-sample, 128-request protocol. `--verify --context` is one
untimed correctness batch. Source identities are pinned in context-manifest.json.
The v3 state workload passes matched untimed verification; its exact record is in
SQLite. Timing remains unmeasured. Earlier v2 records retain their original
closure-only semantics and are not v3 performance baselines.

`--producer` selects bounded application production and body collection. Each of
128 bodies contains six seven-byte chunks through a capacity-one channel; the
independent checksum is 128 * 6 * 7 = 5376. Node uses a highWaterMark-one object
stream and validates the same complete body. This is in-process streaming, not
network throughput or a claim of identical scheduler costs. The usual timing
protocol is registered; run `--producer --verify` for one untimed batch.
