# Default worker cleanup and global Result propagation

Accepted implementation and measurements, 2026-10-14. Source checkpoint `7c3a326c`.

Use `spawn` from `std:threads`. It starts a dedicated OS thread immediately;
`join` waits when called. Several workers started before the first join can overlap.
A handle owns its worker. If that handle leaves scope, cleanup waits for completion
and discards its result or error. This includes return, throw, loop exit and owner
replacement. Moving a handle transfers cleanup responsibility. Cleanup never
turns a worker error into a caller error. `join` observes completion and `detach`
explicitly allows the worker to outlive the owner without waiting.

Global `unwrap(result)` consumes a Result, returns its Ok payload or throws its
original Err payload. Ordinary try/catch/finally handles that typed error.
`unwrap(join(worker))` observes a worker failure; implicit cleanup does not.

```ts
import { spawn, join } from 'std:threads';
function main() {
    const first = unwrap(spawn((): int => 20));
    const second = unwrap(spawn((): int => 22));
    console.log(unwrap(join(first)) + unwrap(join(second)));
}
// 42
```

The [readable worker example](../../examples/scoped-workers/main.tsr) uses this
surface. Existing `spawnScoped`, `ScopedThread` and `std:result` imports remain
compatible; new code needs none of those spellings.

Ownership restrictions remain: no borrowed captures across workers, shared handle
aliases, handle cloning, callback-held handles or handle containers. Spawn errors
still require handling when an expression result is discarded. Conditional handle
expressions retain their existing rejection. Cleanup may wait indefinitely if a
worker does not finish; it does not imply cancellation or detached process survival.

## Next thread priority: configurable default pool

The owner-approved pool remains pending, ahead of further thread features. Expose
configurable global pool management in the standard library, retain a dedicated
thread option, and evaluate Rayon reuse against these contracts before choosing a
backend. No dependency or performance claim is added in this pass.

- Track queued, running and completed-but-unjoined tasks.
- Completion frees its worker immediately. Its outstanding-task slot stays occupied
  until join or owner cleanup consumes completion.
- Bound outstanding tasks separately from worker count. Saturation returns a typed
  spawn error; admission must not silently block a caller that still needs to join.
- Preserve single ownership and typed completion, including cleanup, detach,
  shutdown, configuration timing and nested submissions.
- Acceptance must prove overlap, saturation and recovery, completed-task accounting,
  nested-submit progress and shutdown. Measure matched workloads before claiming
  reduced startup costs. Pool limits and configuration defaults need documentation.

## Additional owner requirements: cancellation, Rayon and channels

Required follow-on work, not optional polish. These APIs are not implemented by
this cleanup pass. Deliver dedicated and pooled cancellation under one completion
contract; ship Rayon bridging and channels for dedicated workers as part of S12.

Cancellation must preserve ownership cleanup. Use cooperative cancellation, with
explicit worker checkpoints and cancellation-aware blocking operations. Requesting
cancellation does not mean the worker has stopped. Keep ownership until join or
cleanup confirms completion; cancelling must not silently detach. A completed task
can win a race with cancellation and return its already-produced result. Queued
work cancelled before starting must dispose its captured data exactly once and
release its worker demand; task accounting lasts until completion is collected.
Represent acknowledged cancellation distinctly from source throws and backend
panics. Document that arbitrary native calls cannot be forcibly interrupted.

Rayon bridging must use the same owned-transfer, typed-error and cancellation
contracts, without hidden copying or shared access to source payloads. Validate
nested submission and join progress, backend panic conversion and shutdown. The
owner requires this bridge; evaluating the integration details is not permission
to omit it or to claim it exists before acceptance.

Dedicated workers need typed channels transferring owned messages. Define bounded
capacity, backpressure/nonblocking admission, sender/receiver close, disconnected
and cancelled waits, and recovery of an unsent owned payload. Cancellation must
wake supported blocking channel operations. Keep channel closure distinct from
successful receipt and from worker cancellation.

Acceptance witnesses: cancel before start, cancel during work, cancellation racing
completion, repeated requests, cleanup after cancellation, queued capacity recovery,
blocking channel wakeup, receiver/sender disconnect, unsent-payload ownership,
Rayon nested tasks and errors. Verify each owned payload is disposed exactly once.

## Focused checkpoint evidence

Compiler acceptance passes: 11 worker tests, 18 Result tests, 7 value-match tests,
and 7 match syntax tests. A native witness preserves global and local user-defined
`unwrap`; existing identifiers are not newly reserved. Runtime overlap and cleanup
witnesses pass. Seven worker editor tests and two match editor tests pass; the
complete typing matrix passes after checked recovery was repaired. Executable catalog verification passes. Consolidated compiler acceptance is
4,008 passed / 7 ignored across 332 targets; editor acceptance is 679 passed /
1 ignored across 54 targets. All three original compiler failures and both original
editor failures were repaired, preserving original records.

Result pattern lowering retains the selected variant before extracting its payload,
matching existing readonly/exclusive provenance. The native-view regression passes
for both a bound error payload and `Err(_)`; success payload ownership remains
protected. Exact original failures and focused repairs are stored in SQLite as
`tsrust-match-worker-final-focus*.log`, `tsrust-match-native-view-final*.log`, and
`tsrust-unwrap-shadow-native.log`. Measurement remains separate from these checks.

## Timing and consumers

The original 3-process / 8-warmup / 31-sample protocols are complete. All native and
Node checksums agree. Median batch milliseconds:

| Workload | Native | Node after tsc |
| --- | ---: | ---: |
| Match values, 2,048 iterations | 0.002167 | 0.012125 |
| Eight sequential workers | 0.139708 | 93.485334 |
| Eight workers in overlapping pairs | 0.101500 | 49.352375 |

These are first baselines, with no comparable earlier timings. Thread costs include
Node isolate/message transport versus native typed thread storage. No application
speedup, pool throughput or cancellation claim follows. Original samples and exact
source/compiler fingerprints are in SQLite; no optimization was justified by these
three native-versus-Node signals.

Release CLI/LSP binaries were built from `7c3a326c` and atomically installed in
`~/.cargo/bin` and `target/editor/debug/tsrust-lsp`. The installed CLI executes the
readable worker example (25205). The primary VS Code server restarted and the
compiler workspace reports No Problems. The second `tsrust_test` window still
requires Developer: Reload Window: its old server was stopped, but UI automation
could not activate its restart command or relaunch VS Code. The updated binary is
installed; do not claim that second running consumer is verified.

The milestone proof is stored as `tsrust-match-worker-acceptance-2026-10-14.json`.
All 558 timing samples are indexed in SQLite and match the original records.
SQLite structure, foreign keys and all 32,830 then-existing blob checksums pass.
The standard path verifier remains blocked by 758 historical absolute record names;
those immutable records were preserved unchanged. Separate storage verification and
cleanup proofs are retained in SQLite. Cleanup removed 390 completed executable
outputs (13.33 GiB logical size), preserving current consumers and owner editor caches.
