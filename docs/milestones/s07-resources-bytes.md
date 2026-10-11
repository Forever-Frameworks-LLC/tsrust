# S07 resources and bytes: family acceptance

Status: focused gates 187–190, family correctness and matched measurements accepted.
Direct Rust wrapper review and final installed-consumer refresh remain pending.

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

## Recorded measurements (October 11, 2026)

The run at `8ebee85c` completed all 17 workloads, 3,162 timing samples and 51
allocation profiles. Five independent correctness cases per workload passed for both
engines. Every allocation sample returned to zero live bytes after output disposal.
All three allocation samples per workload agree. The original protocol and all
samples, including slow file samples, are retained.

The table reports native mean throughput divided by Node-after-tsc mean throughput.
Allocation counts are per complete 2,048-iteration native batch, excluding input
adaptation and result observation. They include retained output construction.

| Workload | Native / Node throughput | Native allocations | Native reallocations |
| --- | ---: | ---: | ---: |
| `exception_completion` | 2.889× | 33,703 | 3,915 |
| `native_bytes` | 3.342× | 4,097 | 8 |
| `native_byte_access` | 2.466× | 4,097 | 8 |
| `native_byte_growth` | 7.706× | 4,097 | 2,056 |
| `native_byte_copy_append` | 2.770× | 6,145 | 2,056 |
| `native_byte_clone` | 1.878× | 8,193 | 8 |
| `native_byte_utf8` | 45.911× | 10,241 | 8 |
| `native_byte_views` | 6.922× | 8,193 | 8 |
| `native_byte_views_recursive` | 2.222× | 34,817 | 8 |
| `native_byte_views_linked` | 1.976× | 34,817 | 8 |
| `native_byte_views_callbacks` | 10.806× | 2,049 | 8 |
| `native_byte_views_stored_callbacks` | 3.524× | 34,817 | 8 |
| `native_byte_views_captured_only` | 2.423× | 34,817 | 8 |
| `native_byte_views_nullable_callbacks` | 2.331× | 34,817 | 8 |
| `native_byte_views_parameter_stores` | 3.503× | 34,817 | 8 |
| `native_file_read_close` | 1.046× | 2,049 | 8 |
| `native_file_scope_drop` | 0.925× | 2,049 | 8 |

Byte workloads favor native in this run. File release results are close and have
substantial first-process variability: native explicit-close process medians were
50.402, 20.861 and 20.618 ms; Node medians were 46.284, 21.607 and 21.486 ms.
Native scope-drop medians were 39.293, 21.824 and 20.810 ms; Node medians were
27.308, 21.374 and 21.399 ms. The overall 0.925× scope-drop signal is retained,
but these samples do not establish a stable compiler regression. Both native file
workloads have identical allocation profiles and disposal counts. No optimization
or selective retiming is justified by this evidence alone.

There were no previously indexed comparable S07 timing rows. Do not claim a
historical speedup or combine unrelated workload epochs. Direct Rust comparison
for byte/view wrapper overhead remains pending, as does the installed-consumer
refresh. These measurements do not close S07 by themselves.

SQLite records:
- `runtime/results/2026-10-11T09-03-12-817Z.json`
- `runtime/diagnostics/2026-10-11T09-03-35-868Z-native-allocations.json`
- `runtime/results/s07-measurement-proof-2026-10-11.json`

The proof validates sample counts, checksums, current source hashes, matching
compiler inputs and zero final live bytes. Raw records and proof were imported,
indexed into throughput/allocation tables, read back byte-for-byte and removed
from loose storage. SQLite integrity passed.

After the checks closed, 485 obsolete test/catalog outputs totaling 9,945,597,118
logical bytes were removed. Current executable consumers and the 17 current S07
benchmark builds remain. The next full regression build will recreate its test
executables; no rebuild was performed merely to repopulate the cache. Exact cleanup
inventory is stored as `tsrust-s07-cleanup.json` in SQLite.

## Direct Rust comparison support

`node benchmarks/runtime/direct-rust-bytes.mjs --verify` checks handwritten Rust
counterparts for `native_bytes` and `native_byte_views_callbacks` against the same
independent result oracles at O0 and O3. Removing `--verify` runs the original
three-process, eight-warmup, 31-sample protocol and three allocation profiles.
Both variants preserve complete nested result rows and observe/drop them outside
the timed interval. Source hashes and exact commands are captured, and input drift
during a run is rejected.

These references call the same checked byte backend directly from handwritten
Rust control flow, using `Vec` output and ordinary borrowed slices/closures. This
isolates compiler-generated representation and call overhead; it does not compare
alternative byte implementations. Construction covers valid, empty and invalid
values. The view witness covers a captured view and a returned argument view,
including a view derived from a borrowed byte owner. The unused fixed input string
is omitted from direct Rust; native adaptation is outside timing, and its bytes
remain separately reported in allocation evidence. Conclusions are limited to
these two representative workloads, not every byte or recursive-view operation.
