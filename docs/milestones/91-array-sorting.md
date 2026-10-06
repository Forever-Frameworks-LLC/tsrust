# Milestone 91: Array.sort and Array.toSorted

Status: implementation, executable documentation, combined regression and 28 matched comparisons accepted at checkpoint 91.

`sort(compare?): void` stably orders the receiver in place; `toSorted(compare?): Element[]` returns an independently mutable sorted copy. Primitive defaults are numeric int order, f64 total ordering (including signed zero and NaN bits), false before true, UTF-8 lexical String order, and equal null/undefined leaves. Nested arrays and records require a comparator even when empty. This differs from JavaScript default string-coercing order.

A reusable comparator accepts zero to two read-only/Copy element parameters and returns one concrete int or Number type. Negative/positive results select order; zero and NaN compare equal. Invocation order and count depend on standard Rust sorting. Receiver and callback construction evaluate once in order. The comparator may change unrelated state. In-place sorting holds exclusive receiver access; copying sorting allows reads of the original source but forbids its mutation or consumption. Sparse/mixed arrays, nullable elements, coercion, once-only callbacks and consuming/mutating element callbacks remain unsupported.

In-place sorting rearranges owners without cloning; copying sorting clones owned descendants into the independent result. On comparison panic, sort may leave a partial reorder while preserving all owners; toSorted drops the copied result and leaves the source unchanged.

[Accepted implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T17-35-53-000Z-array-sorting-implementation.json) binds four retained compiler groups and supplements plus one editor group. Six catalog examples produce seven exact lines at O0/O3, including signed-zero ordering, stable record ties and the UTF-8 ﬁ/😀 ordering difference. The editor's initial ENOSPC build failure is retained; no tests ran until space was restored and the exact group passed.

Twenty-eight matched comparisons are measured at checkpoint 91. Rates include receiver/callback setup, native ordering and complete retained output; they do not isolate comparison latency. SQLite is canonical analytics and Markdown exports are optional.

[Completed checkpoint 91](../../benchmarks/runtime/diagnostics/2026-10-03T18-25-41-000Z-checkpoint91-completed-proof.json) accepts 890 distinct checks without a full restart, 68 matched native/compiled-Node comparisons and 204 fully disposed profiles. Native is faster on 34/68; there is no universal speed claim. SQLite preserves exact original records and restoration. Across history, 622 workloads are measured among 623 registered sources; the separate peer path workload remains unmeasured.
