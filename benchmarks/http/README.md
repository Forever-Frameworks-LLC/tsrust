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
