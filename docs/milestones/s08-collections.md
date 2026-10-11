# S08 collections acceptance

Status: implementation, complete regression acceptance and family measurements are
verified, including native-reference review. Targeted performance improvements and
installed-consumer refresh remain pending.
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

## Native reference review

At `7bdd6947`, handwritten Rust controls exercise VecDeque deep clone, BinaryHeap,
BTreeSet and BTreeMap with the same observable scenarios. Independent arithmetic
oracles pass five cases for every control at both optimization levels 0 and 3.
The measured controls retain three processes, eight warmups and 31 samples, plus
three allocation profiles each: 372 timing samples and 12 profiles in total.
Every profile releases all tracked allocations and agrees across its three runs.

| Workload | Generated native / direct Rust throughput | Direct Rust allocations | Direct Rust reallocations |
| --- | ---: | ---: | ---: |
| Deque deep clone | 2.370× | 10,241 | 2,057 |
| Heap | 0.694× | 10,241 | 4,105 |
| OrderedSet | 1.044× | 6,145 | 9 |
| OrderedMap | 1.052× | 6,145 | 9 |

These controls use standard containers and Vec results rather than generated
wrappers and array representations. The result oracle is identical, but this is
not an isolated backend comparison. Generated fixed numeric arrays avoid heap
storage used by the Deque control's Vec values. Tree controls have matching
allocation counts; small timing differences do not justify tree optimization.
Heap's generated path has 4,096 more allocations per 2,048-iteration batch and
warrants inspection of borrowed rest/spread materialization.

Source inspection also identifies two costs in the slower existing workloads.
String iteration allocates an owned string for every scalar and a fresh buffer
for every binary concatenation. Map/Set stores each hash bucket's slot indices
in a separate Vec, allocating even when a bucket contains only one key. An inline
first slot can remove the common-case bucket allocation without changing key
identity, insertion order or collision behavior. Validate collisions, deletion,
reinsertion, cloning and allocation changes before accepting that optimization.

Exact reference inputs, commands, binaries' hashes and observations remain in
SQLite under `runtime/diagnostics/2026-10-11T14-29-47-367Z-direct-rust-collections.json`.
The `s08-direct-rust-proof-7bdd6947.json` proof binds derived execution/profile
records to that original. Derived samples use the `direct-rust` engine and
`direct-rust-reference` scope, separately from generated-native baselines.
The verified control record is
`runtime/diagnostics/2026-10-11T14-29-16-118Z-direct-rust-collections-verify.json`;
an earlier verification record retains a superseded description copied from the
byte reference runner and is not the accepted scope description.

## Remaining acceptance

Complete the identified Map/Set bucket optimization and inspect Heap spread
allocations. Make only evidence-backed changes. Preserve this baseline and rerun only affected workloads
after changes. Refresh all installed consumers and verify editor diagnostics.
Then begin the requested project-wide, multiple-error diagnostics milestone before
S09. The measurement proof accepts the recorded measurements; it does not declare
the complete S08 milestone accepted.
