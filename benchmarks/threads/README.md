# Owned spawn/join workload

This workload starts and joins eight workers sequentially, returning the sum of
squares from 0 through 7 (independent oracle: 140). It measures thread creation,
execution, completion transport and joining. It does not measure multicore speedup.

The Node-after-tsc reference waits for each worker's exit, not only its message.
Node creates an isolate and transports a scalar through worker messages; native
moves the scalar through typed thread storage. These startup and transport costs
are intentional differences, not identical backend work. No pooling is included.

`node benchmarks/threads/run.mjs --verify` builds both fixtures, checks formatting,
and verifies one batch per engine without recording timing. A measured run omits
`--verify` and retains three fresh processes, eight warmups and 31 samples per
engine. Each native or Node process measures its batches internally; executable
startup and compilation are outside that interval. The result retains source
hashes, compiler identity, toolchain versions, host identity and all durations.

Set `TSRUST` to the current verified compiler and `CARGO_TARGET_DIR` to the shared
Cargo target. The harness uses the pinned runtime benchmark TypeScript install.
Import accepted output into SQLite before removing loose evidence. Timings remain
unmeasured until the native-thread family milestone.
