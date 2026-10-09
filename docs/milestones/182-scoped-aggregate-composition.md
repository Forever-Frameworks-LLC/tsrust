# Gate 182: scoped aggregate composition family

Status: family verification in progress. B5 in the compiler completion ledger.

## Agreed scope

Borrowing callback records compose through local/nested records, dense arrays,
Maps, nullable local/field/array storage, readonly views, mutable owned replacement,
parameters, results and independent parameter environments. Capturing records remain
invalid Set/Map keys under the owner's structural-key policy. No implicit clone.

The owner assigned mixed-record unions containing callbacks to a dedicated D2
discriminated-union feature on 2026-10-09. That requirement remains open and is
tracked in docs/discriminated-unions-plan.md. It does not block B5's agreed scope.

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
