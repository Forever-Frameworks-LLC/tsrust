# Local LRU workload

`TSRUST=/path/to/tsrust node run.mjs --verify` checks matched native and
Node-after-tsc results without retaining timing samples. Omit `--verify` only at
the cache family measurement milestone. Protocol: three processes per engine,
eight warmups and 31 samples; 256 inserts/lookups per batch, capacity 16.
The independent checksum is `3 * 256 * 255 / 2 + 16 = 97936`.

The TypeScript counterpart implements the exercised owned-result LRU operations
with Map insertion order. This workload does not measure TTL, resource destruction,
concurrent loading or stampede prevention. Native scalar keys and values fit exact
JavaScript integers. Every engine observes each lookup and final size. Source
fingerprints and mutation controls protect the matched scenario.
