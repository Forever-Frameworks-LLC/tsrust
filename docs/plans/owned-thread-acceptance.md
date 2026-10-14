# Owned native worker slice

The [scoped-worker ergonomics slice](scoped-worker-acceptance.md) adds opt-in
fallback joining and concise typed error propagation to this foundation.

The initial S12 slice adds `spawn`, `join` and `detach` from `std:threads`. Channels, pools,
Rust-module imports, cancellation and async tasks remain separate work.

`spawn(callback)` consumes a concrete, owned, zero-argument callable and starts
one OS thread. Captures, returns and source throws must pass the existing transfer
classifier. No borrowed caller storage, hidden clones, reference-counted wrappers
or payload serialization are introduced. `Result<Thread<T,E>,SpawnError>` reports
creation failure. E is the inferred thrown payload type, or undefined for a worker
without source failures. A callback returning void produces `Thread<void,E>`.

`join(worker)` consumes the handle, waits, and returns `Result<T,ThreadError<E>>`.
The worker error code retains its original typed thrown payload; panic reports a
backend panic without exposing arbitrary Rust panic storage. `detach(worker)`
consumes the handle without waiting. The detached worker owns its remaining work;
process exit need not wait for it.

Every handle must be joined, detached or transferred on every source path. The
initial storage surface is direct handles and Result wrappers. Containers,
retained callbacks, handles carried into loops, conditional owner expressions and exception-region cleanup
require broader lifecycle analysis and are diagnosed instead of silently dropping
handles. Fallible work while a handle is live is rejected. Ordinary explicit if
branches, owned function parameters and owned returns retain existing move rules.

The local Rust integration direction remains compatible: native callable wrappers
can use the same owned-transfer boundary. Importing a function does not schedule
it. A future pooled submission operation is distinct from this one-OS-thread API;
this slice does not freeze pool admission, scheduler compatibility or shutdown
contracts, and does not add a Rayon dependency prematurely.

Focused acceptance passes on macOS. The paired workload in `benchmarks/threads` measures sequential creation
and joining, with explicit Node isolate/transport differences. Multicore throughput
remains unmeasured and belongs to the broader native-thread family milestone.


## Acceptance evidence

- Workspace/all-target Cargo check: no warnings or errors, normal worker stacks.
- Compiler: seven transfer/runtime tests, 184 native regression tests, six generic
  record checks, 54 integration tests across threads, native bindings, callable
  moves, owned arguments and Result matching; 31 formatting/default-void tests.
- Formatter corpus: 240 lexable fixtures, 84 unchanged standalone contracts;
  whole-project formatting is idempotent and preserves code tokens.
- Editor: compiler agreement, complete typing matrix (only the repaired row
  rerun after its initial failure), consistency, schema editing and six thread
  tests. Completion covers imports and typed worker errors, including unfinished
  members and unrelated syntax errors. Catalog examples execute at Rust O0/O3.
- Native/Node-after-tsc workload verification: both produce checksum 140.
  Source/counterpart/harness/compiler fingerprints and observed checksums retained.
  No timing, multicore-speedup or cross-platform acceptance claim.
- Rust 1.96.1, Node 24.11.1 and TypeScript 5.9.3; one shared Cargo target,
  incremental compilation disabled. No dependency or scheduler added.

Original passing logs and failed attempts are retained in `benchmarks/records.sqlite`.
The summary record is `tsrust-s12-owned-workers-acceptance-2026-10-14.json`.
Consumer refresh and open-editor verification are recorded separately in
`tsrust-s12-consumers-2026-10-14.json`. Full compiler/LSP regressions and timed
comparisons remain due at the broader S12 family milestone.
