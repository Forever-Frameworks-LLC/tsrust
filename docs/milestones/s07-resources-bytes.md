# S07 resources and bytes: family acceptance

Status: focused gates 187–190 and family correctness accepted. Performance review
and final installed-consumer refresh remain pending. No S07 timing is claimed here.

## Complete regression evidence

The full workspace run at source checkpoint `28467b54` exercised all 300 Cargo
test targets and both library doctest targets. The target inventory was obtained
from Cargo metadata. No targets were missing, no tests aborted, and every target
produced a terminal result. The compiler passed 3,053 tests with three intentional
ignores. The editor initially passed 499 tests with one failure and one intentional
ignore. The original run exited 101 and remains preserved.

The failure was in the general executable catalog runner: newly added `std:fs`
examples require the documented four-byte `input.bin`, but that runner did not
create it. The dedicated file catalog test already supplied isolated input and
passed. The general runner now creates its own temporary directory and input,
executes examples there, removes the directory on scope exit, and identifies the
entry and stderr when execution fails. The affected catalog test passes after the
repair. Combined evidence therefore accepts 3,053 compiler and 500 editor tests;
the other completed suites were not rerun.

The four ignored tests are the opt-in source-worklist diagnostic, native-cache
subprocess worker, frontend fanout measurement and editor scaling measurement.
The subprocess worker is exercised by its parent test. Ignored measurement tests
are not included in the correctness pass count.

VS Code build and grammar checks pass. Full VS Code integration passes against the
current scratch language server, including native run tasks. Three stale assertions
were repaired: two expected four-space indentation, and one expected the old
inferred callable spelling. Explicit `Function<...>` annotations still retain their
written spelling. No language or ownership semantics changed. The initial sandboxed
Electron launch aborted before tests; the authorized unsandboxed run supplied the
integration evidence. Intermediate failed expectations and probes are retained.

All 24 selected byte/file benchmark registration, source-contract, manifest and
measurement-protocol tests pass. The full regression includes formatter token
preservation and idempotence. Whole-project formatting changed no files or hashes.

Exact logs and the target audit are stored in `benchmarks/records.sqlite` under
`tsrust-s07-*` record paths, including `tsrust-s07-regressions.log`,
`tsrust-s07-regression-audit.json`, `tsrust-s07-doc-examples-fixed.log` and
`tsrust-s07-extension-integration-final.log`. Closed records are read back exactly
before loose copies are removed. Installed consumers remain at `4367f4cc`.

## Remaining measurement and integration acceptance

The scope is 17 workloads: `exception_completion`, `native_bytes`, the five byte
access/growth/copy-append/clone/UTF-8 workloads, eight view workloads including
recursive storage and callbacks, and the two file read/release workloads. The
manifest is authoritative for exact IDs and operations. Use native versus
Node-after-tsc execution-only measurements with the existing three processes,
eight warmups and 31 samples. Preserve the five correctness cases per workload.
Review allocations, comparable historical evidence and direct Rust when assessing
wrapper overhead. Preserve original samples before justified optimization.

After measurement review, update the ledger, coverage, roadmap, acceptance map,
guide and handoff, then refresh and verify all five installed consumers atomically.
S08–S21 remain subsequent roadmap work.
