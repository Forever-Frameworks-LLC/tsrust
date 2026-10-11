# Gate 184: generic adapter compile time

Date: 2026-10-10. Ledger row: B8 in `docs/plans/compiler-completion-plan.md`.
Investigation history: `generic-state-join-efficiency.md`,
`../plans/resumable-callable-summaries-plan.md`.

## Result

B8 is accepted as a reconciliation. This gate changes no compiler source. The slow
check of `generic_value_adapters.tsr` was repaired by the resumable callable summary
work accepted at `294af82`. The ledger row and the investigation record still read
"incomplete", so this gate measures the current compiler and closes the row.

## Evidence

Compiler source at `7ee033d9`. Wall-clock `tsrust check`, serialized runs, five release
and three debug runs per workload. Medians:

| Workload | Lines | Release | Debug |
| --- | --- | --- | --- |
| `generic_value_adapters` | 528 | 2.444 s | 10.394 s |
| `generic_callable_replacement` | 244 | 1.266 s | 6.171 s |
| `scoped_record_composition` | 2,985 | 1.195 s | 8.003 s |
| `caller_borrowed_record_stores` | 739 | 0.025 s | 0.127 s |
| six smaller generic workloads | 33 to 53 | at most 0.034 s | at most 0.142 s |

The earlier limits were a 55 s release observation limit and a 120 s debug bound. Both
probes used to be terminated at the limit. The adapter workload now completes about 22
times below the release limit and 11 times below the debug bound. These are compiler
check times. They are not runtime measurements and not a before/after throughput ratio,
because the earlier probes never completed.

Other checks, all passing:

- 103 generic-flow unit controls, including the three prefix-replay budget witnesses
  and both resumption witnesses. One opt-in whole-source diagnostic stays ignored by
  design.
- Seven generic integration suites: 168 tests.
- Registration tests for `generic_value_adapters` and `generic_callable_replacement`.
- Untimed native and Node-after-tsc verification of both workloads.

SQLite records: `tsrust-b8-check-times.json`, `tsrust-b8-generic-suites.log`,
`tsrust-b8-in-place-join-rejected.patch`,
`runtime/results/2026-10-10T00-55-39-818Z-verify.json`.

## Rejected experiment

`State::join` copies every binding, joins the copy and compares it with the original.
A candidate joined the bindings in place after the fallible steps. Three release runs
took 2.18 s to 2.43 s against a 2.44 s median before. That is within run-to-run
variation, so the candidate was removed. Its patch is preserved in SQLite.

## Remaining cost

A sample of the release check shows the time spread across the flow analysis:
`BodyState::drive` copies the binding state per node, guard partitions merge states and
`Binding` values are cloned and dropped. No single site dominates. A further reduction
needs shared binding storage (for example reference-counted bindings with
copy-on-write), which is a structural change to `generic_call_validation`. It is not
required by any current bound. Reopen it only if a real program exceeds an interactive
check time.

## Not done

No whole compiler or editor regression: no source changed. No runtime timing: both
workloads keep their accepted measurements. Consumers stay at `2c232b4e`.
