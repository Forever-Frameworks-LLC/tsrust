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
measurement protocol applies if `--verify` is omitted. Owned ThreadPool setup/shutdown remains
outside batch timing. `pool-manifest.json` tracks acceptance and source hashes;
`node --test benchmarks/threads/pool-registration.test.mjs` guards the workload.
Current ThreadPool measurement: `pool-join-2026-10-14T23-03-02-577Z.json` in SQLite.
All 93 samples per engine produce checksum 140. Median native/Node batch times
are 0.014875/0.068792 ms. This measures eight tiny tasks across two reusable
workers; it is not an application speedup claim or a comparison with dedicated
thread startup. Current dedicated workloads also passed untimed checks.

The prior global-pool record `pool-join-2026-10-14T22-25-15-491Z.json` remains
immutable in SQLite. The current native median is 1.7% above that record; this
small change does not justify optimization.

Use `--throughput --verify` for four equal CPU jobs returning results through a
bounded native channel. Omit `--verify` to measure serial, two-worker and
four-worker variants using the same 3-process/8-warmup/31-sample protocol. Pool
construction is excluded. Node uses worker messages; native uses owned channel
transfers. A separate BigInt oracle verifies every checksum. Compare within-engine
serial/parallel ratios to assess multicore scaling; cross-engine ratios also
include arithmetic and message-transport implementation differences.

Accepted channel measurement: `channel-throughput-2026-10-15T00-24-43-676Z.json`
in SQLite, with 93 samples per variant/engine and checksum 4791373295 throughout.
On Apple M3, native medians are 3.511500/1.735416/1.052875 ms for serial/two/four
workers (2.02x/3.34x speedups). Node-after-tsc medians are
27.922833/14.795958/7.765750 ms (1.89x/3.60x). These are workload-specific results.

The unchanged tiny-task pool sources were remeasured after channel/cancellation
integration: `pool-join-2026-10-15T00-25-18-930Z.json`. Native/Node medians are
0.015458/0.099500 ms. Native increased 3.9%; the unchanged Node reference increased
44.6%. This does not isolate a compiler regression; no optimization was made from
that signal alone. Prior measurements remain immutable.

Use `--bridge --verify` for the matched Rust recurrence wrapper workload. Omit
`--verify` for the unchanged serial/two/four worker measurement protocol. The
harness copies and fingerprints the complete local Rust implementation, derives
its checked signature, and retains the independent BigInt oracle. This measures
ordinary scalar bridge calls and pool dispatch; heap-buffer transfer is separately
verified by native pointer-identity assertions. `rust-bridge-manifest.json` tracks
acceptance and `rust-bridge-registration.test.mjs` guards exact inputs and protocol.

Accepted bridge record: `rust-bridge-throughput-2026-10-15T01-48-44-424Z.json` in
SQLite. All 93 samples per engine/variant retain checksum 4791373295. Native
serial/two/four-worker medians are 3.282708/1.717542/0.960291 ms (1.91x/3.42x
speedups); Node-after-tsc medians are 26.895666/14.003791/7.701292 ms
(1.92x/3.49x). The equivalent earlier source-only native recurrence medians were
3.511500/1.735416/1.052875 ms. This comparison shows no material slowdown but does
not isolate per-call overhead or establish application-wide speedup.
