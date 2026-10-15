# S14 Tokio tasks

Scope: Tokio runtime bootstrap, owned Task.start()/Job settlement, cancellation,
deadlines, delay/yield, race/allSettled and bounded asynchronous channels. Existing
dedicated threads, ThreadPool and Rust/Rayon APIs remain distinct. No detach,
Promise sharing, async arrows, networking or blocking-work adapter is added.

Implementation retains lazy construction and eager argument ownership. Local
borrowed futures remain supported; scheduling requires owned transferable inputs
and results. Error payloads retain their checked types. Final error-effect
validation rejects stale inferred transport contracts before native emission.

Started obligations follow fields, arrays, captures, returned jobs and lazy group
wrappers. Cancellation alone does not settle a job. Splice on Job-bearing arrays
is rejected until replacement and removal owners can be tracked independently.
Mixed group literals accept Tasks and Jobs with one result type; stored arrays
use one input type. No general callable-union array support is claimed.

Runtime witnesses verify nested cancellation, deadline cleanup, returned-job
ownership transfer, race loser cleanup, pending-message disposal, bounded-channel
backpressure, completed outcomes, worker overlap and repeated exact-once drops.
A three-party barrier proves both scheduled workers execute before either join.
Source finally is preserved during ordinary return/throw, but is not promised
when cancellation drops a suspended future. CPU work without suspension remains
cooperative; cancellation and deadlines may take longer while it runs.

Final correctness acceptance: 4,095 compiler tests pass, with seven pre-existing
ignored tests; 691 editor tests pass. These totals combine the complete runs
with focused repaired targets, without counting reruns twice. Compiler coverage
includes 1,232 library tests (eight Tokio runtime witnesses), 18 CLI unit tests,
23 S13 source tests, 12 S14 source tests, 333 other integration targets and the
new offline async export witness added during the run. Original failures remain
in SQLite alongside their passing repairs.

The milestone caught and repaired optional Tokio dependency filtering in package
management, cached native runtime linking, final inferred error contracts and
editor completion/catalog coverage. All seven package-management scenarios pass,
including transactional preservation on failure and concurrent feature changes.
Offline relocated async exports build and execute in debug and release profiles.
Cancellation tests use a long-lived task instead of racing a one-millisecond timer.

Editor agreement, typing-speed completion and consistency pass for Task, Job,
AsyncError and asynchronous Channel/Sender. All catalog examples execute natively.
The active VS Code server was restarted; six stale diagnostics cleared. Its Task
hover now describes start, timeout, scheduling and ownership. Readable examples
are in examples/tokio-tasks. Public yield() naming and Task hover wording use
“returns” for values and explain scheduling separately.

Matched scheduled-batch verification passes for native and Node after tsc with
checksum 16777216. Its workload measures task scheduling overhead, not CPU speedup.
Both async workloads retain three processes, eight warmups and 31 samples per
process when measured. Verification is untimed and uses one sample.

Evidence is stored byte-exact in benchmarks/records.sqlite. Closed initial logs
use the tsrust-s14- prefix; failed attempts are retained alongside repaired runs.
Source repairs are checkpointed in b36ae1d8, d482b1f5 and 6fce2b90.

## Matched measurements and consumer refresh

Final release compiler source: 6fce2b90. Both workloads passed all checksum
observations with the original three-process, eight-warmup, 31-sample protocol
(93 samples per engine). Only acceptance documentation was uncommitted during
measurement; the records retain that dirty-tree flag.

- Lazy await: native median 0.009625 ms, Node-after-tsc 0.132084 ms per batch.
  The S13 native baseline was 0.009208 ms: +4.5%, or 0.000417 ms. Individual
  process medians overlap the prior range; this small signal does not justify
  a compiler optimization or establish a reliable regression from one run.
- Scheduled batch: native median 11.258250 ms, Node-after-tsc 0.750916 ms for
  4,096 tiny jobs in groups of four. Native is about 15 times slower here.
  Tokio worker scheduling and owned cleanup have substantial overhead for this
  tiny-workload case; Node microtasks do not provide the same worker parallelism.
  This is a scheduling-cost result, not a multicore throughput claim. Worker
  overlap is established separately by the runtime barrier test.

Exact records: `benchmarks/async/lazy-await-2026-10-15T15-40-05.409Z.json` and
`benchmarks/async/scheduled-batch-2026-10-15T15-40-27.350Z.json` in SQLite.
No speculative scheduling optimization was added to this acceptance pass.

Release CLI and both installed server paths were atomically refreshed. The active
VS Code server was restarted again after the final build and reports no problems.
The CLI SHA256 is cae21ac7d026d98f879656cc0776cb5080a46a681251ec7e3e9160bbc0b20c6d;
the server SHA256 is d82aea9c659bf0532d85064f57c0aa1a30373613f3fd9d40265084e32de852f1.

Closed evidence was imported and byte-verified in SQLite. Obsolete intermediates
and test executables were pruned from the single Cargo target while retaining
CLI/server consumers: target size fell from about 24 GB to 162 MB; 90 GiB disk
space remains free. Future Cargo builds will be cold. Active editor-managed
workspace caches were left intact.

Storage validation: SQLite integrity and foreign keys pass; all 33,218 existing
blob hashes and lengths pass. The standard storage verifier rejects 758 historical
absolute record paths; these predate S14 and were not rewritten. New S14 records
use relative paths. Legacy path migration remains separate from runtime acceptance.
