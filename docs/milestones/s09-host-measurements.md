# S09 synchronous host measurements

Measurement acceptance is complete on macOS at source checkpoint `0347e340`.
S09 family acceptance is complete on macOS after full regressions, final installed
consumer checks and editor verification at `b921b2fb`. This report does not accept
S10, additional host APIs or untested native platforms.

The canonical SQLite proof is `runtime/diagnostics/s09-measurement-acceptance.json`
with SHA-256 `ba8f3981a78c3b52d7881e15116569a3ae4ac3c28eb603de03a9af9b7cc162db`.
It binds nine original runtime records and one allocation record. All original
bytes were restored, verified, indexed and removed from loose storage. The database
retains exact originals, toolchain/source identities, startup/compilation results,
individual execution samples and allocation counters.

## Protocol and results

Each workload uses native and Node-after-tsc, three processes, eight warmups and
31 execution samples per process, plus 11 startup samples under the existing
runtime/startup/compilation protocol. Five independent correctness cases pass per
workload. There are 1,674 execution samples and 27 native allocation samples.

The table reports median batch duration across 93 samples per engine. The ratio
is Node duration divided by native duration; values above one favor native.
These are workload results, not isolated API-call timings.

| Workload | Native ms | Node-after-tsc ms | Ratio |
|---|---:|---:|---:|
| File write | 88.019666 | 93.898916 | 1.067 |
| File writeAll | 87.567125 | 95.233166 | 1.088 |
| Full-file write/read | 89.747916 | 93.016250 | 1.036 |
| Buffered file write/read | 99.212292 | 111.778250 | 1.127 |
| Environment get | 2.225584 | 3.460875 | 1.555 |
| Monotonic clock | 0.571083 | 0.487208 | 0.853 |
| OS platform | 0.031708 | 0.114834 | 3.622 |
| OS architecture | 0.034334 | 0.112000 | 3.262 |
| OS endianness | 0.033875 | 0.059750 | 1.764 |

Every allocation sample returns to zero tracked live bytes after output disposal
and final disposal. This is evidence for these workloads, not a universal leak
proof. Buffered-file construction allocates buffers and descriptors repeatedly;
its 35,201 allocations per batch reflect the registered workload's construction
pattern. Do not compare its raw allocation count with an unrelated workload.

## Performance review

All nine workloads have only one indexed execution record each. There is no
earlier indexed baseline for a same-workload regression comparison. Existing path
and S07 read/close measurements remain retained and were not rerun or relabeled.

The clock workload is about 17% slower than Node, a batch difference of 0.084 ms
over 20,000 iterations. This includes result collection and validation, so it
does not isolate clock-call overhead. Source inspection finds a process-local
`OnceLock<Instant>` origin lookup, elapsed-clock read and fractional-millisecond
conversion on each call. The profile has one output allocation and ten growth
reallocations, with no retained allocation after disposal. Current evidence does
not establish which operation explains the difference. No speculative runtime
rewrite or weakened clock/result contract is justified by this comparison alone.
Preserve this baseline for targeted optimization if later evidence identifies a
material application cost. Native winning every workload is not an acceptance
requirement.

## Final family acceptance

`cargo test --offline --locked -p tsrust-compiler --no-fail-fast` passes 3,207 tests
with zero failures and three intentional ignores. The equivalent `tsrust-lsp`
command passes 532 tests with zero failures and one intentional ignore. Both use
`CARGO_INCREMENTAL=0` and the shared `/private/tmp/tsrust-gate100-editor-target`.
SQLite retains `tsrust-s09-full-compiler.log` and `tsrust-s09-full-lsp.log`.

A locked offline release build supplies eight atomically refreshed CLI/LSP
consumers, with identical source/destination hashes recorded in
`tsrust-s09-final-consumers.json`. Installed smoke checks pass ten clock cases,
ten environment cases and 32 write/full-file/buffered-file native/Node cases.
A fresh installed LSP publishes empty diagnostics and the clock hover, then
shuts down cleanly. Evidence is retained as `tsrust-s09-installed-*.log`.

The existing owner editor executes `Tsrust: Restart Language Server`; its new
installed process starts at 16:11:06 local time on 2026-10-11, PID 13362. The
`main.tsr — tsrust_test` window shows `Tsrust: checking enabled` and an empty,
unfiltered workspace Problems view. No owner files were edited. See
`tsrust-s09-final-editor.json` for the final observation.

Cleanup removes 324 test executables, obsolete workspace incremental outputs and
disposable runtime benchmark outputs: 11,542,234,168 bytes total. All eight current
consumer hashes remain unchanged. The shared target is approximately 4.7 GiB;
the workspace target is 275 MiB. `tsrust-s09-output-cleanup.json` records exact
removed paths and preserved hashes. Future tests rebuild removed executables;
no rebuild was run merely to refill those outputs.
