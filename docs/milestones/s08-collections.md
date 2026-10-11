# S08 collections acceptance

Status: S08 is accepted on macOS. Implementation, complete regressions, matched
measurements, native-reference review, justified optimizations and installed
CLI/editor verification are complete. Project-wide, multiple-error diagnostics
is the next required milestone before S09.

## Regression evidence

The complete run at `f414c3ea` observed all 304 compiler/LSP test and doctest
targets from Cargo metadata. It preserved 53 failed tests across six completed
targets and six stack-overflow aborts. All other targets completed successfully.

The retained full regression and affected reruns accept 3,121 compiler tests and
511 editor tests,
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

## Inline hash bucket optimization

Map/Set now retain a single slot index inline and allocate a vector only for
colliding hashes. Deletion collapses a one-element collision vector back to inline
storage. Key storage, equality, insertion order and explicit cloning are unchanged.
A focused allocator witness originally observed 256 bucket allocations for 128
distinct keys in each container; it now observes zero after reserving the outer
storage. The same witness runs 2,048 forced-collision operations against an
independent sequence model and checks deletion, replacement, reinsertion and
independent clones at optimization levels 0 and 3. All 74 focused collection,
Map/Set, iteration, relation and algebra checks pass. Matched performance reruns now pass for all 14 affected workloads; the original
baseline stays immutable in SQLite.

### Measured bucket results

At `7f6c3887`, the affected workload rerun retains all five correctness cases per
engine, eight warmups, 31 samples and three processes. It adds 2,604 execution
samples and 42 allocation profiles, without retiming startup or compilation.
Native throughput improves by 2.5–38.8% across the 14 workloads. Unchanged Node
controls vary from 0.975× to 1.031×; the smallest native change is within that
variation and should not be treated as a firm speedup. Allocation counts and
requested bytes decrease in every affected workload, with deterministic counters
and zero live bytes after disposal.

Representative results include Map values (1.209× native throughput, 51,201 to
38,913 allocations), Set algebra (1.288×, 141,313 to 88,065 allocations), and
Map byte views (1.388×, 53,249 to 20,481 allocations). Native remains slower than
Node on many complete workloads; this optimization does not establish universal
native superiority. No affected workload shows a measured throughput regression.

SQLite proof `runtime/diagnostics/s08-bucket-optimization-proof-7f6c3887.json`
binds exact source/counterpart identities, emitted-source agreement and protocol
counts to both records and includes every baseline comparison:

- `runtime/results/2026-10-11T14-36-31-471Z.json`
- `runtime/diagnostics/2026-10-11T14-36-56-622Z-native-allocations.json`

All originals were verified byte-for-byte after import before removing loose
copies. The bucket optimization is accepted. Heap still stages each numeric
spread into an intermediate Array before copying it into its final rest array;
its two spread expressions explain the two extra allocations per iteration
relative to the direct Rust control. Preserve eager snapshot and argument-order
semantics when investigating a single-spread rest-pack transfer.

## Single scalar spread transfer

When one scalar spread supplies an entire rest parameter, the emitter now creates
the eager snapshot in the final rest-array representation and transfers that
array into the call. It does not create and copy a second intermediate array.
This applies only to matching scalar element types, one spread, no fixed arguments
and no built-in special handling. Other spread paths retain their existing staging.
Source copying, argument evaluation order and ownership remain unchanged.

An allocator witness fails on the prior implementation with two allocations and
passes with one at optimization levels 0 and 3. It verifies all nine returned
values and that mutating the result leaves the source unchanged. All 99 affected
spread, rest and collection checks pass, including existing later-argument mutation
and depth witnesses. Performance reruns are accepted below.

### Measured spread results

The emitted-code audit identified five previously measured workloads affected by
the change. Their five correctness cases per engine, 930 timing samples and 15
allocation profiles pass at `d6de4131`, with the original eight warmups, 31 samples
and three processes. Heap allocations fall from 14,337 to 10,241 per batch, matching
the direct Rust control's count. Heap native throughput improves 29.5%; its unchanged
Node control improves 2.0%. Native/Node throughput is now 0.744×, so this complete
workload remains slower than Node despite the improvement.

The current iterable-array and iterable-call baselines show 1.037× and 0.997×
native throughput respectively, with stable Node controls and unchanged allocation
counts. Those small differences do not establish meaningful regressions. Older
nested-function-rest and generic-value-adapter records have different source hashes;
the new measurements establish current results without a historical speedup claim.

SQLite proof `runtime/diagnostics/s08-single-spread-proof-d6de4131.json` binds:

- `runtime/results/2026-10-11T14-42-47-703Z.json`
- `runtime/diagnostics/2026-10-11T14-43-24-106Z-native-allocations.json`

The emitted-code audit is `tsrust-s08-single-spread-affected.json`. Original records,
focused failing/passing allocation witnesses and affected reruns are preserved.
String concatenation costs remain visible in the baseline; broader string-storage
changes are not justified as part of this collection optimization. No universal
native performance claim is made.

## Installed consumers and milestone acceptance

All six consumers were atomically refreshed from `d6de4131`: workspace debug and
editor compiler/LSP binaries, workspace release CLI and the installed CLI. Hashes
match their shared-target build outputs. The installed server reports zero
diagnostics for 32 byte-view/collection documents, and four installed CLI assertion
fixtures pass natively. Restarting the active VSCode server clears the stale
OrderedMap error; checking remains enabled and the Problems panel reports no
problems in the already open OrderedMap and recursive-array documents.

`tsrust-s08-consumers-d6de4131.json` in SQLite records consumer hashes, checked
source hashes, native smoke results and editor verification. It also records
1,295,219,361 bytes of removed obsolete incremental, benchmark and test outputs.
All current consumers were rehashed after cleanup. Closed records were restored
byte-for-byte before deleting their loose copies.

S08 acceptance covers all six collection kinds, their ownership/invalidation and
clone boundaries, executable catalog documentation, full regressions with retained
repair evidence, matched measurements, native references and measured optimization.
The two new optimization regressions supplement the original complete run; unaffected
passing targets were retained. No S08 check remains failed, aborted or unrun.
The next work is [project diagnostics](../plans/project-diagnostics-plan.md), including
unopened files and multiple independent errors per file, before beginning S09.
