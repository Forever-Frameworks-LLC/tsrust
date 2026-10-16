# S19/S20 owned coordination milestone

Source checkpoint: `6164c22a`. S19 implements owned local LRU/TTL and bounded
serialized loading. S20 uses existing Channel/Sender and task primitives for
bounded workers; it adds no duplicate queue abstraction. Their focused acceptance
is recorded in [cache acceptance](s19-local-cache-acceptance.md),
[loading contracts](../plans/cache-plan.md) and
[queue acceptance](../plans/work-queue-plan.md).

## Combined regressions

The complete compiler and editor milestone ran once, with one Cargo job, disabled
incremental compilation and serial test execution. Final acceptance combines the
original pass and exact affected reruns: **4,194 compiler tests and 731 editor
tests pass**. Eleven compiler and one editor opt-in tests remain ignored. Existing
three-driver SQL evidence is reused; no SQL or task-runtime lifecycle behavior
changed in this section.

The original sandboxed pass had 17 compiler and one editor failure, all caused by
loopback binding returning `Operation not permitted`. Independent socket probing
confirmed the sandbox restriction. Exactly those 18 tests passed outside the
network sandbox, including HTTP/TLS cleanup, relocated offline packages and the
editor catalog example test. No source repair or full rerun was required.

Both separate doctest drivers pass (zero doctest cases). SQLite stores the original
and rerun logs under `tsrust-coordination-family-*`, including the accepted proof.
Every imported record was restored byte-for-byte and SHA-256/integrity checked
before its loose copy was removed.

## First measurements

Both native and Node-after-tsc workloads retain three processes, eight warmups and
31 samples per process. All 372 measured samples match their independent checksum:
97,936 for the LRU workload and 2,080 for the bounded queue. SQLite contains exact
raw records and queryable derived samples/summaries.

| Workload | Native median ms | Node median ms | Native process medians ms | Node process medians ms |
| --- | ---: | ---: | --- | --- |
| LRU, 256 insert/read pairs, capacity 16 | 0.041917 | 0.021875 | 0.040917–0.042667 | 0.019958–0.022333 |
| Queue, 64 jobs, two consumers, capacity 2 | 0.222125 | 0.017041 | 0.182458–0.236084 | 0.015333–0.017625 |

Native is approximately 1.92 times slower for this cache workload and 13.03 times
slower for this queue workload. No earlier comparable records exist, so these are
first baselines, not measured regressions. The cache compares ordinary non-TTL
operations; native still checks expiration and maintains uniquely owned indexed
entries, whereas the scoped Node reference uses Map without a TTL implementation.
Code inspection identifies different costs, not an isolated causal attribution.
Queue timing includes setup, owned handoff, yields and settlement. Tokio worker
scheduling and Node microtasks have different execution costs; this is not a CPU
parallelism or universal runtime comparison. The scoped results do not justify a
scheduler redesign or unrelated optimization before service implementation.

Raw record identities:

- `benchmarks/cache/cache-lru-2026-10-16T11-21-13-388Z.json`
- `benchmarks/work-queue/bounded-work-queue-2026-10-16T11-21-30-688Z.json`

The dirty-tree flag reflects documentation and unrelated owner handoffs; compiler
and workload sources are checkpointed at `6164c22a`. Installed consumers already
contain this source and passed queue execution/editor restart acceptance. Continue
S21 using the [service inventory](../plans/service-readiness-plan.md).

## Retention

Removed 432 completed test executables (18,138,729,688 logical bytes). Shared
library/build caches and all five current CLI/LSP consumers remain. Future affected
test binaries will relink; no rebuild was run merely to refill caches.
