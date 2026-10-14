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
Import accepted output into SQLite before removing loose evidence. The first timing baselines were accepted with the match/default-cleanup milestone
on 2026-10-14.

Use `--scoped` for paired concurrent workers with explicit fallback joining and
`unwrap` propagation. It preserves eight workers, the independent sum-of-squares
oracle (140), and the original timing protocol. The Node counterpart starts each
pair before awaiting completion and waits for both in finally. Source, counterpart
and harness fingerprints are recorded automatically. `--verify --scoped` checks
results without recording timing samples; no speedup is claimed from that check.

`scoped-join` exercises dedicated worker cleanup and global `unwrap`.
Both original workloads retain dedicated `spawn` and their startup semantics. The workload
name and original measurement protocol remain stable; records retain source hashes.

Accepted SQLite records: `spawn-join-2026-10-14T21-27-32-540Z.json` and
`scoped-join-2026-10-14T21-27-41-620Z.json`. Each engine retains 93 samples and
checksum 140 throughout. Native/Node median batch times are respectively
0.139708/93.485334 ms sequentially and 0.101500/49.352375 ms for overlapping
pairs. These are startup/transport microbenchmarks with different backend costs;
there is no earlier comparable timing or application-speedup claim. These original records do not measure a pool.


Use `--pool --verify` for the new two-worker pool workload. Both engines reuse two
workers across paired tasks and all batches; native uses Rayon, Node uses worker
messages. The checksum remains 140. The original 3-process/8-warmup/31-sample
measurement protocol applies if `--verify` is omitted. Pool setup/shutdown remains
outside batch timing. `pool-manifest.json` tracks acceptance and source hashes;
`node --test benchmarks/threads/pool-registration.test.mjs` guards the workload.
No pooled performance measurements have been accepted yet.
