# Gate 182: scoped aggregate composition family

Status: accepted. B5 in the compiler completion ledger.

## Agreed scope

Borrowing callback records compose through local/nested records, dense arrays,
Maps, nullable local/field/array storage, readonly views, mutable owned replacement,
parameters, results and independent parameter environments. Capturing records remain
invalid Set/Map keys under the owner's structural-key policy. No implicit clone.

The owner assigned mixed-record unions containing callbacks to a dedicated D2
discriminated-union feature on 2026-10-09. That requirement remains open and is
tracked in docs/plans/discriminated-unions-plan.md. It does not block B5's agreed scope.

## Verification discovered repair

The whole compiler run completed 241 groups: 2,585 passes, one failure and three
intentional ignores. The failed historical boundary expected an omitted nullable
record parameter to be rejected. A stronger native witness exposed a real defect:
explicit present arguments lost their capture environment in the default-argument
path, then moved their fields during native injection.

Nullable record injection now preserves borrowing for direct records as well as
existing unions. The native parameter accepts the nominal readonly record bridge
for both owning records and field views. Its fields are accessed through that
bridge. Omission, explicit absence, present arguments, repeated reads and nested
string/data fields are covered by native assertions and formatter round trips.

## Evidence pending publication

Retain the complete initial run and supplementary affected suites. Finish the whole
editor crate, all eight depth canaries and matched timing with three processes,
eight warmups and 31 execution samples per engine per process. The workload retains
136 observations. Its default-argument path preserves the existing oracle values.
Record final evidence and performance review here before claiming acceptance.

## Supplementary correctness evidence

After the native bridge repair, all 208 affected compiler tests pass (seven record
boundaries and 201 composition tests). The initial whole-editor run has 427 passes
and two stale rejection assertions. The two assertions described saved nullable
views and owned stores through mutable parameters, both already covered by native
acceptance. They now assert supported behavior and retain capture conflict/escape
controls. All eight depth canaries pass. The focused native and Node default
parameter probe passes. Whole-project formatting is stable with no changed files
or fingerprints after the semantic workload hashes were updated.

The repaired intelligence suite passes all 279 tests. Combined with retained whole
run evidence, compiler correctness is 2,586 passes with three intentional ignores;
editor correctness is 429 passes with no ignores. These counts retain successful
groups and substitute the repaired affected suites, rather than claiming another
complete rerun. Timing and final family publication remain pending.

## Final measurement and review

Both workloads pass five correctness cases per engine and retain the original
three-process, eight-warmup, 31-sample execution-only protocol. SQLite stores
`runtime/results/2026-10-09T16-27-56-788Z.json` and exact proof
`tsrust-b5-family-acceptance.json`; 372 normalized execution samples are verified.
Startup, compilation and allocation profiles were not measured.

| Complete workload batch | Native median (ms) | Node-after-tsc median (ms) | Node/native |
| --- | ---: | ---: | ---: |
| Scoped record composition | 153.913583 | 155.239750 | 1.009 |
| Fallible record callbacks | 0.072625 | 2.184375 | 30.08 |

Composition is effectively tied in this run. Native process medians are stable
(153.886 to 154.056 ms); Node process medians range from 152.480 to 158.061 ms.
The near-unity ratio provides no material win or regression signal. Fallible
callbacks have a substantial native advantage for this exact pipeline. Neither
workload has an earlier comparable timing baseline. No optimization is justified
by a demonstrated regression. Existing scoped-callable measurements remain intact.

B5 is accepted for its agreed scope. B7 caller-borrowing parameter stores remain
separate and incomplete. Discriminated unions remain a dedicated D2 feature.

SQLite also retains min/mean/max throughput, in complete scenarios per second:

| Workload / engine | Minimum | Mean | Maximum |
| --- | ---: | ---: | ---: |
| Composition / native | 12,726.74 | 13,316.14 | 13,798.61 |
| Composition / Node | 5,680.40 | 13,048.30 | 13,504.50 |
| Fallible / native | 25,666,733.51 | 28,071,981.58 | 28,777,787.15 |
| Fallible / Node | 784,011.07 | 936,474.44 | 985,069.08 |

The Node composition minimum reflects a slower sample; the median comparison above
remains the useful central signal. All samples remain preserved rather than filtered.

## Correction (2026-10-10)

The whole-compiler run recorded for this gate was not clean. One or two suites aborted
with a stack overflow and were not counted, because an aborted suite prints no result
line. The cause was stack growth in the expression checker, not a wrong result. It is
repaired at gate 185; see [the gate 185 record](185-project-editor.md).
