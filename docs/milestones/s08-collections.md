# S08 collections acceptance

Status: implementation, complete regression acceptance and family measurements are
verified. Performance investigation, native-reference review and installed-consumer
refresh remain pending.
S08 is not yet accepted as a complete roadmap milestone.

## Regression evidence

The complete run at `f414c3ea` observed all 304 compiler/LSP test and doctest
targets from Cargo metadata. It preserved 53 failed tests across six completed
targets and six stack-overflow aborts. All other targets completed successfully.

Focused repairs and reruns now accept 3,119 compiler tests and 511 editor tests,
with three and one intentional ignores respectively. No target remains failed,
aborted or unrun. Completed successful targets were retained rather than restarting
the complete run. The collection suite adds a regression covering 15 invalid
borrowed-query selections. All 40 collection catalog examples execute natively.

The repairs address three causes:

- Borrowed-call detection used for nested Deque projections also made complete
  conditional, logical and non-null selections look like readonly place aliases.
  Restrict that detection to projections and preserve existing ownership diagnostics.
- Borrowed record projections were annotated with their owned representation.
  Infer their borrowed native representation while preserving checked source types.
- Native constructor temporaries enlarged recursive analyzer/emitter frames.
  Extract constructor and collection-hint handling without increasing worker stacks
  or relaxing the original depth guards. All six original depth targets pass.

The general catalog witness runner now recognizes the collection families, and
the dedicated executable runner includes Deque members as well as constructors.

SQLite evidence includes `tsrust-s08-regression-audit-f414c3ea.json`, the complete
original log `tsrust-s08-family-regressions-f414c3ea.log`, and
`tsrust-s08-repaired-regression-audit.json`. Original failures, probes and affected
reruns remain available. The repaired audit maps each target to retained or
replacement evidence and distinguishes compiler targets from similarly named LSP
targets.

## Family measurement evidence

At `c7dc044e`, all 21 pending collection workloads passed five independent
correctness cases per engine and completed native versus Node-after-tsc timing.
The unchanged protocol uses eight warmups, 31 samples and three processes per
engine, with 11 startup samples and three compilation samples. The record contains
3,906 execution samples. All 63 allocation profiles agree within each workload
and return to zero live bytes after output disposal.

The table reports mean native throughput divided by mean Node throughput. A ratio
below one means native was slower. These are complete workload comparisons, not
isolated container operations: construction, strings, snapshots, clones and result
construction contribute to the measured work. Ordered collections and Heap use
documented TypeScript counterparts; these are equivalent result workloads, not
identical storage implementations.

| Workload | Native / Node throughput | Native allocations | Native reallocations |
| --- | ---: | ---: | ---: |
| `set_values` | 0.342× | 69,633 | 3,112 |
| `map_values` | 0.332× | 51,201 | 9 |
| `map_snapshots` | 0.703× | 40,961 | 3,112 |
| `set_entries` | 0.757× | 49,153 | 9 |
| `set_for_each` | 0.270× | 38,913 | 9 |
| `map_for_each` | 0.290× | 47,105 | 9 |
| `set_relations` | 0.494× | 55,297 | 9 |
| `set_algebra` | 0.353× | 141,313 | 6,285 |
| `map_iteration` | 0.261× | 57,345 | 9 |
| `string_iteration` | 0.199× | 344,065 | 9 |
| `array_from_collections` | 0.942× | 79,873 | 5,067 |
| `iterable_array_spread` | 0.539× | 104,449 | 18,440 |
| `iterable_call_spread` | 0.809× | 102,401 | 12,297 |
| `map_group_by` | 0.421× | 114,689 | 24,557 |
| `native_byte_views_maps` | 0.602× | 53,249 | 4,104 |
| `native_byte_views_deque` | 3.506× | 6,145 | 4,104 |
| `native_byte_views_deque_callbacks` | 2.923× | 6,145 | 4,104 |
| `native_deque_clone` | 1.080× | 6,145 | 8 |
| `native_heap` | 0.587× | 14,337 | 4,104 |
| `native_ordered_set` | 3.005× | 6,145 | 8 |
| `native_ordered_map` | 7.252× | 6,145 | 8 |

Five workloads have higher native throughput; sixteen have lower native throughput.
String iteration is the slowest relative result and performs 344,065 allocations
per batch. Its concatenations and per-character values warrant investigation.
Map/Set workloads also warrant representation and allocation review. These signals
do not yet establish the cause or justify a particular optimization.

No indexed earlier throughput records exist for these workload IDs. This pass
establishes their baseline; it does not demonstrate a historical regression or
speedup. Do not combine measurements from different workloads or semantic epochs.

SQLite stores the exact originals and normalized samples:

- `runtime/results/2026-10-11T14-24-18-678Z.json`
- `runtime/diagnostics/2026-10-11T14-24-44-679Z-native-allocations.json`
- `runtime/diagnostics/s08-measurement-proof-c7dc044e.json`

The proof verifies source/counterpart fingerprints, emitted-source agreement,
protocol counts, deterministic profiles and complete disposal. All records were
read back byte-for-byte before removing their loose copies. Release build and
runner logs are retained in SQLite under their `tsrust-s08-*-c7dc044e.log` names.

## Remaining acceptance

Investigate slower workloads, complete native-reference review and make only
justified optimizations. Preserve this baseline and rerun only affected workloads
after changes. Refresh all installed consumers and verify editor diagnostics.
Then begin the requested project-wide, multiple-error diagnostics milestone before
S09. The measurement proof accepts the recorded measurements; it does not declare
the complete S08 milestone accepted.
